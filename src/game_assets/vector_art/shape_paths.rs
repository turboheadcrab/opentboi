//! Rebuilds Flash's edge-based shape records into closed fill contours and stroke runs.
//!
//! Flash stores each edge once, tagged with the fill on its left (`fill_style_0`) and on its right
//! (`fill_style_1`). Every fill is recovered by collecting its edges (left-side edges reversed so all
//! run the same way) and chaining them end to start into closed contours.

use std::collections::HashMap;

use swf::{FillStyle, LineStyle, Shape, ShapeRecord, ShapeStyles};

/// A point in twips, kept as integers so chained endpoints compare exactly.
pub type TwipsPoint = (i32, i32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge {
    pub start: TwipsPoint,
    /// Present for quadratic curves.
    pub control: Option<TwipsPoint>,
    pub end: TwipsPoint,
}

impl Edge {
    fn reversed(self) -> Self {
        Self {
            start: self.end,
            control: self.control,
            end: self.start,
        }
    }
}

/// Connected edges; closed when the last edge ends where the first starts.
pub type EdgeRun = Vec<Edge>;

/// Everything drawn with one style table, in paint order: fills first, then strokes.
pub struct StyleGroup<'shape> {
    pub fills: Vec<(&'shape FillStyle, Vec<EdgeRun>)>,
    pub strokes: Vec<(&'shape LineStyle, Vec<EdgeRun>)>,
}

/// Edges sorted by the style (1-based index) they belong to, for one style table.
struct PendingGroup<'shape> {
    styles: &'shape ShapeStyles,
    fill_edges: HashMap<u32, Vec<Edge>>,
    line_edges: HashMap<u32, Vec<Edge>>,
}

impl<'shape> PendingGroup<'shape> {
    fn new(styles: &'shape ShapeStyles) -> Self {
        Self {
            styles,
            fill_edges: HashMap::new(),
            line_edges: HashMap::new(),
        }
    }

    fn finish(self) -> StyleGroup<'shape> {
        let mut fill_indices: Vec<_> = self.fill_edges.keys().copied().collect();
        fill_indices.sort_unstable();
        let mut line_indices: Vec<_> = self.line_edges.keys().copied().collect();
        line_indices.sort_unstable();
        StyleGroup {
            fills: fill_indices
                .into_iter()
                .filter_map(|index| {
                    let style = self.styles.fill_styles.get(index as usize - 1)?;
                    Some((style, chain_into_closed_contours(&self.fill_edges[&index])))
                })
                .collect(),
            strokes: line_indices
                .into_iter()
                .filter_map(|index| {
                    let style = self.styles.line_styles.get(index as usize - 1)?;
                    Some((style, split_into_connected_runs(&self.line_edges[&index])))
                })
                .collect(),
        }
    }
}

pub fn style_groups(shape: &Shape) -> Vec<StyleGroup<'_>> {
    let mut finished_groups = Vec::new();
    let mut group = PendingGroup::new(&shape.styles);
    let (mut left_fill, mut right_fill, mut line) = (0, 0, 0);
    let mut pen: TwipsPoint = (0, 0);

    for record in &shape.shape {
        let edge = match record {
            ShapeRecord::StyleChange(change) => {
                if let Some(new_styles) = &change.new_styles {
                    finished_groups.push(std::mem::replace(&mut group, PendingGroup::new(new_styles)).finish());
                }
                if let Some(position) = change.move_to {
                    pen = (position.x.get(), position.y.get());
                }
                left_fill = change.fill_style_0.unwrap_or(left_fill);
                right_fill = change.fill_style_1.unwrap_or(right_fill);
                line = change.line_style.unwrap_or(line);
                continue;
            }
            edge_record => edge_from(edge_record, pen),
        };
        pen = edge.end;
        if right_fill != 0 {
            group.fill_edges.entry(right_fill).or_default().push(edge);
        }
        if left_fill != 0 {
            group.fill_edges.entry(left_fill).or_default().push(edge.reversed());
        }
        if line != 0 {
            group.line_edges.entry(line).or_default().push(edge);
        }
    }
    finished_groups.push(group.finish());
    finished_groups
}

/// The edge an edge record draws from `pen`; style changes are handled by the caller.
fn edge_from(record: &ShapeRecord, pen: TwipsPoint) -> Edge {
    match record {
        ShapeRecord::StraightEdge { delta } => Edge {
            start: pen,
            control: None,
            end: (pen.0 + delta.dx.get(), pen.1 + delta.dy.get()),
        },
        ShapeRecord::CurvedEdge {
            control_delta,
            anchor_delta,
        } => {
            let control = (pen.0 + control_delta.dx.get(), pen.1 + control_delta.dy.get());
            Edge {
                start: pen,
                control: Some(control),
                end: (control.0 + anchor_delta.dx.get(), control.1 + anchor_delta.dy.get()),
            }
        }
        ShapeRecord::StyleChange(_) => Edge {
            start: pen,
            control: None,
            end: pen,
        },
    }
}

/// Joins edges end to start until each contour returns to its beginning.
fn chain_into_closed_contours(edges: &[Edge]) -> Vec<EdgeRun> {
    let mut unused_by_start: HashMap<TwipsPoint, Vec<usize>> = HashMap::new();
    for (index, edge) in edges.iter().enumerate().rev() {
        unused_by_start.entry(edge.start).or_default().push(index);
    }
    let mut is_used = vec![false; edges.len()];
    let mut contours = Vec::new();
    for first_index in 0..edges.len() {
        if is_used[first_index] {
            continue;
        }
        let mut contour = Vec::new();
        let mut current_index = first_index;
        loop {
            is_used[current_index] = true;
            contour.push(edges[current_index]);
            let end = edges[current_index].end;
            if end == edges[first_index].start {
                break;
            }
            let next_index = unused_by_start
                .get_mut(&end)
                .and_then(|candidates| take_first_unused(candidates, &is_used));
            match next_index {
                Some(next_index) => current_index = next_index,
                None => break,
            }
        }
        contours.push(contour);
    }
    contours
}

fn take_first_unused(candidates: &mut Vec<usize>, is_used: &[bool]) -> Option<usize> {
    while let Some(candidate) = candidates.pop() {
        if !is_used[candidate] {
            return Some(candidate);
        }
    }
    None
}

/// Strokes keep their drawing order; a new run starts wherever the pen jumped.
fn split_into_connected_runs(edges: &[Edge]) -> Vec<EdgeRun> {
    let mut runs: Vec<EdgeRun> = Vec::new();
    for &edge in edges {
        match runs.last_mut() {
            Some(run) if run.last().is_some_and(|previous| previous.end == edge.start) => run.push(edge),
            _ => runs.push(vec![edge]),
        }
    }
    runs
}
