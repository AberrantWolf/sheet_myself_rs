use chrono::{Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::ops::Add;

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

    #[serde(skip)]
    pub potential_bonus: f64,
    #[serde(skip)]
    pub total_exp: f64,
}

impl Default for Skill {
    fn default() -> Self {
        Self {
            name: "new skill".to_string(),
            records: Vec::new(),
            potential_bonus: 0f64,
            total_exp: 0f64,
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

        let clear_old_streaks =
            |date: &NaiveDate, streak_list: &mut VecDeque<&mut SheetActionRecord>| {
                while let Some(back) = streak_list.pop_back() {
                    let duration = date.signed_duration_since(back.date).num_days();
                    if duration <= max_bonus_days {
                        streak_list.push_back(back);
                        break;
                    }
                }
            };

        let calc_streak_bonus =
            |date: &NaiveDate, streak_list: &VecDeque<&mut SheetActionRecord>| -> f64 {
                // Go through all remaining items in the streak-day list, calculate their total,
                // multiply by the number of days' degredation, and then add to our running bonus exp.
                let mut running_bonus: f64 = 0f64;
                streak_list.iter().for_each(|s| {
                    let num_days = date.signed_duration_since(s.date).num_days() as f64;
                    let multiplier = streak_max_daily_bonus - (daily_degredation * num_days);
                    let this_bonus = (s.base_exp + s.bonus_exp) * multiplier;
                    running_bonus += this_bonus;
                });
                running_bonus
            };

        let mut exp_total = 0f64;
        let mut streak_list: VecDeque<&mut SheetActionRecord> = VecDeque::new();
        self.records.iter_mut().for_each(|r| {
            r.base_exp = (r.duration as f64 / 60f64) * exp_per_hour;

            let date = &r.date;

            // This should drain dates that are too old.
            clear_old_streaks(&date, &mut streak_list);
            r.bonus_exp = calc_streak_bonus(&date, &streak_list);

            exp_total += r.base_exp + r.bonus_exp;

            streak_list.push_back(r);
        });
        self.total_exp = exp_total;

        // Try to calculate how much bonus to expect if you do the thing today (or tomorrow if
        // you already did it today)
        let today = Utc::now().naive_local().date();
        let next_day = if let Some(front) = streak_list.front() {
            if today.signed_duration_since(front.date).is_zero() {
                today.add(Duration::days(1))
            } else {
                today
            }
        } else {
            today
        };
        clear_old_streaks(&next_day, &mut streak_list);
        self.potential_bonus = calc_streak_bonus(&next_day, &streak_list);
    }
}
