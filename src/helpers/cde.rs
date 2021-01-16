use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
};

use crate::{error::*, RollHistory, RollResult};

enum Element {
    Fire([Outcome; 10]),
    Earth([Outcome; 10]),
    Metal([Outcome; 10]),
    Water([Outcome; 10]),
    Wood([Outcome; 10]),
}

enum Side {
    Yin,
    Yang,
}

enum Outcome {
    Success,
    Lucky,
    Ill,
    Loksyu(Side),
    TinJi,
}

// Result mapping, according to the French Starter Kit p. 26
const FIRE: [Outcome; 10] = [
    Outcome::TinJi,              // 1
    Outcome::Success,            // 2
    Outcome::Loksyu(Side::Yang), // 3
    Outcome::Ill,                // 4
    Outcome::Lucky,              // 5
    Outcome::TinJi,              // 6
    Outcome::Success,            // 7
    Outcome::Loksyu(Side::Yin),  // 8
    Outcome::Ill,                // 9
    Outcome::Lucky,              // 10
];

const EARTH: [Outcome; 10] = [
    Outcome::Loksyu(Side::Yang), // 1
    Outcome::Ill,                // 2
    Outcome::Lucky,              // 3
    Outcome::TinJi,              // 4
    Outcome::Success,            // 5
    Outcome::Loksyu(Side::Yin),  // 6
    Outcome::Ill,                // 7
    Outcome::Lucky,              // 8
    Outcome::TinJi,              // 9
    Outcome::Success,            // 10
];

const METAL: [Outcome; 10] = [
    Outcome::Lucky,              // 1
    Outcome::TinJi,              // 2
    Outcome::Success,            // 3
    Outcome::Loksyu(Side::Yin),  // 4
    Outcome::Ill,                // 5
    Outcome::Lucky,              // 6
    Outcome::TinJi,              // 7
    Outcome::Success,            // 8
    Outcome::Loksyu(Side::Yang), // 9
    Outcome::Ill,                // 10
];

const WATER: [Outcome; 10] = [
    Outcome::Success,            // 1
    Outcome::Loksyu(Side::Yin),  // 2
    Outcome::Ill,                // 3
    Outcome::Lucky,              // 4
    Outcome::TinJi,              // 5
    Outcome::Success,            // 6
    Outcome::Loksyu(Side::Yang), // 7
    Outcome::Ill,                // 8
    Outcome::Lucky,              // 9
    Outcome::TinJi,              // 10
];

const WOOD: [Outcome; 10] = [
    Outcome::Ill,                // 1
    Outcome::Lucky,              // 2
    Outcome::TinJi,              // 3
    Outcome::Success,            // 4
    Outcome::Loksyu(Side::Yang), // 5
    Outcome::Ill,                // 6
    Outcome::Lucky,              // 7
    Outcome::TinJi,              // 8
    Outcome::Success,            // 9
    Outcome::Loksyu(Side::Yin),  // 10
];

#[derive(Debug, Default)]
/// This struct represent the repartition of the dices according to an element
pub struct CdeResult {
    /// Number of dice that fall under the rolling element
    pub success: u32,
    /// Number of dice that fall under the element generated by the rolling element
    pub lucky: u32,
    /// Number of dice that fall under the element generating the rolling element
    pub ill: u32,
    /// Number of dice that fall under the element dominated by the rolling element
    /// Splitted by Yin/Yang
    pub loksyu: (u32, u32), // Yin, Yang
    /// Number of dice that fall under the element dominating the rolling elemeent
    pub tin_ji: u32,
    /// The history to have all the dice results so you can manually check the distribution
    pub history: Option<RollHistory>, // Option to make derive Default happy
    /// The element names to use when printing
    pub elements: [String; 5],
}

impl PartialEq for CdeResult {
    fn eq(&self, other: &Self) -> bool {
        self.success == other.success
            && self.lucky == other.lucky
            && self.ill == other.ill
            && self.loksyu == other.loksyu
            && self.tin_ji == other.tin_ji
    }
}

impl TryFrom<&str> for Element {
    type Error = &'static str;

    fn try_from(s: &str) -> std::result::Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "feu" | "fire" => Ok(Element::Fire(FIRE)),
            "earth" | "terre" => Ok(Element::Earth(EARTH)),
            "metal" | "métal" => Ok(Element::Metal(METAL)),
            "eau" | "water" => Ok(Element::Water(WATER)),
            "bois" | "wood" => Ok(Element::Wood(WOOD)),
            _ => Err("Element must be one of `fire`, `earth`, `metal`, `fire` or `wood"),
        }
    }
}

impl Display for CdeResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{}
Success ({}): {}
Lucky dice ({}): {}
Ill dice ({}): {}
Loksyu ({}): {} ● Yin / {} ○ Yang
Tin Ji ({}): {}
"#,
            self.history.as_ref().unwrap().to_string(),
            self.elements[0],
            self.success,
            self.elements[1],
            self.lucky,
            self.elements[2],
            self.ill,
            self.elements[3],
            self.loksyu.0,
            self.loksyu.1,
            self.elements[4],
            self.tin_ji
        )
    }
}

/// Interpret a [`RollResult`](crate::RollResult) according to the RPG
/// "Hong Kong : Chroniques de l'étrange"
pub fn compute_cde(res: &RollResult, element: &str) -> Result<CdeResult> {
    let history = res
        .as_single()
        .ok_or("Not a single roll result")?
        .get_history();
    if history.len() != 1 {
        Err("Should have only one roll".into())
    } else {
        let res = history
            .iter()
            .flat_map(|v| {
                if let RollHistory::Roll(dices_res) = v {
                    Some(dices_res)
                } else {
                    None
                }
            })
            .next()
            .ok_or("RollHistory must be a Roll variant")?
            .clone();

        let mapping: Element = element.try_into()?;
        let (mapping, elements) = match mapping {
            Element::Fire(m) => (
                m,
                [
                    "㊋ fire".to_string(),
                    "㊏ earth".to_string(),
                    "㊍ wood".to_string(),
                    "㊎ metal".to_string(),
                    "㊌ water".to_string(),
                ],
            ),
            Element::Earth(m) => (
                m,
                [
                    "㊏ earth".to_string(),
                    "㊎ metal".to_string(),
                    "㊋ fire".to_string(),
                    "㊌ water".to_string(),
                    "㊍ wood".to_string(),
                ],
            ),
            Element::Metal(m) => (
                m,
                [
                    "㊎ metal".to_string(),
                    "㊌ water".to_string(),
                    "㊏ earth".to_string(),
                    "㊍ wood".to_string(),
                    "㊋ fire".to_string(),
                ],
            ),
            Element::Water(m) => (
                m,
                [
                    "㊌ water".to_string(),
                    "㊍ wood".to_string(),
                    "㊎ metal".to_string(),
                    "㊋ fire".to_string(),
                    "㊏ earth".to_string(),
                ],
            ),
            Element::Wood(m) => (
                m,
                [
                    "㊍ wood".to_string(),
                    "㊋ fire".to_string(),
                    "㊌ water".to_string(),
                    "㊏ earth".to_string(),
                    "㊎ metal".to_string(),
                ],
            ),
        };

        let mut result = res.iter().fold(CdeResult::default(), |mut acc, v| {
            match &mapping[(v.res - 1) as usize] {
                Outcome::Success => acc.success += 1,
                Outcome::Lucky => acc.lucky += 1,
                Outcome::Ill => acc.ill += 1,
                Outcome::Loksyu(side) => match side {
                    Side::Yin => acc.loksyu.0 += 1,
                    Side::Yang => acc.loksyu.1 += 1,
                },
                Outcome::TinJi => acc.tin_ji += 1,
            }
            acc
        });

        result.history = history.get(0).cloned();
        result.elements = elements;
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{tests::IteratorDiceRollSource, Critic, DiceResult, Roller};

    #[test]
    fn test_cde() {
        let roll_mock = vec![1, 2, 3, 4, 5, 7, 10, 5];
        let r = Roller::new(&format!("{}d10", roll_mock.len())).unwrap();
        // fire
        let roll_res = r
            .roll_with_source(&mut IteratorDiceRollSource {
                iterator: &mut roll_mock.clone().into_iter(),
            })
            .unwrap();

        let res = compute_cde(&roll_res, "fire").unwrap();
        let expected = CdeResult {
            success: 2,
            lucky: 3,
            ill: 1,
            loksyu: (0, 1),
            tin_ji: 1,
            history: Some(RollHistory::Roll(
                roll_mock
                    .iter()
                    .map(|v| DiceResult {
                        res: *v,
                        crit: Critic::No,
                    })
                    .collect(),
            )),
            elements: Default::default(), // not used in comparison
        };

        assert_eq!(expected, res);
        println!("{}", res);

        // earth
        let roll_res = r
            .roll_with_source(&mut IteratorDiceRollSource {
                iterator: &mut roll_mock.clone().into_iter(),
            })
            .unwrap();

        let res = compute_cde(&roll_res, "earth").unwrap();
        let expected = CdeResult {
            success: 3,
            lucky: 1,
            ill: 2,
            loksyu: (0, 1),
            tin_ji: 1,
            history: Some(RollHistory::Roll(
                roll_mock
                    .iter()
                    .map(|v| DiceResult {
                        res: *v,
                        crit: Critic::No,
                    })
                    .collect(),
            )),
            elements: Default::default(), // not used in comparison
        };

        assert_eq!(expected, res);
        println!("{}", res);

        // metal
        let roll_res = r
            .roll_with_source(&mut IteratorDiceRollSource {
                iterator: &mut roll_mock.clone().into_iter(),
            })
            .unwrap();

        let res = compute_cde(&roll_res, "metal").unwrap();
        let expected = CdeResult {
            success: 1,
            lucky: 1,
            ill: 3,
            loksyu: (1, 0),
            tin_ji: 2,
            history: Some(RollHistory::Roll(
                roll_mock
                    .iter()
                    .map(|v| DiceResult {
                        res: *v,
                        crit: Critic::No,
                    })
                    .collect(),
            )),
            elements: Default::default(), // not used in comparison
        };

        assert_eq!(expected, res);
        println!("{}", res);

        // water
        let roll_res = r
            .roll_with_source(&mut IteratorDiceRollSource {
                iterator: &mut roll_mock.clone().into_iter(),
            })
            .unwrap();

        let res = compute_cde(&roll_res, "water").unwrap();
        let expected = CdeResult {
            success: 1,
            lucky: 1,
            ill: 1,
            loksyu: (1, 1),
            tin_ji: 3,
            history: Some(RollHistory::Roll(
                roll_mock
                    .iter()
                    .map(|v| DiceResult {
                        res: *v,
                        crit: Critic::No,
                    })
                    .collect(),
            )),
            elements: Default::default(), // not used in comparison
        };

        assert_eq!(expected, res);
        println!("{}", res);

        // wood
        let roll_res = r
            .roll_with_source(&mut IteratorDiceRollSource {
                iterator: &mut roll_mock.clone().into_iter(),
            })
            .unwrap();

        let res = compute_cde(&roll_res, "wood").unwrap();
        let expected = CdeResult {
            success: 1,
            lucky: 2,
            ill: 1,
            loksyu: (1, 2),
            tin_ji: 1,
            history: Some(RollHistory::Roll(
                roll_mock
                    .iter()
                    .map(|v| DiceResult {
                        res: *v,
                        crit: Critic::No,
                    })
                    .collect(),
            )),
            elements: Default::default(), // not used in comparison
        };

        assert_eq!(expected, res);
        println!("{}", res);
    }
}
