# 최종 결과보고서 — task_m100_1472

**이슈**: upstream edwardkim/rhwp #1472 — ParaShape indent 2배 어긋남 (HwpUnitChar 경로)
**브랜치**: `local/task1472` (devel 분기)
**마일스톤**: M100 (v1.0.0 "한컴 동일 조판")
**완료일**: 2026-06-25
**방향**: 작업지시자 결정 — 재설계(indent IR 정확성과 미주 페이지네이션 보정 분리)

---

## 1. 결과 요약

`ParaShape.indent` 를 IR 에서 **정답(full HWPUNIT, HWPX 일치)** 로 유지하도록 재설계하고,
종전 `is_hwp3_variant` 의 `indent /= 2` 가 (의도치 않게) 수행하던 미주 페이지네이션 보정을
**값 훼손이 아닌 미주 TAC 수식 흐름의 `indent_scale` 로 이전**했다. 결과:

- 3-11월/3-09월 등 모의고사군 본문 내어쓰기 indent **ir-diff 0건**(HWPX 완전 일치, 정답).
- 미주 한컴-PDF parity 테스트 전부 통과(`issue_1082/1139/1256/1284`).
- 전체 `cargo test` **148 타깃 0 실패**(diag_1042·비변환본·sample16 회귀 없음).

## 2. 근본 원인 (요약 — 상세 `tech/task_m100_1472_rootcause.md`)

- HWP5 바이너리는 indent 를 full 로 저장. `-1608` 은 `parser/mod.rs` 의 `is_hwp3_variant` 분기
  `ps.indent /= 2` 산출물(#1042). 이 분기는 summary 의 옛 연도(예: 1997 템플릿)로 오탐.
- 핵심: 본문은 IR `line_segs` 를 그대로 써 indent 가 X offset 만 바꾸지만, **미주 TAC 수식 흐름**은
  `available_width`(= 열폭 − effective_margin_left[indent 포함] − margin_right) 로 수식을 행에
  packing 하므로 indent 가 미주 높이/페이지네이션에 직접 영향.
- 종전 코드는 `indent /= 2`(IR 절반) + 미주 수식 `indent_scale = 2.0`(비셀) 으로,
  **수식 effective indent = (indent/2)×2 = full** 을 만들어 한컴 미주 조판과 정합시켰다.
  즉 indent /2 는 단순 버그가 아니라 미주 페이지네이션의 load-bearing 보정이었다.

## 3. 재설계 (불변식 보존)

**불변식**: 미주 수식 effective indent 는 그대로(=full), 본문 indent 만 half→full.

- `src/parser/mod.rs`: `is_hwp3_variant` 블록에서 **`ps.indent /= 2` 제거**(spacing /2 는 유지 —
  변환본 세로 밀도 한컴 정합에 필요). → IR indent = full.
- 미주 수식 `indent_scale` 을 **변환본에서만 절반**(2.0→1.0, 셀 1.0→0.5)으로 보정하여
  effective indent(=IR×scale) 를 종전과 동일(full)하게 유지. 적용 사이트(4):
  - `src/renderer/layout/paragraph_layout.rs:2142` (수식 available_width, 렌더)
  - `src/renderer/layout/paragraph_layout.rs:4356` (수식 row base_x, 렌더)
  - `src/renderer/height_measurer.rs:447` (Paginator 측정 경로)
  - `src/renderer/typeset.rs:format_paragraph` (TypesetEngine 높이/페이지네이션 — 핫패스)
    · `TypesetEngine` 에 `is_hwp3_variant: Cell<bool>` 추가, typeset 진입 시 set.

> 비변환본은 IR·scale 모두 불변 → 거동 무변경. 셀/비셀 모두 effective indent 보존.

## 4. 검증

| 항목 | 결과 |
|------|------|
| 3-11월 ir-diff indent | **0건** (full = HWPX) |
| 3-09'24 ir-diff indent/sb/sa | **0건** |
| `issue_1082` 미주 드리프트(5케이스) | **통과** (종전 재설계 1차 시 113px → 정합) |
| `issue_1139/1256/1284` 미주 parity | **통과** |
| hwp3-sample16-hwp5 (진짜 변환본) | 64쪽 불변, 미주 세로밀도 한컴 PDF 정합 유지 |
| 비변환본(exam_eng/science/21_언어 등) | 무변경(test_490/521/548/624 포함 통과) |
| 전체 `cargo test` | **148 타깃 0 실패** |
| cargo fmt(변경 파일) | 적용 |

## 5. 영향 / 비고

- 모든 HWP5 문서의 내어쓰기 indent 가 IR 정답(full)으로 정규화 → 한컴 정합 개선.
  변환본 미주 페이지네이션은 보정 이전으로 동일 유지(회귀 0).
- serializer(HWP5)·HWPX 라운드트립 무변경(indent 훼손 제거로 IR↔파일 정합도 개선).
- `detect_hwp3_variant` 오탐 자체는 남아 있으나(모던 vs 변환본 구분 불가 — 파일 신호 동일),
  본 재설계로 그 오탐이 **indent 에는 무해**(full 유지)해졌다. spacing /2 는 변환본 세로밀도에
  필요하므로 유지하며, 오탐 모던 파일의 일부 spacing 값(대개 0)에 한해 영향은 경미.

## 6. 산출물

- 소스: `parser/mod.rs`, `renderer/layout/paragraph_layout.rs`, `renderer/height_measurer.rs`,
  `renderer/typeset.rs`.
- 문서: `plans/task_m100_1472.md`(진단), `plans/task_m100_1472_v2.md`(재설계 수행계획),
  `tech/task_m100_1472_rootcause.md`(근본 원인+메커니즘), 본 보고서.
