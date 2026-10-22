//! `ZM_FFMPEG_INPUT_OPTIONS` and `ZM_FFMPEG_OUTPUT_OPTIONS` as encoder
//! settings.
//!
//! ZoneMinder appends these strings to an ffmpeg command line, so whoever
//! can write the Config table chooses ffmpeg's arguments: another input
//! (`-i`), a filter graph that reads or writes files, a second output. Here
//! they are parsed against an allowlist instead. What is kept only sets
//! encoder or muxer properties. Anything else is dropped and reported.

/// Encoder and muxer settings taken from the two option strings.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct EncodeOptions {
    /// `-c:v`: an encoder name, checked against the linked libavcodec when
    /// the video is made.
    pub codec: Option<String>,
    /// `-pix_fmt`.
    pub pix_fmt: Option<String>,
    /// Output `-r`.
    pub fps: Option<f64>,
    /// Input `-r` / `-framerate`: the rate capture JPEGs are read at.
    pub input_fps: Option<f64>,
    /// Encoder options, by libavcodec option name.
    pub encoder: Vec<(String, String)>,
    /// Muxer options, by libavformat option name.
    pub muxer: Vec<(String, String)>,
}

/// What an option sets.
#[derive(Clone, Copy)]
enum Kind {
    Codec,
    PixFmt,
    Fps,
    /// An encoder option, under this libavcodec name.
    Encoder(&'static str),
    /// A muxer option, under this libavformat name.
    Muxer(&'static str),
}

/// What a value must look like.
#[derive(Clone, Copy)]
enum Value {
    /// A number within the range.
    Number(f64, f64),
    /// A bit rate: digits with an optional `k`/`M` suffix.
    Rate,
    /// A name or flag list: `[A-Za-z0-9_.:+-]`.
    Word,
}

/// Options that take a value, what they set, and what the value must be.
fn lookup(flag: &str, input: bool) -> Option<(Kind, Value)> {
    use Kind::*;
    use Value::*;
    Some(match flag {
        "-r" | "-framerate" if input => (Fps, Number(0.1, 240.0)),
        "-r" => (Fps, Number(0.1, 240.0)),
        "-c:v" | "-vcodec" | "-codec:v" if !input => (Codec, Word),
        "-pix_fmt" if !input => (PixFmt, Word),
        "-preset" if !input => (Encoder("preset"), Word),
        "-tune" if !input => (Encoder("tune"), Word),
        "-profile:v" | "-profile" if !input => (Encoder("profile"), Word),
        "-level" if !input => (Encoder("level"), Word),
        "-crf" if !input => (Encoder("crf"), Number(0.0, 63.0)),
        "-qp" if !input => (Encoder("qp"), Number(0.0, 69.0)),
        "-b:v" | "-b" if !input => (Encoder("b"), Rate),
        "-maxrate" if !input => (Encoder("maxrate"), Rate),
        "-minrate" if !input => (Encoder("minrate"), Rate),
        "-bufsize" if !input => (Encoder("bufsize"), Rate),
        "-g" if !input => (Encoder("g"), Number(1.0, 10_000.0)),
        "-bf" if !input => (Encoder("bf"), Number(0.0, 16.0)),
        "-threads" if !input => (Encoder("threads"), Number(0.0, 64.0)),
        "-movflags" if !input => (Muxer("movflags"), Word),
        _ => return None,
    })
}

/// Flags without a value that change nothing here: the video never has
/// audio, subtitles or data, and the output is always replaced.
fn harmless_flag(flag: &str) -> bool {
    matches!(
        flag,
        "-an" | "-sn" | "-dn" | "-y" | "-nostdin" | "-hide_banner"
    )
}

fn valid(value: &str, rule: Value) -> bool {
    if value.is_empty() || value.len() > 32 {
        return false;
    }
    match rule {
        Value::Number(min, max) => value
            .parse::<f64>()
            .is_ok_and(|n| n.is_finite() && (min..=max).contains(&n)),
        Value::Rate => {
            let digits = value.trim_end_matches(['k', 'K', 'M']);
            !digits.is_empty()
                && digits.len() + 1 >= value.len()
                && digits.chars().all(|c| c.is_ascii_digit())
        }
        Value::Word => value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | ':' | '+' | '-')),
    }
}

fn apply(into: &mut EncodeOptions, kind: Kind, value: &str, input: bool) {
    match kind {
        Kind::Codec => into.codec = Some(value.to_string()),
        Kind::PixFmt => into.pix_fmt = Some(value.to_string()),
        Kind::Fps if input => into.input_fps = value.parse().ok(),
        Kind::Fps => into.fps = value.parse().ok(),
        Kind::Encoder(name) => set(&mut into.encoder, name, value),
        Kind::Muxer(name) => set(&mut into.muxer, name, value),
    }
}

/// A later setting replaces an earlier one, as on ffmpeg's command line.
fn set(list: &mut Vec<(String, String)>, name: &str, value: &str) {
    list.retain(|(k, _)| k != name);
    list.push((name.to_string(), value.to_string()));
}

fn parse_into(into: &mut EncodeOptions, dropped: &mut Vec<String>, text: &str, input: bool) {
    let mut tokens = text.split_whitespace().peekable();
    while let Some(flag) = tokens.next() {
        if harmless_flag(flag) {
            continue;
        }
        match lookup(flag, input) {
            Some((kind, rule)) => match tokens.next() {
                Some(value) if valid(value, rule) => apply(into, kind, value, input),
                Some(value) => dropped.push(format!("{flag} {value}")),
                None => dropped.push(flag.to_string()),
            },
            None => {
                // An unknown option: drop it and the value that follows it.
                match tokens.next_if(|t| !t.starts_with('-')) {
                    Some(value) => dropped.push(format!("{flag} {value}")),
                    None => dropped.push(flag.to_string()),
                }
            }
        }
    }
}

/// Parse `ZM_FFMPEG_INPUT_OPTIONS` and `ZM_FFMPEG_OUTPUT_OPTIONS`. Returns
/// the settings kept, and each option dropped as it was written.
pub fn parse(input: &str, output: &str) -> (EncodeOptions, Vec<String>) {
    let mut options = EncodeOptions::default();
    let mut dropped = Vec::new();
    parse_into(&mut options, &mut dropped, input, true);
    parse_into(&mut options, &mut dropped, output, false);
    (options, dropped)
}

/// For a Config update: `Err` names the options zm-api would drop.
pub fn check(name: &str, value: &str) -> Result<(), String> {
    let dropped = match name {
        "ZM_FFMPEG_INPUT_OPTIONS" => parse(value, "").1,
        "ZM_FFMPEG_OUTPUT_OPTIONS" => parse("", value).1,
        _ => return Ok(()),
    };
    if dropped.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{name}: not allowed: {}. Allowed: input -r/-framerate; output -r, -c:v, \
             -pix_fmt, -preset, -tune, -profile:v, -level, -crf, -qp, -b:v, -maxrate, \
             -minrate, -bufsize, -g, -bf, -threads, -movflags, -an",
            dropped.join(", ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zoneminders_default_sets_the_output_rate() {
        let (o, dropped) = parse("", "-r 25");
        assert_eq!(o.fps, Some(25.0));
        assert!(dropped.is_empty());
    }

    #[test]
    fn encoder_settings_are_kept() {
        let (o, dropped) = parse(
            "-framerate 5",
            "-c:v libx264 -preset veryfast -crf 23 -pix_fmt yuv420p -b:v 800k \
             -movflags +faststart -an -y",
        );
        assert!(dropped.is_empty(), "{dropped:?}");
        assert_eq!(o.input_fps, Some(5.0));
        assert_eq!(o.codec.as_deref(), Some("libx264"));
        assert_eq!(o.pix_fmt.as_deref(), Some("yuv420p"));
        assert_eq!(
            o.encoder,
            [
                ("preset".to_string(), "veryfast".to_string()),
                ("crf".to_string(), "23".to_string()),
                ("b".to_string(), "800k".to_string()),
            ]
        );
        assert_eq!(
            o.muxer,
            [("movflags".to_string(), "+faststart".to_string())]
        );
    }

    #[test]
    fn options_that_reach_files_or_programs_are_dropped() {
        let (o, dropped) = parse(
            "-i /etc/passwd -f concat",
            "-vf movie=/etc/shadow -x264-params dump-yuv=/tmp/x -passlogfile /tmp/p \
             -filter_complex x /tmp/second.mp4 -crf 23",
        );
        assert_eq!(o.encoder, [("crf".to_string(), "23".to_string())]);
        assert_eq!(
            dropped,
            [
                "-i /etc/passwd",
                "-f concat",
                "-vf movie=/etc/shadow",
                "-x264-params dump-yuv=/tmp/x",
                "-passlogfile /tmp/p",
                "-filter_complex x",
                "/tmp/second.mp4",
            ]
        );
    }

    #[test]
    fn values_must_be_plain() {
        let (o, dropped) = parse(
            "",
            "-c:v ../x -preset a;b -crf 99 -b:v 8kk -movflags $(id) -r 0",
        );
        assert_eq!(o, EncodeOptions::default());
        assert_eq!(dropped.len(), 6, "{dropped:?}");
    }

    #[test]
    fn input_only_takes_a_rate() {
        let (o, dropped) = parse("-c:v libx264 -r 7.5", "");
        assert_eq!(o.input_fps, Some(7.5));
        assert_eq!(o.codec, None);
        assert_eq!(dropped, ["-c:v libx264"]);
    }

    #[test]
    fn config_updates_are_checked() {
        assert!(check("ZM_FFMPEG_OUTPUT_OPTIONS", "-r 25 -crf 20").is_ok());
        let err = check("ZM_FFMPEG_OUTPUT_OPTIONS", "-r 25 -vf movie=/x").unwrap_err();
        assert!(err.contains("-vf movie=/x"), "{err}");
        assert!(check("ZM_FFMPEG_INPUT_OPTIONS", "-i x").is_err());
        assert!(
            check("ZM_LANG_DEFAULT", "-i x").is_ok(),
            "other options untouched"
        );
    }
}
