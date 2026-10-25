//! Each original function judged against its counterparts in our code: the functions whose doc
//! comment names it (`` `slotf()` ``) and the one that holds most of its telling computations.
//! Inside that narrow scope even a one-operator comparison or a single number is evidence, so short
//! functions made of plain tests (`x < 30`) can be judged too, which the whole-code search cannot
//! do. Tests count whichever way round they are written, and numbers that differ (a unit or a
//! parameter we changed) still count as the same structure.

use std::collections::{HashMap, HashSet};

use super::original_side::roots_of;
use super::report::{Finding, Report};
use super::rust_side::{RustShapes, Site};
use super::{Numbers, Shape};
use crate::game_assets::script::{GameScript, Statement};

/// Our function holding most of an original function's telling computations is taken as its
/// counterpart only when it holds at least this many, and this share of them: a few shared shapes
/// (random rolls are everywhere) are coincidence.
const MINIMUM_BEST_FIT_STATEMENTS: usize = 3;
const MINIMUM_BEST_FIT_SHARE: f64 = 0.3;

pub struct ScopedFunction {
    pub name: String,
    pub counterparts: Vec<Site>,
    counts: HashMap<Finding, usize>,
    /// Each judged statement's finding and the shapes of what it computes (local reading only).
    pub details: Vec<(Finding, String)>,
}

impl ScopedFunction {
    pub fn count(&self, finding: Finding) -> usize {
        self.counts.get(&finding).copied().unwrap_or(0)
    }

    pub fn judged(&self) -> usize {
        [Finding::Exact, Finding::SameStructure, Finding::Part, Finding::NotFound]
            .map(|finding| self.count(finding))
            .iter()
            .sum()
    }

    /// Statements whose whole computation is in the counterparts, with equal or other numbers.
    pub fn covered(&self) -> usize {
        self.count(Finding::Exact) + self.count(Finding::SameStructure)
    }
}

/// What the counterparts of one original function compute, as test keys.
#[derive(Default)]
struct Scope {
    exact: HashSet<String>,
    /// A structure key, and the exact keys of our shapes that have it.
    structure: HashMap<String, HashSet<String>>,
    /// Exact keys of every computation of the original function: a shape of ours that equals one of
    /// them is that statement's counterpart, not another statement's with other numbers.
    original: HashSet<String>,
}

impl Scope {
    fn of_counterparts(rust: &RustShapes, counterparts: &[Site]) -> Self {
        let mut scope = Self::default();
        for (shape, site) in &rust.found {
            if !counterparts.contains(site) {
                continue;
            }
            for part in shape.parts() {
                let exact = part.test_key(Numbers::Exact);
                scope.exact.insert(exact.clone());
                let structure = match part.operator_count() {
                    0 => continue,
                    1 => part.test_key(Numbers::Scaled),
                    _ => part.test_key(Numbers::Hidden),
                };
                scope.structure.entry(structure).or_default().insert(exact);
            }
        }
        scope
    }

    /// How a computation of the original turns up here; `Nothing` if it says nothing (only
    /// names, or one operation between names).
    fn judge(&self, root: &Shape) -> Finding {
        let operators = root.operator_count();
        if operators == 0 {
            // A lone telling number (a result like 10) or a sound's name.
            let telling = root.is_sound_name() || matches!(root, Shape::Number(_)) && root.telling_number_count() > 0;
            return if !telling {
                Finding::Nothing
            } else if self.exact.contains(&root.test_key(Numbers::Exact)) {
                Finding::Exact
            } else {
                Finding::NotFound
            };
        }
        if !root.has_number() && operators < 2 {
            return Finding::Nothing;
        }
        // One operation may differ from ours only in the units of its number; a longer one in any
        // of its numbers.
        let structure = if operators == 1 {
            Numbers::Scaled
        } else {
            Numbers::Hidden
        };
        if self.exact.contains(&root.test_key(Numbers::Exact)) {
            Finding::Exact
        } else if self
            .structure
            .get(&root.test_key(structure))
            .is_some_and(|ours| ours.iter().any(|exact| !self.original.contains(exact)))
        {
            Finding::SameStructure
        } else if root
            .parts()
            .into_iter()
            .skip(1)
            .filter(|part| part.operator_count() > 0 && part.telling_number_count() > 0)
            .any(|part| self.exact.contains(&part.test_key(Numbers::Exact)))
        {
            Finding::Part
        } else {
            Finding::NotFound
        }
    }
}

/// Every original function that has a counterpart, judged statement by statement.
pub fn judge(script: &GameScript, rust: &RustShapes, report: &Report) -> Vec<ScopedFunction> {
    let mut cited: HashMap<&str, Vec<Site>> = HashMap::new();
    for (name, site) in &rust.citations {
        let sites = cited.entry(name.as_str()).or_default();
        if !sites.contains(site) {
            sites.push(site.clone());
        }
    }
    script
        .functions
        .iter()
        .filter_map(|function| {
            let mut counterparts = cited.get(function.name.as_str()).cloned().unwrap_or_default();
            let best_fit = report
                .functions
                .iter()
                .find(|judged| judged.name == function.name)
                .and_then(|judged| {
                    let (site, count) = judged.best_site()?;
                    let share = count as f64 / judged.judged().max(1) as f64;
                    (count >= MINIMUM_BEST_FIT_STATEMENTS && share >= MINIMUM_BEST_FIT_SHARE).then(|| site.clone())
                });
            counterparts.extend(best_fit.filter(|site| !counterparts.contains(site)));
            (!counterparts.is_empty()).then(|| judge_function(&function.name, &function.statements, rust, counterparts))
        })
        .collect()
}

fn judge_function(name: &str, statements: &[Statement], rust: &RustShapes, counterparts: Vec<Site>) -> ScopedFunction {
    let mut scope = Scope::of_counterparts(rust, &counterparts);
    scope.original = statements
        .iter()
        .flat_map(roots_of)
        .map(|root| root.test_key(Numbers::Exact))
        .collect();
    let mut function = ScopedFunction {
        name: name.to_owned(),
        counterparts,
        counts: HashMap::new(),
        details: Vec::new(),
    };
    for statement in statements {
        let roots = roots_of(statement);
        let finding = roots
            .iter()
            .map(|root| scope.judge(root))
            .min()
            .unwrap_or(Finding::Nothing);
        *function.counts.entry(finding).or_default() += 1;
        let shapes: Vec<String> = roots.iter().map(|root| root.render(true)).collect();
        function.details.push((finding, shapes.join(" ; ")));
    }
    function
}

#[cfg(test)]
mod tests;
