//! 인라인 HTML의 내용·서식·그림과 CSS 모델 변환.

use super::*;

impl DocumentCore {
    /// <p> 태그 내부의 인라인 콘텐츠를 파싱하여 Paragraph에 채운다.
    pub(crate) fn parse_inline_content(&mut self, para: &mut Paragraph, html: &str) {
        let mut full_text = String::new();
        // (char_start, char_end, char_shape_id) 형태의 스타일 범위
        let mut style_runs: Vec<(usize, usize, u32)> = Vec::new();
        // (그림 앞 글자 수, 그림) — 글 사이에 든 그림
        let mut pictures: Vec<(usize, crate::model::image::Picture)> = Vec::new();

        let chars: Vec<char> = html.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        // 열린 서식 요소(span·b·i·u 등)의 이름과 style. 서식은 닫힐 때까지 안쪽 글에 이어진다.
        let mut open_styles: Vec<(String, String)> = Vec::new();

        while pos < len {
            if chars[pos] == '<' {
                let tag_end = find_char(&chars, pos, '>');
                if tag_end >= len {
                    break;
                }

                let tag_str: String = chars[pos..=tag_end].iter().collect();
                let tag_lower = tag_str.to_lowercase();

                if let Some((name, closing)) = inline_format_tag(&tag_lower) {
                    // span 도 건너뛰지 않고 안쪽을 계속 읽는다.
                    // 그래야 span 자기 style 과 안쪽 <strong>·<u>·<br>·그림이 함께 남는다.
                    if closing {
                        if let Some(i) = open_styles.iter().rposition(|(open, _)| open == name) {
                            open_styles.truncate(i);
                        }
                    } else {
                        let mut css = parse_inline_style(&tag_str);
                        // Chrome 은 <b style="…"> 처럼 style 을 붙여 쓴다.
                        // style 이 그 속성을 정하면 따른다(Google 문서의 <b style="font-weight:normal">).
                        let implied = match name {
                            "b" | "strong" => Some(("font-weight", "bold")),
                            "i" | "em" => Some(("font-style", "italic")),
                            "u" => Some(("text-decoration", "underline")),
                            _ => None,
                        };
                        if let Some((property, value)) = implied {
                            if parse_css_value(&css.to_lowercase(), property).is_none() {
                                css = format!("{css};{property}:{value}");
                            }
                        }
                        open_styles.push((name.to_string(), css));
                    }
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<br") {
                    full_text.push('\n');
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<img") {
                    // 글 사이 그림은 그 자리에 글자처럼 취급하는 그림으로 넣는다.
                    // 종전에는 기타 태그로 버려 `<p>앞<img>뒤</p>` 의 그림이 사라졌다.
                    if let Some(pic) =
                        crate::document_core::html_table_import::html_img_src(&tag_str)
                            .filter(|src| src.starts_with("data:"))
                            .and_then(|src| self.html_data_img_picture(&tag_str, src))
                    {
                        pictures.push((full_text.chars().count(), pic));
                    }
                    pos = tag_end + 1;
                    continue;
                } else {
                    // 기타 태그 무시
                    pos = tag_end + 1;
                    continue;
                }
            } else {
                // 태그 밖의 일반 텍스트
                let text_start = pos;
                while pos < len && chars[pos] != '<' {
                    pos += 1;
                }
                let raw: String = chars[text_start..pos].iter().collect();
                let decoded = decode_html_entities(&raw);
                if !decoded.is_empty() {
                    // 서식 구간 뒤의 평문도 구간을 새로 연다.
                    // 안 열면 앞 구간의 굵게 등이 문단 끝까지 이어진다.
                    if !open_styles.is_empty() || !style_runs.is_empty() {
                        // parse_css_value 는 처음 찾은 값을 쓰므로 안쪽 요소를 앞에 둔다.
                        let css: Vec<&str> = open_styles
                            .iter()
                            .rev()
                            .map(|(_, css)| css.as_str())
                            .collect();
                        let char_shape_id = self.css_to_char_shape_id(&css.join(";"));
                        let start = full_text.chars().count();
                        full_text.push_str(&decoded);
                        let end = full_text.chars().count();
                        style_runs.push((start, end, char_shape_id));
                    } else {
                        full_text.push_str(&decoded);
                    }
                }
                continue;
            }
        }

        para.text = full_text;
        // [#3494] char_count 는 문단 종결자를 포함한다 (model/paragraph.rs:1042).
        para.char_count = para.text.encode_utf16().count() as u32 + 1;
        para.char_offsets = para
            .text
            .chars()
            .scan(0u32, |acc, c| {
                let off = *acc;
                *acc += c.len_utf16() as u32;
                Some(off)
            })
            .collect();

        // 스타일 범위를 CharShapeRef로 변환
        for (start, _end, char_shape_id) in &style_runs {
            // char index → UTF-16 위치
            let utf16_pos: u32 = para
                .text
                .chars()
                .take(*start)
                .map(|c| c.len_utf16() as u32)
                .sum();
            para.char_shapes
                .push(crate::model::paragraph::CharShapeRef {
                    start_pos: utf16_pos,
                    char_shape_id: *char_shape_id,
                });
        }

        // 그림은 확장 제어문자 8칸을 차지한다. 그 자리를 char_offsets 의 갭으로 남겨야
        // control_text_positions 가 그림을 글 사이에 되짚는다.
        for (char_idx, pic) in pictures {
            para.shift_for_inline_control_insert(char_idx);
            para.char_count += 8;
            para.controls.push(Control::Picture(Box::new(pic)));
            para.ctrl_data_records.push(None);
            para.control_mask |= 0x0000_0800;
            para.has_para_text = true;
        }
    }

    /// CSS 인라인 스타일 → CharShape ID 변환 (기존에서 검색 또는 신규 생성).
    pub(crate) fn css_to_char_shape_id(&mut self, css: &str) -> u32 {
        use crate::model::style::{CharShape, UnderlineType};

        // 기본 CharShape를 기반으로 수정
        let base_id = if !self.document.doc_info.char_shapes.is_empty() {
            0u32
        } else {
            self.document
                .doc_info
                .char_shapes
                .push(CharShape::default());
            0
        };
        let mut cs = self.document.doc_info.char_shapes[base_id as usize].clone();
        // 파싱된 문서의 CharShape 는 원본 CHAR_SHAPE 레코드 바이트를 raw_data 로 들고 있고
        // (parser/doc_info.rs), 직렬화기는 raw_data 가 있으면 필드 대신 그 바이트를 그대로
        // 쓴다(serializer/doc_info.rs). 아래에서 굵기·색·크기를 바꿔도 raw_data 를 비우지
        // 않으면 저장 시 원본 서식 바이트가 나가 붙여넣은 서식이 통째로 사라진다.
        // PartialEq 가 raw_data 를 비교에서 제외하므로 아래 중복 검색도 이를 걸러내지 못한다.
        // CharShapeMods::apply_to(model/style.rs)가 같은 이유로 첫 줄에서 raw_data 를 비운다.
        cs.raw_data = None;

        // CSS 속성 파싱 및 적용
        let css_lower = css.to_lowercase();

        // font-family
        //
        // 문서에 없는 글꼴이면 **새로 등록**한다.
        // 종전에는 `find_font_id` 가 못 찾으면 조용히 기본 글꼴로 떨어졌다 — 표본 문서의
        // `바탕` 이 양식의 `맑은 고딕` 으로 바뀌면서 줄 높이가 커져 **표 셀 안 둘째 줄이
        // 행 경계에 잘렸고**, 글자 폭이 넓어져 쪽수도 어긋났다.
        // 값은 대소문자를 보존해야 하므로 `css_lower` 가 아니라 원본 `css` 에서 읽는다
        // (속성 이름은 `parse_css_value` 가 완전일치로 비교하므로
        // `mso-fareast-font-family` 는 걸리지 않는다).
        //
        // 🔴 한글은 **동아시아 글꼴을 `mso-fareast-font-family` 에** 적는다. `font-family` 는
        // 라틴 글꼴이다(실측: `<span style="font-family:바탕;mso-fareast-font-family:바탕">` 도 있고
        // `mso-fareast-font-family` 만 있는 span 도 많다). 라틴만 읽으면 **본문 한글이 통째로
        // 대상 문서 기본 글꼴로 떨어진다** — 표 셀 글자가 고딕으로 바뀌어 줄 높이가 커지고
        // 셀에서 잘렸다.
        // 슬롯 순서는 `serializer/hwpx/canonical_defaults.rs` FONTFACE_LANG_NAMES 와 같다:
        // 0 HANGUL · 1 LATIN · 2 HANJA · 3 JAPANESE · 4 OTHER · 5 SYMBOL · 6 USER.
        let first_family = |value: String| -> String {
            value
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(|c: char| c == '\'' || c == '"')
                .trim()
                .to_string()
        };
        let latin_name = parse_css_value(css, "font-family")
            .map(first_family)
            .filter(|n| !n.is_empty());
        let east_name = parse_css_value(css, "mso-fareast-font-family")
            .map(first_family)
            .filter(|n| !n.is_empty());
        let east_ids = east_name
            .as_deref()
            .or(latin_name.as_deref())
            .and_then(|n| self.find_or_register_font_ids(n));
        let latin_ids = latin_name
            .as_deref()
            .or(east_name.as_deref())
            .and_then(|n| self.find_or_register_font_ids(n));
        if let Some(ids) = east_ids {
            for lang_idx in [0usize, 2, 3] {
                cs.font_ids[lang_idx] = ids[lang_idx];
            }
        }
        if let Some(ids) = latin_ids {
            for lang_idx in [1usize, 4, 5, 6] {
                cs.font_ids[lang_idx] = ids[lang_idx];
            }
        }

        // font-size
        if let Some(size_str) = parse_css_value(&css_lower, "font-size") {
            if let Some(pt) = parse_pt_value(&size_str) {
                // pt → HWPUNIT: 1pt = 100 HWPUNIT (base_size 단위)
                cs.base_size = (pt * 100.0) as i32;
            }
        }

        // letter-spacing(자간)
        //
        // 한글은 줄을 맞추려고 글자마다 `letter-spacing:-0.2pt` 같은 음수 자간을 넣어 내보낸다.
        // 이를 버리면 같은 글이 원본보다 넓어져 **줄바꿈과 쪽수가 어긋난다**
        // (실측: 원본 11쪽짜리 요약이 붙여넣기 뒤 12쪽, 문단마다 줄 하나씩 늘어남).
        // `CharShape.spacings` 는 글꼴 크기 대비 **퍼센트**다
        // (`renderer/style_resolver.rs`: letter_spacing_px = font_size × spacing / 100).
        // 한글 자간 입력 범위(±50%)로 자른다.
        if let Some(ls_str) = parse_css_value(&css_lower, "letter-spacing") {
            if let Some(pt) = parse_pt_value(&ls_str) {
                let base_pt = (f64::from(cs.base_size) / 100.0).max(1.0);
                let pct = (pt / base_pt * 100.0).round().clamp(-50.0, 50.0) as i8;
                cs.spacings = [pct; 7];
            }
        }

        // font-weight — 안쪽 요소의 값이 앞에 오므로 처음 찾은 값을 따른다.
        // GitHub 등은 <strong> 을 600 으로 쓰므로 600 이상을 굵게 본다.
        cs.bold = parse_css_value(&css_lower, "font-weight")
            .is_some_and(|w| w.starts_with("bold") || w.parse::<u16>().is_ok_and(|n| n >= 600));

        // font-style
        let is_italic =
            css_lower.contains("font-style:italic") || css_lower.contains("font-style: italic");
        cs.italic = is_italic;

        // color
        if let Some(color_str) = parse_css_value(&css_lower, "color") {
            if let Some(bgr) = css_color_to_hwp_bgr(&color_str) {
                cs.text_color = bgr;
            }
        }

        // text-decoration
        let has_underline = css_lower.contains("text-decoration:underline")
            || css_lower.contains("text-decoration: underline")
            || css_lower.contains("text-decoration-line:underline")
            || css_lower.contains("text-decoration-line: underline");
        cs.underline_type = if has_underline {
            UnderlineType::Bottom
        } else {
            UnderlineType::None
        };

        let has_strikethrough = css_lower.contains("text-decoration:line-through")
            || css_lower.contains("text-decoration: line-through")
            || css_lower.contains("line-through");
        cs.strikethrough = has_strikethrough;

        // 동일한 CharShape 검색
        for (i, existing) in self.document.doc_info.char_shapes.iter().enumerate() {
            if *existing == cs {
                return i as u32;
            }
        }

        // 새로 추가
        let new_id = self.document.doc_info.char_shapes.len() as u32;
        self.document.doc_info.char_shapes.push(cs);
        self.document.doc_info.raw_stream_dirty = true;
        // 스타일 세트 갱신
        self.rebuild_resolved_styles();
        new_id
    }

    /// CSS 인라인 스타일 → ParaShape ID 변환.
    pub(crate) fn css_to_para_shape_id(&mut self, css: &str) -> u16 {
        use crate::model::style::{Alignment, LineSpacingType};

        if css.is_empty() && !self.document.doc_info.para_shapes.is_empty() {
            return 0;
        }

        let base_id: u16 = 0;
        let mut ps = self
            .document
            .doc_info
            .para_shapes
            .get(base_id as usize)
            .cloned()
            .unwrap_or_default();
        // CharShape 쪽과 동일 — 원본 PARA_SHAPE 바이트를 비우지 않으면 정렬·줄간격 변경이
        // 저장 시 사라진다(ParaShapeMods::apply_to 와 같은 처리).
        ps.raw_data = None;

        let css_lower = css.to_lowercase();

        // text-align
        if let Some(align) = parse_css_value(&css_lower, "text-align") {
            ps.alignment = match align.trim() {
                "left" => Alignment::Left,
                "right" => Alignment::Right,
                "center" => Alignment::Center,
                "justify" => Alignment::Justify,
                _ => ps.alignment,
            };
        }

        // line-height
        if let Some(lh) = parse_css_value(&css_lower, "line-height") {
            let lh = lh.trim();
            if lh.ends_with('%') {
                if let Ok(pct) = lh.trim_end_matches('%').parse::<i32>() {
                    ps.line_spacing = pct;
                    ps.line_spacing_type = LineSpacingType::Percent;
                }
            } else if lh.ends_with("px") {
                if let Ok(px) = lh.trim_end_matches("px").parse::<f64>() {
                    // px → HWPUNIT (1px ≈ 75 HWPUNIT at 96dpi)
                    ps.line_spacing = (px * 7200.0 / 25.4 / (self.dpi / 25.4)).round() as i32;
                    ps.line_spacing_type = LineSpacingType::Fixed;
                }
            }
        }

        // 여백·들여쓰기 — 한글은 개조식 문단을 `margin-left:15pt;text-indent:-15pt`
        // 로 내보낸다. 종전에는 이 둘을 무시해 **둘째 줄부터 왼쪽으로 튀어나왔다**.
        // CSS 길이(pt·px·cm·mm) → ParaShape 여백 단위(= HWPUNIT × 2).
        //
        // 🔴 ParaShape 의 margin_left/right·indent·spacing_* 는 **HWPUNIT 의 2배 스케일**로
        // 저장한다(`renderer/style_resolver.rs` 가 `/2` 로 되돌린다. HWPX 파서도
        // `hp:case`(HwpUnitChar) 값을 2배로 올려 읽는다 — `parser/hwpx/header.rs`).
        // 1배로 넣으면 방향은 맞고 **크기가 정확히 절반**이 되어, 개조식 문단의 둘째 줄이
        // 글머리표 아래로 덜 들어가 "왼쪽으로 튀어나온" 것처럼 보인다.
        // 실측(2026-09-04): CSS `margin-left:30.3pt;text-indent:-30.3pt`
        // → 1배 −3030 이면 내어쓰기 20.2px, 한글 원본(`hp:default` −6060)은 40.4px.
        let css_len_to_hwpunit = |value: &str| -> Option<i32> {
            let v = value.trim();
            let (num, unit): (&str, &str) = if let Some(rest) = v.strip_suffix("pt") {
                (rest, "pt")
            } else if let Some(rest) = v.strip_suffix("px") {
                (rest, "px")
            } else if let Some(rest) = v.strip_suffix("cm") {
                (rest, "cm")
            } else if let Some(rest) = v.strip_suffix("mm") {
                (rest, "mm")
            } else {
                (v, "pt")
            };
            let n: f64 = num.trim().parse().ok()?;
            // 마지막 ×2 가 위에서 말한 ParaShape IR 스케일이다.
            Some(match unit {
                "px" => (n * 72.0 / 96.0 * 200.0).round() as i32,
                "cm" => (n * 72.0 / 2.54 * 200.0).round() as i32,
                "mm" => (n * 72.0 / 25.4 * 200.0).round() as i32,
                _ => (n * 200.0).round() as i32,
            })
        };
        let css_margin_left =
            parse_css_value(&css_lower, "margin-left").and_then(|v| css_len_to_hwpunit(&v));
        let css_text_indent =
            parse_css_value(&css_lower, "text-indent").and_then(|v| css_len_to_hwpunit(&v));
        if let Some(v) = parse_css_value(&css_lower, "margin-right") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.margin_right = hu.max(0);
            }
        }
        if css_margin_left.is_some() || css_text_indent.is_some() {
            // CSS: 첫 줄 x = margin-left + text-indent, 나머지 = margin-left.
            // HWP: 첫 줄 x = left(+indent 가 양수면 더함), 내어쓰기(indent<0)면 나머지가 left+|indent|.
            // 따라서 left = margin-left + text-indent(첫 줄 위치), indent = text-indent 로 옮긴다.
            let ml = css_margin_left.unwrap_or(0);
            let ti = css_text_indent.unwrap_or(0);
            ps.margin_left = (ml + ti).max(0);
            ps.indent = ti;
            // 평문 표기를 유지하되, 여백은 직렬화기가 공통 IR에서 물리 단위로 되돌린다.
            ps.hwpx_plain_para_margin = true;
        }
        if let Some(v) = parse_css_value(&css_lower, "margin-top") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.spacing_before = hu.max(0);
                ps.hwpx_plain_para_margin = true;
            }
        }
        if let Some(v) = parse_css_value(&css_lower, "margin-bottom") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.spacing_after = hu.max(0);
                ps.hwpx_plain_para_margin = true;
            }
        }

        // 동일한 ParaShape 검색
        for (i, existing) in self.document.doc_info.para_shapes.iter().enumerate() {
            if *existing == ps {
                return i as u16;
            }
        }

        let new_id = self.document.doc_info.para_shapes.len() as u16;
        self.document.doc_info.para_shapes.push(ps);
        self.document.doc_info.raw_stream_dirty = true;
        self.rebuild_resolved_styles();
        new_id
    }

    /// 폰트 이름으로 font_faces에서 ID를 찾는다.
    pub(crate) fn find_font_id(&self, name: &str) -> Option<u16> {
        let name_lower = name.to_lowercase();
        // 한글 폰트 (인덱스 0)를 먼저, 영어 폰트 (인덱스 1)를 다음으로 검색
        for lang_idx in 0..self.document.doc_info.font_faces.len() {
            for (font_idx, font) in self.document.doc_info.font_faces[lang_idx]
                .iter()
                .enumerate()
            {
                if font.name.to_lowercase() == name_lower {
                    return Some(font_idx as u16);
                }
            }
        }
        None
    }

    /// 글꼴 이름으로 ID 를 찾고, 문서에 없으면 **새로 등록**해 그 ID 를 돌려준다.
    ///
    /// 붙여넣기 원본의 글꼴이 대상 문서에 없을 때 조용히 기본 글꼴로
    /// 떨어지면 줄 높이·글자 폭이 달라져 **표 셀 글자가 잘리고 쪽수가 어긋난다**
    /// (표본: 본문 글꼴 `바탕` 이 대상 문서 기본 글꼴 `맑은 고딕` 으로 바뀌었다).
    /// `Font` 는 `raw_data` 없이도 HWPX(`serializer/hwpx/header.rs`)·
    /// HWP5(`serializer/hwp5/doc_info.rs` `serialize_face_name`) 양쪽이 모델 필드로 써낸다.
    ///
    /// 🔴 언어 슬롯마다 **글꼴 목록이 다르다**(실측한 문서: HANGUL·LATIN 8개, OTHER·USER 7개.
    /// 같은 index 가 슬롯마다 다른 글꼴을 가리킨다).
    /// 그래서 하나의 ID 를 7칸에 복사하면 안 되고, **슬롯마다 찾거나 넣어서 각자의 index** 를 쓴다.
    fn find_or_register_font_ids(&mut self, name: &str) -> Option<[u16; 7]> {
        let trimmed = name.trim();
        // 총칭 글꼴은 등록하지 않는다 — 이름이 아니라 분류다.
        const GENERIC: [&str; 8] = [
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "-apple-system",
            "inherit",
        ];
        if trimmed.is_empty()
            || trimmed.chars().count() > 64
            || GENERIC.contains(&trimmed.to_lowercase().as_str())
        {
            return None;
        }
        let name_lower = trimmed.to_lowercase();
        let faces = &mut self.document.doc_info.font_faces;
        if faces.is_empty() {
            return None;
        }
        let slot_count = faces.len().min(7);
        let mut ids = [0u16; 7];
        for (lang_idx, slot) in faces.iter_mut().take(7).enumerate() {
            let idx = match slot
                .iter()
                .position(|font| font.name.to_lowercase() == name_lower)
            {
                Some(found) => found,
                None => {
                    // 한 문서에 무한정 늘리지 않는다(중복·악성 CSS 방어).
                    if slot.len() >= 256 {
                        return None;
                    }
                    slot.push(crate::model::style::Font {
                        name: trimmed.to_string(),
                        alt_type: 1, // TTF
                        ..Default::default()
                    });
                    slot.len() - 1
                }
            };
            ids[lang_idx] = u16::try_from(idx).ok()?;
        }
        // 슬롯이 7개보다 적은 문서는 남은 칸을 첫 슬롯 값으로 채운다.
        let first = ids[0];
        for id in ids.iter_mut().skip(slot_count) {
            *id = first;
        }
        Some(ids)
    }
}
