use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

//====================================================
// SheetActionRecord
//====================================================
#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct SheetActionRecord {
    pub date: NaiveDate,
    pub duration: u64,
    pub base_exp: f64,
    pub bonus_exp: f64,
}

impl Default for SheetActionRecord {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            date: now.naive_local().date(),
            duration: 0,
            base_exp: 0.0,
            bonus_exp: 0.0,
        }
    }
}

//====================================================
// Skill
//====================================================
#[derive(Deserialize, Serialize)]
pub struct Skill {
    pub name: String,
    pub records: Vec<SheetActionRecord>,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: "new skill".to_string(),
            records: Vec::new(),
        }
    }
}

impl Skill {
    pub fn sort_actions(&mut self) {
        self.records.sort_by(|a, b| a.date.cmp(&b.date));
    }

    pub fn calculate_exp(&mut self) {
        // This function assumes that all records are pre-sorted before arriving here. Otherwise
        // it will probably produce incorrect streak bonuses.

        let exp_per_hour: f64 = 55.0;
        let streak_max_daily_bonus: f64 = 0.5;
        let max_bonus_days: i64 = 5;
        let daily_degredation = streak_max_daily_bonus / max_bonus_days as f64;

        let mut streak_list: VecDeque<&mut SheetActionRecord> = VecDeque::new();
        self.records.iter_mut().for_each(|r| {
            r.base_exp = (r.duration as f64 / 60f64) * exp_per_hour;

            let date = &r.date;

            // This should drain dates that are too old.
            while let Some(back) = streak_list.pop_back() {
                let duration = date.signed_duration_since(back.date).num_days();
                if duration <= max_bonus_days {
                    streak_list.push_back(back);
                    break;
                }
            }

            // Now we go through all remaining items in the streak-day list, calculate their total,
            // multiply by the number of days' degredation, and then add to our running bonus exp.
            let mut running_bonus: f64 = 0f64;
            streak_list.iter().for_each(|s| {
                let num_days = date.signed_duration_since(s.date).num_days() as f64;
                let multiplier = streak_max_daily_bonus - (daily_degredation * num_days);
                let this_bonus = (s.base_exp + s.bonus_exp) * multiplier;
                running_bonus += this_bonus;
            });

            r.bonus_exp = running_bonus;

            streak_list.push_back(r);
        });
    }
}
