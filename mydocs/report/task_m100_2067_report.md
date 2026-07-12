# 최종 결과보고서 — Task M100 #2067: 리팩토링 라운드 8 (layout_composed_paragraph 잔여 분해 3차 회전)

- 이슈: #2067 / 브랜치: `local/task2067` / 기간: 2026-07-08 ~ 07-09
- 수행계획서: `plans/task_m100_2067.md` / 구현계획서: `plans/task_m100_2067_impl.md`
- 거버넌스: #1582 → #1883 마스터 플랜 v2

## 1. 결과 요약

목표(CC 146→100 내외)를 초과 달성 — 계획한 추출 5건 전부 완료.

| 지표 | 라운드 시작 (r7) | 완료 (r8) |
|---|---|---|
| `layout_composed_paragraph` 공식 CC | **146** (전체 1위) | **98** (파일 내 1위, 전체 순위권 밖) |
| 동 함수 줄수 · 분기 | 2,093 · 365 | 1,589 · 237 |
| 전체 최대 CC | 146 | **138** (`typeset_endnote_paragraphs`) |
| CC>25 예외 함수 | 87개 | 87개 (신규 진입 0) |
| 행동 회귀 | — | **0건** (통산 유지) |

신규 함수 5건 — **전부 공식 CC 25 이하** (§5 예외 심사 불요):

| 함수 | 원 국면 | 줄수 | 공식 CC | 방식 |
|---|---|---|---|---|
| `compute_line_extra_spacing` | justify 여분 간격 (145줄+전용 클로저) | 178 | 18 | free fn, 순수, `in_cell: bool` 치환 |
| `collect_shape_marker_labels` | 조판부호 마커 라벨 (63줄) | 48 | <25 | free fn, 순수 |
| `place_unmatched_line_tac_pictures` | TAC 그림 배치 (68줄) | 88 | <25 | `TacPictureLineVars`(Copy 7) + x 값 왕복 + &mut reserved |
| `place_empty_line_tac_forms` | 빈 문단 TAC 양식 (39줄) | 55 | <25 | 직접 파라미터, x 값 왕복 |
| `place_empty_line_inline_equations` | 빈 runs TAC 수식 인라인 (241줄) | 274 | **18** | `EquationTacLineVars`(Copy 20) + guard 조기 반환 |

> 3c 의 공식 CC 18 은 원 위치의 깊은 중첩(라인 루프 내부 3~5단)이 사라진 효과 —
> cognitive complexity 의 중첩 가중이 추출로 해소된 전형 사례.

## 2. 단계별 이력

1. **1단계** (a11a0362) — 정밀 분석: 후보 1 실범위 2659 확정(외부 변이 의심 1건 지역
   `let mut` 판명), 후보 2 대입 8건 match arm 오탐(체크리스트 ③), TAC 군 `continue`
   전부 블록-지역 판명, **3c 소스분기 1건 발견**(`is_hwp3_variant` indent 배율).
   구현계획서에서 4단계→5단계 세분.
2. **2단계** (052ef629) — 추출 1+2 (순수 함수 2건). OVR 게이트를 manifest §5 정식
   명령으로 정정 1건.
3. **3단계** (bd2b9ce2) — 추출 3a+3b. 슬라이스 순회 표기 2건 컴파일러 검출(E0277) 보정.
4. **4단계** (f25b301f) — 추출 3c. **소스분기 caller 유지**: `hwp3_indent_scale: f64`
   값 전달로 추출 함수 내부 소스분기 0 (치환 사전 assert 검증). `end`→`line_end`,
   `char_offset`→`line_char_end` 의미 명시.
5. **5단계** — 재평가 `--snapshot r8 --no-coverage` → `mydocs/metrics/2026-07-08-r8/`.

## 3. 게이트 (매 단계 전수 통과)

fmt ✓ / clippy 0 / `--tests` **2,944/0** / issue_1116 **13/13** /
OVR 5샘플 회귀 **0건** (00014ecf ×4 + a05e6f1b).

## 4. 사각지대 체크리스트 (6종 적용 결과)

신규 유형 없음. ③(match arm 리터럴 오탐) 실전 재확인 1건, ⑥(클로저 캡처 변이)은
3c 클로저 3개 전수 점검 — 캡처 변이 0 확인 후 통이동.

## 5. 다음 라운드 후보 (r8 공식 CC 기준)

1. `typeset_endnote_paragraphs` 2차 회전 (P6 fit 판정 연쇄) — **138** (전체 1위 복귀)
2. `typeset.rs:12338` — 129 / `layout_table_cells` — 124 (표 계열 2차)
3. `layout.rs:5913` — 121 / `typeset.rs:2313` — 120
