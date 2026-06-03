# Task #990 구현계획서

**관련 이슈**: [#990](https://github.com/edwardkim/rhwp/issues/990)
**브랜치**: `local/task990`
**수행계획서**: `task_m100_990.md`

---

## 단계 구성 (4단계)

### Stage 1 — 재현 + 진단 (RED)

- 빈 문단 위 treat-as-char 글상자 2개 이상 연속 구조의 committed 회귀 샘플 탐색.
  없으면 최소 재현 HWPX(`samples/` 또는 `tests/` 픽스처) 추가.
- TDD: 박스 advance = LINE_SEG 1회분(`lh + ls`)임을 검사하는 통합 테스트 작성
  → 현재 코드에서 **FAIL(RED)** 확인 (advance ≈ 2배).
- `layout_paragraph()`(`layout.rs:3100`)가 빈 호스트 문단에서 advance 하는 양과,
  `layout_shape_item()`(`layout.rs:4622~4692`)의 재진행을 로그/덤프로 확정.
- 산출물: `task_m100_990_stage1.md`

### Stage 2 — 정정 (GREEN)

- `layout_shape_item()` 의 `!has_real_text` 분기 정정:
  - `shape_y` 를 `para_start_y[para_index]`(호스트 문단 시작)로 산출.
  - `result_y` 는 입력 `y_offset` 유지(재진행 제거) — `FullParagraph` 항목이
    이미 LINE_SEG 만큼 진행한 경우에 한함.
  - 진행 여부 판정: `y_offset > para_start_y[para_index]` (FullParagraph 선행 emit 확인).
- Stage 1 RED 테스트 → **GREEN** 전환 확인.
- 변경 최소화 — `treat_as_char` 글상자 + 빈 문단 경로만 영향, 그 외 분기 불변.
- 산출물: `task_m100_990_stage2.md` + 소스 커밋

### Stage 3 — 검증 (회귀)

- 전체 `cargo test` — 0 회귀 확인.
- 광범위 회귀: 기존 샘플 다수 `export-svg` 전/후 차분
  (treat-as-char 글상자/그림 보유 샘플 우선) — 음의 시프트 0건 확인.
- 내부 검증 문서: 글상자 3개 advance 측정 → PDF 비율(1.92) 정합 확인(로컬).
- `cargo fmt`(신규/수정 파일 한정), `cargo clippy`.
- 산출물: `task_m100_990_stage3.md`

### Stage 4 — WASM 빌드 + 최종 보고

- Docker WASM 빌드(`pkg/` 갱신) — rhwp-studio 시각 확인.
- 최종 결과보고서 `mydocs/report/task_m100_990_report.md` 작성.
- `mydocs/orders/20260518.md` 상태 갱신.
- 산출물: `task_m100_990_report.md` + 커밋 + merge 요청

---

## 영향 범위

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | `layout_shape_item()` `!has_real_text` 분기 (shape_y / result_y) |
| `tests/` | 통합 테스트 1건 (신규) |
| `samples/` 또는 `tests/` 픽스처 | 최소 재현 샘플 (기존 샘플로 대체 가능 시 생략) |

## 리스크

- treat-as-char 글상자가 **텍스트 있는 문단**에 놓인 경우는 `has_real_text=true`
  분기로 본 수정과 무관 — 회귀 위험 낮음.
- `para_start_y` 미등록 케이스(이론상) — `unwrap_or(y_offset)` 폴백 유지.
