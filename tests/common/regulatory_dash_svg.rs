// 원본과 실제 선택 프로그램 metadata로 검증한다. 설치 글꼴은 필요 없다.
use rhwp::{renderer::font_environment::FontEnvironment, DocumentCore};
use std::{collections::BTreeMap, path::Path};

pub fn core() -> DocumentCore {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1891/86712_regulatory_analysis.hwpx");
    let mut core = DocumentCore::from_bytes(&std::fs::read(source).unwrap()).unwrap();
    core.set_hangul2024_compat(true);
    core.set_font_environment(Some(
        FontEnvironment::from_json(include_str!(
            "../../mydocs/pr/assets/issues7754_7765/source32/regulatory-hwpx-font-environment.json"
        ))
        .unwrap(),
    ))
    .unwrap();
    core
}
pub fn render_page_svg(page: u32) -> String {
    core()
        .render_page_svg_layer_with_profile_native(page, rhwp::paint::RenderProfile::Print)
        .unwrap()
}

pub fn hyphen_glyphs(svg: &str) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (attrs, rest) = chunk.split_at(end);
        if !rest.starts_with(">-</text>") {
            continue;
        }
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            tail[..stop].parse::<f64>().ok()
        };
        if let (Some(x), Some(y), Some(fs)) = (pick("x"), pick("y"), pick("font-size")) {
            out.push((y, x, fs));
        }
    }
    out
}

pub struct Run {
    pub baseline: f64,
    pub pts: Vec<(f64, f64)>,
}

impl Run {
    /// 글자당 advance (px).
    pub fn advance(&self) -> f64 {
        (self.pts[self.pts.len() - 1].0 - self.pts[0].0) / (self.pts.len() - 1) as f64
    }
    pub fn font_size(&self) -> f64 {
        self.pts[0].1
    }
}

/// 같은 baseline 에 놓인 하이픈을 이어 런으로 묶는다 (issue_5804 와 같은 방식).
pub fn hyphen_runs(svg: &str, min_len: usize) -> Vec<Run> {
    let mut by_line: BTreeMap<i64, Vec<(f64, f64)>> = BTreeMap::new();
    for (y, x, fs) in hyphen_glyphs(svg) {
        by_line
            .entry((y * 10.0).round() as i64)
            .or_default()
            .push((x, fs));
    }
    let mut runs = Vec::new();
    for (key, mut pts) in by_line {
        let baseline = key as f64 / 10.0;
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("좌표 비교"));
        let mut cur: Vec<(f64, f64)> = vec![pts[0]];
        for &(x, fs) in &pts[1..] {
            let last = cur[cur.len() - 1];
            // 과신장 회귀도 한 런으로 묶여야 검출된다 — 자연폭(≈0.6em)의 4배까지 허용.
            if x - last.0 < last.1 * 2.5 {
                cur.push((x, fs));
            } else {
                if cur.len() >= min_len {
                    runs.push(Run {
                        baseline,
                        pts: std::mem::take(&mut cur),
                    });
                }
                cur = vec![(x, fs)];
            }
        }
        if cur.len() >= min_len {
            runs.push(Run { baseline, pts: cur });
        }
    }
    runs
}
