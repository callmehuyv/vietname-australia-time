use chrono::{Offset, Timelike, Utc};
use chrono_tz::Tz;
use eframe::egui;

// Preset timezones using IANA names — DST is handled automatically
const PRESETS: &[(&str, &str, &str)] = &[
    ("Sydney", "Australia/Sydney", "Australia Eastern"),
    ("Melbourne", "Australia/Melbourne", "Australia Eastern"),
    ("Brisbane", "Australia/Brisbane", "Australia Eastern (no DST)"),
    ("Adelaide", "Australia/Adelaide", "Australia Central"),
    ("Darwin", "Australia/Darwin", "Australia Central (no DST)"),
    ("Perth", "Australia/Perth", "Australia Western"),
    ("Ho Chi Minh", "Asia/Ho_Chi_Minh", "Vietnam"),
    ("Hanoi", "Asia/Ho_Chi_Minh", "Vietnam"),
    ("Tokyo", "Asia/Tokyo", "Japan"),
    ("Seoul", "Asia/Seoul", "Korea"),
    ("Shanghai", "Asia/Shanghai", "China"),
    ("Singapore", "Asia/Singapore", "Singapore"),
    ("Kolkata", "Asia/Kolkata", "India"),
    ("Dubai", "Asia/Dubai", "Gulf"),
    ("Moscow", "Europe/Moscow", "Russia"),
    ("Berlin", "Europe/Berlin", "Central Europe"),
    ("Paris", "Europe/Paris", "Central Europe"),
    ("London", "Europe/London", "UK"),
    ("New York", "America/New_York", "US Eastern"),
    ("Chicago", "America/Chicago", "US Central"),
    ("Denver", "America/Denver", "US Mountain"),
    ("Los Angeles", "America/Los_Angeles", "US Pacific"),
    ("Auckland", "Pacific/Auckland", "New Zealand"),
];

struct TzClock {
    display_name: String,
    iana_tz: Tz,
    /// Current UTC offset in seconds (updated dynamically for DST)
    current_offset_secs: i32,
    /// Current abbreviation like "AEDT" or "AEST"
    current_abbrev: String,
    hour: i32,
    min: i32,
    sec: i32,
}

impl TzClock {
    fn new(display_name: &str, tz: Tz) -> Self {
        Self {
            display_name: display_name.to_string(),
            iana_tz: tz,
            current_offset_secs: 0,
            current_abbrev: String::new(),
            hour: 0,
            min: 0,
            sec: 0,
        }
    }

    fn refresh_from_utc(&mut self, utc: chrono::DateTime<Utc>) {
        let local = utc.with_timezone(&self.iana_tz);
        self.hour = local.hour() as i32;
        self.min = local.minute() as i32;
        self.sec = local.second() as i32;
        self.current_offset_secs = local.offset().fix().local_minus_utc();
        self.current_abbrev = local.format("%Z").to_string();
    }

    fn offset_hours_display(&self) -> String {
        let h = self.current_offset_secs / 3600;
        let m = (self.current_offset_secs.abs() % 3600) / 60;
        if m == 0 {
            format!("UTC{:+}", h)
        } else {
            format!("UTC{:+}:{:02}", h, m)
        }
    }
}

struct ClockApp {
    clocks: Vec<TzClock>,
    synced_to_now: bool,
    show_add_dialog: bool,
    add_search: String,
    custom_name: String,
    custom_iana: String,
}

impl ClockApp {
    fn new() -> Self {
        let mut app = Self {
            clocks: vec![
                TzClock::new("Vietnam", "Asia/Ho_Chi_Minh".parse().unwrap()),
                TzClock::new("Sydney", "Australia/Sydney".parse().unwrap()),
            ],
            synced_to_now: true,
            show_add_dialog: false,
            add_search: String::new(),
            custom_name: String::new(),
            custom_iana: String::new(),
        };
        app.sync_to_now();
        app
    }

    fn sync_to_now(&mut self) {
        let now = Utc::now();
        for tz in &mut self.clocks {
            tz.refresh_from_utc(now);
        }
        self.synced_to_now = true;
    }

    fn update_others_from(&mut self, source_idx: usize) {
        let src = &self.clocks[source_idx];
        // Convert source local time to UTC minutes
        let src_offset_mins = src.current_offset_secs / 60;
        let src_utc_mins = src.hour * 60 + src.min - src_offset_mins;
        let src_sec = src.sec;

        for i in 0..self.clocks.len() {
            if i == source_idx {
                continue;
            }
            let dst_offset_mins = self.clocks[i].current_offset_secs / 60;
            let total = src_utc_mins + dst_offset_mins;
            let wrapped = ((total % 1440) + 1440) % 1440;
            self.clocks[i].hour = wrapped / 60;
            self.clocks[i].min = wrapped % 60;
            self.clocks[i].sec = src_sec;
        }
    }
}

fn time_button(ui: &mut egui::Ui, label: &str) -> bool {
    let btn = egui::Button::new(egui::RichText::new(label).size(18.0).monospace())
        .min_size(egui::vec2(36.0, 32.0));
    ui.add(btn).clicked()
}

fn wrap_hour(h: i32) -> i32 {
    ((h % 24) + 24) % 24
}

fn wrap_min(m: i32) -> i32 {
    ((m % 60) + 60) % 60
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([500.0, 500.0])
            .with_min_inner_size([420.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Timezone Clock",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClockApp::new()))
        }),
    )
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.synced_to_now {
            self.sync_to_now();
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(6.0);
                    ui.heading("Timezone Clock");
                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        if ui
                            .button(if self.synced_to_now {
                                "Live (synced)"
                            } else {
                                "Reset to now"
                            })
                            .clicked()
                        {
                            self.sync_to_now();
                        }
                        if ui.button("+ Add timezone").clicked() {
                            self.show_add_dialog = true;
                            self.add_search.clear();
                        }
                    });
                    ui.add_space(8.0);
                });

                let mut changed_idx: Option<usize> = None;
                let mut remove_idx: Option<usize> = None;

                for (idx, tz) in self.clocks.iter_mut().enumerate() {
                    ui.group(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{}  ({} / {})",
                                        tz.display_name,
                                        tz.current_abbrev,
                                        tz.offset_hours_display(),
                                    ))
                                    .strong()
                                    .size(13.0),
                                );
                                if idx > 0 {
                                    if ui
                                        .small_button(egui::RichText::new("X").color(
                                            egui::Color32::from_rgb(200, 80, 80),
                                        ))
                                        .clicked()
                                    {
                                        remove_idx = Some(idx);
                                    }
                                }
                            });
                            ui.add_space(2.0);

                            ui.label(
                                egui::RichText::new(format!(
                                    "{:02} : {:02} : {:02}",
                                    tz.hour, tz.min, tz.sec
                                ))
                                .size(36.0)
                                .monospace(),
                            );
                            ui.add_space(2.0);

                            let mut changed = false;
                            ui.horizontal(|ui| {
                                ui.add_space(50.0);
                                if time_button(ui, "-") {
                                    tz.hour = wrap_hour(tz.hour - 1);
                                    changed = true;
                                }
                                ui.label(egui::RichText::new("Hr").size(13.0));
                                if time_button(ui, "+") {
                                    tz.hour = wrap_hour(tz.hour + 1);
                                    changed = true;
                                }
                                ui.add_space(16.0);
                                if time_button(ui, "-") {
                                    tz.min = wrap_min(tz.min - 1);
                                    changed = true;
                                }
                                ui.label(egui::RichText::new("Min").size(13.0));
                                if time_button(ui, "+") {
                                    tz.min = wrap_min(tz.min + 1);
                                    changed = true;
                                }
                            });
                            if changed {
                                changed_idx = Some(idx);
                            }
                        });
                    });
                    ui.add_space(4.0);
                }

                if let Some(idx) = changed_idx {
                    self.synced_to_now = false;
                    self.update_others_from(idx);
                }
                if let Some(idx) = remove_idx {
                    self.clocks.remove(idx);
                }
            });
        });

        // Add timezone dialog
        if self.show_add_dialog {
            egui::Window::new("Add Timezone")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("Search:");
                    ui.text_edit_singleline(&mut self.add_search);
                    ui.add_space(4.0);

                    let search = self.add_search.to_lowercase();
                    let mut added = false;

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for (name, iana, region) in PRESETS {
                                let haystack =
                                    format!("{} {} {}", name, iana, region).to_lowercase();
                                if !search.is_empty() && !haystack.contains(&search) {
                                    continue;
                                }
                                // Show current offset for this tz
                                let tz: Tz = iana.parse().unwrap();
                                let now = Utc::now().with_timezone(&tz);
                                let off = now.offset().fix().local_minus_utc();
                                let oh = off / 3600;
                                let om = (off.abs() % 3600) / 60;
                                let abbr = now.format("%Z").to_string();
                                let offset_str = if om == 0 {
                                    format!("UTC{:+}", oh)
                                } else {
                                    format!("UTC{:+}:{:02}", oh, om)
                                };

                                if ui
                                    .button(format!(
                                        "{} - {} ({} / {})",
                                        name, region, abbr, offset_str
                                    ))
                                    .clicked()
                                {
                                    self.clocks.push(TzClock::new(name, tz));
                                    added = true;
                                }
                            }
                        });

                    ui.add_space(6.0);
                    ui.separator();
                    ui.label("Or enter IANA timezone (e.g. Europe/Rome):");
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.custom_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("IANA:");
                        ui.text_edit_singleline(&mut self.custom_iana);
                    });
                    if ui.button("Add custom").clicked() && !self.custom_name.is_empty() {
                        if let Ok(tz) = self.custom_iana.parse::<Tz>() {
                            self.clocks
                                .push(TzClock::new(&self.custom_name.clone(), tz));
                            added = true;
                            self.custom_name.clear();
                            self.custom_iana.clear();
                        }
                    }

                    ui.add_space(4.0);
                    if ui.button("Close").clicked() || added {
                        self.show_add_dialog = false;
                        if self.synced_to_now {
                            self.sync_to_now();
                        } else if !self.clocks.is_empty() {
                            self.update_others_from(0);
                        }
                    }
                });
        }
    }
}
