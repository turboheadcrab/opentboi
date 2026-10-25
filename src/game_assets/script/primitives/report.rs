//! Looks every statement of the original up in our code and sums the result up per function.

use std::collections::{HashMap, HashSet};

use super::Shape;
use super::original_side::roots_of;
use super::rust_side::{RustShapes, Site};
use crate::game_assets::script::{GameScript, Statement};

/// A function needs this many statements with something telling in them to be judged at all.
const MINIMUM_JUDGED_STATEMENTS: usize = 3;
/// Share of its judged statements that our best-fitting function must reproduce for the original
/// function to count as fully implemented.
const COMPLETE_SHARE: f64 = 0.75;
/// Only shapes with this many operators are looked up with their numbers left out.
const MINIMUM_OPERATORS_FOR_STRUCTURE_ONLY: usize = 3;
/// A shape found in more of our functions than this is too common to say where a statement went.
const MAXIMUM_FUNCTIONS_PER_SHAPE: usize = 3;
const FRAME_SCRIPTS: &str = "(frame scripts)";

/// How a statement's computation turned up in our code, best result first.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Finding {
    /// Its whole computation, numbers included.
    Exact,
    /// A telling part of it, numbers included.
    Part,
    /// The whole computation with other numbers (a parameter, another unit).
    SameStructure,
    /// Found, but in so many of our functions that it proves nothing.
    Ambiguous,
    NotFound,
    /// Nothing in it says anything (only names and common numbers).
    Nothing,
}

pub struct FunctionReport {
    pub name: String,
    pub findings: HashMap<Finding, usize>,
    /// Our functions that reproduce its statements, with how many each.
    pub sites: HashMap<Site, usize>,
    /// For looking at one function: each statement's finding and the shapes of what it computes.
    pub details: Vec<(Finding, String)>,
}

impl FunctionReport {
    pub fn count(&self, finding: Finding) -> usize {
        self.findings.get(&finding).copied().unwrap_or(0)
    }

    pub fn judged(&self) -> usize {
        self.findings
            .iter()
            .filter(|(finding, _)| **finding != Finding::Nothing)
            .map(|(_, count)| count)
            .sum()
    }

    pub fn found(&self) -> usize {
        self.count(Finding::Exact) + self.count(Finding::Part)
    }

    /// Our function that reproduces most of it.
    pub fn best_site(&self) -> Option<(&Site, usize)> {
        self.sites
            .iter()
            .max_by_key(|&(site, count)| (*count, site.function.clone()))
            .map(|(site, count)| (site, *count))
    }
}

pub struct Report {
    pub functions: Vec<FunctionReport>,
    exact: HashMap<String, Vec<Site>>,
    structure: HashMap<String, Vec<Site>>,
    /// Every shape in the original, for telling what in our code has no counterpart.
    original_exact: HashSet<String>,
    original_structure: HashSet<String>,
    rust: Vec<(Shape, Site)>,
}

impl Report {
    pub fn build(script: &GameScript, rust: &RustShapes) -> Self {
        let mut report = Self {
            functions: Vec::new(),
            exact: HashMap::new(),
            structure: HashMap::new(),
            original_exact: HashSet::new(),
            original_structure: HashSet::new(),
            rust: Vec::new(),
        };
        for (shape, site) in &rust.found {
            report.exact.entry(shape.render(true)).or_default().push(site.clone());
            if shape.operator_count() >= MINIMUM_OPERATORS_FOR_STRUCTURE_ONLY {
                report
                    .structure
                    .entry(shape.render(false))
                    .or_default()
                    .push(site.clone());
            }
        }
        report.rust = rust.found.clone();
        let frame = (FRAME_SCRIPTS, script.frame_statements.as_slice());
        let named = script
            .functions
            .iter()
            .map(|function| (function.name.as_str(), function.statements.as_slice()));
        for (name, statements) in std::iter::once(frame).chain(named) {
            let function = report.judge_function(name, statements);
            report.functions.push(function);
        }
        report
    }

    fn judge_function(&mut self, name: &str, statements: &[Statement]) -> FunctionReport {
        let mut function = FunctionReport {
            name: name.to_owned(),
            findings: HashMap::new(),
            sites: HashMap::new(),
            details: Vec::new(),
        };
        for statement in statements {
            let roots = roots_of(statement);
            for part in roots.iter().flat_map(Shape::parts) {
                self.original_exact.insert(part.render(true));
                if part.operator_count() >= MINIMUM_OPERATORS_FOR_STRUCTURE_ONLY {
                    self.original_structure.insert(part.render(false));
                }
            }
            let (finding, sites) = self.judge_statement(&roots);
            let shapes: Vec<String> = roots.iter().map(|root| root.render(true)).collect();
            function.details.push((finding, shapes.join(" ; ")));
            *function.findings.entry(finding).or_default() += 1;
            for site in sites {
                *function.sites.entry(site).or_default() += 1;
            }
        }
        function
    }

    fn judge_statement(&self, roots: &[Shape]) -> (Finding, HashSet<Site>) {
        let mut best = Finding::Nothing;
        let mut sites = HashSet::new();
        for root in roots {
            let mut candidates = Vec::new();
            if root.is_telling() {
                candidates.push((Finding::Exact, self.exact.get(&root.render(true))));
                candidates.push((Finding::SameStructure, self.structure.get(&root.render(false))));
            }
            for part in root.parts().into_iter().skip(1).filter(|part| part.is_telling()) {
                candidates.push((Finding::Part, self.exact.get(&part.render(true))));
            }
            let telling = root.parts().iter().any(|part| part.is_telling());
            let mut finding = if telling { Finding::NotFound } else { Finding::Nothing };
            for (candidate, found) in candidates {
                let Some(found) = found.filter(|found| !found.is_empty()) else {
                    continue;
                };
                let functions: HashSet<&Site> = found.iter().collect();
                if functions.len() > MAXIMUM_FUNCTIONS_PER_SHAPE {
                    finding = finding.min(Finding::Ambiguous);
                    continue;
                }
                finding = finding.min(candidate);
                if matches!(candidate, Finding::Exact | Finding::Part) {
                    sites.extend(functions.into_iter().cloned());
                }
            }
            best = best.min(finding);
        }
        (best, sites)
    }

    /// Telling expressions of our function `site` that match nothing in the original: what we added
    /// or computed differently. Returns (those, all telling expressions in it).
    pub fn extras(&self, site: &Site) -> (usize, usize) {
        let mut seen = HashSet::new();
        let mut extra = 0;
        let mut total = 0;
        for (shape, found_at) in &self.rust {
            if found_at != site || !shape.is_telling() || !seen.insert(shape.render(true)) {
                continue;
            }
            total += 1;
            let known = self.original_exact.contains(&shape.render(true))
                || (shape.operator_count() >= MINIMUM_OPERATORS_FOR_STRUCTURE_ONLY
                    && self.original_structure.contains(&shape.render(false)));
            extra += usize::from(!known);
        }
        (extra, total)
    }

    /// Original functions that one of our functions reproduces (nearly) completely.
    pub fn complete_ports(&self) -> Vec<(&FunctionReport, &Site, usize)> {
        self.functions
            .iter()
            .filter(|function| function.judged() >= MINIMUM_JUDGED_STATEMENTS)
            .filter_map(|function| {
                let (site, count) = function.best_site()?;
                (count as f64 >= COMPLETE_SHARE * function.judged() as f64).then_some((function, site, count))
            })
            .collect()
    }
}
