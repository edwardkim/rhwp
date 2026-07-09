# Task #2093 재현 샘플

## saved_single_line_spacing_after.hwpx (합성)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격 기반). 실문서 재현원은
  hwpdocs `1192000-201900021_D0150004-1-001_해양수산 수소경제 기술 활성화 방안 연구.hwp`
  (9.1 MB — 크기 사유로 미수록, rhwp 17쪽 vs 한글 16쪽).
- 형상: pi=0 채움 줄(vpos=0, 68800HU) → pi=1 **단일 줄 + 아래 간격 sa=1000HU**
  (vpos=68800, lh=1200, gap=840, 시각 경계 bottom 70000 ≤ 본문 70018HU)
  → pi=2 vpos=1000 리셋(새 쪽 증거).
- 결함(수정 전): pi=1 의 누적 fit 이 layout-drift 안전마진 4px 구간에서 탈락
  (917.3+16.0=933.3 > 가용 933.6−4=929.6px — 실문서 pi61 도 동일하게 안전마진
  1.6px 차 탈락)하고, `saved_single_line_bottom_fits` 의 `spacing_after <= 0.5`
  게이트에 걸려 saved-bounds 신뢰에서도 배제 → 2쪽으로 단독 과분할.
- 기대(한글 정합): 한글은 쪽 마지막 줄의 아래 간격을 쪽 하단에서 소비하지 않는다.
  pi=1 은 1쪽 하단, pi=2 는 2쪽, 전체 2쪽.
- 검증: `rhwp dump-pages samples/task2093/saved_single_line_spacing_after.hwpx` /
  `cargo test --test issue_2093_saved_single_line_spacing_after`
