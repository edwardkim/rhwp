# Task #2136 재현 샘플

## neartop_reset_sb2500.hwpx (합성)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격). 실문서 재현원은 hwpdocs
  `148753276_제3회연구노트확산세미나(김호영_최종).hwp` pi46 (p4 used 942px > 본문
  933.6px 과적, 한글 p5 — 10k r12 PI TAIL_PUSH 계열).
- 형상: pi0 채움(저장 하단 64000HU > 60000) → pi1 텍스트 문단, **저장 vpos=2500 =
  sb(5000유닛=2500HU) 정확 일치**의 새 쪽 리셋.
- 결함(수정 전): `native_near_top_reset` 상한 cv≤2000 에 500HU 차로 배제 → 측정
  fit 으로 pi1 이 1쪽 말미 과적 (1쪽).
- 기대(한글 정합): 상한 2500HU 로 리셋 인식 → pi1 새 쪽 시작 (2쪽).
- 검증: `rhwp dump-pages samples/task2136/neartop_reset_sb2500.hwpx` /
  `cargo test --test issue_2136_neartop_reset_sb2500`
