//! The report as text for the terminal. It names original functions and counts, never the
//! original's expressions.

use super::report::{Finding, FunctionReport, Report};
use super::scoped::ScopedFunction;

const LISTED_FUNCTIONS: usize = 25;
/// Names an original function whose statements to list one by one.
const DETAIL_VARIABLE: &str = "OPENTBOI_PRIMITIVES_FUNCTION";

pub fn print(report: &Report, scoped: &[ScopedFunction]) {
    let total = |finding| {
        report
            .functions
            .iter()
            .map(|function| function.count(finding))
            .sum::<usize>()
    };
    let judged: usize = report.functions.iter().map(FunctionReport::judged).sum();
    println!("{} original functions (and frame scripts)", report.functions.len());
    println!("statements with something telling in them: {judged}");
    for (finding, label) in [
        (Finding::Exact, "whole computation found in our code, numbers equal"),
        (Finding::Part, "a telling part found, numbers equal"),
        (
            Finding::SameStructure,
            "same structure, other numbers (a parameter or unit)",
        ),
        (
            Finding::Ambiguous,
            "found, but in too many of our functions to say anything",
        ),
        (Finding::NotFound, "not found"),
        (
            Finding::Nothing,
            "nothing telling (names and common numbers only), not judged",
        ),
    ] {
        println!("  {:>5}  {label}", total(finding));
    }
    if let Ok(name) = std::env::var(DETAIL_VARIABLE) {
        print_details(report, &name);
        print_scoped_details(scoped, &name);
    }
    print_best_matches(report);
    print_complete_ports(report);
    print_scoped(scoped);
}

/// Every original function with a counterpart: how much of it the counterpart reproduces.
fn print_scoped(scoped: &[ScopedFunction]) {
    let mut ranked: Vec<&ScopedFunction> = scoped.iter().filter(|function| function.judged() > 0).collect();
    ranked.sort_by(|first, second| {
        let share = |function: &ScopedFunction| function.covered() as f64 / function.judged() as f64;
        share(second)
            .total_cmp(&share(first))
            .then(second.judged().cmp(&first.judged()))
    });
    println!("\noriginal functions judged against their counterparts (named in our doc comments, or the best fit):");
    println!("  function                 covered  (equal + other numbers)  part  missing  counterparts");
    for function in ranked {
        let counterparts: Vec<String> = function
            .counterparts
            .iter()
            .map(|site| format!("{}::{}", site.file.display(), site.function))
            .collect();
        println!(
            "  {:<24} {:>3} / {:<3} ({:>3} + {:<3})              {:>4}  {:>7}  {}",
            function.name,
            function.covered(),
            function.judged(),
            function.count(Finding::Exact),
            function.count(Finding::SameStructure),
            function.count(Finding::Part),
            function.count(Finding::NotFound),
            counterparts.join(", ")
        );
    }
}

fn print_scoped_details(scoped: &[ScopedFunction], name: &str) {
    for function in scoped.iter().filter(|function| function.name == name) {
        println!("\n{name} against its counterparts: statement by statement");
        for (finding, shapes) in &function.details {
            println!("  {finding:?}: {shapes}");
        }
    }
}

fn print_best_matches(report: &Report) {
    let mut ranked: Vec<&FunctionReport> = report
        .functions
        .iter()
        .filter(|function| function.found() > 0)
        .collect();
    ranked.sort_by_key(|function| std::cmp::Reverse(function.found()));
    println!("\nfunctions with the most statements found (found / judged, then our best fit):");
    for function in ranked.into_iter().take(LISTED_FUNCTIONS) {
        let fit = function.best_site().map_or(String::new(), |(site, count)| {
            format!("{} in {}::{}", count, site.file.display(), site.function)
        });
        println!(
            "  {:<24} {:>3} / {:<3}  {fit}",
            function.name,
            function.found(),
            function.judged()
        );
    }
}

fn print_complete_ports(report: &Report) {
    println!("\noriginal functions that one of our functions reproduces nearly completely, and what else it holds:");
    for (function, site, count) in report.complete_ports() {
        let (extra, total) = report.extras(site);
        println!(
            "  {:<24} {:>3} of {:<3} in {}::{}; telling expressions there with no counterpart: {extra} of {total}",
            function.name,
            count,
            function.judged(),
            site.file.display(),
            site.function
        );
    }
}

fn print_details(report: &Report, name: &str) {
    for function in report.functions.iter().filter(|function| function.name == name) {
        println!("\n{name}: statement by statement");
        for (finding, shapes) in &function.details {
            if *finding != Finding::Nothing {
                println!("  {finding:?}: {shapes}");
            }
        }
    }
}
