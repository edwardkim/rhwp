# 최종 결과보고서 — Task #1355 (v2)

## 이슈
해설(미주) 제목 앞 세로 여백 이중계상 → p18 문30 위 여백 과다 → 문24 답안 본문 초과,
p19 문28 드리프트 (#1355, M100)

## 원인
미주 문제-제목 직전 문단이 **수식 전용 tail**이고 그 trailing line-spacing 이 흐름에
"미주 사이" gap 을 이미 만들었는데, 제목의 saved LINE_SEG vpos 가 직전 bottom 보다
크게 점프(원본 단/쪽 경계)하면 상류 `vpos_adjust` 가 saved 기준 gap 을 **한 번 더** 더해
제목 앞 여백이 약 2배가 된다.

## v1 폐기와 v2 해결
- **v1(PR #1356 closed)**: `flow_advance ≥ gap` 단일 게이트 → 이중계상과 정상 gap 의
  시그니처가 동일해 구분 불가 → 2022_oct/sep 4개 PDF-정합 테스트 회귀. 폐기.
- **v2**: 계측으로 distinguisher 확정 — **직전 문단 textless(수식전용)** +
  **flow_advance≈gap** + **saved-vpos 점프 > 5000HU**(원본 단/쪽 경계) 일 때만 제목을
  흐름 위치(y_before_vpos)로 정정. 직전이 텍스트거나 점프가 작은 순차 미주는 제외.

`src/renderer/layout.rs` vpos_adjust 직후 조건부 클램프(page/lazy base null + preserve
비활성).

## 검증
- 시각: p18 문30/문23/문24 PDF 정합(510/1230/1470), 문24 본문 수용; p19 문28(690) 정합
- **전체 `cargo test`: 0 failed** (121 바이너리) — v1 회귀 4종(issue_1139/1189/1274/1284)
  전부 통과, issue_1082 미주 드리프트 5 passed
- 신규 `tests/issue_1355_endnote_title_gap_double.rs`: p18 첫 미주 제목 y<350 가드
  (이중계상 시 ~362로 FAIL)
- clippy 경고 없음

## 교훈
- 미주 gap 판별은 `flow_advance` 단독으론 불가 — **saved-vpos 점프 + 직전 textless**
  복합 신호 필요([[tech_endnote_title_gap_double_count]])
- 레이아웃/미주 변경은 **PR 전 전체 cargo test** 필수, `--lib` 만으론 통합 테스트 누락
  ([[feedback_full_cargo_test_before_pr]])

## 범위
미주 제목 배치 한정. p21→p22 페이지높이 오버플로는 별건 #1357(누적기 정합, 바운드 유지).
