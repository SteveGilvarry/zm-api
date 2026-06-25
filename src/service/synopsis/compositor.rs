//! Pixel compositing for the motion synopsis.
//!
//! **Anti-recompute rule:** this module never decodes the source clip or re-runs
//! detection. Every pixel comes from the pre-rendered cutout/plate JPEGs the
//! manifest references. It only does compositing — drawing premultiplied cutouts
//! over a time-appropriate background plate, using the per-sample mask (polygon
//! or RLE) for a clean alpha edge.
//!
//! The P1 "composite still" stamps one representative cutout per tube onto a
//! single plate; the P3 video renderer reuses [`composite_cutout`] /
//! [`build_plate_canvas`] per output frame.

use std::path::{Path, PathBuf};

use image::{imageops::FilterType, RgbImage};

use super::SynopsisError;
use crate::service::zmnext::detail::{Mask, Plate, TubeManifest, TubeSample};

/// A loaded RGB canvas at source resolution that cutouts are composited onto.
pub type Canvas = RgbImage;

/// Safely join an asset-relative path under `base`, rejecting `..` traversal
/// exactly like `resolve_event_storage_path`. Absolute `rel` is allowed only
/// when it equals/escapes nothing — but cutout/plate paths are relative, so an
/// absolute one is rejected to keep reads inside the asset tree.
pub fn safe_asset_path(base: &Path, rel: &str) -> Option<PathBuf> {
    if rel.is_empty() || crate::util::path::contains_traversal(rel) {
        return None;
    }
    let candidate = Path::new(rel);
    if candidate.is_absolute() {
        return None;
    }
    Some(base.join(candidate))
}

/// Decode an RGB image from disk. `None` (not an error) when the file is missing
/// or undecodable — a pruned asset degrades the render, it does not fail it.
pub fn load_rgb(path: &Path) -> Option<RgbImage> {
    match image::open(path) {
        Ok(img) => Some(img.to_rgb8()),
        Err(e) => {
            tracing::debug!("synopsis: skipping unreadable asset {:?}: {}", path, e);
            None
        }
    }
}

/// Rescale an image to exactly `w × h`. No-op when already that size.
pub fn rescale_to(img: RgbImage, w: u32, h: u32) -> RgbImage {
    if img.width() == w && img.height() == h {
        img
    } else {
        image::imageops::resize(&img, w.max(1), h.max(1), FilterType::Triangle)
    }
}

/// Pick the plate whose `wallclock_ms` is nearest `target_ms` (time-varying
/// background). `None` when there are no plates.
pub fn nearest_plate(plates: &[Plate], target_ms: i64) -> Option<&Plate> {
    plates
        .iter()
        .filter(|p| !p.path.is_empty())
        .min_by_key(|p| (p.wallclock_ms - target_ms).abs())
}

/// Build the background canvas for a synopsis frame: the time-nearest plate,
/// rescaled to source resolution, or a black canvas when no plate is usable.
pub fn build_plate_canvas(
    manifest: &TubeManifest,
    asset_dir: &Path,
    target_ms: i64,
    source_w: u32,
    source_h: u32,
) -> Canvas {
    if let Some(plate) = nearest_plate(&manifest.plates, target_ms) {
        if let Some(path) = safe_asset_path(asset_dir, &plate.path) {
            if let Some(img) = load_rgb(&path) {
                return rescale_to(img, source_w, source_h);
            }
        }
    }
    RgbImage::new(source_w.max(1), source_h.max(1))
}

/// Per-cutout-pixel alpha in `[0,1]`, row-major `cw × ch`. Empty when there is
/// no usable mask (the caller then luma-keys the premultiplied cutout).
pub fn rasterize_mask(mask: Option<&Mask>, bbox: [i32; 4], cw: u32, ch: u32) -> Vec<f32> {
    let (cw, ch) = (cw as usize, ch as usize);
    match mask {
        Some(Mask::Polygon { points }) if points.len() >= 3 => {
            // Polygon points are in source coords; shift to cutout-local.
            let local: Vec<[f32; 2]> = points
                .iter()
                .map(|p| [p[0] - bbox[0] as f32, p[1] - bbox[1] as f32])
                .collect();
            rasterize_polygon(&local, cw, ch)
        }
        Some(Mask::Rle { w, h, counts }) if *w as usize == cw && *h as usize == ch => {
            decode_rle(counts, cw, ch)
        }
        // Mismatched RLE dims, polygon too small, Unknown, or absent → no mask.
        _ => Vec::new(),
    }
}

/// Even-odd scanline polygon fill → a `cw × ch` alpha buffer (1.0 inside).
fn rasterize_polygon(points: &[[f32; 2]], cw: usize, ch: usize) -> Vec<f32> {
    let mut alpha = vec![0.0f32; cw * ch];
    if points.len() < 3 || cw == 0 || ch == 0 {
        return alpha;
    }
    for y in 0..ch {
        let yc = y as f32 + 0.5;
        // Collect x-intersections of the scanline with all edges.
        let mut xs: Vec<f32> = Vec::new();
        for i in 0..points.len() {
            let a = points[i];
            let b = points[(i + 1) % points.len()];
            let (y0, y1) = (a[1], b[1]);
            // Edge crosses this scanline (half-open to avoid double-counting
            // shared vertices).
            if (y0 <= yc && y1 > yc) || (y1 <= yc && y0 > yc) {
                let t = (yc - y0) / (y1 - y0);
                xs.push(a[0] + t * (b[0] - a[0]));
            }
        }
        if xs.len() < 2 {
            continue;
        }
        xs.sort_by(|p, q| p.partial_cmp(q).unwrap_or(std::cmp::Ordering::Equal));
        // Fill between consecutive intersection pairs.
        let mut k = 0;
        while k + 1 < xs.len() {
            let x_start = xs[k].ceil().max(0.0) as usize;
            let x_end = (xs[k + 1].floor() as isize).min(cw as isize - 1);
            if x_end >= 0 {
                for x in x_start..=(x_end as usize) {
                    if x < cw {
                        alpha[y * cw + x] = 1.0;
                    }
                }
            }
            k += 2;
        }
    }
    alpha
}

/// Decode a column-major (COCO-style) RLE into a row-major `cw × ch` alpha
/// buffer. Runs alternate background/foreground starting with background.
fn decode_rle(counts: &[u32], cw: usize, ch: usize) -> Vec<f32> {
    let mut alpha = vec![0.0f32; cw * ch];
    if cw == 0 || ch == 0 {
        return alpha;
    }
    let total = cw * ch;
    let mut pos = 0usize; // column-major linear position
    let mut value = 0.0f32; // first run is background
    for &run in counts {
        let end = (pos + run as usize).min(total);
        if value > 0.0 {
            for p in pos..end {
                let col = p / ch;
                let row = p % ch;
                if row < ch && col < cw {
                    alpha[row * cw + col] = 1.0;
                }
            }
        }
        pos = end;
        value = 1.0 - value;
        if pos >= total {
            break;
        }
    }
    alpha
}

/// Separable box-blur of an alpha buffer to feather hard mask edges, suppressing
/// cutout halos. `radius == 0` is a no-op.
pub fn feather_alpha(alpha: &mut [f32], cw: usize, ch: usize, radius: u32) {
    if radius == 0 || cw == 0 || ch == 0 || alpha.len() != cw * ch {
        return;
    }
    let r = radius as isize;
    let mut tmp = alpha.to_vec();
    // Horizontal pass.
    for y in 0..ch {
        for x in 0..cw {
            let mut sum = 0.0;
            let mut n = 0.0;
            for dx in -r..=r {
                let xx = x as isize + dx;
                if xx >= 0 && (xx as usize) < cw {
                    sum += alpha[y * cw + xx as usize];
                    n += 1.0;
                }
            }
            tmp[y * cw + x] = sum / n;
        }
    }
    // Vertical pass.
    for y in 0..ch {
        for x in 0..cw {
            let mut sum = 0.0;
            let mut n = 0.0;
            for dy in -r..=r {
                let yy = y as isize + dy;
                if yy >= 0 && (yy as usize) < ch {
                    sum += tmp[yy as usize * cw + x];
                    n += 1.0;
                }
            }
            alpha[y * cw + x] = sum / n;
        }
    }
}

/// Luma-key alpha for a premultiplied cutout with no explicit mask: near-black
/// (the premultiplied background) is transparent, brighter pixels opaque.
fn luma_key(px: &image::Rgb<u8>) -> f32 {
    let maxc = px.0.iter().copied().max().unwrap_or(0) as f32;
    ((maxc - 4.0) * (1.0 / 24.0)).clamp(0.0, 1.0)
}

/// Composite one premultiplied cutout onto the canvas at `bbox`, using `alpha`
/// (row-major `cutout_w × cutout_h`) when present, else a luma key. Premultiplied
/// "over": `out = src + dst·(1 − a)`.
pub fn composite_cutout(
    canvas: &mut Canvas,
    cutout: &RgbImage,
    bbox: [i32; 4],
    alpha: &[f32],
    feather_px: u32,
) {
    let cw = cutout.width();
    let ch = cutout.height();
    let has_mask = alpha.len() == (cw * ch) as usize;

    // Feather a copy of the mask so the caller's buffer is reusable.
    let mask = if has_mask && feather_px > 0 {
        let mut m = alpha.to_vec();
        feather_alpha(&mut m, cw as usize, ch as usize, feather_px);
        Some(m)
    } else if has_mask {
        Some(alpha.to_vec())
    } else {
        None
    };

    let (ox, oy) = (bbox[0], bbox[1]);
    let (canvas_w, canvas_h) = (canvas.width() as i32, canvas.height() as i32);

    for cy in 0..ch {
        let dy = oy + cy as i32;
        if dy < 0 || dy >= canvas_h {
            continue;
        }
        for cx in 0..cw {
            let dx = ox + cx as i32;
            if dx < 0 || dx >= canvas_w {
                continue;
            }
            let src = cutout.get_pixel(cx, cy);
            let a = match &mask {
                Some(m) => m[(cy * cw + cx) as usize],
                None => luma_key(src),
            };
            if a <= 0.0 {
                continue;
            }
            let inv = 1.0 - a;
            let dst = canvas.get_pixel_mut(dx as u32, dy as u32);
            for c in 0..3 {
                // src is already premultiplied by its own coverage; the mask
                // narrows that coverage, so scale src by `a` before adding.
                let s = src.0[c] as f32 * a;
                let d = dst.0[c] as f32 * inv;
                dst.0[c] = (s + d).round().clamp(0.0, 255.0) as u8;
            }
        }
    }
}

/// Choose the most prominent sample of a tube for the still: largest bbox area
/// (ties → the middle sample, which is usually mid-motion).
pub fn representative_sample(samples: &[TubeSample]) -> Option<&TubeSample> {
    if samples.is_empty() {
        return None;
    }
    samples
        .iter()
        .enumerate()
        .max_by_key(|(i, s)| {
            let area = (s.bbox[2].max(0) as i64) * (s.bbox[3].max(0) as i64);
            // Prefer larger area; break ties toward the middle index.
            let mid = samples.len() as i64 / 2;
            (area, -(*i as i64 - mid).abs())
        })
        .map(|(_, s)| s)
}

/// Render the P1 composite still: one representative cutout per tube stamped onto
/// the time-central plate. Returns encoded JPEG bytes. Degrades over missing
/// assets (skips them); only errors when the manifest lacks source dimensions.
pub fn render_still(
    manifest: &TubeManifest,
    asset_dir: &Path,
    feather_px: u32,
) -> Result<Vec<u8>, SynopsisError> {
    let (sw, sh) = (manifest.source_w, manifest.source_h);
    if sw == 0 || sh == 0 {
        return Err(SynopsisError::InvalidManifest(
            "manifest missing source_w/source_h".to_string(),
        ));
    }

    // Plate nearest the event midpoint.
    let mid_ms = if manifest.t_start_us != 0 || manifest.t_end_us != 0 {
        (manifest.t_start_us / 2 + manifest.t_end_us / 2) / 1000
    } else {
        0
    };
    let mut canvas = build_plate_canvas(manifest, asset_dir, mid_ms, sw, sh);

    let mut drawn = 0usize;
    for tube in &manifest.tubes {
        let Some(sample) = representative_sample(&tube.samples) else {
            continue;
        };
        let Some(path) = safe_asset_path(asset_dir, &sample.cutout) else {
            continue;
        };
        let Some(cutout) = load_rgb(&path) else {
            continue;
        };
        let alpha = rasterize_mask(
            sample.mask.as_ref(),
            sample.bbox,
            cutout.width(),
            cutout.height(),
        );
        composite_cutout(&mut canvas, &cutout, sample.bbox, &alpha, feather_px);
        drawn += 1;
    }
    tracing::debug!(
        "synopsis still: {} of {} tubes drawn over {}x{}",
        drawn,
        manifest.tubes.len(),
        sw,
        sh
    );

    encode_jpeg(&canvas)
}

/// Render a P4 **overview** still: a montage of one representative cutout per
/// tube across *many* events (a time range), stamped onto a single black canvas.
/// Each tube's cutout is resolved under its own event's `asset_dir`. Capped at
/// `max_tubes` eligible tubes; returns the encoded JPEG and the count dropped by
/// the cap (so callers can log/report — never a silent truncation).
///
/// A plain black background is used (plates are per-event and camera-level; a
/// single shared plate is a future refinement — see the spec's plate-sharing
/// open question).
pub fn render_overview_still(
    manifests: &[(String, TubeManifest)],
    class_filter: &[i64],
    max_tubes: usize,
    feather_px: u32,
) -> Result<Vec<u8>, SynopsisError> {
    // Canvas dimensions from the first manifest with usable source dims.
    let (sw, sh) = manifests
        .iter()
        .map(|(_, m)| (m.source_w, m.source_h))
        .find(|&(w, h)| w > 0 && h > 0)
        .unwrap_or((1280, 720));
    let mut canvas = RgbImage::new(sw.max(1), sh.max(1));

    // Eligible (asset_dir, sample) pairs across all events, class-filtered.
    let mut eligible: Vec<(&str, &TubeSample)> = Vec::new();
    for (asset_dir, manifest) in manifests {
        for tube in &manifest.tubes {
            if !class_filter.is_empty() && !class_filter.contains(&tube.class_id) {
                continue;
            }
            if let Some(sample) = representative_sample(&tube.samples) {
                eligible.push((asset_dir.as_str(), sample));
            }
        }
    }

    let cap = max_tubes.max(1);
    let dropped = eligible.len().saturating_sub(cap);
    for (asset_dir, sample) in eligible.into_iter().take(cap) {
        let Some(path) = safe_asset_path(Path::new(asset_dir), &sample.cutout) else {
            continue;
        };
        let Some(cutout) = load_rgb(&path) else {
            continue;
        };
        let alpha = rasterize_mask(
            sample.mask.as_ref(),
            sample.bbox,
            cutout.width(),
            cutout.height(),
        );
        composite_cutout(&mut canvas, &cutout, sample.bbox, &alpha, feather_px);
    }
    if dropped > 0 {
        tracing::info!("synopsis overview: capped at {cap} tubes, {dropped} dropped");
    }

    encode_jpeg(&canvas)
}

/// Encode an RGB canvas to JPEG bytes.
pub fn encode_jpeg(canvas: &Canvas) -> Result<Vec<u8>, SynopsisError> {
    let mut bytes = Vec::new();
    canvas
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Jpeg,
        )
        .map_err(|e| SynopsisError::RenderFailed(format!("jpeg encode failed: {e}")))?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::zmnext::detail::Mask;

    #[test]
    fn safe_asset_path_rejects_traversal_and_absolute() {
        let base = Path::new("/data/3/2026/512/synopsis");
        assert_eq!(
            safe_asset_path(base, "t17/000001.jpg"),
            Some(PathBuf::from("/data/3/2026/512/synopsis/t17/000001.jpg"))
        );
        assert_eq!(safe_asset_path(base, "../../etc/passwd"), None);
        assert_eq!(safe_asset_path(base, "/etc/passwd"), None);
        assert_eq!(safe_asset_path(base, ""), None);
    }

    #[test]
    fn polygon_fills_a_triangle() {
        // Right triangle covering the lower-left half of a 4x4, bbox at origin.
        let pts = [[0.0, 0.0], [4.0, 0.0], [0.0, 4.0]];
        let a = rasterize_polygon(&pts, 4, 4);
        // Top-left pixel center (0.5,0.5) is inside.
        assert_eq!(a[0], 1.0);
        // Bottom-right pixel center (3.5,3.5) is outside.
        assert_eq!(a[4 * 4 - 1], 0.0);
    }

    #[test]
    fn rle_round_trips_column_major() {
        // 2 wide x 2 tall, column-major counts: bg,fg,bg,fg of length 1 each →
        // checkerboard: (col0,row1) and (col1,row1) foreground? Walk it out:
        // pos0 col0row0 bg, pos1 col0row1 fg, pos2 col1row0 bg, pos3 col1row1 fg.
        let a = decode_rle(&[1, 1, 1, 1], 2, 2);
        // row-major indices: [r0c0, r0c1, r1c0, r1c1]
        assert_eq!(a, vec![0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn rasterize_mask_offsets_polygon_by_bbox() {
        // Polygon in source coords around a bbox at (10,10), 4x4.
        let mask = Mask::Polygon {
            points: vec![[10.0, 10.0], [14.0, 10.0], [10.0, 14.0]],
        };
        let a = rasterize_mask(Some(&mask), [10, 10, 4, 4], 4, 4);
        assert_eq!(a.len(), 16);
        assert_eq!(a[0], 1.0); // local (0,0) inside
    }

    #[test]
    fn composite_premultiplied_over_black_keeps_src() {
        let mut canvas = RgbImage::new(8, 8);
        let mut cutout = RgbImage::new(4, 4);
        for p in cutout.pixels_mut() {
            *p = image::Rgb([200, 100, 50]);
        }
        // Full-alpha mask → src lands verbatim over black.
        let alpha = vec![1.0; 16];
        composite_cutout(&mut canvas, &cutout, [2, 2, 4, 4], &alpha, 0);
        assert_eq!(*canvas.get_pixel(2, 2), image::Rgb([200, 100, 50]));
        // Outside the bbox stays black.
        assert_eq!(*canvas.get_pixel(0, 0), image::Rgb([0, 0, 0]));
    }

    #[test]
    fn render_still_errors_without_source_dims() {
        let manifest = TubeManifest::default();
        let err = render_still(&manifest, Path::new("/nonexistent"), 1).unwrap_err();
        assert!(matches!(err, SynopsisError::InvalidManifest(_)));
    }

    #[test]
    fn render_still_produces_jpeg_over_black_when_assets_missing() {
        // A tube whose cutout file doesn't exist → skipped, still renders.
        let manifest = TubeManifest {
            source_w: 16,
            source_h: 16,
            tubes: vec![crate::service::zmnext::detail::Tube {
                samples: vec![TubeSample {
                    bbox: [1, 1, 4, 4],
                    cutout: "missing.jpg".to_string(),
                    cutout_w: 4,
                    cutout_h: 4,
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };
        let bytes = render_still(&manifest, Path::new("/nonexistent"), 1).unwrap();
        // JPEG SOI marker.
        assert_eq!(&bytes[0..2], &[0xFF, 0xD8]);
    }
}
