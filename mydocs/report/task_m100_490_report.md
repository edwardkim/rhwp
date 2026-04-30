# Task #490 최종 결과보고서 — 빈 텍스트 + TAC 수식 셀 alignment 정정

**이슈**: GitHub #490
**브랜치**: `local/task490`
**기간**: 2026-04-30
**관련**: Task #488 (Stage 2 시각 검증에서 발견)

## 1. 결함 요약

`samples/exam_science.hwp` 페이지 1 의 3번 표 (이온 결합 화합물 4×4) "전체 전자의 양" 컬럼의 28/36 수식이 셀 중앙이 아닌 좌측(셀 안 x≈0)에 정렬됨.

## 2. 근본 원인

`src/renderer/layout/paragraph_layout.rs:2227` 의 빈 runs 분기 (Task #287 도입) 가 paragraph alignment 를 적용하지 않아, `text_len=0 + ctrls=1+` (텍스트 없이 수식만 있는) 셀 paragraph 의 수식이 좌측 고정.

동일 함수의 텍스트 runs 케이스는 `Alignment::Center | Distribute | Right` 분기로 x_start 보정. 빈 runs 케이스만 누락됐던 것.

## 3. 수정

### `src/renderer/layout/paragraph_layout.rs`

빈 runs 분기에 paragraph alignment 보정 추가:

```rust
let line_tac_width: f64 = tac_offsets_px.iter()
    .filter(|(pos, _, _)| *pos >= line_start_char && *pos < line_end_char)
    .map(|(_, w, _)| *w)
    .sum();
let align_offset = match alignment {
    Alignment::Center | Alignment::Distribute => (available_width - line_tac_width).max(0.0) / 2.0,
    Alignment::Right => (available_width - line_tac_width).max(0.0),
    _ => 0.0,
};
let mut inline_x = effective_col_x + effective_margin_left + align_offset;
```

### `src/renderer/layout/integration_tests.rs`

`test_490_empty_para_with_tac_equation_respects_alignment` 추가.

## 4. 검증

| 항목 | 결과 |
|------|------|
| `cargo test --lib --release` | **1094 passed; 0 failed** (1093 + 신규 1) |
| `cargo test --release --test svg_snapshot` | **6/6 passed** |
| 9 종 샘플 263 페이지 byte 비교 | 257 동일 / 6 정정 (회귀 0) |
| 변경 방향 분석 (59 수식 shift) | 모두 양의 방향(우측 이동) — alignment 적용 일치 |

### 4.1 정정 페이지 분포

- exam_science p1: 28/36 두 수식 x=358.7 → 415.5 (셀 7/11)
- exam_science p2: 14 수식 우측 이동 (+15~+26)
- exam_science p3: 18 수식 우측 이동 (+8~+28)
- exam_science p4: 14 수식 우측 이동 (+26~+137)
- exam_math p13: 10 수식 우측 이동 (+2~+29)
- kps-ai p46: 1 수식 우측 이동 (+80)

광범위한 정정 효과: exam_science 전 페이지 + exam_math + kps-ai 의 수식만 있는 셀 모두 alignment 정상화.

### 4.2 회귀 영역

- Justify/Left ParaShape: 변경 없음 (match `_ => 0.0`)
- 텍스트+수식 혼재 줄: 본 분기 미진입
- Task #287 (수식 y 위치): x 만 변경, y/height 미변경

## 5. 잔존 후속 이슈 (Task #488 분리 4 건 중 #490 완료)

- [x] **#489** — Picture+Square wrap 호스트 텍스트 LINE_SEG 적용
- [x] **#490** — 빈 텍스트 + TAC 수식 셀 alignment (본 작업)
- [x] **#492** — 5번 밑단 짤림 (#489 정정으로 자연 해소)
- [ ] #491 — exam_science p1 2번 답안지 위치 미세 차이

## 6. 결론

빈 텍스트 + TAC 수식만 있는 셀 paragraph 의 alignment 무시 결함을 정정. 단일 함수 분기 1 곳 수정으로 9 종 샘플 263 페이지 중 6 페이지 (59 수식) 정정. 회귀 0.
