use crate::skill::{SheetActionRecord, Skill};
use chrono::{Datelike, Utc};
use eframe::{egui, epi};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use uuid::Uuid;

fn get_default_file_path() -> Box<Path> {
    Path::new("myself.sht").into()
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SheetMyselfApp {
    player_name: String,
    skills_list: HashMap<Uuid, Skill>,
    // this how you opt-out of serialization of a member
    // #[cfg_attr(feature = "persistence", serde(skip))]
}

impl SheetMyselfApp {
    fn save_json(&self) {
        if let Ok(json_data) = serde_json::to_string(&self) {
            let path = get_default_file_path();
            if let Ok(mut file) = File::create(path) {
                file.write_all(json_data.as_bytes());
            }
        }
    }

    fn reload_from_json(&mut self) {
        let other = Self::from_default_path();

        self.player_name = other.player_name;
        self.skills_list = other.skills_list;
    }

    pub fn from_path(path: &Path) -> Self {
        if path.exists() {
            if let Ok(file) = File::open(path) {
                let reader = BufReader::new(file);
                if let Ok(app_data) = serde_json::from_reader(reader) {
                    return app_data;
                }
            }
        }

        Self::default()
    }

    pub fn from_default_path() -> Self {
        let path = get_default_file_path();
        Self::from_path(&path)
    }
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
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Reload").clicked() {
                        self.reload_from_json();
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        self.save_json();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        let Self {
            player_name,
            skills_list,
        } = self;

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
            // The central panel the region left after adding TopPanel's and SidePanel's
            skills_list.iter_mut().for_each(|(skill_id, skill)| {
                // TODO: The sorting is WAY too aggressive -- it sorts any time a value changes
                // and additionally your cursor stays in the same physical place even though the
                // row you were editing has shifted.
                //
                // I think I'll need to add some UUIDs or something to each action struct so that
                // the UI can track which one you were editing and make sure you're always scrolled
                // to it if nothing else.
                //
                // Alternatively, I could implement an editor for rows and then only change and sort
                // and recalculate when the editor closes with "accept" rather than "cancel"?
                //
                // Alter-alternatively, go through and look for a focus lost but none gained across
                // all the text edit fields?
                let mut need_sort = false;
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
                            ui.label("EXP");
                            ui.label("(from streak)");
                            ui.end_row();

                            let mut idx = 0;
                            while idx < records.len() {
                                let mut rec = &mut records[idx];
                                let mut year = rec.date.year().to_string();
                                let mut month = rec.date.month().to_string();
                                let mut day = rec.date.day().to_string();
                                let mut duration = rec.duration.to_string();

                                let year_field = ui.text_edit_singleline(&mut year);
                                let month_field = ui.text_edit_singleline(&mut month);
                                let day_field = ui.text_edit_singleline(&mut day);
                                let duration_field = ui.text_edit_singleline(&mut duration);

                                let total_exp = rec.base_exp + rec.bonus_exp;
                                ui.label(total_exp.to_string());
                                ui.label(format!("({})", rec.bonus_exp));

                                if year_field.changed() {
                                    if let Ok(i) = year.parse::<i32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_year(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                }
                                if month_field.changed() {
                                    if let Ok(i) = month.parse::<u32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_month(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                }
                                if day_field.changed() {
                                    if let Ok(i) = day.parse::<u32>() {
                                        rec.date = if let Some(new_rec) = rec.date.with_day(i) {
                                            new_rec
                                        } else {
                                            rec.date
                                        };
                                    }
                                }
                                if duration_field.changed() {
                                    if let Ok(i) = duration.parse::<u64>() {
                                        rec.duration = i;
                                    }
                                }

                                // Hack to prevent the UI from sorting while you're editing fields
                                // This should execute when you press enter, click outside the
                                // fields, or tab away from the fields in this record.
                                if !year_field.has_focus()
                                    && !month_field.has_focus()
                                    && !day_field.has_focus()
                                    && !duration_field.has_focus()
                                {
                                    if year_field.lost_focus()
                                        || month_field.lost_focus()
                                        || day_field.lost_focus()
                                        || duration_field.lost_focus()
                                    {
                                        need_sort = true;
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

                if need_sort {
                    skill.sort_actions();
                    skill.calculate_exp();
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
