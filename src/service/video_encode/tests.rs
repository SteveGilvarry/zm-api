use super::*;

/// `n` capture JPEGs of `w`x`h`, red, numbered from 1 as zmc writes them.
fn jpegs(dir: &Path, n: usize, w: u32, h: u32) {
    for i in 1..=n {
        let img = image::RgbImage::from_pixel(w, h, image::Rgb([200, 20, 20]));
        img.save(dir.join(format!("{i:05}-capture.jpg"))).unwrap();
    }
}

fn job(source: Source, output: PathBuf) -> Render {
    Render {
        source,
        speed: None,
        fps: None,
        width: 64,
        height: 48,
        greyscale: false,
        options: EncodeOptions::default(),
        output,
    }
}

/// Width, height and frame count of a video file's video stream.
fn probe(path: &Path) -> (u32, u32, u64) {
    let mut ictx = format::input(path).unwrap();
    let ist = ictx.streams().best(media::Type::Video).unwrap();
    let index = ist.index();
    let mut dec = codec::context::Context::from_parameters(ist.parameters())
        .unwrap()
        .decoder()
        .video()
        .unwrap();
    let (w, h) = (dec.width(), dec.height());
    let mut n = 0;
    let mut f = frame::Video::empty();
    for (s, p) in ictx.packets() {
        if s.index() == index {
            dec.send_packet(&p).unwrap();
            while dec.receive_frame(&mut f).is_ok() {
                n += 1;
            }
        }
    }
    dec.send_eof().unwrap();
    while dec.receive_frame(&mut f).is_ok() {
        n += 1;
    }
    (w, h, n)
}

fn no_cancel() -> AtomicBool {
    AtomicBool::new(false)
}

#[test]
fn jpegs_become_an_mp4_at_their_rate() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 10, 80, 60);
    let out = dir.path().join("e.mp4");
    let src = Source::Jpegs {
        dir: dir.path().into(),
        digits: 5,
        fps: 5.0,
    };
    let frames = render(&job(src, out.clone()), &no_cancel()).unwrap();
    assert_eq!(frames, 10);
    assert_eq!(probe(&out), (64, 48, 10));
}

#[test]
fn an_output_rate_repeats_frames() {
    // ZoneMinder's default ZM_FFMPEG_OUTPUT_OPTIONS is `-r 25`.
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 5, 64, 48);
    let out = dir.path().join("e.mp4");
    let mut j = job(
        Source::Jpegs {
            dir: dir.path().into(),
            digits: 5,
            fps: 5.0,
        },
        out.clone(),
    );
    j.options = options::parse("", "-r 25").0;
    let frames = render(&j, &no_cancel()).unwrap();
    assert!((20..=26).contains(&frames), "{frames} frames for 1 s at 25");
}

#[test]
fn greyscale_takes_the_colour_out() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 3, 64, 48);
    let out = dir.path().join("g.mp4");
    let mut j = job(
        Source::Jpegs {
            dir: dir.path().into(),
            digits: 5,
            fps: 5.0,
        },
        out.clone(),
    );
    j.greyscale = true;
    j.options.pix_fmt = Some("yuv420p".into());
    render(&j, &no_cancel()).unwrap();
    // Chroma planes sit at 128 (no colour) once decoded.
    let mut ictx = format::input(&out).unwrap();
    let ist = ictx.streams().best(media::Type::Video).unwrap();
    let index = ist.index();
    let mut dec = codec::context::Context::from_parameters(ist.parameters())
        .unwrap()
        .decoder()
        .video()
        .unwrap();
    let mut f = frame::Video::empty();
    let mut checked = false;
    for (s, p) in ictx.packets() {
        if s.index() != index {
            continue;
        }
        dec.send_packet(&p).unwrap();
        if dec.receive_frame(&mut f).is_ok() {
            let u = f.data(1)[0];
            assert!((120..=136).contains(&u), "chroma {u}");
            checked = true;
            break;
        }
    }
    if !checked {
        dec.send_eof().unwrap();
        dec.receive_frame(&mut f).unwrap();
        let u = f.data(1)[0];
        assert!((120..=136).contains(&u), "chroma {u}");
    }
}

#[test]
fn other_containers_use_their_own_encoders() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 6, 64, 48);
    for ext in ["avi", "mpg", "mov", "mkv"] {
        let out = dir.path().join(format!("e.{ext}"));
        let src = Source::Jpegs {
            dir: dir.path().into(),
            digits: 5,
            fps: 5.0,
        };
        // MPEG-1 only takes standard rates; 5 fps becomes the nearest.
        render(&job(src, out.clone()), &no_cancel()).unwrap_or_else(|e| panic!("{ext}: {e}"));
        let (w, h, n) = probe(&out);
        assert_eq!((w, h), (64, 48), "{ext}");
        assert!(n >= 6, "{ext}: {n} frames");
    }
}

#[test]
fn recordings_are_retimed() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 20, 64, 48);
    let rec = dir.path().join("1-video.mp4");
    let src = Source::Jpegs {
        dir: dir.path().into(),
        digits: 5,
        fps: 10.0,
    };
    render(&job(src, rec.clone()), &no_cancel()).unwrap();

    let out = dir.path().join("fast.mp4");
    let mut j = job(Source::Recording(rec), out.clone());
    j.speed = Some(2.0);
    j.width = 32;
    j.height = 24;
    let frames = render(&j, &no_cancel()).unwrap();
    // As zmvideo.pl: twice as fast keeps every frame at twice the rate, so
    // 2 s of recording plays in 1 s.
    assert!((19..=21).contains(&frames), "{frames}");
    let (w, h, _) = probe(&out);
    assert_eq!((w, h), (32, 24));
    let secs = format::input(&out).unwrap().duration() as f64 / 1e6;
    assert!((0.9..=1.15).contains(&secs), "{secs} s");
}

#[test]
fn a_recording_cannot_point_elsewhere() {
    let dir = Path::new("/var/cache/zm/events/1/2026-10-23/5");
    assert!(file_in(dir, "5-video.mp4").is_ok());
    for bad in [
        "",
        "..",
        "../../x.mp4",
        "/etc/passwd",
        "http://x/a.mp4",
        "concat:a|b",
        "-i",
        "a b.mp4",
    ] {
        assert!(file_in(dir, bad).is_err(), "{bad:?}");
    }
}

#[test]
fn a_playlist_cannot_pull_in_other_files() {
    // A file in the event folder that is really a playlist naming another
    // file: the demuxer is limited to recording containers and local files.
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 3, 64, 48);
    let secret = dir.path().join("secret.mp4");
    let src = Source::Jpegs {
        dir: dir.path().into(),
        digits: 5,
        fps: 5.0,
    };
    render(&job(src, secret.clone()), &no_cancel()).unwrap();
    let list = dir.path().join("list.mp4");
    std::fs::write(
        &list,
        format!("ffconcat version 1.0\nfile '{}'\n", secret.display()),
    )
    .unwrap();
    let j = job(Source::Recording(list), dir.path().join("out.mp4"));
    assert!(render(&j, &no_cancel()).is_err());

    let m3u = dir.path().join("hls.mp4");
    std::fs::write(&m3u, "#EXTM3U\n#EXTINF:1,\nhttp://127.0.0.1:9/x.ts\n").unwrap();
    let j = job(Source::Recording(m3u), dir.path().join("out2.mp4"));
    assert!(render(&j, &no_cancel()).is_err());
}

#[test]
fn an_unknown_encoder_is_an_error_not_a_fallback() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 2, 64, 48);
    let mut j = job(
        Source::Jpegs {
            dir: dir.path().into(),
            digits: 5,
            fps: 5.0,
        },
        dir.path().join("e.mp4"),
    );
    j.options.codec = Some("nosuchcodec".into());
    let e = render(&j, &no_cancel()).unwrap_err();
    assert!(e.contains("nosuchcodec"), "{e}");
}

#[test]
fn cancel_stops_between_frames() {
    let dir = tempfile::tempdir().unwrap();
    jpegs(dir.path(), 3, 64, 48);
    let src = Source::Jpegs {
        dir: dir.path().into(),
        digits: 5,
        fps: 5.0,
    };
    let e = render(&job(src, dir.path().join("e.mp4")), &AtomicBool::new(true)).unwrap_err();
    assert_eq!(e, "cancelled");
}

#[test]
fn no_jpegs_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let src = Source::Jpegs {
        dir: dir.path().into(),
        digits: 5,
        fps: 5.0,
    };
    assert!(render(&job(src, dir.path().join("e.mp4")), &no_cancel()).is_err());
}
