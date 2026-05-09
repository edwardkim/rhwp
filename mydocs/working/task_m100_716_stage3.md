# Task #716 Stage 3 (GREEN) 완료 보고서

**Issue**: [#716](https://github.com/edwardkim/rhwp/issues/716)
**Stage**: 3 — GREEN (수정 적용 + RED PASS)
**작성일**: 2026-05-08

---

## 산출물

`src/renderer/layout.rs:1559-1582` 의 Task #9 `fix_overlay_active` push 블록에 `is_empty_para` 가드 추가.

### 변경 diff (개념)

```rust
// Percent 전환: 표 하단과 비교 (Task #9)
if fix_overlay_active {
    let is_fixed = ...;
    // [Task #716] 빈 paragraph 는 push 대상에서 제외
    let is_empty_para = paragraphs.get(item_para)
        .map(|p| p.text.is_empty()
            || p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}'))
        .unwrap_or(false);
    if !is_fixed && !is_empty_para {     // ← !is_empty_para 추가
        let table_bottom = fix_table_start_y + fix_table_visual_h;
        if y_offset < table_bottom {
            y_offset = table_bottom;
        }
        fix_overlay_active = false;
    }
    // 빈 paragraph 일 때는 fix_overlay_active 를 해제하지 않아 후속 비-empty
    // paragraph 가 push 대상이 될 수 있도록 한다.
}
```

### 빈 paragraph 판정 기준

`p.text.is_empty()` 또는 모든 문자가 `c <= '\u{001F}'` (control 문자) 또는 `c == '\u{FFFC}'` (object placeholder).

→ 시각적으로 보이는 글자가 없는 문단을 보수적으로 식별. TAC 표 호스트 paragraph (FFFC 만 포함) 도 본 가드 대상이지만, 실제 layout 경로에서는 다른 분기로 처리되므로 영향 없음.

---

## RED 테스트 결과

```
$ cargo test --test issue_716 -- --nocapture

LAYOUT_OVERFLOW: page=0, col=0, para=15, type=PartialParagraph, y=1039.4, bottom=1028.0, overflow=11.3px
[issue_716] page 0 body=[x=75.59 y=94.47 w=642.53 h=933.57 bottom=1028.04]
            text_lines=38 max_bottom=1028.19 overflow=+0.15
test issue_716_page1_last_text_line_within_body ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured
```

**RED PASS** — `max_bottom=1028.19` (Body 하단 1028.04 대비 +0.15 px, 허용 0.5 px 이내).

## stderr 출력 비교

| 메시지 | RED (Stage 1) | GREEN (Stage 3) | 의미 |
|--------|---------------|-----------------|------|
| `LAYOUT_OVERFLOW_DRAW: pi=15 line=2 overflow=20.1px` | 발생 | **0건** ✓ | 실제 시각 cropping (TextLine bbox 가 컬럼 하단 초과) |
| `LAYOUT_OVERFLOW: pi=15 PartialParagraph overflow=31.3px` | 발생 | `overflow=11.3px` (잔존) | y_offset 누적 (trailing line_spacing 포함) — 시각 영향 없음 |

### 잔존 11.3 px 의 정체

`pi=15 line=2` 의 trailing line_spacing (840 HU = 11.2 px) 가 layout y_offset 에 누적된 결과.

- pi=15 line=2 bbox: `y=999.65 → bottom=1029.52` (line_height=29.87 합)
  - 실제 그려지는 글자 영역은 1028.19 까지 (sub-pixel rounding 후 1028.04 이내 0.15 px 초과)
- 그 후 trailing ls 11.2 px 가산 → y_offset = 1040.72 ≈ 1039.4 (실측)
- LAYOUT_OVERFLOW 메시지가 trigger (y_offset > col_bottom + 2.0)

**시각적 영향 없음**: trailing ls 영역에는 글자가 그려지지 않음. `LAYOUT_OVERFLOW_DRAW` (실제 cropping 측정) 가 0건이 됨이 핵심.

이 11.3 px 잔존은 Task #452 (`layout-only trailing 제외 → pagination 1 ls drift`) 의 의도된 trailing 가산 결과로, 본 task 의 결함과 별개. 추후 별도 검토 가능.

---

## 페이지네이션 영향

`dump-pages` 비교:

```
=== 페이지 1 (global_idx=0, section=0, page_num=1) ===
  body_area: x=75.6 y=94.5 w=642.5 h=933.6
  단 0 (items=16, used=929.4px, hwp_used≈918.2px, diff=+11.2px)
```

- 페이지 수: 4쪽 (변경 없음)
- 페이지 1 의 항목 수: 16 (변경 없음)
- `used` 값: 929.4 → 비변동 (push 제거에도 multi-line paragraph 의 trailing ls 가산이 그대로)

→ **페이지네이션 동작 변경 없음**. 본 정정은 layout 의 y_offset 만 정정 (drift 누적 차단), pagination 은 별도 height_measurer 가 독립 산출.

---

## 수정 영향 검토

### 본 정정의 직접 효과

1. **빈 paragraph 의 fix_overlay push 차단** — TAC 표(ls<0) 직후 첫 빈 paragraph 가 표 하단으로 점프하지 않음
2. **누적 drift 차단** — 본 샘플의 +20 px (pi=1 +8, pi=3 +12) 누적 제거
3. **시각 정합 유지** — 빈 paragraph 는 보이지 않으므로 push/no-push 차이가 시각적으로 없음

### Task #9 본래 의도 보존

- 텍스트 paragraph 의 push: **유지** (TAC 표 위에 텍스트 침범 차단)
- Fixed line spacing paragraph: **유지** (기존대로 push 면제)
- 빈 paragraph 만: **신규 면제** (의미 없는 push 차단)

### 회귀 위험

낮음:
- 빈 paragraph 의 위치는 시각적으로 무의미
- `is_empty_para` 가드는 보수적 (control + FFFC 만 허용)
- TAC 표 host paragraph 자체는 본 분기 진입 전에 다른 경로로 처리

다음 단계 Stage 4 / 5 에서 광범위 회귀 검증.

---

## 다음 단계 (Stage 4 — 회귀)

1. `cargo test --release` 전체 통과 확인
2. 페이지 1 SVG 시각 점검 (`output/svg/`)
3. `RHWP_VPOS_DEBUG=1` 출력 확인 (참고)
4. 골든 SVG 회귀 (회귀 테스트 셋트 실행)
5. `mydocs/working/task_m100_716_stage4.md`

## 승인 요청

Stage 3 GREEN 완료. RED PASS, LAYOUT_OVERFLOW_DRAW 0건 확인. Stage 4 (회귀 검증) 진입 승인 요청.
