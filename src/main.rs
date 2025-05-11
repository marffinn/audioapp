#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console window on Windows in release

use eframe::egui;
use egui::{Color32, RichText, Slider};
use cpal::traits::{DeviceTrait, HostTrait};
use windows_volume_control::{AudioController, CoinitMode};

#[cfg(target_os = "windows")]
mod win_utils {
    use winapi::um::winuser::{ReleaseCapture, SendMessageW, PostMessageW};
    use winapi::um::winuser::{WM_NCLBUTTONDOWN, HTCAPTION, WM_SYSCOMMAND, SC_MINIMIZE};
    use winapi::shared::windef::HWND;
    use winapi::shared::minwindef::{LPARAM, WPARAM};

    pub fn drag_window(hwnd: HWND) {
        unsafe {
            ReleaseCapture();
            SendMessageW(hwnd, WM_NCLBUTTONDOWN, HTCAPTION as WPARAM, 0 as LPARAM);
        }
    }

    pub fn minimize_window(hwnd: HWND) {
        unsafe {
            PostMessageW(hwnd, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0 as LPARAM);
        }
    }
}

// Application state
struct AudioApp {
    device_names: Vec<String>,
    selected_device_idx: Option<usize>,
    volume: f32,
    is_muted: bool,
    audio_controller: Option<AudioController>,
}

impl AudioApp {
    fn new() -> Self {
        // Get audio devices
        let host = cpal::default_host();

        // Get all devices and their names
        let devices: Vec<cpal::Device> = match host.output_devices() {
            Ok(devices) => devices.collect(),
            Err(_) => Vec::new(),
        };

        let device_names: Vec<String> = devices
            .iter()
            .filter_map(|device| device.name().ok())
            .collect();

        // Try to find the default device
        let default_device = host.default_output_device();
        let default_device_name = default_device.as_ref().and_then(|d| d.name().ok());

        // Find the index of the default device in our list
        let selected_device_idx = if let Some(default_name) = default_device_name {
            device_names.iter().position(|name| name == &default_name).map(Some).unwrap_or(None)
        } else {
            None
        };

        // Initialize with default values
        let mut app = Self {
            audio_controller: None,
            device_names,
            selected_device_idx,
            volume: 0.5,
            is_muted: false,
        };

        // Initialize audio controller with apartment threading
        unsafe {
            let mut controller = AudioController::init(Some(CoinitMode::ApartmentThreaded));
            controller.GetSessions();
            controller.GetDefaultAudioEnpointVolumeControl();
            controller.GetAllProcessSessions();

            // Get initial volume
            if let Some(session) = controller.get_session_by_name("master".to_string()) {
                app.volume = session.getVolume();
                app.is_muted = session.getMute();
            }

            app.audio_controller = Some(controller);
        }

        app
    }

    fn update_volume(&mut self) {
        if let Some(controller) = &self.audio_controller {
            unsafe {
                if let Some(session) = controller.get_session_by_name("master".to_string()) {
                    self.volume = session.getVolume();
                    self.is_muted = session.getMute();
                }
            }
        }
    }

    fn set_volume(&mut self, volume: f32) {
        if let Some(controller) = &self.audio_controller {
            unsafe {
                if let Some(session) = controller.get_session_by_name("master".to_string()) {
                    session.setVolume(volume);
                    self.volume = volume;
                }
            }
        }
    }

    fn toggle_mute(&mut self) {
        if let Some(controller) = &self.audio_controller {
            unsafe {
                if let Some(session) = controller.get_session_by_name("master".to_string()) {
                    let new_mute_state = !session.getMute();
                    session.setMute(new_mute_state);
                    self.is_muted = new_mute_state;
                }
            }
        }
    }

    // Set the default audio device in Windows by index
    fn set_default_device(&mut self, device_idx: usize) {
        if device_idx >= self.device_names.len() {
            return;
        }

        // Get the device name - clone it to avoid borrow issues
        let device_name = self.device_names[device_idx].clone();
        self.set_default_device_by_name(&device_name);
    }

    // Refresh the list of audio devices
    fn refresh_devices(&mut self) {
        // Get audio devices
        let host = cpal::default_host();
        let mut device_names = Vec::new();

        // Get output devices
        if let Ok(devices) = host.output_devices() {
            for device in devices {
                if let Ok(name) = device.name() {
                    device_names.push(name);
                }
            }
        }

        // Update the device list
        self.device_names = device_names;

        // Reset selected device if it's no longer valid
        if let Some(idx) = self.selected_device_idx {
            if idx >= self.device_names.len() {
                self.selected_device_idx = None;
            }
        }
    }

    // Set the default audio device in Windows by name
    fn set_default_device_by_name(&mut self, device_name: &str) {
        #[cfg(target_os = "windows")]
        {
            // First try using the Windows API directly through winapi
            if let Err(_) = self.set_default_device_winapi(device_name) {
                // Fall back to PowerShell if the direct approach fails
                self.set_default_device_powershell(device_name);
            }
        }
    }

    #[cfg(target_os = "windows")]
    fn set_default_device_winapi(&self, device_name: &str) -> Result<(), &'static str> {
        use winapi::um::objbase::CoInitialize;
        use std::ptr;

        // Since implementing the full COM interface for audio device management is complex,
        // we'll just initialize COM and then fall back to PowerShell for simplicity
        unsafe {
            // Initialize COM
            CoInitialize(ptr::null_mut());

            // For a full implementation, we would:
            // 1. Create an MMDeviceEnumerator
            // 2. Enumerate audio endpoints
            // 3. Find the device by name
            // 4. Set it as the default device

            // But for simplicity, we'll just return an error to fall back to PowerShell
            Err("Using PowerShell fallback")
        }
    }

    #[cfg(target_os = "windows")]
    fn set_default_device_powershell(&self, device_name: &str) {
        use std::process::Command;

        // Try multiple approaches to set the default audio device

        // Approach 1: Using AudioDeviceCmdlets module (if installed)
        let ps_command1 = format!(
            "if (Get-Command Get-AudioDevice -ErrorAction SilentlyContinue) {{ \
             Get-AudioDevice -List | Where-Object {{ $_.Name -eq '{}' }} | Set-AudioDevice \
             }}",
            device_name.replace("'", "''") // Escape single quotes for PowerShell
        );

        // Approach 2: Using SoundVolumeView (if available)
        let ps_command2 = format!(
            "if (Test-Path 'C:\\Windows\\SoundVolumeView.exe') {{ \
             C:\\Windows\\SoundVolumeView.exe /SetDefault \"{}\" all \
             }}",
            device_name.replace("\"", "\\\"") // Escape quotes
        );

        // Approach 3: Using Windows API directly through PowerShell
        let ps_command3 = format!(
            "$devices = Get-WmiObject -Class Win32_SoundDevice; \
             foreach ($device in $devices) {{ \
             if ($device.Name -like '*{}*') {{ \
             $device.SetDefault() \
             }} \
             }}",
            device_name.replace("'", "''")
        );

        // Combine all approaches
        let full_command = format!("{} ; {} ; {}", ps_command1, ps_command2, ps_command3);

        // Try to run the command
        let _ = Command::new("powershell")
            .args(&["-Command", &full_command])
            .spawn();
    }
}

// Extension trait to get the window handle from eframe
trait FrameExt {
    fn hwnd(&self) -> Option<isize>;
}

impl FrameExt for eframe::Frame {
    fn hwnd(&self) -> Option<isize> {
        #[cfg(target_os = "windows")]
        {
            // Try to get the native window handle
            // This is a simplified approach - in a real app, we'd use raw_window_handle
            // but for this demo we'll use a simpler approach
            use std::ptr;
            use winapi::um::winuser::GetForegroundWindow;

            unsafe {
                let hwnd = GetForegroundWindow();
                if hwnd != ptr::null_mut() {
                    return Some(hwnd as isize);
                }
            }
        }
        None
    }
}

impl eframe::App for AudioApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update volume from system
        self.update_volume();

        // We'll implement a simpler dragging mechanism

        // Use the central panel directly instead of creating a nested window
        egui::CentralPanel::default().show(ctx, |ui| {
            // Add some padding around the entire UI
            ui.spacing_mut().item_spacing = egui::vec2(10.0, 10.0);

            // Custom title bar with proper dragging
            let _title_bar = egui::Frame::none()
                .fill(ui.visuals().window_fill())
                .inner_margin(egui::style::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Make the title draggable
                        let title_label = ui.add(
                            egui::Label::new(RichText::new("Audio Controller").size(16.0).strong())
                                .sense(egui::Sense::click_and_drag())
                        );

                        // Handle dragging
                        if title_label.dragged() {
                            #[cfg(target_os = "windows")]
                            {
                                // Get the window handle from the native window ID
                                if let Some(hwnd) = _frame.hwnd() {
                                    // Convert to HWND
                                    let hwnd = hwnd as winapi::shared::windef::HWND;
                                    // Call our drag function
                                    win_utils::drag_window(hwnd);
                                }
                            }
                        }

                        // Push buttons to the right with flexible space
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Close button
                            if ui.button(RichText::new("✖").size(16.0)).clicked() {
                                std::process::exit(0);  // Just exit the process
                            }

                            // Minimize button - use Windows API to minimize
                            if ui.button(RichText::new("_").size(16.0)).clicked() {
                                #[cfg(target_os = "windows")]
                                {
                                    // Get the window handle from the native window ID
                                    if let Some(hwnd) = _frame.hwnd() {
                                        // Convert to HWND
                                        let hwnd = hwnd as winapi::shared::windef::HWND;
                                        // Call our minimize function
                                        win_utils::minimize_window(hwnd);
                                    }
                                }
                            }

                            // Add some space between buttons
                            ui.add_space(10.0);

                            // Refresh button - update the device list
                            if ui.add(egui::Button::new(RichText::new("🔄").size(16.0))
                                .min_size(egui::vec2(24.0, 24.0))
                                .fill(ui.visuals().widgets.active.bg_fill))
                                .clicked()
                            {
                                // Refresh the device list
                                self.refresh_devices();
                            }
                        });
                    });
                }).response;

            ui.separator();
            ui.add_space(5.0);

            // Device selection - make it responsive with padding
            ui.add_space(10.0); // Add more padding above

            let _device_frame = egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color) // Slightly different background
                .inner_margin(egui::style::Margin::same(12.0)) // Add more padding inside
                .rounding(egui::Rounding::same(6.0)) // Add rounded corners
                .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)) // Add border
                .show(ui, |ui| {
                    // Use vertical layout for better organization
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Output Device:").strong().size(16.0));
                        ui.add_space(8.0);

                        // Use full width for the combo box
                        // Track if a device was selected
                        let mut selected_device = None;

                        // Make the combo box take the full width with better visibility
                        let combo = egui::ComboBox::from_label("")
                            .selected_text(
                                self.selected_device_idx
                                    .map(|idx| {
                                        // Truncate long device names for display
                                        let name = self.device_names[idx].clone();
                                        if name.len() > 25 {
                                            format!("{}...", &name[0..22])
                                        } else {
                                            name
                                        }
                                    })
                                    .unwrap_or_else(|| "Select a device".to_string()),
                            )
                            .width(ui.available_width()) // Use full width
                            .height(250.0) // Increase maximum height for the dropdown
                            .wrap(false); // Prevent text wrapping in dropdown

                        combo.show_ui(ui, |ui| {
                            // Add a scrolling area for many devices
                            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                                for (idx, name) in self.device_names.iter().enumerate() {
                                    let response = ui.selectable_value(&mut self.selected_device_idx, Some(idx), name);

                                    // If a new device is selected, store the index
                                    if response.clicked() {
                                        selected_device = Some(idx);
                                    }
                                }
                            });
                        });

                        // If a device was selected, set it as default
                        if let Some(idx) = selected_device {
                            // We need to clone the device name because we'll need to borrow self mutably
                            let device_name = self.device_names[idx].clone();
                            self.set_default_device_by_name(&device_name);
                        }
                    });
                });

            ui.add_space(10.0); // Add more padding below

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            // Volume control - in a frame with padding for better appearance
            let _volume_frame = egui::Frame::none()
                .fill(ui.visuals().extreme_bg_color)
                .inner_margin(egui::style::Margin::same(10.0))
                .show(ui, |ui| {
                    // Use a vertical layout for better responsiveness
                    ui.vertical(|ui| {
                        // First row: Mute button and volume percentage
                        ui.horizontal(|ui| {
                            // Use different icons for mute/unmute
                            let mute_btn_text = if self.is_muted {
                                RichText::new("🔇").color(Color32::RED).size(20.0)
                            } else {
                                RichText::new("🎵").color(Color32::GREEN).size(20.0)
                            };

                            // Make button a bit larger
                            if ui.add(egui::Button::new(mute_btn_text).min_size(egui::vec2(36.0, 36.0))).clicked() {
                                self.toggle_mute();
                            }

                            // Push volume percentage to the right
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.label(RichText::new(format!("{}%", (self.volume * 100.0) as i32)).size(18.0));
                            });
                        });

                        // Second row: Full-width slider with better visibility
                        ui.add_space(4.0); // Add some space above the slider

                        // Create a frame for the slider to make it more visible
                        let slider_frame = egui::Frame::none()
                            .fill(ui.visuals().widgets.inactive.bg_fill)
                            .inner_margin(egui::style::Margin::same(12.0)) // Increased padding
                            .rounding(egui::Rounding::same(6.0)) // Increased rounding
                            .stroke(egui::Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color)) // Add border
                            .show(ui, |ui| {
                                // Add some extra space for better visibility
                                ui.add_space(4.0);

                                // Make the slider larger and more visible
                                let volume_response = ui.add_sized(
                                    [ui.available_width(), 30.0], // Make the slider taller
                                    Slider::new(&mut self.volume, 0.0..=1.0)
                                        .text("Volume")
                                        .show_value(false)
                                        .trailing_fill(true) // Fill the slider to show current level
                                );

                                ui.add_space(4.0);
                                volume_response
                            }).inner;

                        ui.add_space(4.0); // Add some space below the slider

                        if slider_frame.changed() {
                            self.set_volume(self.volume);
                        }
                    });
                });
        });

        // Request a repaint for smooth updates
        ctx.request_repaint();
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 350.0])  // Much larger size to ensure all content is visible
            .with_always_on_top()
            .with_decorations(false)  // No default window decorations
            .with_transparent(false)
            .with_title("Audio Controller"),  // Title for taskbar
        ..Default::default()
    };

    eframe::run_native(
        "Audio Controller",
        options,
        Box::new(|_cc| Box::new(AudioApp::new())),
    )
    .unwrap();
}
