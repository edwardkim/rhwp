//! 원문 주소·전진을 유지하면서 방향별 구간의 실제 표시 원점을 구한다.
use super::render_tree::{FieldMarkerType, RenderNode, RenderNodeType};
use std::borrow::Cow;
use unicode_bidi::{bidi_class, BidiClass, BidiInfo, Level};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct BidiVisualRun {
    start: usize,
    end: usize,
    x: f64,
    rtl: bool,
}

/// 한 줄에서 해소한 방향을 런의 표시 스칼라 주소에 투영한 결과.
/// 논리 전진과 원문 주소는 바꾸지 않는다.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BidiLayout {
    scalar_count: usize,
    logical_width: f64,
    #[serde(skip)]
    source_text: Box<str>,
    #[serde(skip)]
    source_positions: Box<[f64]>,
    runs: Box<[BidiVisualRun]>,
}

fn has_rtl(text: &str) -> bool {
    text.chars().any(|ch| {
        matches!(
            bidi_class(ch),
            BidiClass::R | BidiClass::AL | BidiClass::RLE | BidiClass::RLO | BidiClass::RLI
        )
    })
}

pub(crate) struct VisualTextPositions<'a> {
    positions: &'a [f64],
    runs: Cow<'a, [BidiVisualRun]>,
}

impl<'a> VisualTextPositions<'a> {
    pub(crate) fn new(text: &str, positions: &'a [f64]) -> Option<Self> {
        if !has_rtl(text) {
            return None;
        }
        let mut offsets: Vec<_> = text.char_indices().map(|(at, _)| at).collect();
        offsets.push(text.len());
        if offsets.len() != positions.len() {
            return None;
        }
        // 현재 수평 문단의 기본 흐름은 LTR이다. 뒤 공백은 본문 바깥으로 뒤집지 않는다.
        let bidi = BidiInfo::new(text, Some(Level::ltr()));
        let mut runs = Vec::new();
        for para in &bidi.paragraphs {
            let (levels, visual_runs) = bidi.visual_runs(para, para.range.clone());
            let start = offsets.binary_search(&para.range.start).ok()?;
            let mut x = positions[start];
            for run in visual_runs {
                let start = offsets.binary_search(&run.start).ok()?;
                let end = offsets.binary_search(&run.end).ok()?;
                runs.push(BidiVisualRun {
                    start,
                    end,
                    x,
                    rtl: levels[run.start].is_rtl(),
                });
                x += positions[end] - positions[start];
            }
        }
        Some(Self {
            positions,
            runs: Cow::Owned(runs),
        })
    }

    /// 표시 문자열이나 전진 배열이 달라지면 같은 길이·폭이어도 국소 해석으로 되돌린다.
    pub(crate) fn for_style(
        text: &str,
        positions: &'a [f64],
        style: &'a super::TextStyle,
    ) -> Option<Self> {
        if let Some(layout) = &style.bidi_layout {
            if layout.source_text.as_ref() == text && layout.source_positions.as_ref() == positions
            {
                return Some(Self {
                    positions,
                    runs: Cow::Borrowed(&layout.runs),
                });
            }
        }
        Self::new(text, positions)
    }

    pub(crate) fn span_origin(&self, start: usize, end: usize) -> Option<f64> {
        if start >= end || end >= self.positions.len() {
            return None;
        }
        let run = self
            .runs
            .iter()
            .find(|run| start >= run.start && end <= run.end)?;
        Some(if run.rtl {
            run.x + self.positions[run.end] - self.positions[end]
        } else {
            run.x + self.positions[start] - self.positions[run.start]
        })
    }

    pub(crate) fn span_scope(&self, start: usize, end: usize) -> Option<(usize, usize, bool)> {
        self.runs
            .iter()
            .find(|r| start >= r.start && end <= r.end)
            .map(|r| (r.start, r.end, r.rtl))
    }

    /// shaping하지 않고 그리는 스칼라만 거울 글자로 바꾼다. 원문과 전진은 유지한다.
    pub(crate) fn mirrored<'b>(&self, text: &'b str, start: usize) -> Cow<'b, str> {
        if self
            .span_scope(start, start + text.chars().count())
            .is_some_and(|(_, _, rtl)| rtl)
            && text
                .chars()
                .any(|c| unicode_bidi_mirroring::get_mirrored(c).is_some())
        {
            Cow::Owned(
                text.chars()
                    .map(|c| unicode_bidi_mirroring::get_mirrored(c).unwrap_or(c))
                    .collect(),
            )
        } else {
            Cow::Borrowed(text)
        }
    }

    /// 같은 원문 범위가 실제로 점유하는 표시 구간. 배경·장식·paint bounds도 공유한다.
    pub(crate) fn segments(&self, end: usize) -> Vec<(f64, f64)> {
        self.runs
            .iter()
            .filter_map(|run| {
                let last = run.end.min(end);
                if run.start >= last {
                    return None;
                }
                let x = self.span_origin(run.start, last)?;
                Some((x, x + self.positions[last] - self.positions[run.start]))
            })
            .collect()
    }
}

/// 원문·서식 런 경계는 방향 해석의 경계가 아니다. 가로 글줄 전체에서 한 번 해소한다.
pub(crate) fn resolve_page(root: &mut RenderNode) {
    for child in &mut root.children {
        resolve_page(child);
    }
    if !matches!(root.node_type, RenderNodeType::TextLine(_)) {
        return;
    }
    let mut runs = Vec::new();
    for (index, node) in root.children.iter().enumerate() {
        let RenderNodeType::TextRun(run) = &node.node_type else {
            return;
        };
        // 개체·필드 표식·겹침·세로 글줄은 일반 텍스트와 다른 소유/배치 계약이다.
        // 아직 그 계약을 논리 텍스트의 전진으로 대체하지 않는다.
        if run.is_vertical
            || run.rotation != 0.0
            || run.char_overlap.is_some()
            || !matches!(run.field_marker, FieldMarkerType::None)
            || run.display_or_text().contains('\u{fffc}')
        {
            return;
        }
        runs.push((index, node, run));
    }
    if !runs
        .iter()
        .any(|(_, _, run)| has_rtl(run.display_or_text()))
    {
        return;
    }
    let Some((_, first, _)) = runs.first() else {
        return;
    };
    let anchor = first.bbox.x;
    let mut text = String::new();
    let mut positions = Vec::new();
    let mut owners = Vec::new();
    let mut end_x = 0.0;
    for (index, node, run) in &runs {
        let local = run.replay_positions_for(run.display_or_text());
        let count = run.display_or_text().chars().count();
        if local.len() != count + 1 {
            return;
        }
        let start_x = node.bbox.x - anchor;
        if start_x < end_x - 1e-6 {
            return;
        }
        if start_x > end_x + 1e-6 {
            // 레이아웃이 예약한 빈 전진도 방향 해석의 중립 구간으로 보존한다.
            positions.push(end_x);
            text.push('\u{fffc}');
        }
        let start = positions.len();
        positions.extend(local[..count].iter().map(|x| start_x + x));
        text.push_str(run.display_or_text());
        end_x = start_x + local[count];
        owners.push((
            *index,
            start,
            start + count,
            start_x,
            local.into_owned(),
            run.display_or_text().to_string(),
        ));
    }
    positions.push(end_x);
    if positions.iter().any(|x| !x.is_finite()) || positions.windows(2).any(|p| p[1] < p[0]) {
        return;
    }
    let Some(visual) = VisualTextPositions::new(&text, &positions) else {
        return;
    };
    let mut layouts = Vec::new();
    for (index, start, end, x, local, source_text) in owners {
        let spans = visual
            .runs
            .iter()
            .filter_map(|r| {
                let a = start.max(r.start);
                let b = end.min(r.end);
                (a < b).then(|| BidiVisualRun {
                    start: a - start,
                    end: b - start,
                    x: visual.span_origin(a, b).unwrap() - x,
                    rtl: r.rtl,
                })
            })
            .collect::<Vec<_>>();
        layouts.push((
            index,
            BidiLayout {
                scalar_count: end - start,
                logical_width: local.last().unwrap() - local.first().unwrap(),
                source_text: source_text.into_boxed_str(),
                source_positions: local.into_boxed_slice(),
                runs: spans.into_boxed_slice(),
            },
        ));
    }
    for (index, layout) in layouts {
        let RenderNodeType::TextRun(run) = &mut root.children[index].node_type else {
            unreachable!()
        };
        run.style.bidi_layout = Some(layout);
    }
}
