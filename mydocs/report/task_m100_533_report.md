# Task #533 최종 보고서 — exam_kor 14p Square wrap 표 다음 문단 baseline 누락

**작성일**: 2026-05-02
**이슈**: [#533](https://github.com/edwardkim/rhwp/issues/533)
**브랜치**: `local/task533`
**최종 commit**: `354a06f` (Stage 4 commit, 시리즈 5 commit)

## 1. 요약

> exam_kor.hwp 14페이지 우측 단의 줄간격 일관성 결함 (`작업지시자 분석 요청 — 14페이지 우측 박스 안에 줄간격이 일정하지 않음`) 의 본질을 **비-TAC Square wrap 인라인 표 직후 y_offset 이 표 bottom 으로만 advance** 로 확정. 호스트 문단 텍스트가 표보다 길게 늘어진 케이스에서 다음 문단이 baseline (~13 px) 만큼 위로 시프트되어 시각적 결함 발생. `src/renderer/layout.rs::layout_table_item` 에 호스트 last LINE_SEG 영역 max() 가산 (+18 라인) 으로 정정. 1116 lib + 6 svg_snapshot + issue_418/501 회귀 0, 광범위 8 샘플 192 페이지 중 190 페이지 byte-identical, 변경 2 페이지 (exam_kor p14 + p17) 모두 의도된 정정.

## 2. 본질 정정 경로

### 2-1. 초기 가설 → 본질 발견

| 단계 | 발견 |
|------|------|
| 사용자 분석 요청 | "14페이지 우측 박스 줄간격 불일치" |
| Stage 1 측정 | pi=51 SVG y 285.44 (기대 298.26), gap 11.73 (기대 24.51), baseline 978 HU 누락 |
| Stage 1 root cause | layout_table_item 비-TAC Square wrap 분기 y_offset advance 결함 |

### 2-2. 핵심 측정 (`RHWP_VPOS_DEBUG=1`)

수정 전:
```
VPOS_CORR: pi=51 vpos_end=5514 base=958 col_y=211.65 y_in=272.40 end_y=272.40
```

수정 후:
```
VPOS_CORR: pi=51 vpos_end=5514 base=0 col_y=211.65 y_in=285.17 end_y=285.17
```

→ `base` 가 958 HU → 0 으로 정상화. y_offset 12.77 px advance 보강.

## 3. 변경

### 3-1. 코드 (`src/renderer/layout.rs::layout_table_item`)

비-TAC Square wrap 표 처리 분기에 호스트 마지막 LINE_SEG 영역 max() 추가:

```rust
// [Task #533] Square wrap 호스트 문단: 표는 floating, 호스트
// 텍스트가 표 옆을 흐른다. 호스트 last LINE_SEG vpos+lh 영역이
// 표 bottom 보다 아래일 때 호스트 텍스트 영역까지 advance.
// 대형 표 (표 > 텍스트) 는 max() 로 표 영역 우선 유지.
// vpos 는 column 누적 좌표이므로 ls[0].vpos 를 차감해 호스트
// 문단 내부 offset 으로 변환.
if !is_tac {
    if let Some(Control::Table(t)) = para.controls.get(control_index) {
        if matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square) {
            if let (Some(first), Some(last)) =
                (para.line_segs.first(), para.line_segs.last()) {
                let para_inner_h = (last.vertical_pos + last.line_height)
                    .saturating_sub(first.vertical_pos);
                let host_text_bottom = para_y_for_table
                    + hwpunit_to_px(para_inner_h, self.dpi);
                if host_text_bottom > y_offset {
                    y_offset = host_text_bottom;
                }
            }
        }
    }
}
```

### 3-2. 변경량

| 영역 | 추가 | 삭제 | 수정 |
|------|------|------|------|
| `src/renderer/layout.rs` | 18 | 0 | 0 |

## 4. 회귀 차단 가드

| 가드 | 보호 영역 |
|------|----------|
| `!is_tac` | TAC 인라인 표 영역 (TAC 표는 인라인 흐름 / 별도 측정 경로) |
| `wrap == Square` | TopAndBottom / InFrontOfText / BehindText 등 영역 분리 |
| `max() (host > y_offset)` | 대형 표 케이스 (표 > 텍스트) 영역 보존 |
| `first.vertical_pos saturating_sub` | column 누적 vpos → paragraph 내부 offset 변환 |

## 5. 검증 결과

### 5-1. 단위/통합 테스트

| 게이트 | 결과 |
|--------|------|
| `cargo test --lib` | **1116 passed** (0 failed, 1 ignored) |
| `cargo test --test svg_snapshot` | **6/6** 통과 |
| `cargo test --test issue_418` | 1/1 통과 |
| `cargo test --test issue_501` | 1/1 통과 |
| `cargo clippy --lib` | 2 pre-existing errors (`object_ops.rs` / `table_ops.rs` `pic.caption.unwrap()`), 본 task 미관여 |

### 5-2. 광범위 샘플 회귀 (`scripts/svg_regression_diff.sh build b848d00 61415cf`)

| 샘플 | total | same | diff |
|------|-------|------|------|
| 2010-01-06 | 6 | 6 | 0 |
| aift | 77 | 77 | 0 |
| exam_eng | 8 | 8 | 0 |
| **exam_kor** | **20** | **18** | **2 (p14, p17)** |
| exam_math_no | 20 | 20 | 0 |
| exam_math | 20 | 20 | 0 |
| exam_science | 6 | 6 | 0 |
| synam-001 | 35 | 35 | 0 |
| **합계** | **192** | **190** | **2** |

### 5-3. 변경 페이지 본질 검증

| 페이지 | 위치 | host inner_h | table h | 차이 | 판정 |
|--------|------|------------|---------|------|------|
| p14 col 0 pi=33 | 88.85 px | 75.6 px | +13.25 | 의도 정정 |
| p14 col 0 pi=37 | 64.35 px | 51.4 px | +12.95 | 의도 정정 |
| p14 col 0 pi=40 | 39.84 px | 26.4 px | +13.44 | 의도 정정 |
| p14 col 0 pi=47 | 64.35 px | 51.4 px | +12.95 | 의도 정정 |
| p14 col 1 pi=50 | 64.35 px | 51.4 px | +12.95 | 의도 정정 (1차 발견) |
| **p17 pi=2** | **211.39 px** | **198.8 px** | **+12.59** | **의도 정정 (동일 본질 흡수)** |

### 5-4. 시각 검증

| 위치 | gap (수정 전) | gap (수정 후) |
|------|--------------|--------------|
| p14 col 1 pi=50→pi=51 | **11.73 ★ 좁음** | **24.51 ✓** |
| p14 col 0 pi=37→pi=38 | 11.41 ★ 좁음 | 24.51 ✓ |
| p14 col 0 pi=40→pi=41 | 11.73 ★ 좁음 | 24.51 ✓ |
| p14 col 0 pi=47→pi=48 | 11.31 ★ 좁음 | 24.51 ✓ |
| p17 pi=3 "사" y | 680.41 | 692.81 (+12.40) |

## 6. 산출물

| 산출물 | 위치 |
|--------|------|
| 수행계획서 | `mydocs/plans/task_m100_533.md` |
| 구현계획서 | `mydocs/plans/task_m100_533_impl.md` |
| Stage 1 보고서 (root cause) | `mydocs/working/task_m100_533_stage1.md` |
| Stage 3 보고서 (코드 적용) | `mydocs/working/task_m100_533_stage3.md` |
| Stage 4 보고서 (회귀 검증) | `mydocs/working/task_m100_533_stage4.md` |
| **본 최종 보고서** | `mydocs/report/task_m100_533_report.md` |
| 코드 변경 | `src/renderer/layout.rs` (+18 라인) |

## 7. 본질 학습

### 7-1. HWP IR vpos 의미

- 비-TAC Square wrap 표는 floating — 표는 호스트 문단 옆에 배치, 호스트 텍스트는 표를 우회하여 흐름
- 호스트 텍스트 영역 ≠ 표 영역. 두 영역이 다를 때 다음 문단의 시작 y 는 max(둘 중 큰 영역) 이어야 함
- `LINE_SEG.vertical_pos` 는 **column 누적 좌표** (paragraph 상대 아님). paragraph 내부 offset 추출 시 `last.vpos - first.vpos` 패턴 필수

### 7-2. 회귀 위험 측면

- vpos 보정 (Task #412/#332) 의 lazy_base 알고리즘은 y_offset 으로부터 역산 — y_offset 자체가 잘못되면 base 가 잘못 계산되어 보정이 무효화 (`base=958` 케이스). 본 결함의 lazy_base 무효화 패턴은 향후 vpos drift 결함 디버깅 시 신호로 활용 가능
- Square wrap host 의 다음 문단 시프트는 baseline 과 정확히 일치 (978 HU = ~13 px) → "baseline 누락" 패턴은 일관 시프트와 결합되어 발견 신호

### 7-3. 메모리 정합

- `feedback_essential_fix_regression_risk` 정합 — 본 fix 는 광범위 회귀 위험 영역이었으나 측정 (192 페이지 중 190 byte-identical) 으로 안전성 확정
- `feedback_rule_not_heuristic` 정합 — HWP LINE_SEG 인코딩 (vpos+lh 가 paragraph 끝) 룰 직접 적용, max() 패턴으로 분기 없음

## 8. Stage 5 영역 (작업지시자 시각 판정)

검증 영역 (생성됨):
- `/tmp/p14_final/exam_kor_014.svg` (수정 후 p14)
- `/tmp/p17_final/exam_kor_017.svg` (수정 후 p17)

확인 영역:
1. **exam_kor p14 우측 단**: 본 결함 직접 대상. pi=50 → pi=51 → pi=52 줄간격 일관성
2. **exam_kor p14 좌측 단**: pi=37/40/47 직후 줄간격 일관성 (동일 본질)
3. **exam_kor p17**: 옛한글 자모 영역 (Task #528) + 본 fix 의 12.40 px 시프트 영향 (페이지 외곽 박스 + pi=3 본문 위치)

## 9. 작업지시자 승인 흐름

| Stage | 단계 | 승인 |
|-------|------|------|
| 1 | Root cause 위치 확정 | ✓ |
| 2 | 구현계획서 (Option A) | ✓ |
| 3 | 코드 적용 + 단위 테스트 (1116) | ✓ |
| 4 | 광범위 회귀 (192 페이지 / 2 변경) | ✓ |
| 5 | 시각 판정 (본 단계) | **대기** |

## 10. close 영역

작업지시자 시각 판정 통과 시:
1. local/task533 → local/devel merge
2. local/devel → devel merge + push
3. issue #533 close (`gh issue close 533 --reason completed`)

작업지시자 시각 판정 부적합 시: 별도 피드백 + 보강 stage 추가.
