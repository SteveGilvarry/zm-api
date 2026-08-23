//! Polygon area for zone `Coords` (GH #43).
//!
//! `Zones.Area` is not decorative: when a zone's `Units` are `Percent`, the
//! alarm thresholds (`MinAlarmPixels`, `MaxAlarmPixels`, `MinFilterPixels`,
//! `MinBlobPixels`) are stored as a proportion *of* it. A zone whose `Area` is
//! wrong has thresholds that silently do not mean what they say, and nothing
//! errors.
//!
//! Every zone created through this API had `Area = 0` hardcoded, and updating
//! a zone's coordinates never recomputed it.
//!
//! Matches ZoneMinder's own `getPolyArea` (`web/includes/functions.php`), which
//! is a plain shoelace over the raw coordinates:
//!
//! ```php
//! // Shoelace formula - works correctly with both integer and float coordinates
//! return round(abs($area) / 2.0);
//! ```
//!
//! Upstream also keeps a `getPolyAreaOld` that inflates each edge by a pixel,
//! but nothing calls it; matching the live function is what keeps this API's
//! value equal to the one the web UI shows for the same zone.

/// A parsed coordinate pair.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Parse a `Coords` string — space-separated `x,y` pairs, as ZoneMinder stores
/// them (`"0,0 639,0 639,479 0,479"`).
///
/// Returns `None` rather than a partial polygon when anything does not parse:
/// a zone is a safety boundary, and silently computing an area from half the
/// points would be worse than refusing.
pub fn parse_coords(coords: &str) -> Option<Vec<Point>> {
    let points: Vec<Point> = coords
        .split_whitespace()
        .map(|pair| {
            let (x, y) = pair.split_once(',')?;
            Some(Point {
                x: x.trim().parse().ok()?,
                y: y.trim().parse().ok()?,
            })
        })
        .collect::<Option<Vec<_>>>()?;

    // Fewer than three points has no area, and is not a polygon.
    (points.len() >= 3).then_some(points)
}

/// Shoelace area, rounded, matching ZoneMinder's `getPolyArea`.
pub fn polygon_area(points: &[Point]) -> u32 {
    if points.len() < 3 {
        return 0;
    }
    let mut area = 0.0f64;
    for i in 0..points.len() {
        let j = (i + 1) % points.len();
        area += points[i].x * points[j].y - points[j].x * points[i].y;
    }
    (area.abs() / 2.0).round() as u32
}

/// Area for a `Coords` string, or `None` if it does not describe a polygon.
///
/// The caller decides what to do with `None` — a create/update should reject
/// the request rather than store a zero.
pub fn area_from_coords(coords: &str) -> Option<u32> {
    parse_coords(coords).map(|p| polygon_area(&p))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rectangle_is_width_times_height() {
        // ZoneMinder's default full-frame zone for a 640x480 monitor. Plain
        // shoelace over inclusive pixel coordinates gives 639*479, which is
        // what the web UI shows — not 640*480.
        let area = area_from_coords("0,0 639,0 639,479 0,479").unwrap();
        assert_eq!(area, 639 * 479);
    }

    #[test]
    fn winding_order_does_not_matter() {
        let clockwise = area_from_coords("0,0 100,0 100,50 0,50").unwrap();
        let anticlockwise = area_from_coords("0,50 100,50 100,0 0,0").unwrap();
        assert_eq!(clockwise, anticlockwise);
        assert_eq!(clockwise, 5000);
    }

    #[test]
    fn a_triangle_is_half_the_bounding_box() {
        assert_eq!(area_from_coords("0,0 100,0 0,100").unwrap(), 5000);
    }

    #[test]
    fn a_concave_polygon_excludes_its_notch() {
        // An L shape: a 100x100 square with a 50x50 corner removed.
        let area = area_from_coords("0,0 100,0 100,50 50,50 50,100 0,100").unwrap();
        assert_eq!(area, 10_000 - 2_500);
    }

    #[test]
    fn float_coordinates_are_accepted() {
        // ZoneMinder's own parser reads these as floats.
        assert_eq!(area_from_coords("0,0 10.5,0 10.5,10 0,10").unwrap(), 105);
    }

    #[test]
    fn malformed_input_yields_none_rather_than_a_wrong_area() {
        // Each of these would produce a plausible-looking number if parsed
        // leniently, which is the failure worth avoiding: a zone with a wrong
        // Area has thresholds that quietly mean something else.
        for bad in [
            "",
            "   ",
            "0,0",                    // one point
            "0,0 100,0",              // two points, no area
            "0,0 100,0 abc,50",       // non-numeric
            "0,0 100,0 100",          // missing y
            "0,0 100,0 100;50",       // wrong separator
            "0,0 100,0 100,50 extra", // trailing junk
        ] {
            assert_eq!(area_from_coords(bad), None, "{bad:?} should not parse");
        }
    }

    #[test]
    fn degenerate_but_valid_polygons_are_zero_not_none() {
        // Three collinear points parse fine and enclose nothing. That is a
        // real answer, distinct from "could not parse".
        assert_eq!(area_from_coords("0,0 50,0 100,0"), Some(0));
    }

    #[test]
    fn extra_whitespace_is_tolerated() {
        assert_eq!(
            area_from_coords("  0,0   100,0\t100,50   0,50  ").unwrap(),
            5000
        );
    }
}
