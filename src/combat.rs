// Copyright (C) 2026 Mai Ooizumi
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::{BTreeMap, HashMap};

#[derive(Default)]
pub(crate) struct Combat {
    first_blood: bool,
    streak: HashMap<i32, usize>,
    headshot_streak: HashMap<i32, usize>,
    combo: HashMap<i32, (usize, f64)>,
    last_kill_tick: HashMap<i32, i32>,
    last_teamkill_tick: HashMap<i32, i32>,
    pending: BTreeMap<(i32, i32), Vec<String>>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Death {
    Enemy {
        killer: i32,
        weapon: String,
        headshot: bool,
    },
    Teamkill {
        killer: i32,
    },
    World,
    Suicide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CueKind {
    Other,
    Teamkill,
    Enemy,
}

pub(crate) fn classify(
    victim: i32,
    attacker: Option<i32>,
    same_team: bool,
    free_for_all: bool,
    weapon: &str,
    headshot: bool,
) -> Death {
    match attacker {
        None if weapon == "planted_c4" => Death::World,
        None => Death::Suicide,
        Some(killer) if killer == victim => Death::Suicide,
        Some(killer) if same_team && !free_for_all => Death::Teamkill { killer },
        Some(killer) => Death::Enemy {
            killer,
            weapon: weapon.strip_prefix("weapon_").unwrap_or(weapon).into(),
            headshot,
        },
    }
}

impl Combat {
    pub fn new_round(&mut self) {
        self.first_blood = false;
        self.headshot_streak.clear();
        self.combo.clear();
        self.last_kill_tick.clear();
        self.last_teamkill_tick.clear();
        self.pending.clear();
        // Streak is per life, not per team or half.
    }

    pub fn disconnect(&mut self, slot: i32) {
        self.streak.remove(&slot);
        self.headshot_streak.remove(&slot);
        self.combo.remove(&slot);
        self.last_kill_tick.remove(&slot);
        self.last_teamkill_tick.remove(&slot);
    }

    // Returns the actor, and the ordered configured-cue candidates for this death.
    pub fn death(
        &mut self,
        victim: i32,
        death: Death,
        tick: i32,
        game_time: f64,
        combo_window: f64,
        shutdown_min: usize,
    ) -> (i32, Vec<String>, CueKind) {
        let previous = self.streak.remove(&victim).unwrap_or_default();
        self.headshot_streak.remove(&victim);
        self.combo.remove(&victim);
        match death {
            Death::World => (victim, Vec::new(), CueKind::Other),
            Death::Suicide => (victim, vec!["suicide".into()], CueKind::Other),
            Death::Teamkill { killer } => (
                killer,
                if self.last_teamkill_tick.get(&killer) == Some(&tick) {
                    Vec::new()
                } else {
                    vec!["teamkill".into()]
                },
                CueKind::Teamkill,
            ),
            Death::Enemy {
                killer,
                weapon,
                headshot,
            } => {
                let streak = self.streak.entry(killer).or_default();
                *streak += 1;
                let streak = *streak;
                let hs_streak = if headshot {
                    let entry = self.headshot_streak.entry(killer).or_default();
                    *entry += 1;
                    *entry
                } else {
                    0
                };
                let combo_count = if combo_window > 0.0 && game_time.is_finite() {
                    let entry = self.combo.entry(killer).or_insert((0, game_time));
                    if entry.0 > 0 && game_time >= entry.1 && (game_time - entry.1) <= combo_window
                    {
                        entry.0 += 1;
                    } else {
                        entry.0 = 1;
                    }
                    entry.1 = game_time;
                    entry.0
                } else {
                    0
                };
                if !self.not_emitted(killer, tick) {
                    return (killer, Vec::new(), CueKind::Enemy);
                }
                let first = !std::mem::replace(&mut self.first_blood, true);
                let mut cues = Vec::with_capacity(8);
                if first {
                    cues.push("first_blood".into());
                }
                if combo_count >= 2 {
                    cues.push(format!("combo.{combo_count}"));
                }
                cues.push(format!("streak.{streak}"));
                if previous >= shutdown_min {
                    cues.push("streak_shutdown".into());
                }
                if !weapon.is_empty() {
                    cues.push(format!("weapons.{weapon}"));
                }
                if headshot {
                    cues.push(format!("headshot_streak.{hs_streak}"));
                    cues.push("headshot".into());
                }
                cues.push("kill".into());
                (killer, cues, CueKind::Enemy)
            }
        }
    }

    fn not_emitted(&self, slot: i32, tick: i32) -> bool {
        self.last_kill_tick.get(&slot) != Some(&tick)
    }

    pub fn emitted(&mut self, slot: i32, tick: i32, enemy_kill: bool) {
        if enemy_kill {
            self.last_kill_tick.insert(slot, tick);
        }
    }

    pub fn emitted_teamkill(&mut self, slot: i32, tick: i32) {
        self.last_teamkill_tick.insert(slot, tick);
    }

    pub fn queue_death_cues(&mut self, slot: i32, tick: i32, cues: Vec<String>) {
        let pending = self.pending.entry((slot, tick)).or_default();
        pending.extend(cues);
        pending.sort_by_key(|cue| cue_priority(cue));
        pending.dedup();
    }

    pub fn take_pending_before(&mut self, tick: i32) -> Vec<(i32, i32, Vec<String>)> {
        let ready: Vec<_> = self
            .pending
            .keys()
            .filter(|(_, pending_tick)| *pending_tick < tick)
            .copied()
            .collect();
        ready
            .into_iter()
            .filter_map(|key| self.pending.remove(&key).map(|cues| (key.0, key.1, cues)))
            .collect()
    }
}

#[derive(PartialEq, Eq, Hash)]
pub(crate) enum Dimension<'a> {
    FirstBlood,
    Combo,
    Streak,
    StreakShutdown,
    Weapon(&'a str),
    Headshot,
    Kill,
    Teamkill,
    Other(&'a str),
}

pub(crate) fn cue_dimension(cue: &str) -> Dimension<'_> {
    if cue == "first_blood" {
        Dimension::FirstBlood
    } else if cue.starts_with("combo.") {
        Dimension::Combo
    } else if cue.starts_with("streak.") {
        Dimension::Streak
    } else if cue == "streak_shutdown" {
        Dimension::StreakShutdown
    } else if let Some(weapon) = cue.strip_prefix("weapons.") {
        Dimension::Weapon(weapon)
    } else if cue.starts_with("headshot_streak.") || cue == "headshot" {
        Dimension::Headshot
    } else if cue == "kill" {
        Dimension::Kill
    } else if cue == "teamkill" {
        Dimension::Teamkill
    } else {
        Dimension::Other(cue)
    }
}

fn cue_priority(cue: &str) -> (u8, usize) {
    if cue == "first_blood" {
        (0, 0)
    } else if let Some(combo) = cue
        .strip_prefix("combo.")
        .and_then(|n| n.parse::<usize>().ok())
    {
        (1, usize::MAX - combo)
    } else if let Some(streak) = cue
        .strip_prefix("streak.")
        .and_then(|n| n.parse::<usize>().ok())
    {
        (2, usize::MAX - streak)
    } else if cue == "streak_shutdown" {
        (3, 0)
    } else if cue.starts_with("weapons.") {
        (4, 0)
    } else if let Some(hs) = cue
        .strip_prefix("headshot_streak.")
        .and_then(|n| n.parse::<usize>().ok())
    {
        (5, usize::MAX - hs)
    } else if cue == "headshot" {
        (6, 0)
    } else if cue == "kill" {
        (7, 0)
    } else if cue == "teamkill" {
        (9, 0)
    } else {
        (8, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enemy(killer: i32, weapon: &str, headshot: bool) -> Death {
        Death::Enemy {
            killer,
            weapon: weapon.into(),
            headshot,
        }
    }

    #[test]
    fn same_tick_multikill_is_silent_but_counts_and_death_enables_shutdown() {
        let mut c = Combat::default();
        assert_eq!(
            c.death(2, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3).1,
            ["first_blood", "streak.1", "weapons.hegrenade", "kill"]
        );
        c.emitted(1, 41, true);
        assert!(
            c.death(3, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3)
                .1
                .is_empty()
        );
        assert_eq!(
            c.death(4, enemy(1, "new_weapon", true), 42, 10.5, 2.0, 3).1,
            [
                "combo.3",
                "streak.3",
                "weapons.new_weapon",
                "headshot_streak.1",
                "headshot",
                "kill"
            ]
        );
        c.new_round(); // A team-side change cannot erase the per-life streak.
        assert_eq!(
            c.death(1, enemy(5, "ak47", false), 43, 15.0, 2.0, 3).1,
            [
                "first_blood",
                "streak.1",
                "streak_shutdown",
                "weapons.ak47",
                "kill"
            ]
        );
        assert_eq!(
            c.death(6, enemy(1, "ak47", false), 44, 15.5, 2.0, 3).1[0],
            "streak.1"
        );
    }

    #[test]
    fn ffa_and_self_or_world_death_are_classified_without_eating_first_blood() {
        assert_eq!(
            classify(1, None, false, true, "planted_c4", false),
            Death::World
        );
        assert_eq!(
            classify(1, None, false, true, "world", false),
            Death::Suicide
        );
        assert_eq!(
            classify(1, Some(1), true, true, "hegrenade", false),
            Death::Suicide
        );
        let mut c = Combat::default();
        let tk = classify(1, Some(2), true, false, "ak47", false);
        assert_eq!(c.death(1, tk, 0, 1.0, 2.0, 3).1, ["teamkill"]);
        assert_eq!(c.death(1, Death::Suicide, 1, 2.0, 2.0, 3).1, ["suicide"]);
        let ffa = classify(1, Some(2), true, true, "weapon_ak47", false);
        assert_eq!(
            c.death(1, ffa, 2, 3.0, 2.0, 3).1,
            ["first_blood", "streak.1", "weapons.ak47", "kill"]
        );
    }

    #[test]
    fn same_tick_teamkill_does_not_consume_first_blood() {
        let mut c = Combat::default();
        let teamkill = classify(2, Some(1), true, false, "ak47", false);
        assert_eq!(c.death(2, teamkill, 41, 10.0, 2.0, 3).1, ["teamkill"]);
        c.emitted(1, 41, false);
        assert_eq!(
            c.death(3, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3).1,
            ["first_blood", "streak.1", "weapons.hegrenade", "kill"]
        );
    }

    #[test]
    fn same_tick_enemy_batch_keeps_first_blood_and_ranks_combo_then_streak() {
        let mut c = Combat::default();
        let first = c.death(2, enemy(1, "ak47", false), 41, 10.0, 2.0, 3);
        c.queue_death_cues(first.0, 41, first.1);
        let second = c.death(3, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3);
        c.queue_death_cues(second.0, 41, second.1);

        let batches = c.take_pending_before(42);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].0, 1);
        assert_eq!(batches[0].1, 41);
        assert_eq!(batches[0].2[0], "first_blood");
        assert_eq!(batches[0].2[1], "combo.2");
        assert_eq!(batches[0].2[2], "streak.2");
        assert!(batches[0].2.contains(&"streak.1".into()));
    }

    #[test]
    fn same_tick_double_kill_after_first_blood_selects_combo_before_streak() {
        let mut c = Combat::default();
        c.death(9, enemy(8, "ak47", false), 40, 5.0, 2.0, 3);
        for victim in [2, 3] {
            let (actor, cues, _) = c.death(victim, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3);
            c.queue_death_cues(actor, 41, cues);
        }
        let (_, _, cues) = c.take_pending_before(42).pop().unwrap();
        assert_eq!(&cues[..3], ["combo.2", "streak.2", "streak.1"]);
    }

    #[test]
    fn same_tick_teamkill_joins_enemy_batch_as_lowest_priority_and_keeps_enemy_quota_free() {
        let mut c = Combat::default();
        let teamkill = c.death(2, Death::Teamkill { killer: 1 }, 41, 10.0, 2.0, 3);
        c.queue_death_cues(teamkill.0, 41, teamkill.1);
        let enemy_batch = c.death(3, enemy(1, "hegrenade", false), 41, 10.0, 2.0, 3);
        c.queue_death_cues(enemy_batch.0, 41, enemy_batch.1);

        let batch = c.take_pending_before(42).pop().unwrap();
        assert_eq!(batch.2[0], "first_blood");
        assert_eq!(batch.2.last().unwrap(), "teamkill");

        c.emitted_teamkill(1, 42);
        assert_eq!(
            c.death(4, enemy(1, "ak47", false), 42, 10.5, 2.0, 3).2,
            CueKind::Enemy
        );
    }

    #[test]
    fn uncredited_world_death_does_not_play_suicide_or_consume_first_blood() {
        let mut c = Combat::default();
        assert!(
            c.death(
                2,
                classify(2, None, false, false, "planted_c4", false),
                1,
                1.0,
                2.0,
                3
            )
            .1
            .is_empty()
        );
        assert_eq!(
            c.death(3, enemy(1, "ak47", false), 2, 2.0, 2.0, 3).1[0],
            "first_blood"
        );
    }

    #[test]
    fn combo_expires_after_window_and_headshot_streak_resets_on_new_round_and_death() {
        let mut c = Combat::default();
        let (_, k1, _) = c.death(2, enemy(1, "ak47", true), 10, 10.0, 2.0, 3);
        assert!(k1.contains(&"headshot_streak.1".into()));
        assert!(!k1.iter().any(|x| x.starts_with("combo.")));

        // Bodyshot within window advances combo to 2, keeps headshot_streak at 1.
        let (_, k2, _) = c.death(3, enemy(1, "ak47", false), 11, 11.5, 2.0, 3);
        assert!(k2.contains(&"combo.2".into()));
        assert!(!k2.iter().any(|x| x.starts_with("headshot")));

        // Headshot after combo window (> 2.0s) resets combo to 1, headshot_streak becomes 2.
        let (_, k3, _) = c.death(4, enemy(1, "ak47", true), 12, 14.0, 2.0, 3);
        assert!(!k3.iter().any(|x| x.starts_with("combo.")));
        assert!(k3.contains(&"headshot_streak.2".into()));

        // New round resets headshot_streak & combo, keeps streak (now 3 -> 4).
        c.new_round();
        let (_, k4, _) = c.death(5, enemy(1, "ak47", true), 13, 14.5, 2.0, 3);
        assert!(k4.contains(&"first_blood".into()));
        assert!(k4.contains(&"streak.4".into()));
        assert!(k4.contains(&"headshot_streak.1".into()));
        assert!(!k4.iter().any(|x| x.starts_with("combo.")));
    }
}
