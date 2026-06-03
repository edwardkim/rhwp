# 구현 계획서 — Task #1195: 표 셀 안 빈 줄/빈 문단 높이 미반영 겹침 보정

- **이슈**: #1195
- **브랜치**: `local/task1195`
- **작성일**: 2026-06-01
- **수행계획서**: `task_m100_1195.md` (승인 완료)
- **대상 결함**: hcar-001 1쪽 표 셀[28] "1. 개인정보…동의[필수]" 제목과 중첩 표 겹침

## 결함 메커니즘 (코드 확정)

셀 안 inline 표는 `paragraph_layout.rs::layout_inline_table_paragraph`(354~) 로 배치됨.
- **L384 `let y = y_start + spacing_before;`** — `spacing_before` 만 적용.
- p[9] 는 `text_len=5`("     ") + 표 anchor 문단. line_segs 2줄(ls[0] vpos=8424 = **표 앞 빈 줄**,
  ls[1] vpos=9324). 그러나 이 함수는 **표 앞 빈 줄(line_seg)의 수직 높이를 표 시작 y 에 더하지 않음**.
- 세그먼트 분할(L431~463)은 텍스트/표 출현 순서만 계산, 빈 줄 높이는 y 전진에 미반영.
- 결과: 표가 제목(vpos=8176) 바로 뒤(표 vpos=8424, 차이 248)에 붙어 겹침.
- 후행 빈 문단(p[10], lh=700 ls=-72 → ~8px)도 별도 점검 대상이나, **1차 본질은 표 앞 빈 줄 미반영**.

> 작업지시자 지적("빈 줄/문단을 조판에 사용 안 함")의 코드 실체 = L384 가 line_segs 의 표-앞 빈 줄
> 수직 흐름(vpos)을 무시하고 spacing_before 만 더하는 것.

## 단계 구성 (4단계)

### Stage 1 — 결함 정밀 계측 (코드 무수정/디버그만)
- `RHWP_LAYOUT_DEBUG=1 rhwp export-svg samples/hcar-001.hwp -p 0` 로
  `LAYOUT_INLINE_TABLE_PARA`(L402~) 로그 수집 → p[9] 의 `y_start`, `y`, line_segs vpos/lh 확인.
- 표 시작 y 와 한컴/PDF 기준 표 시작 y 의 차이(px) 정량화.
- HWPX 동일 계측. 산출물 `output/poc/issue1195/stage1_*`.
- 빈 줄 높이를 어디서(=어느 line_seg) 얼마만큼 더해야 PDF 정합인지 수치 확정.
- → Stage1 보고서.

### Stage 2 — 표 앞 빈 줄 높이 반영 보정 (구조 가드)
- `layout_inline_table_paragraph` 에서 **표가 텍스트 앞(선행 컨트롤, L441 `tables_to_prepend`)
  에 올 때**, 표 앞 line_seg(들)의 수직 높이를 표 시작 y 에 가산.
- **가드 (회귀 차단, feedback_hancom_compat_specific_over_general)**:
  - 선행 빈 세그먼트(empty_seg before table)가 존재하는 경우에 한정.
  - line_seg vpos 기반(첫 표 line_seg.vpos − 문단 첫 line_seg.vpos) 또는 빈 줄 line_height 합산
    중 PDF 계측(Stage1)에 맞는 방식 선택.
  - 표가 문단 맨 앞(빈 줄 없음)인 기존 케이스는 y 불변 → 무회귀.
- 본문(셀 밖) inline 표도 같은 함수를 쓰므로, **셀 안/밖 공통**으로 안전한지 Stage1 계측으로 확인.
  필요 시 "표 앞 빈 줄 존재" 구조 조건으로만 한정.
- → Stage2 보고서 + 단일 회귀 테스트.

### Stage 3 — 후행 빈 문단(p[10]) 높이 점검 + 회귀 테스트
- 표 종료 후 빈 문단(p[10], "2." 앞)의 높이가 `para_y` 누적에 반영되는지 SVG 로 재확인.
  Stage2 후 잔여 겹침/간격 부족이 있으면 셀 다문단 누적 경로(`table_cell_content.rs` 654~702)
  보정.
- `tests/issue_1195_*.rs`: hcar-001 셀[28] 제목·표 비겹침(좌표 단언) + 기존 표/빈셀/inline표 보호.
- → Stage3 보고서.

### Stage 4 — 검증 + 시각 판정 + 보고서
- `cargo fmt --all --check` / `cargo build` / `cargo test --tests`(0 failed) / WASM 빌드.
- SVG 산출(`output/poc/issue1195/`) + PDF 정답지 비교.
- **작업지시자 직접 시각 판정** (셀[28] 제목·표 간격 PDF 정합).
- 최종 보고서 `task_m100_1195_report.md` + orders.

## 회귀 가드 / 위험

| 항목 | 위험 | 가드 |
|------|------|------|
| L384 y 보정 | 본문/타 셀 inline 표 과간격 | "표 앞 빈 줄(선행 empty seg) 존재" 구조 조건 한정 + Stage1 계측 |
| 후행 빈 문단 | 빈 셀/단일 빈문단 과간격 | para_count>1 + 비율 줄간격 구조 가드, 측정 clamp 회피 |
| 공통 경로 | 광범위 회귀 | cargo test --tests 전체 + 기존 표 샘플 시각 무회귀 |

## 검증 기준

- hcar-001 1쪽 셀[28] 제목·중첩 표 겹침 제거, PDF 정답지 간격 정합 (시각 판정).
- HWP·HWPX 동일 결과.
- `cargo test --tests` 0 failed + 신규 회귀.
- 기존 표/inline표/빈셀 샘플 무회귀.

## 적용/머지

- 4 stage 모두 `local/task1195` 커밋(소스+stage 보고서 동반).
- 완료 후 devel 머지 + push + WASM + 작업지시자 시각 판정 → 통과 시 이슈 #1195 close.

> 본 문서는 구현 계획서. 승인 후 Stage 1 착수.
