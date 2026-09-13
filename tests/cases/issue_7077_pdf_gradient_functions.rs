//! #7077/#7078: PDF reader limits must not reduce source gradient colors or stops.
//! Exercise the production SVG-to-PDF chunk path; evaluate its actual PDF functions
//! at source interval midpoints and preserve the original HWP step=100/255 contract.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::{BTreeMap, BTreeSet};

fn array(dict: &str, key: &str) -> Vec<f64> {
    dict.split_once(&format!("/{key} ["))
        .expect(key)
        .1
        .split_once(']')
        .expect("array end")
        .0
        .split_whitespace()
        .filter(|s| *s != "R")
        .map(|s| s.parse().expect("PDF number"))
        .collect()
}

fn evaluate(id: u32, x: f64, functions: &BTreeMap<u32, String>, depth: usize) -> Vec<f64> {
    assert!(depth < 10, "unbounded function recursion");
    let f = &functions[&id];
    let domain = array(f, "Domain");
    let x = x.clamp(domain[0], domain[1]);
    if f.contains("/FunctionType 2") {
        let a = array(f, "C0");
        let b = array(f, "C1");
        return a.iter().zip(b).map(|(a, b)| a + x * (b - a)).collect();
    }
    let refs = array(f, "Functions");
    let bounds = array(f, "Bounds");
    let encode = array(f, "Encode");
    let count = refs.len() / 2;
    assert!(
        (1..=256).contains(&count),
        "reader rejects {count} subfunctions"
    );
    assert_eq!(bounds.len(), count - 1);
    assert_eq!(encode.len(), count * 2);
    let i = bounds.iter().position(|b| x < *b).unwrap_or(count - 1);
    let low = if i == 0 { domain[0] } else { bounds[i - 1] };
    let high = if i == count - 1 { domain[1] } else { bounds[i] };
    let mapped = if high == low {
        encode[2 * i]
    } else {
        encode[2 * i] + (x - low) / (high - low) * (encode[2 * i + 1] - encode[2 * i])
    };
    evaluate(refs[2 * i] as u32, mapped, functions, depth + 1)
}

fn check_gradient(stops: &[(f64, [u8; 3], f64)], radial: bool) {
    let tag = if radial {
        "radialGradient"
    } else {
        "linearGradient"
    };
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="1024" height="64"><defs><{tag} id="g">"#
    );
    for (offset, rgb, opacity) in stops {
        svg.push_str(&format!(
            r#"<stop offset="{offset}" stop-color="rgb({},{},{})" stop-opacity="{opacity}"/>"#,
            rgb[0], rgb[1], rgb[2]
        ));
    }
    svg.push_str(&format!(
        r##"</{tag}></defs><rect width="1024" height="64" fill="url(#g)"/></svg>"##
    ));
    let tree = usvg::Tree::from_str(&svg, &usvg::Options::default()).expect("SVG");
    let (chunk, _) =
        rhwp::renderer::pdf::svg2pdf_to_chunk(&tree, false).expect("production PDF chunk");
    let text = String::from_utf8_lossy(chunk.as_bytes());
    let mut functions = BTreeMap::new();
    for object in text.split("endobj") {
        let Some((header, body)) = object.split_once(" 0 obj") else {
            continue;
        };
        let body = body.split("stream").next().unwrap();
        if body.contains("/FunctionType ") {
            let id = header
                .split_whitespace()
                .last()
                .unwrap()
                .parse::<u32>()
                .unwrap();
            functions.insert(id, body.to_string());
        }
    }
    assert!(!functions.is_empty());
    let mut children = BTreeSet::new();
    for f in functions.values().filter(|f| f.contains("/FunctionType 3")) {
        let refs = array(f, "Functions");
        assert!(refs.len() / 2 <= 256, "PDF reader subfunction limit");
        children.extend(refs.chunks_exact(2).map(|r| r[0] as u32));
    }
    let roots: Vec<_> = functions
        .keys()
        .filter(|id| !children.contains(id))
        .copied()
        .collect();
    assert_eq!(
        roots.len(),
        if stops.iter().any(|s| s.2 < 1.0) {
            2
        } else {
            1
        }
    );
    for pair in stops.windows(2) {
        // usvg separates duplicate hard-edge offsets by epsilon. Sample each
        // nonzero source interval well inside its bounds, not that epsilon ramp.
        if pair[1].0 - pair[0].0 < 0.00001 {
            continue;
        }
        let x = (pair[0].0 + pair[1].0) / 2.0;
        for id in &roots {
            let actual = evaluate(*id, x, &functions, 0);
            let expected = if actual.len() == 1 {
                vec![(pair[0].2 + pair[1].2) / 2.0]
            } else {
                (0..3)
                    .map(|c| (f64::from(pair[0].1[c]) + f64::from(pair[1].1[c])) / 510.0)
                    .collect()
            };
            for (a, e) in actual.iter().zip(expected) {
                assert!((a - e).abs() < 0.0001, "x={x}: {a} != {e}");
            }
        }
    }
}

#[test]
fn pdf_preserves_every_hwp_band_and_shifted_boundary() {
    for count in [100, 129, 255, 1024] {
        for center in [0, 8, 50, 92, 100] {
            let (colors, positions) =
                rhwp::renderer::expand_gradient_steps(&[0, 0xffffff], &[0.0, 1.0], count, center);
            assert_eq!(
                colors.len(),
                count as usize * 2,
                "must not cap common bands"
            );
            let stops: Vec<_> = colors
                .iter()
                .zip(positions)
                .map(|(c, p)| (p, [*c as u8, (*c >> 8) as u8, (*c >> 16) as u8], 1.0))
                .collect();
            check_gradient(&stops, false);
            check_gradient(&stops, true);
        }
    }
}

#[test]
fn pdf_preserves_large_multicolor_ramps_and_opacity_masks() {
    // 256 subfunctions is the last flat case; 257 is the first nested case.
    for intervals in [1, 256, 257, 1024] {
        let stops: Vec<_> = (0..=intervals)
            .map(|i| {
                let x = i as f64 / intervals as f64;
                (
                    x,
                    [
                        (i % 251) as u8,
                        ((i * 7) % 253) as u8,
                        ((i * 13) % 255) as u8,
                    ],
                    0.2 + 0.8 * x,
                )
            })
            .collect();
        check_gradient(&stops, false);
        check_gradient(&stops, true);
    }
}
