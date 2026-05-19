# Task #1007 최종 결과 보고서 — HWP5 변환본 페이지 강제 나눔 한컴 정합

이슈: [#1007](https://github.com/edwardkim/rhwp/issues/1007)

## 1. 목표

HWP5 변환본 (HWP3 → HWP5 변환) 의 페이지 강제 나눔이 한컴 정답지와 일치하지 않는 문제 해결.

대표 fixture: `samples/hwp3-sample16-hwp5.hwp` (한컴 64페이지 vs rhwp 67페이지)

## 2. 결과

| 항목 | Before | After |
|------|--------|-------|
| 페이지 수 (sample16-hwp5) | 67 | **64** (한컴 정합) ✓ |
| 페이지별 첫 단락 정합률 | 측정 안됨 | **63/64 (98.4%)** ✓ |
| HWP3 sample16 페이지 수 | 64 | 64 (회귀 없음) ✓ |
| `cargo test` | 1303 passed | 1303 passed ✓ |
| `cargo clippy` | 0 warnings | 0 warnings ✓ |
| 다른 sample 회귀 | — | 없음 (5 개 sweep) ✓ |
| WASM 빌드 | ✓ | ✓ |

## 3. 핵심 fix

### 3-1. variant cross-paragraph vpos reset 감지 (메인)

`src/renderer/typeset.rs` / `src/renderer/pagination/engine.rs` 에 추가:

```rust
// 한컴 변환본의 page-reset 인코딩 감지:
// 1. prev_end_vpos = vpos + line_height > body × 0.85
// 2. curr_first_vpos < 1500 HU (또는 < 4000 HU for empty line_segs)
// 3. prev/curr line_seg 누락 paragraph 는 인접 walk-back/forward
// 4. aux_trigger: empty paragraph bridge 가 page-bottom 까지 채운 경우
```

이를 통해 변환본 encoder 가 `line_segs.vertical_pos` 를 page boundary 마다 작은 값으로 reset 한 신호를 paginator 가 직접 인식.

### 3-2. CHARS_PER_LINE 45 → 50 (보조)

`src/renderer/composer.rs`: 변환본 CharShape `spacing=-12%` (char 압축) 반영. 50 char paragraph (pi=400 등) 가 2 line → 1 line 으로 정합.

### 3-3. variant 식별 + IR 보존

`src/parser/mod.rs`: HwpSummaryInformation 의 HWP3 시대 텍스트 + ParaShape/CharShape 비율 → `Document.is_hwp3_variant=true` (Task #1001 에서 이미 도입). 본 task 에서 typeset/engine 으로 전파.

## 4. 변경 파일

| 파일 | 변경 |
|------|------|
| `src/renderer/pagination.rs` | `PaginationOpts::is_hwp3_variant` 필드 |
| `src/renderer/pagination/engine.rs` | variant vpos reset 감지 + walk-back/forward |
| `src/renderer/typeset.rs` | `typeset_section_with_variant` + 동일 로직 |
| `src/renderer/composer.rs` | CHARS_PER_LINE 50 |
| `src/document_core/queries/rendering.rs` | `is_hwp3_variant` 전달 |

## 5. 잔존 / Follow-up

1. **페이지 33** — pi=535 "(빈)" 1 paragraph alignment 차이 (시각 무영향)
2. **variant paragraph 외곽선 micro-overflow** — 변환본 일부 paragraph 의 PARA_LINE_SEG 누락 → render 누적 y drift → 일부 페이지 마지막 paragraph 가 외곽선 9~45px overflow. 본 task 의 메인 scope (페이지 split) 와 별개 root cause. **별도 follow-up issue 권장**.

## 6. 검증 trail

- Stage 1: 페이지 강제 나눔 root cause 분석 ([`stage1`](../working/task_m100_1007_stage1.md))
- Stage 2: 구현 계획서 ([`stage2`](../working/task_m100_1007_stage2.md))
- Stage 3: 핵심 fix 구현 + 단위 검증 (62 → 67 페이지) ([`stage3`](../working/task_m100_1007_stage3.md))
- Stage 4: tuning + 회귀 검증 (67 → 64 페이지 정합) ([`stage4`](../working/task_m100_1007_stage4.md))

## 7. 결론

본 task 의 메인 목표 (HWP5 변환본 페이지 강제 나눔 한컴 정합) 달성. sample16-hwp5 가 한컴 정답지 64 페이지와 일치하며, 페이지별 단락 alignment 가 98.4% perfect match. 회귀 없음.
