# Task M100 #1749 v2 최종 결과보고서 — saved bounds 페이지-마지막 증거에 명시적 쪽나누기 추가

## 요약

#1749 1차 게이트(`saved_flow_marks_page_last`)가 hwpdocs 코퍼스 600건 교차 검증에서
회귀 1건(결재문서 36375752: 5쪽→6쪽)을 유발함을 확인하고, 페이지-마지막 증거에
"다음 문단 명시적 쪽/구역나누기"를 추가해 해소했다.

## 변경 사항

- `src/renderer/typeset.rs`
  - `saved_flow_marks_page_last`: 다음 실줄 탐색 스캔 중 `ColumnBreakType::Page | Section`
    문단을 만나면 페이지-마지막 증거로 인정(true). 독스트링을 세 증거(문서 끝 / vpos
    리셋 / 명시적 쪽나누기) 체계로 갱신.
  - 단위 테스트 케이스 (e) 추가: 누적 vpos + 다음 문단 column_type=Page → true.
- `tests/issue_1749_saved_bounds_page_break.rs` 신설: 재현 샘플 5쪽 + pi=26 2쪽 배치 검증.
- `samples/task1749/saved_bounds_cumulative_page_break.hwpx` 추가 (36375752, PII 방침 A)
  + README 갱신.

## 검증 결과

| 항목 | 결과 |
|------|------|
| 단위 `test_saved_flow_marks_page_last` (a)~(e) | 통과 |
| 통합 `issue_1749_saved_bounds_page_break` (신규, 5쪽+pi26) | 통과 |
| 통합 `issue_1749_saved_bounds_cumulative` (1차 원 케이스 보존) | 통과 |
| 전체 `cargo test` | 145 바이너리 통과, 실패 1건은 기존 이슈* |
| `cargo clippy --release --lib` | 경고 없음 |
| hwpdocs 코퍼스 600건 render-diff(--via hwp) inventory | devel 과 완전 일치 |

\* `issue_852_hwpx_to_hwp_contract_streams::form_01_keeps_nine_cfb_streams` — CFB 스트림
경로 구분자(`/` vs `\`) Windows 환경 문제. origin/devel 에서도 동일 실패, 본 타스크 무관.

## 회귀 발견 경위 (재현 자료)

- hwpdocs 코퍼스(서울시 결재문서 + 행정규칙 + 법령 123,681건)에서 등간격 600건 추출,
  `render-diff --via hwp` 배치를 PR 적용본/devel 두 바이너리로 교차 실행.
- 유일한 inventory 차이: seoul_053(36375752) pages 5→6.
- 이등분: #1751 까지 적용 빌드는 5쪽 정상, #1752 추가 시 6쪽 → #1749 1차 게이트 단독 기인.
- 정답 근거: 저장 lineseg pi=25(vpos=134764)–pi=26(vpos=137484) 한 줄(2720HU) 간격 연속
  = 한글은 pi=26 을 2쪽 마지막 줄로 인코딩. pi=27 은 명시적 [쪽나누기].

## 남은 리스크

- 코퍼스 불일치 16건(PAGE_MISMATCH 1, STRUCT 2, OVER 13)은 devel 에서도 동일 재현되는
  기존 결함으로 본 타스크 범위 외 (별도 이슈 후보: 분할 표 라운드트립 높이 붕괴,
  HWPX/HWP5 파스 경로 saved vpos 신뢰 차이).
