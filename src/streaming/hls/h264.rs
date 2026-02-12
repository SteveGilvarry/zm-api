//! H.264 bitstream parsing utilities
//!
//! Provides Exp-Golomb decoding and SPS (Sequence Parameter Set) parsing
//! to extract video dimensions from H.264 NAL units.

/// Bit-level reader for H.264 Exp-Golomb coded bitstreams.
pub struct BitReader<'a> {
    data: &'a [u8],
    byte_offset: usize,
    bit_offset: u8, // 0-7, bits consumed in current byte
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            byte_offset: 0,
            bit_offset: 0,
        }
    }

    /// Read a single bit, returning 0 or 1.
    pub fn read_bit(&mut self) -> Option<u32> {
        if self.byte_offset >= self.data.len() {
            return None;
        }
        let bit = (self.data[self.byte_offset] >> (7 - self.bit_offset)) & 1;
        self.bit_offset += 1;
        if self.bit_offset == 8 {
            self.bit_offset = 0;
            self.byte_offset += 1;
        }
        Some(bit as u32)
    }

    /// Read `n` bits (up to 32) as an unsigned integer.
    pub fn read_bits(&mut self, n: u8) -> Option<u32> {
        let mut value = 0u32;
        for _ in 0..n {
            value = (value << 1) | self.read_bit()?;
        }
        Some(value)
    }

    /// Skip `n` bits.
    pub fn skip_bits(&mut self, n: u32) -> Option<()> {
        for _ in 0..n {
            self.read_bit()?;
        }
        Some(())
    }

    /// Read an unsigned Exp-Golomb coded value (ue(v)).
    pub fn read_ue(&mut self) -> Option<u32> {
        let mut leading_zeros = 0u32;
        loop {
            let bit = self.read_bit()?;
            if bit == 1 {
                break;
            }
            leading_zeros += 1;
            if leading_zeros > 31 {
                return None; // protect against malformed data
            }
        }
        if leading_zeros == 0 {
            return Some(0);
        }
        let suffix = self.read_bits(leading_zeros as u8)?;
        Some((1 << leading_zeros) - 1 + suffix)
    }

    /// Read a signed Exp-Golomb coded value (se(v)).
    pub fn read_se(&mut self) -> Option<i32> {
        let code = self.read_ue()?;
        let value = (code.div_ceil(2)) as i32;
        if code % 2 == 0 {
            Some(-value)
        } else {
            Some(value)
        }
    }
}

/// Parse H.264 SPS NAL unit to extract video dimensions.
///
/// `sps` must be the raw SPS NAL data starting at the NAL type byte (0x67).
/// Returns `(width, height)` in pixels, or `None` if parsing fails.
pub fn parse_sps_dimensions(sps: &[u8]) -> Option<(u32, u32)> {
    if sps.len() < 4 {
        return None;
    }

    // Byte 0: nal_type (0x67 = SPS)
    // Byte 1: profile_idc
    // Byte 2: constraint_set flags
    // Byte 3: level_idc
    let profile_idc = sps[1];

    let mut reader = BitReader::new(&sps[4..]); // skip nal_type + profile + constraints + level

    // seq_parameter_set_id
    reader.read_ue()?;

    // High Profile and related profiles need extended parsing
    if matches!(
        profile_idc,
        100 | 110 | 122 | 244 | 44 | 83 | 86 | 118 | 128 | 138 | 139 | 134 | 135
    ) {
        let chroma_format_idc = reader.read_ue()?;
        if chroma_format_idc == 3 {
            reader.skip_bits(1)?; // separate_colour_plane_flag
        }
        reader.read_ue()?; // bit_depth_luma_minus8
        reader.read_ue()?; // bit_depth_chroma_minus8
        reader.skip_bits(1)?; // qpprime_y_zero_transform_bypass_flag

        let seq_scaling_matrix_present = reader.read_bit()?;
        if seq_scaling_matrix_present == 1 {
            let count = if chroma_format_idc != 3 { 8 } else { 12 };
            for i in 0..count {
                let present = reader.read_bit()?;
                if present == 1 {
                    let size = if i < 6 { 16 } else { 64 };
                    skip_scaling_list(&mut reader, size)?;
                }
            }
        }
    }

    // log2_max_frame_num_minus4
    reader.read_ue()?;

    let pic_order_cnt_type = reader.read_ue()?;
    match pic_order_cnt_type {
        0 => {
            reader.read_ue()?; // log2_max_pic_order_cnt_lsb_minus4
        }
        1 => {
            reader.skip_bits(1)?; // delta_pic_order_always_zero_flag
            reader.read_se()?; // offset_for_non_ref_pic
            reader.read_se()?; // offset_for_top_to_bottom_field
            let num_ref_frames_in_poc_cycle = reader.read_ue()?;
            for _ in 0..num_ref_frames_in_poc_cycle {
                reader.read_se()?; // offset_for_ref_frame[i]
            }
        }
        2 => {} // nothing extra
        _ => return None,
    }

    reader.read_ue()?; // max_num_ref_frames
    reader.skip_bits(1)?; // gaps_in_frame_num_value_allowed_flag

    let pic_width_in_mbs_minus1 = reader.read_ue()?;
    let pic_height_in_map_units_minus1 = reader.read_ue()?;

    let frame_mbs_only_flag = reader.read_bit()?;
    if frame_mbs_only_flag == 0 {
        reader.skip_bits(1)?; // mb_adaptive_frame_field_flag
    }

    reader.skip_bits(1)?; // direct_8x8_inference_flag

    let frame_cropping_flag = reader.read_bit()?;
    let (crop_left, crop_right, crop_top, crop_bottom) = if frame_cropping_flag == 1 {
        let l = reader.read_ue()?;
        let r = reader.read_ue()?;
        let t = reader.read_ue()?;
        let b = reader.read_ue()?;
        (l, r, t, b)
    } else {
        (0, 0, 0, 0)
    };

    // crop_unit_x and crop_unit_y depend on chroma format, but for 4:2:0 (most common):
    // crop_unit_x = 2, crop_unit_y = 2 * (2 - frame_mbs_only_flag)
    // For simplicity and correctness with the most common case:
    let crop_unit_x = 2u32;
    let crop_unit_y = 2 * (2 - frame_mbs_only_flag);

    let width = (pic_width_in_mbs_minus1 + 1) * 16 - crop_unit_x * (crop_left + crop_right);
    let height = (2 - frame_mbs_only_flag) * (pic_height_in_map_units_minus1 + 1) * 16
        - crop_unit_y * (crop_top + crop_bottom);

    Some((width, height))
}

/// Skip a scaling list in the SPS bitstream.
fn skip_scaling_list(reader: &mut BitReader, size: usize) -> Option<()> {
    let mut last_scale = 8i32;
    let mut next_scale = 8i32;
    for _ in 0..size {
        if next_scale != 0 {
            let delta_scale = reader.read_se()?;
            next_scale = (last_scale + delta_scale + 256) % 256;
        }
        last_scale = if next_scale == 0 {
            last_scale
        } else {
            next_scale
        };
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_reader_read_bits() {
        let data = [0b10110100, 0b01100000];
        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_bits(4), Some(0b1011));
        assert_eq!(reader.read_bits(4), Some(0b0100));
        assert_eq!(reader.read_bits(4), Some(0b0110));
    }

    #[test]
    fn test_exp_golomb_ue() {
        // ue(0) = 1 (binary: 1)
        // ue(1) = 010 (binary)
        // ue(2) = 011
        // ue(3) = 00100
        let data = [0b1010_0110, 0b0100_0000];
        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_ue(), Some(0)); // 1
        assert_eq!(reader.read_ue(), Some(1)); // 010
        assert_eq!(reader.read_ue(), Some(2)); // 011
        assert_eq!(reader.read_ue(), Some(3)); // 00100
    }

    #[test]
    fn test_exp_golomb_se() {
        // se(0) = ue(0) = 1 → 0
        // se(1) = ue(1) = 010 → +1
        // se(-1) = ue(2) = 011 → -1
        let data = [0b1010_0110];
        let mut reader = BitReader::new(&data);
        assert_eq!(reader.read_se(), Some(0));
        assert_eq!(reader.read_se(), Some(1));
        assert_eq!(reader.read_se(), Some(-1));
    }

    /// Real SPS bytes generated by ffmpeg for 1920x1080 (High profile).
    /// ffmpeg -f lavfi -i "color=c=black:s=1920x1080:r=25" -frames:v 1 -c:v libx264 -f h264
    fn sps_1920x1080_high() -> Vec<u8> {
        vec![
            0x67, 0x64, 0x00, 0x28, 0xAC, 0xD9, 0x40, 0x78, 0x02, 0x27, 0xE5, 0xC0, 0x44, 0x00,
            0x00, 0x03, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0xC8, 0x3C, 0x60, 0xC6, 0x58,
        ]
    }

    /// Real SPS bytes generated by ffmpeg for 1920x1080 (Baseline profile).
    /// ffmpeg -f lavfi -i "color=c=black:s=1920x1080:r=25" -frames:v 1 -c:v libx264 -profile:v baseline -f h264
    fn sps_1920x1080_baseline() -> Vec<u8> {
        vec![
            0x67, 0x42, 0xC0, 0x28, 0xD9, 0x00, 0x78, 0x02, 0x27, 0xE5, 0xC0, 0x44, 0x00, 0x00,
            0x03, 0x00, 0x04, 0x00, 0x00, 0x03, 0x00, 0xC8, 0x3C, 0x60, 0xC9, 0x20,
        ]
    }

    /// Real SPS bytes generated by ffmpeg for 3840x2160 (High profile, 4K).
    fn sps_3840x2160() -> Vec<u8> {
        vec![
            0x67, 0x64, 0x00, 0x33, 0xAC, 0xD9, 0x40, 0x3C, 0x00, 0x43, 0xEC, 0x04, 0x40, 0x00,
            0x00, 0x03, 0x00, 0x40, 0x00, 0x00, 0x0C, 0x83, 0xC6, 0x0C, 0x65, 0x80,
        ]
    }

    /// Real SPS bytes generated by ffmpeg for 1280x720 (High profile).
    fn sps_1280x720() -> Vec<u8> {
        vec![
            0x67, 0x64, 0x00, 0x1F, 0xAC, 0xD9, 0x40, 0x50, 0x05, 0xBB, 0x01, 0x10, 0x00, 0x00,
            0x03, 0x00, 0x10, 0x00, 0x00, 0x03, 0x03, 0x20, 0xF1, 0x83, 0x19, 0x60,
        ]
    }

    /// Real SPS bytes generated by ffmpeg for 2560x1440 (High profile).
    fn sps_2560x1440() -> Vec<u8> {
        vec![
            0x67, 0x64, 0x00, 0x32, 0xAC, 0xD9, 0x40, 0x28, 0x00, 0xB5, 0xB0, 0x11, 0x00, 0x00,
            0x03, 0x00, 0x01, 0x00, 0x00, 0x03, 0x00, 0x32, 0x0F, 0x18, 0x31, 0x96,
        ]
    }

    #[test]
    fn test_sps_parsing_1920x1080_high() {
        let dims = parse_sps_dimensions(&sps_1920x1080_high());
        assert_eq!(dims, Some((1920, 1080)));
    }

    #[test]
    fn test_sps_parsing_1920x1080_baseline() {
        let dims = parse_sps_dimensions(&sps_1920x1080_baseline());
        assert_eq!(dims, Some((1920, 1080)));
    }

    #[test]
    fn test_sps_parsing_3840x2160() {
        let dims = parse_sps_dimensions(&sps_3840x2160());
        assert_eq!(dims, Some((3840, 2160)));
    }

    #[test]
    fn test_sps_parsing_1280x720() {
        let dims = parse_sps_dimensions(&sps_1280x720());
        assert_eq!(dims, Some((1280, 720)));
    }

    #[test]
    fn test_sps_parsing_2560x1440() {
        let dims = parse_sps_dimensions(&sps_2560x1440());
        assert_eq!(dims, Some((2560, 1440)));
    }

    #[test]
    fn test_sps_parsing_too_short() {
        assert_eq!(parse_sps_dimensions(&[0x67, 0x42, 0x00]), None);
    }

    #[test]
    fn test_sps_parsing_empty() {
        assert_eq!(parse_sps_dimensions(&[]), None);
    }
}
