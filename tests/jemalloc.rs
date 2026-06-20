//! Verifies the process-wide global allocator is jemalloc.
//!
//! The server is a long-running, allocation-bursty media workload (HLS
//! segmenting, WebRTC, ffmpeg decode/mux buffers). Under the system glibc
//! allocator, freed buffers are retained in per-arena free lists rather than
//! returned to the OS, so RSS sits at its high-water mark for the life of the
//! process. jemalloc returns freed pages far more aggressively, keeping RSS
//! close to the live working set. This test guards that the jemalloc global
//! allocator stays wired up — `#[global_allocator]` is easy to drop silently.
//!
//! The check is discriminating: it advances jemalloc's stats epoch, makes a
//! large heap allocation, and asserts jemalloc's own `allocated` counter grew
//! by roughly that much. If some other allocator (e.g. glibc) were installed
//! globally, the allocation would bypass jemalloc and its counter would not
//! move.

#![cfg(not(target_env = "msvc"))]

// Force the `zm_api` library to be linked into this test binary so its
// `#[global_allocator]` (defined in src/lib.rs) actually registers. Without a
// reference to the crate, the linker may drop it and the test would observe the
// default system allocator instead.
use zm_api as _;

use tikv_jemalloc_ctl::{epoch, stats};

#[test]
fn global_allocator_is_jemalloc() {
    // Force a fresh stats epoch so reads reflect current state.
    epoch::advance().expect("advance jemalloc epoch");
    let before = stats::allocated::read().expect("read allocated before");

    // A large allocation that comfortably exceeds cross-thread stat noise.
    const ALLOC: usize = 16 * 1024 * 1024;
    let mut buf: Vec<u8> = Vec::with_capacity(ALLOC);
    // Touch the pages so the allocation is realized, and keep it live across
    // the second epoch read below.
    buf.resize(ALLOC, 0xA5);
    std::hint::black_box(&buf);

    epoch::advance().expect("advance jemalloc epoch");
    let after = stats::allocated::read().expect("read allocated after");

    // Keep `buf` alive until after the measurement.
    std::hint::black_box(&buf);

    assert!(
        after >= before + ALLOC / 2,
        "jemalloc 'allocated' did not grow with a {ALLOC}-byte allocation \
         (before={before}, after={after}); the global allocator is likely \
         not jemalloc"
    );
}
