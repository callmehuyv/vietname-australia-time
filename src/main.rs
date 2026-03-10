use chrono::{Offset, Timelike, Utc};
use chrono_tz::Tz;
use eframe::egui;

const PRESETS: &[(&str, &str, &str)] = &[
    ("Sydney", "Australia/Sydney", "Australia Eastern"),
    ("Melbourne", "Australia/Melbourne", "Australia Eastern"),
    ("Brisbane", "Australia/Brisbane", "AU Eastern (no DST)"),
    ("Adelaide", "Australia/Adelaide", "Australia Central"),
    ("Darwin", "Australia/Darwin", "AU Central (no DST)"),
    ("Perth", "Australia/Perth", "Australia Western"),
    ("Ho Chi Minh", "Asia/Ho_Chi_Minh", "Vietnam"),
    ("Hanoi", "Asia/Ho_Chi_Minh", "Vietnam"),
    ("Tokyo", "Asia/Tokyo", "Japan"),
    ("Seoul", "Asia/Seoul", "Korea"),
    ("Shanghai", "Asia/Shanghai", "China"),
    ("Hong Kong", "Asia/Hong_Kong", "Hong Kong"),
    ("Singapore", "Asia/Singapore", "Singapore"),
    ("Kolkata", "Asia/Kolkata", "India"),
    ("Dubai", "Asia/Dubai", "Gulf"),
    ("Moscow", "Europe/Moscow", "Russia"),
    ("Berlin", "Europe/Berlin", "Central Europe"),
    ("Paris", "Europe/Paris", "Central Europe"),
    ("London", "Europe/London", "United Kingdom"),
    ("New York", "America/New_York", "US Eastern"),
    ("Chicago", "America/Chicago", "US Central"),
    ("Denver", "America/Denver", "US Mountain"),
    ("Los Angeles", "America/Los_Angeles", "US Pacific"),
    ("Auckland", "Pacific/Auckland", "New Zealand"),
];

#[derive(PartialEq)]
enum Page {
    Clock,
    AddTimezone,
}

struct TzClock {
    display_name: String,
    iana_tz: Tz,
    current_offset_secs: i32,
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

    fn offset_display(&self) -> String {
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
    page: Page,
    add_search: String,
    custom_name: String,
    custom_iana: String,
}

// Colors
const ACCENT: egui::Color32 = egui::Color32::from_rgb(110, 150, 210);
const ACCENT_DIM: egui::Color32 = egui::Color32::from_rgb(65, 90, 135);
const CARD_BG: egui::Color32 = egui::Color32::from_rgb(52, 56, 66);
const REMOVE_COLOR: egui::Color32 = egui::Color32::from_rgb(195, 95, 95);
const SUBTITLE: egui::Color32 = egui::Color32::from_rgb(150, 155, 168);
const TIME_COLOR: egui::Color32 = egui::Color32::from_rgb(205, 210, 225);
const TEXT_PRIMARY: egui::Color32 = egui::Color32::from_rgb(200, 205, 215);
const BTN_BG: egui::Color32 = egui::Color32::from_rgb(62, 67, 80);

impl ClockApp {
    fn new() -> Self {
        let mut app = Self {
            clocks: vec![
                TzClock::new("Vietnam", "Asia/Ho_Chi_Minh".parse().unwrap()),
                TzClock::new("Sydney", "Australia/Sydney".parse().unwrap()),
            ],
            synced_to_now: true,
            page: Page::Clock,
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

    fn render_clock_page(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        // Header
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Timezone Clock").size(22.0).strong().color(TEXT_PRIMARY));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(16.0);
                let add_btn = egui::Button::new(
                    egui::RichText::new("+ Add").size(14.0).color(TEXT_PRIMARY),
                )
                .fill(ACCENT_DIM)
                .corner_radius(6.0)
                .min_size(egui::vec2(70.0, 30.0));
                if ui.add(add_btn).clicked() {
                    self.page = Page::AddTimezone;
                    self.add_search.clear();
                }

                // Sync button
                let (sync_label, _sync_color) = if self.synced_to_now {
                    ("LIVE", egui::Color32::from_rgb(80, 200, 120))
                } else {
                    ("SYNC", ACCENT)
                };
                let sync_btn = egui::Button::new(
                    egui::RichText::new(sync_label).size(13.0).strong().color(TEXT_PRIMARY),
                )
                .fill(if self.synced_to_now {
                    egui::Color32::from_rgb(40, 100, 60)
                } else {
                    ACCENT_DIM
                })
                .corner_radius(6.0)
                .min_size(egui::vec2(60.0, 30.0));
                if ui.add(sync_btn).clicked() {
                    self.sync_to_now();
                }
            });
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(8.0);

        // Clock cards
        let mut changed_idx: Option<usize> = None;
        let mut remove_idx: Option<usize> = None;

        for (idx, tz) in self.clocks.iter_mut().enumerate() {
            let card = egui::Frame::new()
                .fill(CARD_BG)
                .corner_radius(10.0)
                .inner_margin(egui::Margin::same(16))
                .outer_margin(egui::Margin::symmetric(12, 4));

            card.show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                // Top row: name + abbrev + remove
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(&tz.display_name)
                            .size(18.0)
                            .strong()
                            .color(TEXT_PRIMARY),
                    );
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new(format!("{} / {}", tz.current_abbrev, tz.offset_display()))
                            .size(12.0)
                            .color(SUBTITLE),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if idx > 0 {
                            let rm_btn = egui::Button::new(
                                egui::RichText::new("Remove").size(11.0).color(REMOVE_COLOR),
                            )
                            .fill(egui::Color32::TRANSPARENT)
                            .stroke(egui::Stroke::new(1.0, REMOVE_COLOR))
                            .corner_radius(4.0);
                            if ui.add(rm_btn).clicked() {
                                remove_idx = Some(idx);
                            }
                        }
                    });
                });

                ui.add_space(8.0);

                // Time display + controls
                ui.horizontal(|ui| {
                    // Big time
                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}:{:02}:{:02}",
                            tz.hour, tz.min, tz.sec
                        ))
                        .size(44.0)
                        .monospace()
                        .color(TIME_COLOR),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut changed = false;

                        // Min controls
                        if adj_button(ui, "+").clicked() {
                            tz.min = wrap(tz.min + 1, 60);
                            changed = true;
                        }
                        ui.label(egui::RichText::new("MIN").size(10.0).color(SUBTITLE));
                        if adj_button(ui, "-").clicked() {
                            tz.min = wrap(tz.min - 1, 60);
                            changed = true;
                        }

                        ui.add_space(8.0);

                        // Hour controls
                        if adj_button(ui, "+").clicked() {
                            tz.hour = wrap(tz.hour + 1, 24);
                            changed = true;
                        }
                        ui.label(egui::RichText::new("HR").size(10.0).color(SUBTITLE));
                        if adj_button(ui, "-").clicked() {
                            tz.hour = wrap(tz.hour - 1, 24);
                            changed = true;
                        }

                        if changed {
                            changed_idx = Some(idx);
                        }
                    });
                });
            });
        }

        if let Some(idx) = changed_idx {
            self.synced_to_now = false;
            self.update_others_from(idx);
        }
        if let Some(idx) = remove_idx {
            self.clocks.remove(idx);
        }
    }

    fn render_add_page(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);

        // Header with back button
        ui.horizontal(|ui| {
            ui.add_space(16.0);
            let back_btn = egui::Button::new(
                egui::RichText::new("< Back").size(14.0).color(ACCENT),
            )
            .fill(egui::Color32::TRANSPARENT)
            .corner_radius(6.0);
            if ui.add(back_btn).clicked() {
                self.page = Page::Clock;
                if self.synced_to_now {
                    self.sync_to_now();
                } else if !self.clocks.is_empty() {
                    self.update_others_from(0);
                }
            }
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Add Timezone").size(22.0).strong().color(TEXT_PRIMARY));
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);

        // Search bar
        let search_frame = egui::Frame::new()
            .fill(CARD_BG)
            .corner_radius(8.0)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .outer_margin(egui::Margin::symmetric(12, 0));
        search_frame.show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Search:").size(14.0).color(SUBTITLE));
                ui.add_space(4.0);
                let te = egui::TextEdit::singleline(&mut self.add_search)
                    .desired_width(ui.available_width() - 8.0)
                    .font(egui::TextStyle::Body);
                ui.add(te);
            });
        });

        ui.add_space(12.0);

        // Preset list
        let search = self.add_search.to_lowercase();
        let mut added = false;

        ui.horizontal(|ui| {
            ui.add_space(16.0);
            ui.label(egui::RichText::new("Popular timezones").size(13.0).color(SUBTITLE));
        });
        ui.add_space(4.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            for (name, iana, region) in PRESETS {
                let haystack = format!("{} {} {}", name, iana, region).to_lowercase();
                if !search.is_empty() && !haystack.contains(&search) {
                    continue;
                }

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
                let current_time = format!("{:02}:{:02}", now.hour(), now.minute());

                let row_frame = egui::Frame::new()
                    .fill(CARD_BG)
                    .corner_radius(8.0)
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .outer_margin(egui::Margin::symmetric(12, 2));

                row_frame.show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(*name)
                                    .size(15.0)
                                    .strong()
                                    .color(TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new(format!("{} - {} / {}", region, abbr, offset_str))
                                    .size(11.0)
                                    .color(SUBTITLE),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let add_btn = egui::Button::new(
                                egui::RichText::new("Add").size(13.0).color(TEXT_PRIMARY),
                            )
                            .fill(ACCENT_DIM)
                            .corner_radius(6.0)
                            .min_size(egui::vec2(50.0, 28.0));
                            if ui.add(add_btn).clicked() {
                                self.clocks.push(TzClock::new(name, tz));
                                added = true;
                            }
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(current_time)
                                    .size(16.0)
                                    .monospace()
                                    .color(TIME_COLOR),
                            );
                        });
                    });
                });
            }

            // Custom IANA section
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.label(egui::RichText::new("Custom IANA timezone").size(13.0).color(SUBTITLE));
            });
            ui.add_space(4.0);

            let custom_frame = egui::Frame::new()
                .fill(CARD_BG)
                .corner_radius(8.0)
                .inner_margin(egui::Margin::symmetric(14, 12))
                .outer_margin(egui::Margin::symmetric(12, 2));
            custom_frame.show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Name:").size(13.0).color(SUBTITLE));
                    ui.add(egui::TextEdit::singleline(&mut self.custom_name).desired_width(120.0));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("IANA:").size(13.0).color(SUBTITLE));
                    ui.add(egui::TextEdit::singleline(&mut self.custom_iana).desired_width(160.0));
                    ui.add_space(8.0);
                    let add_btn = egui::Button::new(
                        egui::RichText::new("Add").size(13.0).color(TEXT_PRIMARY),
                    )
                    .fill(ACCENT_DIM)
                    .corner_radius(6.0)
                    .min_size(egui::vec2(50.0, 28.0));
                    if ui.add(add_btn).clicked() && !self.custom_name.is_empty() {
                        if let Ok(tz) = self.custom_iana.parse::<Tz>() {
                            self.clocks.push(TzClock::new(&self.custom_name.clone(), tz));
                            added = true;
                            self.custom_name.clear();
                            self.custom_iana.clear();
                        }
                    }
                });
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new("e.g. Europe/Rome, America/Toronto, Asia/Bangkok")
                        .size(11.0)
                        .color(SUBTITLE),
                );
            });
        });

        if added {
            self.page = Page::Clock;
            if self.synced_to_now {
                self.sync_to_now();
            } else if !self.clocks.is_empty() {
                self.update_others_from(0);
            }
        }
    }
}

fn adj_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    let btn = egui::Button::new(
        egui::RichText::new(label).size(16.0).monospace().strong().color(ACCENT),
    )
    .fill(BTN_BG)
    .corner_radius(6.0)
    .min_size(egui::vec2(32.0, 32.0));
    ui.add(btn)
}

fn wrap(val: i32, modulo: i32) -> i32 {
    ((val % modulo) + modulo) % modulo
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([520.0, 520.0])
            .with_min_inner_size([440.0, 350.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Timezone Clock",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            visuals.panel_fill = egui::Color32::from_rgb(38, 41, 50);
            visuals.window_fill = egui::Color32::from_rgb(38, 41, 50);
            cc.egui_ctx.set_visuals(visuals);
            Ok(Box::new(ClockApp::new()))
        }),
    )
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.synced_to_now && self.page == Page::Clock {
            self.sync_to_now();
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.page {
                    Page::Clock => self.render_clock_page(ui),
                    Page::AddTimezone => self.render_add_page(ui),
                }
            });
        });
    }
}
