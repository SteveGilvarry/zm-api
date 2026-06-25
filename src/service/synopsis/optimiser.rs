//! Temporal optimiser for motion synopsis (P2).
//!
//! Condensation works by **time-shifting** object tubes along the synopsis
//! timeline — never moving them spatially (an object must stay where it was).
//! Tubes that originally played at different times are slid to play together,
//! minimising synopsis length while keeping spatial collisions within a budget.
//!
//! The canonical synopsis energy is
//! `E = Σ activity + Σ (α·temporal_consistency + β·collision)`. Rather than expose
//! raw weights we expose a single **condensation** knob — a *collision-area
//! budget* (default ~5% of tube pixels) — and pack greedily against it: tubes are
//! placed in original-start order at the earliest slot whose added overlap keeps
//! the running collision under budget, extending the synopsis only when forced.
//!
//! **Interactions are preserved:** tubes that co-occur and are spatially close
//! (a hand-off, two people talking) are grouped and shifted as one rigid unit, so
//! the optimiser never splits them across different synopsis times.

use crate::service::zmnext::detail::{Tube, TubeSample};

/// Tunable parameters for the greedy optimiser. Defaults match the spec's
/// starting parameters.
#[derive(Debug, Clone)]
pub struct OptimiserParams {
    /// Synopsis timeline resolution (frames per second).
    pub fps: u32,
    /// Collision-area budget as a fraction of total tube pixels (2–10%).
    pub collision_budget: f32,
    /// Cap on tubes shown in one synopsis frame (0 = unlimited).
    pub max_tubes_per_frame: usize,
    /// Group two tubes when their temporal overlap exceeds this fraction…
    pub interaction_overlap: f32,
    /// …and their closest-box gap is under `distance_factor × object width`.
    pub distance_factor: f32,
    /// Target-length condensation knob: when set, the synopsis is capped at this
    /// many microseconds and groups that would only fit by extending past it are
    /// dropped (and reported) — the "too crowded → fast-forward" fallback. `None`
    /// lets the synopsis grow as long as packing needs.
    pub max_length_us: Option<i64>,
}

impl Default for OptimiserParams {
    fn default() -> Self {
        Self {
            fps: 12,
            collision_budget: 0.05,
            max_tubes_per_frame: 8,
            interaction_overlap: 0.75,
            distance_factor: 2.0,
            max_length_us: None,
        }
    }
}

/// Where one tube ends up on the synopsis timeline.
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
pub struct TubePlacement {
    pub track_id: i64,
    /// Interaction group this tube belongs to (members share a `start_shift_us`).
    pub group_id: usize,
    /// Microseconds added to every original timestamp of the tube to move it onto
    /// the synopsis timeline (can be negative — tubes usually shift earlier).
    pub start_shift_us: i64,
}

/// The optimiser's output: per-tube shifts plus the resulting length and any
/// tubes dropped because the frame cap was hit (never silently — logged).
#[derive(Debug, Clone, PartialEq, serde::Serialize, utoipa::ToSchema)]
pub struct SynopsisLayout {
    pub placements: Vec<TubePlacement>,
    /// Resulting synopsis length, microseconds.
    pub length_us: i64,
    /// Track ids dropped (too crowded). Surfaced so callers can log/report.
    pub dropped: Vec<i64>,
}

// --- geometry helpers -------------------------------------------------------

/// Area of a `[x, y, w, h]` bbox (negative dims clamp to 0).
fn bbox_area(b: [i32; 4]) -> i64 {
    (b[2].max(0) as i64) * (b[3].max(0) as i64)
}

/// Intersection area of two `[x, y, w, h]` bboxes.
fn intersection_area(a: [i32; 4], b: [i32; 4]) -> i64 {
    let ax2 = a[0] as i64 + a[2].max(0) as i64;
    let ay2 = a[1] as i64 + a[3].max(0) as i64;
    let bx2 = b[0] as i64 + b[2].max(0) as i64;
    let by2 = b[1] as i64 + b[3].max(0) as i64;
    let ix = (ax2.min(bx2) - (a[0] as i64).max(b[0] as i64)).max(0);
    let iy = (ay2.min(by2) - (a[1] as i64).max(b[1] as i64)).max(0);
    ix * iy
}

/// Gap (0 if overlapping) between two bboxes, in pixels (Chebyshev-ish on axes).
fn bbox_gap(a: [i32; 4], b: [i32; 4]) -> i64 {
    let ax2 = a[0] as i64 + a[2].max(0) as i64;
    let ay2 = a[1] as i64 + a[3].max(0) as i64;
    let bx2 = b[0] as i64 + b[2].max(0) as i64;
    let by2 = b[1] as i64 + b[3].max(0) as i64;
    let dx = ((a[0] as i64) - bx2).max((b[0] as i64) - ax2).max(0);
    let dy = ((a[1] as i64) - by2).max((b[1] as i64) - ay2).max(0);
    dx.max(dy)
}

// --- per-frame tube sampling ------------------------------------------------

/// A tube discretised onto the synopsis frame grid: its bbox at each frame from
/// its start, plus class/track metadata.
#[derive(Debug, Clone)]
struct FrameTube {
    track_id: i64,
    t_start_us: i64,
    /// bbox per frame, index 0 = tube start.
    boxes: Vec<[i32; 4]>,
}

impl FrameTube {
    fn frames(&self) -> usize {
        self.boxes.len()
    }
    fn pixel_area(&self) -> i64 {
        self.boxes.iter().copied().map(bbox_area).sum()
    }
}

/// Interpolate a tube's bbox onto `fps` frames spanning `[t_start, t_end]`.
fn interpolate(tube: &Tube, fps: u32) -> FrameTube {
    let fps = fps.max(1) as i64;
    let uspf = 1_000_000 / fps;
    // Span from the tube's declared bounds, falling back to its samples.
    let (t0, t1) = tube_span(tube);
    let span = (t1 - t0).max(0);
    let n = (span / uspf) as usize + 1;

    let mut boxes = Vec::with_capacity(n);
    for f in 0..n {
        let t = t0 + f as i64 * uspf;
        boxes.push(sample_bbox_at(&tube.samples, t));
    }
    FrameTube {
        track_id: tube.track_id,
        t_start_us: t0,
        boxes,
    }
}

/// The tube's time span: its declared `t_start_us/t_end_us`, falling back to the
/// min/max sample pts when those are unset (0).
fn tube_span(tube: &Tube) -> (i64, i64) {
    if tube.t_start_us != 0 || tube.t_end_us != 0 {
        return (tube.t_start_us, tube.t_end_us.max(tube.t_start_us));
    }
    let mut lo = i64::MAX;
    let mut hi = i64::MIN;
    for s in &tube.samples {
        lo = lo.min(s.pts_us);
        hi = hi.max(s.pts_us);
    }
    if lo == i64::MAX {
        (0, 0)
    } else {
        (lo, hi)
    }
}

/// Nearest-time bbox from a tube's samples for media-clock time `t`. Linear
/// interpolation between the bracketing samples; clamps at the ends.
fn sample_bbox_at(samples: &[TubeSample], t: i64) -> [i32; 4] {
    if samples.is_empty() {
        return [0, 0, 0, 0];
    }
    if t <= samples[0].pts_us {
        return samples[0].bbox;
    }
    if t >= samples[samples.len() - 1].pts_us {
        return samples[samples.len() - 1].bbox;
    }
    for w in samples.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if t >= a.pts_us && t <= b.pts_us {
            let denom = (b.pts_us - a.pts_us).max(1);
            let frac = (t - a.pts_us) as f64 / denom as f64;
            let mut out = [0i32; 4];
            for (o, (av, bv)) in out.iter_mut().zip(a.bbox.iter().zip(b.bbox.iter())) {
                *o = (*av as f64 + frac * (*bv - *av) as f64).round() as i32;
            }
            return out;
        }
    }
    samples[samples.len() - 1].bbox
}

// --- interaction grouping ---------------------------------------------------

/// Fraction of the shorter tube's span that overlaps the other in media time.
fn temporal_overlap(a: &FrameTube, b: &FrameTube, uspf: i64) -> f32 {
    let a0 = a.t_start_us;
    let a1 = a.t_start_us + a.frames() as i64 * uspf;
    let b0 = b.t_start_us;
    let b1 = b.t_start_us + b.frames() as i64 * uspf;
    let overlap = (a1.min(b1) - a0.max(b0)).max(0);
    let shorter = (a1 - a0).min(b1 - b0).max(1);
    overlap as f32 / shorter as f32
}

/// Closest box gap between two tubes over their co-occurring frames; `None` when
/// they never co-occur. Also returns the representative object width.
fn closest_gap(a: &FrameTube, b: &FrameTube, uspf: i64) -> Option<(i64, i64)> {
    let mut best: Option<i64> = None;
    let mut width = 1i64;
    for (fa, ba) in a.boxes.iter().enumerate() {
        let ta = a.t_start_us + fa as i64 * uspf;
        // index into b at the same media time
        let fb = (ta - b.t_start_us) / uspf;
        if fb < 0 || fb as usize >= b.boxes.len() {
            continue;
        }
        let bb = b.boxes[fb as usize];
        let gap = bbox_gap(*ba, bb);
        width = width.max(ba[2] as i64).max(bb[2] as i64);
        best = Some(best.map_or(gap, |g| g.min(gap)));
    }
    best.map(|g| (g, width))
}

/// Union-find group ids: tubes that co-occur (temporal overlap above threshold)
/// and stay spatially close (gap < `distance_factor × width`) join one group.
fn group_tubes(fts: &[FrameTube], params: &OptimiserParams, uspf: i64) -> Vec<usize> {
    let n = fts.len();
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while parent[r] != r {
            r = parent[r];
        }
        let mut c = x;
        while parent[c] != c {
            let next = parent[c];
            parent[c] = r;
            c = next;
        }
        r
    }
    for i in 0..n {
        for j in (i + 1)..n {
            if temporal_overlap(&fts[i], &fts[j], uspf) <= params.interaction_overlap {
                continue;
            }
            if let Some((gap, width)) = closest_gap(&fts[i], &fts[j], uspf) {
                if (gap as f32) < params.distance_factor * width as f32 {
                    let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                    if ri != rj {
                        parent[ri] = rj;
                    }
                }
            }
        }
    }
    (0..n).map(|i| find(&mut parent, i)).collect()
}

// --- greedy placement -------------------------------------------------------

/// A rigid interaction group: member tube indices and the group's base start.
struct Group {
    members: Vec<usize>,
    base_start_us: i64,
}

/// Place tubes by greedy time-shift against a collision budget, preserving
/// interaction groups as rigid units. Returns per-tube shifts + the resulting
/// length, and any dropped (too-crowded) tubes.
pub fn optimise(tubes: &[Tube], params: &OptimiserParams) -> SynopsisLayout {
    let fps = params.fps.max(1) as i64;
    let uspf = 1_000_000 / fps;

    if tubes.is_empty() {
        return SynopsisLayout {
            placements: Vec::new(),
            length_us: 0,
            dropped: Vec::new(),
        };
    }

    let fts: Vec<FrameTube> = tubes.iter().map(|t| interpolate(t, params.fps)).collect();
    let raw_groups = group_tubes(&fts, params, uspf);

    // Coalesce union-find roots into contiguous groups, each with a base start.
    let mut groups: Vec<Group> = Vec::new();
    let mut root_to_group: std::collections::HashMap<usize, usize> = Default::default();
    for (i, &root) in raw_groups.iter().enumerate() {
        let gid = *root_to_group.entry(root).or_insert_with(|| {
            groups.push(Group {
                members: Vec::new(),
                base_start_us: i64::MAX,
            });
            groups.len() - 1
        });
        groups[gid].members.push(i);
        groups[gid].base_start_us = groups[gid].base_start_us.min(fts[i].t_start_us);
    }

    // Chronological order keeps the synopsis readable.
    let mut order: Vec<usize> = (0..groups.len()).collect();
    order.sort_by_key(|&g| groups[g].base_start_us);

    let total_area: i64 = fts.iter().map(|f| f.pixel_area()).sum();
    let budget = (total_area as f64 * params.collision_budget.clamp(0.0, 1.0) as f64) as i64;
    let max_frames = params.max_length_us.map(|m| (m / uspf).max(0) as usize);

    // Synopsis-frame occupancy: each frame holds (tube_idx, bbox) of placed tubes.
    let mut occupancy: Vec<Vec<(usize, [i32; 4])>> = Vec::new();
    let mut running_collision: i64 = 0;
    let mut placements: Vec<TubePlacement> = Vec::new();
    let mut dropped: Vec<i64> = Vec::new();

    for (group_id, &g) in order.iter().enumerate() {
        let group = &groups[g];
        // Member frame layout relative to the group base start.
        let member_frames: Vec<(usize, usize)> = group
            .members
            .iter()
            .map(|&m| {
                let off = ((fts[m].t_start_us - group.base_start_us) / uspf).max(0) as usize;
                (m, off)
            })
            .collect();
        let group_len = member_frames
            .iter()
            .map(|&(m, off)| off + fts[m].frames())
            .max()
            .unwrap_or(0);

        // Try the earliest synopsis start frame whose added collision keeps us
        // under budget and respects the per-frame cap; else fall back to the
        // minimum-collision start (extending the synopsis).
        let horizon = occupancy.len() + 1;
        let mut best_start = 0usize;
        let mut best_added = i64::MAX;
        let mut placed_within_budget = false;

        for start in 0..=horizon {
            let (added, cap_ok) = trial_collision(
                &occupancy,
                &fts,
                &member_frames,
                start,
                params.max_tubes_per_frame,
            );
            if !cap_ok {
                continue;
            }
            if running_collision + added <= budget {
                best_start = start;
                best_added = added;
                placed_within_budget = true;
                break;
            }
            if added < best_added {
                best_added = added;
                best_start = start;
            }
        }

        // Drop the group when every start violated the frame cap, or when a
        // target length is set and the only feasible slot would push the group
        // past it (the "too crowded → fast-forward" fallback).
        let over_cap = max_frames.is_some_and(|mf| best_start + group_len > mf);
        if (!placed_within_budget && best_added == i64::MAX) || over_cap {
            for &(m, _) in &member_frames {
                dropped.push(fts[m].track_id);
            }
            continue;
        }

        // Commit the placement.
        commit(&mut occupancy, &fts, &member_frames, best_start, group_len);
        running_collision += best_added.max(0);

        let start_shift_us = best_start as i64 * uspf - group.base_start_us;
        for &(m, _) in &member_frames {
            placements.push(TubePlacement {
                track_id: fts[m].track_id,
                group_id,
                start_shift_us,
            });
        }
    }

    let length_us = occupancy.len() as i64 * uspf;
    // Stable, original order for determinism.
    placements.sort_by_key(|p| p.track_id);
    dropped.sort_unstable();
    SynopsisLayout {
        placements,
        length_us,
        dropped,
    }
}

/// Added collision (and frame-cap feasibility) of placing a group's members at
/// synopsis frame `start`, without mutating occupancy.
fn trial_collision(
    occupancy: &[Vec<(usize, [i32; 4])>],
    fts: &[FrameTube],
    member_frames: &[(usize, usize)],
    start: usize,
    max_per_frame: usize,
) -> (i64, bool) {
    let mut added = 0i64;
    for &(m, off) in member_frames {
        for (f, &bbox) in fts[m].boxes.iter().enumerate() {
            let sf = start + off + f;
            if sf >= occupancy.len() {
                continue;
            }
            if max_per_frame > 0 && occupancy[sf].len() >= max_per_frame {
                return (0, false);
            }
            for &(_, other) in &occupancy[sf] {
                added += intersection_area(bbox, other);
            }
        }
    }
    (added, true)
}

/// Commit a group's members into the occupancy grid, growing it as needed.
fn commit(
    occupancy: &mut Vec<Vec<(usize, [i32; 4])>>,
    fts: &[FrameTube],
    member_frames: &[(usize, usize)],
    start: usize,
    group_len: usize,
) {
    let needed = start + group_len;
    if occupancy.len() < needed {
        occupancy.resize_with(needed, Vec::new);
    }
    for &(m, off) in member_frames {
        for (f, &bbox) in fts[m].boxes.iter().enumerate() {
            occupancy[start + off + f].push((m, bbox));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::zmnext::detail::Tube;

    fn tube(track_id: i64, t0: i64, t1: i64, bbox: [i32; 4]) -> Tube {
        Tube {
            track_id,
            t_start_us: t0,
            t_end_us: t1,
            samples: vec![
                TubeSample {
                    pts_us: t0,
                    bbox,
                    ..Default::default()
                },
                TubeSample {
                    pts_us: t1,
                    bbox,
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
    }

    #[test]
    fn geometry_helpers() {
        assert_eq!(bbox_area([0, 0, 4, 5]), 20);
        assert_eq!(intersection_area([0, 0, 4, 4], [2, 2, 4, 4]), 4);
        assert_eq!(intersection_area([0, 0, 4, 4], [10, 10, 2, 2]), 0);
        assert_eq!(bbox_gap([0, 0, 2, 2], [5, 0, 2, 2]), 3);
        assert_eq!(bbox_gap([0, 0, 4, 4], [2, 2, 4, 4]), 0);
    }

    #[test]
    fn interpolation_spans_expected_frames() {
        // 1s tube at 10 fps → 11 frames (inclusive of both ends).
        let t = tube(1, 0, 1_000_000, [10, 10, 20, 40]);
        let ft = interpolate(&t, 10);
        assert_eq!(ft.frames(), 11);
        assert_eq!(ft.boxes[0], [10, 10, 20, 40]);
    }

    #[test]
    fn empty_input_is_empty_layout() {
        let layout = optimise(&[], &OptimiserParams::default());
        assert!(layout.placements.is_empty());
        assert_eq!(layout.length_us, 0);
    }

    #[test]
    fn distant_sequential_tubes_pack_earlier() {
        // Two spatially-separate tubes that play 10s apart. With a generous
        // budget they should pack to overlap in time → synopsis shorter than the
        // original 11s span, and the later tube shifts earlier.
        let params = OptimiserParams {
            fps: 10,
            collision_budget: 0.10,
            ..Default::default()
        };
        let tubes = vec![
            tube(1, 0, 1_000_000, [0, 0, 20, 20]),
            tube(2, 10_000_000, 11_000_000, [500, 0, 20, 20]),
        ];
        let layout = optimise(&tubes, &params);
        assert_eq!(layout.placements.len(), 2);
        // Different groups (far apart spatially & temporally).
        assert_ne!(layout.placements[0].group_id, layout.placements[1].group_id);
        // The second tube is pulled earlier (negative shift) to pack.
        let p2 = layout.placements.iter().find(|p| p.track_id == 2).unwrap();
        assert!(
            p2.start_shift_us < 0,
            "tube 2 should shift earlier, got {}",
            p2.start_shift_us
        );
        // Packed synopsis is far shorter than the original ~11s span.
        assert!(
            layout.length_us < 3_000_000,
            "packed length {}us should be well under 11s",
            layout.length_us
        );
    }

    #[test]
    fn interacting_tubes_share_a_rigid_group() {
        // Two tubes overlapping in time AND spatially adjacent → one group, same
        // shift (a hand-off must not be split across synopsis times).
        let params = OptimiserParams {
            fps: 10,
            ..Default::default()
        };
        let tubes = vec![
            tube(1, 0, 2_000_000, [100, 100, 40, 40]),
            tube(2, 0, 2_000_000, [150, 100, 40, 40]), // adjacent (gap 10 < 2×40)
        ];
        let layout = optimise(&tubes, &params);
        assert_eq!(layout.placements[0].group_id, layout.placements[1].group_id);
        assert_eq!(
            layout.placements[0].start_shift_us, layout.placements[1].start_shift_us,
            "rigid group members share a shift"
        );
    }

    #[test]
    fn frame_cap_extends_synopsis_rather_than_dropping() {
        // Three identical tubes, cap 2/frame, no length cap: the third can't
        // share a frame so the synopsis *extends* to fit it (overflow → longer
        // synopsis). Nothing is dropped.
        let params = OptimiserParams {
            fps: 10,
            max_tubes_per_frame: 2,
            collision_budget: 1.0,
            interaction_overlap: 2.0, // separate groups
            ..Default::default()
        };
        let tubes = vec![
            tube(1, 0, 1_000_000, [0, 0, 100, 100]),
            tube(2, 0, 1_000_000, [0, 0, 100, 100]),
            tube(3, 0, 1_000_000, [0, 0, 100, 100]),
        ];
        let layout = optimise(&tubes, &params);
        assert!(layout.dropped.is_empty(), "no length cap → nothing dropped");
        assert_eq!(layout.placements.len(), 3);
        // Two fit in the first second; the third extends past it.
        assert!(
            layout.length_us > 1_500_000,
            "extended to fit the third tube"
        );
    }

    #[test]
    fn target_length_drops_overflow() {
        // Same three identical tubes, but a target length of ~1.2s. Two fit; the
        // third would extend past the cap and is dropped (logged, never silent).
        let params = OptimiserParams {
            fps: 10,
            max_tubes_per_frame: 2,
            collision_budget: 1.0,
            interaction_overlap: 2.0,
            max_length_us: Some(1_200_000),
            ..Default::default()
        };
        let tubes = vec![
            tube(1, 0, 1_000_000, [0, 0, 100, 100]),
            tube(2, 0, 1_000_000, [0, 0, 100, 100]),
            tube(3, 0, 1_000_000, [0, 0, 100, 100]),
        ];
        let layout = optimise(&tubes, &params);
        assert_eq!(layout.dropped, vec![3], "the overflow tube is dropped");
        assert_eq!(layout.placements.len(), 2);
    }

    #[test]
    fn tighter_budget_yields_longer_synopsis() {
        // Same two overlapping-in-place tubes; a tighter collision budget should
        // push them apart in time (longer synopsis) vs a loose budget.
        let tubes = vec![
            tube(1, 0, 1_000_000, [0, 0, 50, 50]),
            tube(2, 0, 1_000_000, [10, 10, 50, 50]), // overlaps tube 1
        ];
        let loose = optimise(
            &tubes,
            &OptimiserParams {
                fps: 10,
                collision_budget: 1.0,
                interaction_overlap: 2.0, // keep separate groups
                ..Default::default()
            },
        );
        let tight = optimise(
            &tubes,
            &OptimiserParams {
                fps: 10,
                collision_budget: 0.0,
                interaction_overlap: 2.0,
                ..Default::default()
            },
        );
        assert!(
            tight.length_us > loose.length_us,
            "tighter budget ({}us) should be longer than loose ({}us)",
            tight.length_us,
            loose.length_us
        );
    }
}
