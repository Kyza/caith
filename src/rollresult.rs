use std::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::parser::TotalModifier;

/// In a RollResult history, we either have a vector of the roll, or a separator between different
/// dices. Ex: `1d6 + 1d6`, we will have a RollHistory::Roll followed by RollHistory::Separator and
/// another RollHistory::Roll
#[derive(Debug, Clone)]
pub enum RollHistory {
    Roll(Vec<u64>),
    Fudge(Vec<u64>),
    Value(u64),
    Separator(&'static str),
}

/// Carry the result of the roll and an history of the steps taken
#[derive(Debug, Clone)]
pub struct RollResult {
    /// Result of the roll. In the case of option `t` and/or `f` used, it's the number of `success -
    /// failure`
    total: i64,
    /// History of the steps taken that lead to this result.
    history: Vec<RollHistory>,
    /// Any provided comment will be available here, without the starting `!`.
    reason: Option<String>,
    /// Internal usage field to avoid computing a total if it's already done.
    dirty: bool,
}

impl RollResult {
    /// Create an empty `RollResult`
    pub(crate) fn new() -> Self {
        Self {
            total: 0,
            history: Vec::new(),
            reason: None,
            dirty: true,
        }
    }

    /// Create a `RollResult` with already a total. Used to carry constant value
    pub(crate) fn with_total(total: i64) -> Self {
        Self {
            total,
            history: vec![RollHistory::Value(total as u64)],
            reason: None,
            dirty: false,
        }
    }

    /// Add a comment to the result
    pub fn add_reason(&mut self, reason: String) {
        self.reason = Some(reason);
    }

    /// Get the comment, if any
    pub fn get_reason(&self) -> Option<&String> {
        self.reason.as_ref()
    }

    /// Get the history of the result
    pub fn get_history(&self) -> &Vec<RollHistory> {
        &self.history
    }

    /// Add a step in the history
    pub(crate) fn add_history(&mut self, mut history: Vec<u64>, is_fudge: bool) {
        self.dirty = true;
        history.sort_unstable_by(|a, b| b.cmp(a));
        self.history.push(if is_fudge {
            RollHistory::Fudge(history)
        } else {
            RollHistory::Roll(history)
        })
    }

    /// Compute the total value according to some modifier
    pub(crate) fn compute_total(&mut self, modifier: TotalModifier) -> i64 {
        if self.dirty {
            self.dirty = false;
            let mut flat = self.history.iter().fold(Vec::new(), |mut acc, h| {
                match h {
                    RollHistory::Roll(r) | RollHistory::Fudge(r) => {
                        let mut c = r.clone();
                        acc.append(&mut c);
                    }
                    RollHistory::Value(v) => acc.push(*v),
                    RollHistory::Separator(_) => (),
                };
                acc
            });
            flat.sort_unstable();
            let flat = flat;
            // theres's no check on bounds as `compute_total` is called after `compute_option` where
            // the check is done
            let slice = match modifier {
                TotalModifier::KeepHi(n) => &flat[flat.len() - n..],
                TotalModifier::KeepLo(n) => &flat[..n],
                TotalModifier::DropHi(n) => &flat[..flat.len() - n],
                TotalModifier::DropLo(n) => &flat[n..],
                TotalModifier::None | TotalModifier::TargetFailure(_, _) | TotalModifier::Fudge => {
                    flat.as_slice()
                }
            };

            self.total = match modifier {
                TotalModifier::TargetFailure(t, f) => slice.iter().fold(0, |acc, x| {
                    if *x >= t {
                        acc + 1
                    } else if *x <= f {
                        acc - 1
                    } else {
                        acc
                    }
                }),
                TotalModifier::Fudge => slice.iter().fold(0, |acc, x| {
                    if *x <= 2 {
                        acc - 1
                    } else if *x <= 4 {
                        acc
                    } else {
                        acc + 1
                    }
                }),
                _ => slice.iter().sum::<u64>() as i64,
            };
        }

        self.total
    }

    /// Get the result value
    pub fn get_total(&self) -> i64 {
        self.total
    }
}

impl Add for RollResult {
    type Output = Self;

    fn add(mut self, mut rhs: Self) -> Self::Output {
        if !rhs.history.is_empty() {
            self.history.push(RollHistory::Separator("+"));
        }
        self.history.append(&mut rhs.history);
        RollResult {
            total: self.total + rhs.total,
            history: self.history,
            reason: self.reason,
            dirty: false,
        }
    }
}

impl Sub for RollResult {
    type Output = Self;

    fn sub(mut self, mut rhs: Self) -> Self::Output {
        if !rhs.history.is_empty() {
            self.history.push(RollHistory::Separator("-"));
        }
        self.history.append(&mut rhs.history);
        RollResult {
            total: self.total - rhs.total,
            history: self.history,
            reason: self.reason,
            dirty: false,
        }
    }
}

impl Mul for RollResult {
    type Output = Self;

    fn mul(mut self, mut rhs: Self) -> Self::Output {
        if !rhs.history.is_empty() {
            self.history.push(RollHistory::Separator("*"));
        }
        self.history.append(&mut rhs.history);
        RollResult {
            total: self.total * rhs.total,
            history: self.history,
            reason: self.reason,
            dirty: false,
        }
    }
}

impl Div for RollResult {
    type Output = Self;

    fn div(mut self, mut rhs: Self) -> Self::Output {
        if !rhs.history.is_empty() {
            self.history.push(RollHistory::Separator("/"));
        }
        self.history.append(&mut rhs.history);
        RollResult {
            total: self.total / rhs.total,
            history: self.history,
            reason: self.reason,
            dirty: false,
        }
    }
}

impl Display for RollResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`")?;
        if self.history.is_empty() {
            write!(f, "{}", self.total)?;
        } else {
            self.history
                .iter()
                .try_for_each::<_, std::fmt::Result>(|v| {
                    match v {
                        RollHistory::Roll(v) => {
                            write!(f, "[")?;
                            let len = v.len();
                            v.iter().enumerate().try_for_each(|(i, r)| {
                                if i == len - 1 {
                                    write!(f, "{}", r)
                                } else {
                                    write!(f, "{}, ", r)
                                }
                            })?;
                            write!(f, "]")?;
                        }
                        RollHistory::Fudge(v) => {
                            write!(f, "[")?;
                            let len = v.len();
                            v.iter().enumerate().try_for_each(|(i, r)| {
                                let r = if *r <= 2 {
                                    "-"
                                } else if *r <= 4 {
                                    "▢"
                                } else {
                                    "+"
                                };
                                if i == len - 1 {
                                    write!(f, "{}", r)
                                } else {
                                    write!(f, "{}, ", r)
                                }
                            })?;
                            write!(f, "]")?;
                        }
                        RollHistory::Value(v) => write!(f, "{}", v)?,
                        RollHistory::Separator(s) => write!(f, " {} ", s)?,
                    }

                    Ok(())
                })?;
        }

        write!(f, "` Result: **{}**", self.get_total())?;

        if let Some(reason) = &self.reason {
            write!(f, ", Reason: `{}`", reason)?;
        }
        Ok(())
    }
}
