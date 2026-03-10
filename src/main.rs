use chrono::{FixedOffset, Timelike, Utc};
use eframe::egui;

const AEDT_OFFSET: i32 = 11 * 3600; // UTC+11
const VNT_OFFSET: i32 = 7 * 3600;   // UTC+7
const HOUR_DIFF: i32 = 4;            // AEDT is 4 hours ahead of VNT

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([420.0, 300.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "AEDT / VNT Clock",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            Ok(Box::new(ClockApp::new()))
        }),
    )
}

struct ClockApp {
    aedt_hour: i32,
    aedt_min: i32,
    aedt_sec: i32,
    vnt_hour: i32,
    vnt_min: i32,
    vnt_sec: i32,
    synced_to_now: bool,
}

impl ClockApp {
    fn new() -> Self {
        let mut app = Self {
            aedt_hour: 0,
            aedt_min: 0,
            aedt_sec: 0,
            vnt_hour: 0,
            vnt_min: 0,
            vnt_sec: 0,
            synced_to_now: true,
        };
        app.sync_to_now();
        app
    }

    fn sync_to_now(&mut self) {
        let now = Utc::now();
        let aedt = now.with_timezone(&FixedOffset::east_opt(AEDT_OFFSET).unwrap());
        let vnt = now.with_timezone(&FixedOffset::east_opt(VNT_OFFSET).unwrap());
        self.aedt_hour = aedt.hour() as i32;
        self.aedt_min = aedt.minute() as i32;
        self.aedt_sec = aedt.second() as i32;
        self.vnt_hour = vnt.hour() as i32;
        self.vnt_min = vnt.minute() as i32;
        self.vnt_sec = vnt.second() as i32;
        self.synced_to_now = true;
    }

    fn update_vnt_from_aedt(&mut self) {
        let total_mins = self.aedt_hour * 60 + self.aedt_min - HOUR_DIFF * 60;
        let wrapped = ((total_mins % 1440) + 1440) % 1440;
        self.vnt_hour = wrapped / 60;
        self.vnt_min = wrapped % 60;
        self.vnt_sec = self.aedt_sec;
    }

    fn update_aedt_from_vnt(&mut self) {
        let total_mins = self.vnt_hour * 60 + self.vnt_min + HOUR_DIFF * 60;
        let wrapped = ((total_mins % 1440) + 1440) % 1440;
        self.aedt_hour = wrapped / 60;
        self.aedt_min = wrapped % 60;
        self.aedt_sec = self.vnt_sec;
    }
}

impl eframe::App for ClockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.synced_to_now {
            self.sync_to_now();
            ctx.request_repaint_after(std::time::Duration::from_secs(1));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("AEDT / VNT Clock");
                ui.add_space(10.0);

                if ui
                    .button(if self.synced_to_now {
                        "Live (synced to now)"
                    } else {
                        "Reset to current time"
                    })
                    .clicked()
                {
                    self.sync_to_now();
                }
                ui.add_space(15.0);

                // --- AEDT ---
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("AEDT (Australia - UTC+11)")
                            .strong()
                            .size(14.0),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Hour:");
                        let r = ui.add(
                            egui::DragValue::new(&mut self.aedt_hour)
                                .range(0..=23)
                                .speed(0.1),
                        );
                        ui.label("Min:");
                        let r2 = ui.add(
                            egui::DragValue::new(&mut self.aedt_min)
                                .range(0..=59)
                                .speed(0.1),
                        );
                        if r.changed() || r2.changed() {
                            self.synced_to_now = false;
                            self.update_vnt_from_aedt();
                        }
                    });
                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}:{:02}:{:02}",
                            self.aedt_hour, self.aedt_min, self.aedt_sec
                        ))
                        .size(36.0)
                        .monospace(),
                    );
                });

                ui.add_space(10.0);

                // --- VNT ---
                ui.group(|ui| {
                    ui.label(
                        egui::RichText::new("VNT (Vietnam - UTC+7)")
                            .strong()
                            .size(14.0),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Hour:");
                        let r = ui.add(
                            egui::DragValue::new(&mut self.vnt_hour)
                                .range(0..=23)
                                .speed(0.1),
                        );
                        ui.label("Min:");
                        let r2 = ui.add(
                            egui::DragValue::new(&mut self.vnt_min)
                                .range(0..=59)
                                .speed(0.1),
                        );
                        if r.changed() || r2.changed() {
                            self.synced_to_now = false;
                            self.update_aedt_from_vnt();
                        }
                    });
                    ui.label(
                        egui::RichText::new(format!(
                            "{:02}:{:02}:{:02}",
                            self.vnt_hour, self.vnt_min, self.vnt_sec
                        ))
                        .size(36.0)
                        .monospace(),
                    );
                });
            });
        });
    }
}
