//! Apply a monitor's `Orientation` to a still image (GH #61).
//!
//! ZoneMinder stores the rotation as monitor configuration and its own web UI
//! serves an already-rotated still, so every existing client assumes the image
//! it receives is upright. Handing back the raw frame instead fails silently:
//! no error, no wrong status, just a sideways picture that only a person
//! looking at the screen notices.
//!
//! Stills only. Rotating live video would mean re-encoding the stream, and a
//! client can do it in CSS for free.

use image::DynamicImage;

use crate::entity::sea_orm_active_enums::Orientation;

/// Whether this orientation changes the pixels at all.
///
/// `ROTATE_0` is overwhelmingly the common case, and the caller can then hand
/// back the bytes it read off disk without a decode/encode round trip.
pub fn is_identity(orientation: &Orientation) -> bool {
    matches!(orientation, Orientation::Rotate0)
}

/// Apply `orientation` to an encoded JPEG, returning re-encoded bytes.
///
/// Returns the input unchanged when the orientation is the identity, so the
/// common path stays zero-copy. Decoding is CPU-bound, so it runs on a blocking
/// thread rather than stalling the reactor.
///
/// A frame that cannot be decoded is returned as-is rather than failing the
/// request: an un-rotated thumbnail is a better outcome than no thumbnail, and
/// the reason is logged.
pub async fn orient_jpeg(data: Vec<u8>, orientation: Orientation) -> Vec<u8> {
    if is_identity(&orientation) {
        return data;
    }

    // Shared rather than cloned so the fallback path costs nothing: if the
    // blocking task panics or is cancelled we still hold the original bytes and
    // can serve them unrotated, instead of returning an empty body.
    let data = std::sync::Arc::new(data);
    let for_task = std::sync::Arc::clone(&data);
    let orientation_for_log = orientation.clone();

    match tokio::task::spawn_blocking(move || rotate_jpeg_blocking(&for_task, &orientation)).await {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!(
                "rotation task for {orientation_for_log:?} failed ({e}); \
                 serving the still unrotated"
            );
            std::sync::Arc::try_unwrap(data).unwrap_or_else(|arc| (*arc).clone())
        }
    }
}

/// Decode, transform, re-encode. Returns the original bytes on any failure so
/// the caller always has something to serve — an unrotated still beats none.
fn rotate_jpeg_blocking(data: &[u8], orientation: &Orientation) -> Vec<u8> {
    let img = match image::load_from_memory(data) {
        Ok(img) => img,
        Err(e) => {
            tracing::warn!("could not decode still for rotation: {e}");
            return data.to_vec();
        }
    };

    let rotated = apply(img, orientation);

    let mut out = std::io::Cursor::new(Vec::with_capacity(data.len()));
    match rotated.write_to(&mut out, image::ImageFormat::Jpeg) {
        Ok(()) => out.into_inner(),
        Err(e) => {
            tracing::warn!("could not re-encode rotated still: {e}");
            data.to_vec()
        }
    }
}

/// The pixel transform for each orientation ZoneMinder defines.
pub fn apply(img: DynamicImage, orientation: &Orientation) -> DynamicImage {
    match orientation {
        Orientation::Rotate0 => img,
        Orientation::Rotate90 => img.rotate90(),
        Orientation::Rotate180 => img.rotate180(),
        Orientation::Rotate270 => img.rotate270(),
        Orientation::FlipHori => img.fliph(),
        Orientation::FlipVert => img.flipv(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageFormat, RgbImage};

    /// A 4x2 image, so a 90-degree rotation is detectable by dimensions alone.
    fn jpeg_4x2() -> Vec<u8> {
        let img = DynamicImage::ImageRgb8(RgbImage::from_fn(4, 2, |x, _| {
            image::Rgb([(x * 60) as u8, 0, 0])
        }));
        let mut out = std::io::Cursor::new(Vec::new());
        img.write_to(&mut out, ImageFormat::Jpeg).unwrap();
        out.into_inner()
    }

    fn dimensions(bytes: &[u8]) -> (u32, u32) {
        let img = image::load_from_memory(bytes).expect("decode");
        (img.width(), img.height())
    }

    #[tokio::test]
    async fn rotate_0_returns_the_input_untouched() {
        // Not merely equal — the identical allocation, since the whole point is
        // to skip decode/encode for the common case.
        let original = jpeg_4x2();
        let out = orient_jpeg(original.clone(), Orientation::Rotate0).await;
        assert_eq!(out, original, "Rotate0 must not re-encode");
    }

    #[tokio::test]
    async fn quarter_turns_swap_the_dimensions() {
        let original = jpeg_4x2();
        assert_eq!(dimensions(&original), (4, 2));

        for orientation in [Orientation::Rotate90, Orientation::Rotate270] {
            let out = orient_jpeg(original.clone(), orientation.clone()).await;
            assert_eq!(
                dimensions(&out),
                (2, 4),
                "{orientation:?} must produce a portrait image"
            );
        }
    }

    #[tokio::test]
    async fn half_turn_and_flips_keep_the_dimensions() {
        let original = jpeg_4x2();
        for orientation in [
            Orientation::Rotate180,
            Orientation::FlipHori,
            Orientation::FlipVert,
        ] {
            let out = orient_jpeg(original.clone(), orientation.clone()).await;
            assert_eq!(
                dimensions(&out),
                (4, 2),
                "{orientation:?} must not swap width and height"
            );
        }
    }

    #[tokio::test]
    async fn a_horizontal_flip_actually_mirrors_the_pixels() {
        // Dimensions alone cannot tell fliph from a no-op, so check content.
        let original = jpeg_4x2();
        let before = image::load_from_memory(&original).unwrap().to_rgb8();
        let out = orient_jpeg(original, Orientation::FlipHori).await;
        let after = image::load_from_memory(&out).unwrap().to_rgb8();

        let left_before = before.get_pixel(0, 0)[0] as i16;
        let left_after = after.get_pixel(0, 0)[0] as i16;
        // The gradient runs dark-to-light left-to-right, so mirroring makes the
        // leftmost column markedly brighter. JPEG is lossy, hence the margin.
        assert!(
            left_after - left_before > 60,
            "fliph did not mirror: {left_before} -> {left_after}"
        );
    }

    #[tokio::test]
    async fn undecodable_input_is_returned_rather_than_failing() {
        // A truncated or non-JPEG file must still yield a response; an
        // unrotated still beats no still.
        let junk = b"not a jpeg at all".to_vec();
        let out = orient_jpeg(junk.clone(), Orientation::Rotate90).await;
        assert_eq!(out, junk);
    }
}
