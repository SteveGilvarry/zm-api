//! Reciprocal Rank Fusion (RRF) of multiple ranked result lists.
//!
//! `score(e) = Σ_lists 1 / (k + rank_list(e))` with `k = 60` (the standard
//! constant). Rank-based fusion is robust to the different score scales of the
//! vector ANN and the lexical FTS lists — no normalization needed.

use super::store::Hit;

/// The conventional RRF damping constant.
pub const RRF_K: usize = 60;

/// Fuse ranked `lists` (each best-first) into one ranked list, deduped by
/// `event_id`. Each hit's `score` becomes its RRF score; the snippet/ts are
/// taken from the first list the event appears in.
pub fn reciprocal_rank_fusion(lists: &[Vec<Hit>], rrf_k: usize) -> Vec<Hit> {
    use std::collections::HashMap;
    let mut acc: HashMap<u64, (f32, Hit)> = HashMap::new();
    for list in lists {
        for (i, hit) in list.iter().enumerate() {
            let rank = i + 1; // 1-based
            let contrib = 1.0 / (rrf_k as f32 + rank as f32);
            acc.entry(hit.event_id)
                .or_insert_with(|| (0.0, hit.clone()))
                .0 += contrib;
        }
    }
    let mut fused: Vec<Hit> = acc
        .into_values()
        .map(|(score, mut hit)| {
            hit.score = score;
            hit
        })
        .collect();
    fused.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            // Stable tie-break by event id for determinism.
            .then(a.event_id.cmp(&b.event_id))
    });
    fused
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: u64) -> Hit {
        Hit {
            event_id: id,
            score: 0.0,
            ts: id as i64,
            snippet: format!("event {id}"),
        }
    }

    #[test]
    fn fuses_and_dedups_across_lists() {
        // event 2 is rank-1 in BOTH lists, event 1 is rank-2 in both — so event 2
        // strictly outscores event 1 (symmetric ranks would tie, so we make 2
        // dominate). events 3 and 4 appear once each.
        let ann = vec![hit(2), hit(1), hit(3)];
        let fts = vec![hit(2), hit(1), hit(4)];
        let fused = reciprocal_rank_fusion(&[ann, fts], RRF_K);

        let order: Vec<u64> = fused.iter().map(|h| h.event_id).collect();
        assert_eq!(fused.len(), 4, "deduped union of both lists");
        assert_eq!(order[0], 2, "event 2 (rank-1 in both) ranks first");
        assert_eq!(order[1], 1, "event 1 (rank-2 in both) next");
        // every fused score is positive
        assert!(fused.iter().all(|h| h.score > 0.0));
    }

    #[test]
    fn symmetric_ranks_tie_break_by_event_id() {
        // event 1 (ranks 1,2) and event 2 (ranks 2,1) have identical RRF scores;
        // the deterministic tie-break orders by ascending event id.
        let ann = vec![hit(1), hit(2)];
        let fts = vec![hit(2), hit(1)];
        let fused = reciprocal_rank_fusion(&[ann, fts], RRF_K);
        assert_eq!(
            fused.iter().map(|h| h.event_id).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[test]
    fn single_list_preserves_order() {
        let fused = reciprocal_rank_fusion(&[vec![hit(5), hit(6), hit(7)]], RRF_K);
        assert_eq!(
            fused.iter().map(|h| h.event_id).collect::<Vec<_>>(),
            vec![5, 6, 7]
        );
    }

    #[test]
    fn empty_input_is_empty() {
        assert!(reciprocal_rank_fusion(&[], RRF_K).is_empty());
    }
}
