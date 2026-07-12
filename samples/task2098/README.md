# Task #2098 재현 샘플

## page_bottom_fixed_anchor_vpos0.hwpx (합성)
- 출처: 수작업 합성 (`samples/tac-host-spacing.hwpx` 골격). 실문서 재현원은 hwpdocs
  opengov 결재문서 계열 (36358528/36376848 — 발신명의 쪽-하단 고정 틀, 2쪽↔한글 1쪽).
- 형상: 본문 2문단(마지막 vpos=6000) → **빈 앵커 문단(vpos=0)** + 쪽-하단 고정 표
  (pageBreak=NONE, wrap=자리차지, vertRelTo=PAGE, vertAlign=BOTTOM, 10000HU=133px).
- 결함(수정 전): vpos-reset 가드(cv==0 && pv>5000)가 쪽 기준 절대배치 앵커의 vpos=0 을
  흐름 리셋으로 오독 → 표 진입 전 새 쪽 → 틀 2쪽 단독 (`RHWP_TABLE_DRIFT` 진단
  cur_h=0.0 으로 확인).
- 기대(한글 정합): 앵커는 리셋 신호 제외 → page-bottom footer 경로(배타영역 fit)가
  1쪽 하단 배치. 전체 1쪽.
- 검증: `rhwp dump-pages samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx` /
  `cargo test --test issue_2098_page_bottom_fixed_anchor_vpos0`
