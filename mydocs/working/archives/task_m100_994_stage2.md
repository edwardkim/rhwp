# Task #994 Stage 2 — G4 구현

- 이슈: [#994](https://github.com/edwardkim/rhwp/issues/994)
- 선행: [Stage 1 진단](task_m100_994_stage1.md)
- 구현 계획: [task_m100_994_impl.md](../plans/task_m100_994_impl.md)

## 1. 변경 위치

[src/renderer/composer.rs:321-368](../../src/renderer/composer.rs#L321-L368) — `compose_lines` 의 `line_segs.is_empty()` fallback.

## 2. 구현 (3차 반복)

### 1차 (G1 시도, 실패)
- `paragraph_layout::layout_inline_table_paragraph` 의 line_height 계산 보정
- 효과 없음 — 실제 paragraph (no controls) 는 `layout_composed_paragraph` path 사용

### 2차 (G4-wide, 시각 정렬 부풀림)
- composer fallback 에서 char-count (35 chars/line) 으로 multi-line 분할
- 효과: 겹침 해소 ✓
- **부작용**: align=Justify 가 짧은 line 의 chars 사이 spacing 부풀림

### 3차 (G4-final, 본 commit)
- Word boundary 분할 (mid-word break 회피)
- `has_line_break=true` 로 non-last synth line marking → Justify 비활성화
- 효과: 겹침 해소 + 자연스러운 chars 정렬 ✓

## 3. 최종 코드

```rust
fn compose_lines(para: &Paragraph) -> Vec<ComposedLine> {
    if para.line_segs.is_empty() {
        if para.text.is_empty() {
            return Vec::new();
        }
        let default_style_id = para.char_shapes.first()
            .map(|cs| cs.char_shape_id).unwrap_or(0);
        
        let chars: Vec<char> = para.text.chars().collect();
        const CHARS_PER_LINE: usize = 35;
        let mut lines = Vec::new();
        let total = chars.len();
        let mut offset = 0;
        while offset < total {
            let max_end = (offset + CHARS_PER_LINE).min(total);
            // Word boundary 분할 — Justify spacing 부풀림 회피
            let mut end = max_end;
            if end < total {
                let min_acceptable = offset + (CHARS_PER_LINE / 2);
                for i in (min_acceptable..max_end).rev() {
                    if chars[i] == ' ' || chars[i] == '\t' {
                        end = i + 1;
                        break;
                    }
                }
            }
            let line_text: String = chars[offset..end].iter().collect();
            let is_last_line = end >= total;
            lines.push(ComposedLine {
                runs: split_runs_by_lang(vec![ComposedTextRun {
                    text: line_text,
                    char_style_id: default_style_id,
                    lang_index: 0,
                    char_overlap: None,
                    footnote_marker: None,
                    display_text: None,
                }]),
                line_height: 400,
                baseline_distance: 320,
                segment_width: 0,
                column_start: 0,
                line_spacing: 0,
                has_line_break: !is_last_line,  // ← Justify 비활성화
                char_start: offset,
            });
            offset = end;
        }
        return lines;
    }
    // ... 기존 line_segs 처리 ...
}
```

## 4. 효과

- HWP5 변환본의 line_segs 누락 paragraph 시각 정합 (sample16 page 19~24 겹침 해소)
- Word wrap + 자연스러운 chars 정렬 (한컴 viewer 와 유사)
- 영향 paragraph: sample16 의 59개 `󰏅 ...` paragraph
