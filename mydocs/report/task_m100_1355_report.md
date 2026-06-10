# 최종 결과보고서 — Task #1355

## 이슈
해설(미주) 영역 세로 여백 누적 → 콘텐츠 페이지 오버플로 (#1355, M100)

## 원인
미주 문제-제목 직전 콘텐츠의 trailing line-spacing 이 "미주 사이" gap 을 이미 만든
경우에도, 상류 `vpos_adjust`(layout.rs)가 saved-vpos 기준 gap 을 **한 번 더** 더해
제목 앞 여백이 약 2배(이중계상)가 되었다. 기존 `compact_endnote_title_gap_*` 게이트는
`prev_gap<50px` 등 조건으로 본 케이스(gap=26.5px)를 잡지 못했다.

## 해결
`src/renderer/layout.rs` 미주 제목 배치 직후, **흐름 전진량(y_before_vpos −
prev_content_bottom_y)이 gap 이상**이면 제목을 흐름 위치로 되돌려 gap 1회만 남기는
조건부 클램프 추가. 흐름이 gap 을 만들지 않은 경우(`prev_bottom == y_before`)는 무영향.

- 전 페이지 계측: doubling 케이스 6건에만 발화, 오탐 0건
- 미주 간격 SSOT 부재 영역이라 전면 통일 대신 **조건부 게이트**로 접근
  ([[tech_trailing_model_no_ssot]], [[tech_endnote_tail_backtrack_atomic_vs_text]])

## 검증
- 시각: p18 문30/문23/문24 라벨이 PDF와 정확히 일치(510/1230/1470), 문24 답안 본문 내
  수용; p19 문28 드리프트(720→690) 교정
- `cargo test --lib`: 1618 passed, 0 failed
- `issue_1082`(미주 드리프트 4 exam) 5 passed — 회귀 없음
- 신규 `issue_1355`: 수정 비활성 시 y=362.1 FAIL → 수정 시 pass (회귀 포착 검증)
- clippy 경고 없음

## 범위 밖 (분리)
p21→p22 `(ⅰ)~(ⅲ)에서` 다음 쪽 밀림은 제목 gap 이중계상과 **다른 원인**(해당 페이지
doubling 제목 없음, 수정 전후 동일). 전 페이지 오버플로 총합 50.1px 는 **#1336 기지
잔여(미주 다단 fit 캡, exam별 하드튜닝 보류)** 로, `issue_1082` 가 60px 바운드로 추적 중.
근본 정정은 별도 타스크 권장.

## 영향
미주 제목 배치 한정. 본문/일반 문단 무영향. 다른 exam 샘플 오버플로 미증가.
