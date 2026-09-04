# 단계 완료 보고 — Task M100 #2067 4단계: 추출 3c (빈 runs TAC 수식 인라인)

- 작성일: 2026-07-08 / 브랜치: `local/task2067`

## 수행 내용 (동작 불변, &self 메서드 1건 + vars struct)

- **추출 3c `place_empty_line_inline_equations`** (241줄 통이동): 빈 runs 줄의 TAC
  수식 인라인 배치 (원본 Task #287). 블록 guard 는 메서드 조기 반환으로 이동.
- **`EquationTacLineVars`** (Copy 20필드, §6 의무 — 읽기 25↑): 줄-스코프 스칼라 묶음.
  `end`→`line_end`, `char_offset`→`line_char_end` 로 의미 명시(각 doc 주석,
  EmptyRunsLineVars 전례).
- **소스분기 caller 유지 (§1)**: `self.is_hwp3_variant.get()` 분기를 caller 에서
  `hwp3_indent_scale: f64` 로 계산해 값 전달 — 추출 함수 내부에서 소스분기 소거
  (치환 1곳, 사전 카운트 assert 로 검증).
- 클로저 3개(`tac_on_line`/`tac_row_for`/`row_base_x`)는 메서드 내부 통이동 —
  캡처 변이 없음(체크리스트 ⑥), 블록-지역 `continue` 1곳 내부 잔류.
- `col_area.y` → `col_area_y` 스칼라 치환 1곳.

## 게이트 (전수 통과)

fmt ✓ / clippy **0** / `--tests` **2,944/0** / issue_1116 **13/13** /
OVR 5샘플 회귀 **0건** (00014ecf ×4 + a05e6f1b).

## 계측 (라운드 8 누적)

| 함수 | 시작 (r7) | 3단계 후 | 현재 |
|---|---|---|---|
| `layout_composed_paragraph` | 2,093줄 · 분기 365 | 1,799 · 282 | **1,589줄 · 분기 237** |
| 신규 `place_empty_line_inline_equations` | — | — | 274줄 · 분기 45 |

신규 함수는 CC 25 초과 예상(§5 예외 심사 대상) — 5단계 재평가에서 공식 CC 확인 후
최종 보고서에 심사 기재. 계획한 추출 5건(1·2·3a·3b·3c) 전부 완료.

## 다음 단계

5단계 — 재평가(`--snapshot r8 --no-coverage`) + 공식 CC 대비(목표 146→100 내외) +
최종 보고서 + devel 반영.
