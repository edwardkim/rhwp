//! Kitty graphics 전송·CLI 계약. 실물 조판 golden을 만들지 않는다.

#[cfg(not(target_arch = "wasm32"))]
fn run(args: &[&str]) -> std::process::Output {
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    std::process::Command::new(bin).args(args).output().unwrap()
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-skia")))]
#[test]
fn missing_native_feature_reports_usage_error() {
    let output = run(&["export-kitty", "unused.hwp"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("native-skia"));
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
mod native {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    use rhwp::renderer::kitty::{encode_kitty_png, write_kitty_png};
    use rhwp::renderer::layer_renderer::RasterColorSpace;
    use rhwp::renderer::raster_pixels::{RasterPixelFormat, RasterPixels};
    use std::collections::BTreeMap;
    use std::io::{self, Write};

    fn packets(stream: &[u8]) -> Vec<(BTreeMap<&str, &str>, &str)> {
        let mut rest = std::str::from_utf8(stream)
            .unwrap()
            .strip_suffix("\r\n")
            .unwrap();
        let mut packets = Vec::new();
        while !rest.is_empty() {
            rest = rest.strip_prefix("\x1b_G").expect("APC prefix");
            let end = rest.find("\x1b\\").expect("APC terminator");
            let (header, payload) = rest[..end].split_once(';').unwrap();
            let mut keys = BTreeMap::new();
            for item in header.split(',') {
                let (key, value) = item.split_once('=').unwrap();
                assert!(keys.insert(key, value).is_none(), "duplicate key {key}");
            }
            packets.push((keys, payload));
            rest = &rest[end + 2..];
        }
        packets
    }

    #[test]
    fn chunk_boundaries_padding_and_continuation_follow_specification() {
        // Arbitrary binary payload tests framing only, not PNG validity.
        for length in [1usize, 2, 3, 3071, 3072, 3073, 6144, 6145] {
            let data: Vec<u8> = (0..length).map(|n| (n % 256) as u8).collect();
            let mut output = Vec::new();
            write_kitty_png(&data, &mut output).unwrap();
            let chunks = packets(&output);
            assert_eq!(chunks.len(), length.div_ceil(3072));
            let mut encoded = String::new();
            for (i, (keys, payload)) in chunks.iter().enumerate() {
                assert!(payload.len() <= 4096);
                assert_eq!(keys["q"], "2");
                assert_eq!(keys["m"], if i + 1 == chunks.len() { "0" } else { "1" });
                if i == 0 {
                    assert_eq!(keys.len(), 5);
                    assert_eq!(keys["a"], "T");
                    assert_eq!(keys["f"], "100");
                    assert_eq!(keys["t"], "d");
                } else {
                    assert_eq!(keys.len(), 2, "continuations contain only m/q");
                }
                if i + 1 < chunks.len() {
                    assert_eq!(payload.len() % 4, 0);
                    assert!(!payload.contains('='));
                }
                encoded.push_str(payload);
            }
            assert_eq!(STANDARD.decode(&encoded).unwrap(), data);
        }
    }

    #[test]
    fn empty_payload_is_rejected_before_any_output() {
        let mut output = Vec::new();
        assert_eq!(
            write_kitty_png(&[], &mut output).unwrap_err().kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(output.is_empty());
    }

    struct BrokenWriter {
        remaining: usize,
    }

    impl Write for BrokenWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.remaining == 0 {
                return Err(io::Error::from(io::ErrorKind::BrokenPipe));
            }
            let count = self.remaining.min(bytes.len());
            self.remaining -= count;
            Ok(count)
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn io_failure_stops_header_payload_and_continuation_writes() {
        for remaining in [1, 30, 4096, 5000] {
            let mut writer = BrokenWriter { remaining };
            assert_eq!(
                write_kitty_png(&vec![17; 6145], &mut writer)
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::BrokenPipe
            );
            assert_eq!(writer.remaining, 0);
        }
    }

    fn pixels(bytes: Vec<u8>, width: i32, height: i32, stride: usize) -> RasterPixels {
        RasterPixels {
            bytes,
            width,
            height,
            stride,
            format: RasterPixelFormat::Rgba8Premultiplied,
            color_space: RasterColorSpace::Srgb,
            dpi: None,
        }
    }

    #[test]
    fn png_preserves_channels_stride_and_white_alpha_composition() {
        let source = pixels(
            vec![
                128, 0, 0, 128, 0, 0, 0, 0, 42, 42, 42, 42, 0, 64, 0, 64, 0, 0, 255, 255, 42, 42,
                42, 42,
            ],
            2,
            2,
            12,
        );
        let png = encode_kitty_png(&source).unwrap();
        let decoded = image::load_from_memory(&png).unwrap().to_rgba8();
        assert_eq!(decoded.dimensions(), (2, 2));
        assert_eq!(
            decoded.as_raw(),
            &[255, 127, 127, 255, 255, 255, 255, 255, 191, 255, 191, 255, 0, 0, 255, 255]
        );
        let mut stream = Vec::new();
        write_kitty_png(&png, &mut stream).unwrap();
        let joined: String = packets(&stream)
            .iter()
            .map(|(_, payload)| *payload)
            .collect();
        assert_eq!(STANDARD.decode(joined).unwrap(), png);
    }

    #[test]
    fn encoder_rejects_bad_raster_and_excessive_width() {
        for source in [
            pixels(Vec::new(), 0, 1, 0),
            pixels(vec![0; 3], 1, 1, 3),
            pixels(vec![1, 0, 0, 0], 1, 1, 4),
            pixels(vec![0; 16385 * 4], 16385, 1, 16385 * 4),
        ] {
            assert!(encode_kitty_png(&source).is_err());
        }
    }

    #[test]
    fn nonexistent_input_is_runtime_error_without_graphics() {
        let output = super::run(&["export-kitty", "/definitely-missing-8150/document.hwp"]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cli_declares_kitty_feature_and_options() {
    let output = run(&["export-kitty", "--help"]);
    assert!(output.status.success(), "Kitty command help must exist");
    let help = String::from_utf8(output.stdout).unwrap();
    for expected in ["--width", "--page", "--output", "--font-path", "stdout"] {
        assert!(help.contains(expected), "missing {expected}");
    }
    let output = run(&["capabilities"]);
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let entry = value["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "export-kitty")
        .expect("Kitty capability must exist");
    assert_eq!(entry["requiresFeature"], "native-skia");
    assert_eq!(entry["available"], cfg!(feature = "native-skia"));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cli_invalid_options_have_empty_stdout() {
    for args in [
        vec!["export-kitty"],
        vec!["export-kitty", "--width", "0"],
        vec!["export-kitty", "--width", "16385"],
        vec!["export-kitty", "--page", "-1"],
        vec!["export-kitty", "--width", "NaN"],
        vec!["export-kitty", "--page"],
        vec!["export-kitty", "--output"],
        vec!["export-kitty", "--json"],
        vec!["export-kitty", "a.hwp", "b.hwpx"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}
