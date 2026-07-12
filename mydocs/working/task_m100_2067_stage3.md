# 단계 완료 보고 — Task M100 #2067 3단계: 추출 3a+3b (TAC 그림/양식 배치)

- 작성일: 2026-07-08 / 브랜치: `local/task2067`

## 수행 내용 (동작 불변, &self 메서드 2건)

- **추출 3a `place_unmatched_line_tac_pictures`**: run 범위 밖 미매칭 TAC 이미지 배치.
  구현계획서대로 `TacPictureLineVars`(Copy 7필드) + `x` 값 왕복(반환) +
  `reserved_tac_picture_height: &mut Option<f64>`. 블록 진입 guard 도 메서드 내부로
  통이동(호출부 무조건 호출, guard 실패 시 x 그대로 반환 — 동작 동일).
- **추출 3b `place_empty_line_tac_forms`**: 빈 문단 TAC 양식 배치. 읽기 9개 —
  직접 파라미터(§6 임계 미만). `x` 값 왕복.
- 파라미터 타입 정리: `cell_ctx.as_ref()` 를 caller 에서 넘겨 `Option<&CellContext>` 로
  수취(내부 `.as_ref()` 소거). 슬라이스 파라미터 순회 `&tac_offsets_px` → `tac_offsets_px`
  보정 2건 (컴파일러 E0277 검출).
- 블록-지역 `continue` 1곳은 메서드 내부 루프에 그대로 잔류 — 외부 제어 흐름 무변.

## 게이트 (전수 통과)

fmt ✓ / clippy **0** / `--tests` **2,944/0** / issue_1116 **13/13** /
OVR 5샘플 회귀 **0건** (00014ecf ×4 + a05e6f1b).

## 계측 (라운드 8 누적)

| 함수 | 시작 (r7) | 2단계 후 | 현재 |
|---|---|---|---|
| `layout_composed_paragraph` | 2,093줄 · 분기 365 | 1,876 · 304 | **1,799줄 · 분기 282** |
| 신규 `place_unmatched_line_tac_pictures` | — | — | 88줄 · 분기 17 |
| 신규 `place_empty_line_tac_forms` | — | — | 55줄 · 분기 5 |

## 다음 단계

4단계 — 추출 3c (빈 runs TAC 수식 인라인 241줄, `EquationTacLineVars` struct +
hwp3 indent 배율 caller 유지).
