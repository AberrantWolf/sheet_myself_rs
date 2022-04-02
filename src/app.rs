use chrono::{Datelike, NaiveDate, Utc};
use eframe::{egui, epi};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// A stored entry for time spend doing a Thing
#[derive(Deserialize, Serialize)]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct SheetActionRecord {
    date: NaiveDate,
    duration: u64,
}

impl Default for SheetActionRecord {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            date: now.naive_local().date(),
            duration: 0,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Skill {
    name: String,
    records: Vec<SheetActionRecord>,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: "new skill".to_string(),
            records: Vec::new(),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state
pub struct SheetMyselfApp {
    player_name: String,
    skills_list: HashMap<Uuid, Skill>,
    // this how you opt-out of serialization of a member
    // #[cfg_attr(feature = "persistence", serde(skip))]
}

impl Default for SheetMyselfApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            player_name: "New Player Name".to_owned(),
            skills_list: HashMap::<Uuid, Skill>::new(),
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
                if ui.button("Skills").clicked() {
                    // TODO: swap to skills page if we're not there...
                }
            });
        });

        egui::TopBottomPanel::top("player_info_top").show(ctx, |ui| {
            ui.text_edit_singleline(player_name);
            // TODO: Add a button to edit the player's name... when you hover over the label...?
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // TODO: have a variable affected by the section selector to view various info pages here

            // The central panel the region left after adding TopPanel's and SidePanel's
            skills_list.iter_mut().for_each(|(skill_id, skill)| {
                let Skill { name, records } = skill;
                let collapse_id = ui.make_persistent_id(skill_id);

                let mut expanded =
                    if let Some(b) = ui.memory().data.get_persisted::<bool>(collapse_id) {
                        b
                    } else {
                        true
                    };

                let expand_text = match expanded {
                    true => " v ",
                    false => " > ",
                };

                ui.horizontal_top(|ui| {
                    if ui.button(expand_text).clicked() {
                        expanded = !expanded;
                        ui.memory().data.insert_persisted(collapse_id, expanded);
                    }
                    ui.text_edit_singleline(name);
                });
                if expanded {
                    ui.indent(collapse_id, |ui| {
                        egui::Grid::new("entry_grid").show(ui, |ui| {
                            // TODO: Add little arrow buttons to sort by year/month/day/etc
                            ui.label("Year");
                            ui.label("Month");
                            ui.label("Day");
                            ui.label("Duration");
                            ui.end_row();

                            let mut idx = 0;
                            while idx < records.len() {
                                let mut rec = &mut records[idx];
                                let mut year = rec.date.year().to_string();
                                let mut month = rec.date.month().to_string();
                                let mut day = rec.date.day().to_string();
                                let mut duration = rec.duration.to_string();

                                if ui.text_edit_singleline(&mut year).changed() {
                                    if let Ok(i) = year.parse::<i32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_year(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                };
                                if ui.text_edit_singleline(&mut month).changed() {
                                    if let Ok(i) = month.parse::<u32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_month(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                };
                                if ui.text_edit_singleline(&mut day).changed() {
                                    if let Ok(i) = day.parse::<u32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_day(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                };

                                if ui.text_edit_singleline(&mut duration).changed() {
                                    if let Ok(i) = duration.parse::<u64>() {
                                        rec.duration = i;
                                    }
                                }

                                if ui.button(" - ").clicked() {
                                    records.remove(idx);
                                } else {
                                    idx += 1;
                                }

                                ui.end_row();
                            }
                        });

                        if ui.button("Add entry...").clicked() {
                            records.push(SheetActionRecord::default());
                        }
                    });
                }
            });

            if ui.button("New Skill").clicked() {
                skills_list.insert(Uuid::new_v4(), Skill::default());
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
