# PR #373 처리 보고서 — 정상 머지 (cherry-pick)

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#373](https://github.com/edwardkim/rhwp/pull/373) |
| 작성자 | [@planet6897](https://github.com/planet6897) (Jaeuk Ryu) |
| base / head | `devel` ← `local/task372` |
| 이슈 | [#372](https://github.com/edwardkim/rhwp/issues/372) |
| 처리 결정 | **정상 머지 (cherry-pick)** |
| 처리 일자 | 2026-04-27 |

## 결함 요약

`samples/hwpx/hwpx-h-02.hwpx` 9쪽 PDF 의 민트색(`#CDF2E4`) 형광펜 배경이 SVG 출력에서 누락.

원인: HWPX → IR → `TextRunStyle.shade_color` 까지 정상 보존되지만 `src/renderer/svg.rs::draw_text` 가 해당 필드 미사용. `html.rs` / `web_canvas.rs` 는 이미 동일 패턴 사용 중이었음.

## 변경 내용

`src/renderer/svg.rs::draw_text` 의 클러스터 분할 직후에 `<rect>` 배경 출력 (13 줄 추가):

```rust
let shade_rgb = style.shade_color & 0x00FFFFFF;
if shade_rgb != 0x00FFFFFF && shade_rgb != 0 {
    let text_width = *char_positions.last().unwrap_or(&0.0);
    if text_width > 0.0 {
        self.output.push_str(&format!(
            "<rect x=\"{:.4}\" y=\"{:.4}\" width=\"{:.4}\" height=\"{:.4}\" fill=\"{}\"/>\n",
            x, y - font_size, text_width, font_size * 1.2,
            color_to_svg(style.shade_color),
        ));
    }
}
```

좌표 시멘틱은 `web_canvas.rs:1271-1279` 와 동일 (`y - font_size` 시작, 높이 `font_size * 1.2`).

## 처리 절차

### Stage 1: cherry-pick
- `local/pr373` 브랜치 (`local/devel` 분기)
- PR #373 의 commit (`4b1bfc5`) cherry-pick
- 작성자 attribution 보존 (Jaeook Ryu)
- 충돌 없음 (svg.rs / plans / pdf 모두 devel 무변경)

### Stage 2: 자동 회귀 + 시각 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | **1014 passed, 0 failed** |
| `cargo test --test svg_snapshot` | 6/6 통과 |
| `cargo test --test issue_301` | 1/1 통과 |
| `cargo test --test page_number_propagation` | 2/2 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |

### Stage 3: 시각 검증

`hwpx-h-02.hwpx` 9쪽 SVG 출력 — `fill="#cdf2e4"` `<rect>` **9건 정상 등장**:
```
<rect x="186.4800" y="145.5333" width="58.2000" height="24.0000" fill="#cdf2e4"/>
<rect x="132.4800" y="231.9333" width="72.8000" height="24.0000" fill="#cdf2e4"/>
... (9 건)
```

### Stage 4: 7 핵심 샘플 회귀

| 샘플 | 페이지 | LAYOUT_OVERFLOW |
|------|------|------|
| form-01 | 1 | 0 |
| aift | 77 | 3 |
| KTX | 27 | 1 |
| k-water-rfp | 27 | 0 |
| exam_eng | 11 | 0 |
| kps-ai | 79 | 5 |
| hwp-multi-001 | 10 | 0 |

→ 모든 샘플 무변화 (Task #361, #362, PR #366, #371 효과 모두 유지).

### Stage 5: orders 갱신
`mydocs/orders/20260427.md` 의 #372 항목 추가 (PR 의 작성자 변경 없으므로 메인테이너 추가).

## 흡수 commit 목록

```
[devel merge commit]
[review docs commit]
696a73d Task #372: SVG 렌더러 형광펜 배경(charPr.shadeColor) 누락 [planet6897]
```

## 작성자 기여

@planet6897 (Jaeuk Ryu) — 결함 진단 (svg.rs 누락, html/web_canvas 동일 패턴 식별) + svg.rs 단일 함수 13 줄 변경 + PDF 와 시각 일치 검증.

## 참고

- 검토 문서: `mydocs/pr/pr_373_review.md`
- 구현계획서: `mydocs/pr/pr_373_review_impl.md`
- 동일 패턴 참조: `src/renderer/html.rs:334-337`, `src/renderer/web_canvas.rs:1271-1279`
