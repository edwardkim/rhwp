# Task #2093 재현 샘플

## saved_single_line_spacing_after.hwpx (합성 — rhwp 저장 레이아웃 신뢰 시맨틱 핀)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격 기반). 실문서 재현원은
  `1192000_hydrogen_policy_research.hwp` (아래, rhwp 17→16쪽 = 한글 16쪽).
- 형상: pi=0 채움 줄(vpos=0, 68800HU) → pi=1 **단일 줄 + 아래 간격 sa=1000HU**
  (vpos=68800, lh=1200, gap=840, 시각 경계 bottom 70000 ≤ 본문 70018HU)
  → pi=2 vpos=1000 리셋(새 쪽 증거).
- 결함(수정 전): pi=1 의 누적 fit 이 layout-drift 안전마진 4px 구간에서 탈락
  (917.3+16.0=933.3 > 가용 933.6−4=929.6px — 실문서 pi61 도 동일하게 안전마진
  1.6px 차 탈락)하고, `saved_single_line_bottom_fits` 의 `spacing_after <= 0.5`
  게이트에 걸려 saved-bounds 신뢰에서도 배제 → 2쪽으로 단독 과분할.
- 기대(rhwp, 저장 레이아웃 신뢰): pi=1 은 1쪽 하단, pi=2 는 2쪽, 전체 **2쪽**.
- **oracle 주의**: 이 fixture 의 저장 LINE_SEG(68800HU 채움 줄 등)는 실제 텍스트
  높이와 무관한 수작업 값이다. 한글 편집기는 열 때 문서를 **재조판**하므로
  (저장 LINE_SEG 무시) 전체가 1쪽에 들어간다 — 한글 2020 MCP
  (`pdf/task2093/saved_single_line_spacing_after-2020.pdf`) / 한글 2022 COM
  (`...-2022.pdf`) 공히 **1쪽**. 따라서 한글 재변환 PDF 는 이 fixture 의 정답지가
  아니며, fixture 는 변환-HWP 계열(한글이 저장한 진짜 LINE_SEG 를 신뢰해야 하는
  경로)의 rhwp 시맨틱 회귀 핀으로만 사용한다. 한글 정합의 권위 검증은 아래
  실문서로 수행한다.
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
