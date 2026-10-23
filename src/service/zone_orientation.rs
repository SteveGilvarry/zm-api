//! Zones and a monitor's Orientation.
//!
//! zmc turns or flips each captured frame by the monitor's `Orientation`
//! before analysis, and zone points are positions in that turned picture.
//! When the Orientation changes, the picture turns under the zones, so each
//! zone's points must turn with it to stay over the same part of the scene.
//!
//! zmc's turns (`rotate_plane` in `zm_image.cpp`) are clockwise:
//! `ROTATE_90` sends source pixel (x, y) to (H-1-y, x). `FLIP_HORI` mirrors
//! left to right and `FLIP_VERT` top to bottom.
//!
//! Points with a decimal point are percentages of the picture (0 to 100 on
//! both axes), as zmc reads them; points without are pixels, in a picture
//! whose width and height swap at 90° and 270°. Thresholds need no change:
//! the zone keeps its area.
//!
//! ZoneMinder's PHP UI only swaps x and y when Width and Height swap, which
//! mirrors a zone rather than turning it, and misses 180° and the flips.

use crate::entity::sea_orm_active_enums::Orientation;

/// A 2×2 matrix acting on points measured from the picture's centre.
type M = [[i8; 2]; 2];

/// Where `Orientation` sends an upright point, about the centre.
fn matrix(o: &Orientation) -> M {
    match o {
        Orientation::Rotate0 => [[1, 0], [0, 1]],
        // (u, v) -> (-v, u): x' = W - y, y' = x.
        Orientation::Rotate90 => [[0, -1], [1, 0]],
        Orientation::Rotate180 => [[-1, 0], [0, -1]],
        Orientation::Rotate270 => [[0, 1], [-1, 0]],
        Orientation::FlipHori => [[-1, 0], [0, 1]],
        Orientation::FlipVert => [[1, 0], [0, -1]],
    }
}

fn transpose(m: M) -> M {
    [[m[0][0], m[1][0]], [m[0][1], m[1][1]]]
}

fn mul(a: M, b: M) -> M {
    let mut r = [[0i8; 2]; 2];
    for (i, row) in r.iter_mut().enumerate() {
        for (j, cell) in row.iter_mut().enumerate() {
            *cell = a[i][0] * b[0][j] + a[i][1] * b[1][j];
        }
    }
    r
}

fn quarter_turned(o: &Orientation) -> bool {
    matches!(o, Orientation::Rotate90 | Orientation::Rotate270)
}

/// The zone's `Coords` after the monitor goes from `from` to `to`, for a
/// camera of `width` × `height` (the Monitors row's own Width and Height,
/// which zmc swaps itself for 90° and 270°). `None` when nothing changes or
/// the coords don't parse, so the zone is left as it is.
pub fn reorient(
    coords: &str,
    from: &Orientation,
    to: &Orientation,
    width: u16,
    height: u16,
) -> Option<String> {
    if from == to {
        return None;
    }
    let points = crate::service::polygon::parse_coords(coords)?;
    let percent = coords.contains('.');

    // The picture's far corner, in the zone's units, before and after.
    let extent = |o: &Orientation| -> (f64, f64) {
        if percent {
            (100.0, 100.0)
        } else {
            let (w, h) = if quarter_turned(o) {
                (height, width)
            } else {
                (width, height)
            };
            (
                f64::from(w.saturating_sub(1)),
                f64::from(h.saturating_sub(1)),
            )
        }
    };
    let (fw, fh) = extent(from);
    let (tw, th) = extent(to);
    // Undo `from` (its matrices are orthogonal, so the inverse is the
    // transpose), then apply `to`.
    let m = mul(matrix(to), transpose(matrix(from)));

    let out: Vec<String> = points
        .iter()
        .map(|p| {
            let (u, v) = (p.x - fw / 2.0, p.y - fh / 2.0);
            let x = f64::from(m[0][0]) * u + f64::from(m[0][1]) * v + tw / 2.0;
            let y = f64::from(m[1][0]) * u + f64::from(m[1][1]) * v + th / 2.0;
            // `+ 0.0` turns -0.0 into 0.0, so it doesn't print as "-0.00".
            if percent {
                format!("{:.2},{:.2}", x + 0.0, y + 0.0)
            } else {
                format!("{},{}", x.round() as i64, y.round() as i64)
            }
        })
        .collect();
    Some(out.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use Orientation::*;

    const QUARTER: &str = "0.00,0.00 50.00,0.00 50.00,50.00 0.00,50.00";

    #[test]
    fn a_quarter_turn_moves_the_top_left_to_the_top_right() {
        assert_eq!(
            reorient(QUARTER, &Rotate0, &Rotate90, 1920, 1080).unwrap(),
            "100.00,0.00 100.00,50.00 50.00,50.00 50.00,0.00"
        );
    }

    #[test]
    fn each_orientation_puts_the_top_left_corner_where_zmc_does() {
        let corner = "0.00,0.00 10.00,0.00 0.00,10.00";
        let first = |to| {
            reorient(corner, &Rotate0, &to, 1920, 1080)
                .unwrap()
                .split(' ')
                .next()
                .unwrap()
                .to_string()
        };
        assert_eq!(first(Rotate90), "100.00,0.00");
        assert_eq!(first(Rotate180), "100.00,100.00");
        assert_eq!(first(Rotate270), "0.00,100.00");
        assert_eq!(first(FlipHori), "100.00,0.00");
        assert_eq!(first(FlipVert), "0.00,100.00");
    }

    #[test]
    fn turning_back_restores_the_zone() {
        for o in [Rotate90, Rotate180, Rotate270, FlipHori, FlipVert] {
            let there = reorient(QUARTER, &Rotate0, &o, 1920, 1080).unwrap();
            let back = reorient(&there, &o, &Rotate0, 1920, 1080).unwrap();
            assert_eq!(back, QUARTER, "{o:?}");
        }
    }

    #[test]
    fn going_between_two_turns_composes() {
        let direct = reorient(QUARTER, &Rotate0, &Rotate270, 1920, 1080).unwrap();
        let via = reorient(QUARTER, &Rotate0, &Rotate90, 1920, 1080).unwrap();
        let via = reorient(&via, &Rotate90, &Rotate270, 1920, 1080).unwrap();
        assert_eq!(direct, via);
    }

    #[test]
    fn pixel_zones_use_the_turned_pictures_size() {
        // 1920×1080 upright; 1080×1920 after a quarter turn. The top-left
        // pixel lands on the top-right one, x = 1079.
        assert_eq!(
            reorient("0,0 100,0 0,100", &Rotate0, &Rotate90, 1920, 1080).unwrap(),
            "1079,0 1079,100 979,0"
        );
        assert_eq!(
            reorient("0,0 100,0 0,100", &Rotate0, &Rotate180, 1920, 1080).unwrap(),
            "1919,1079 1819,1079 1919,979"
        );
    }

    #[test]
    fn no_change_or_bad_coords_leave_the_zone_alone() {
        assert!(reorient(QUARTER, &Rotate90, &Rotate90, 1920, 1080).is_none());
        assert!(reorient("nonsense", &Rotate0, &Rotate90, 1920, 1080).is_none());
    }
}
