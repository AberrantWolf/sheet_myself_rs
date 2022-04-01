use chrono::{Duration, NaiveDate, Utc};
use eframe::egui::RichText;
use eframe::{egui, epi};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// A stored entry for time spend doing a Thing
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct SheetActionRecord {
    date: NaiveDate,
    duration: u64,

    #[serde(skip_serializing)]
    editing: bool,
}

impl Default for SheetActionRecord {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            date: now.naive_local().date(),
            duration: 0,
            editing: false,
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct SheetMyselfApp {
    player_name: String,
    skills_list: HashMap<String, Vec<SheetActionRecord>>,

    // this how you opt-out of serialization of a member
    #[cfg_attr(feature = "persistence", serde(skip))]
    value: f32,
}

impl Default for SheetMyselfApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            player_name: "New Player Name".to_owned(),
            skills_list: HashMap::<String, Vec<SheetActionRecord>>::new(),
            value: 2.7,
        }
    }
}

impl epi::App for SheetMyselfApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        let Self {
            player_name,
            skills_list,
            value,
        } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        // Info bar at the bottom...?
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            egui::warn_if_debug_build(ui);
        });

        egui::SidePanel::left("section_chooser").show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.button("Skills");
            });
        });

        egui::TopBottomPanel::top("player_info_top").show(ctx, |ui| {
            ui.label(RichText::new(player_name.clone()).heading());
            // TODO: Add a button to edit the player's name... when you hover over the label...?
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: have a variable affected by the section selector to view various info pages here

            // The central panel the region left after adding TopPanel's and SidePanel's
            skills_list.iter().for_each(|(skill_name, records)| {
                ui.collapsing(RichText::new(skill_name.clone()).heading(), |ui| {
                    for rec in records {
                        ui.horizontal(|ui| {
                            ui.label(rec.date.format("%Y %m %d").to_string());
                        });
                    }
                    // TODO: add a dummy row that if you click on it'll auto add a new item
                    ui.label("Click to add entry...");
                });
            });

            if ui.button("New Skill").clicked() {
                skills_list.insert("new skill".into(), Vec::new());
            }
        });
    }

    /// Called once before the first frame.
    fn setup(
        &mut self,
        _ctx: &egui::Context,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }
    }

    /// Called by the framework to save state before shutdown.
    /// Note that you must enable the `persistence` feature for this to work.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Sheet Myself"
    }
}
