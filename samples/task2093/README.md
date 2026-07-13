# Task #2093 재현 샘플

## saved_single_line_spacing_after.hwpx (합성 — HWP 2020 1쪽 재조판 회귀)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격 기반). 실문서 재현원은
  `1192000_hydrogen_policy_research.hwp` (아래, rhwp 17→16쪽 = 한글 16쪽).
- 형상: pi=0 채움 줄(vpos=0, 68800HU) → pi=1 **단일 줄 + 아래 간격 sa=1000HU**
  (vpos=68800, lh=1200, gap=840, 시각 경계 bottom 70000 ≤ 본문 70018HU)
  → pi=2 vpos=1000 리셋(새 쪽 증거).
- 결함: pi=0의 `line_height=text_height=68800HU`가 10pt/160% 스타일과 모순되지만,
  rhwp가 917.3px로 신뢰해 pi=2를 2쪽으로 분리했다.
- 기대: 한글 2020 MCP PDF와 한글 2022 COM처럼 pi=0, pi=1, pi=2가 모두 **1쪽**에
  배치된다. 순수 텍스트 줄에서 저장 line/text height가 스타일상 가능한 줄 advance의
  40배를 모두 넘을 때만 rhwp가 font metrics로 재조판한다.
- 한글 기준 PDF: `pdf/task2093/saved_single_line_spacing_after-2020.pdf`와
  `pdf/task2093/saved_single_line_spacing_after-2022.pdf`는 모두 1쪽이다.
- 쪽 하단 saved-bounds의 실제 문서 회귀는 아래 1192000 문서가 담당한다.
- 검증: `rhwp dump-pages samples/task2093/saved_single_line_spacing_after.hwpx` /
  `cargo test --test issue_2093_saved_single_line_spacing_after`

## 1192000_hydrogen_policy_research.hwp (실문서 — 한글 정합 권위 검증)
- 출처: hwpdocs 코퍼스 `prism_downloads/해양수산부/1192000-201900021_D0150004-1-001_
  해양수산 수소경제 기술 활성화 방안 연구.hwp` (PRISM 공개 정책연구 보고서, 원본
  그대로 복사, 9.1MB).
- #2093 실결함 문서: 수정 전 rhwp **17쪽** (abs pi66부터 문서 끝까지 83건 연쇄
  +1 밀림 — 단일 줄(sa=1000) 문단이 안전마진 구간 탈락 + sa 게이트로 saved-bounds
  신뢰 배제) → 수정 후 **16쪽 = 한글 16쪽** (오라클 PAGE_DELTA 83건 → MATCH).
- 기준 PDF: `pdf/task2093/1192000_hydrogen_policy_research-2022.pdf`
  (한글 2022 COM, Print 액션 1-up 강제 출력 16쪽 = 편집기 PageCount 16 정합).
- 검증: `cargo test --test issue_2093_1192000_real_doc_pin`
