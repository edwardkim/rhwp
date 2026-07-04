# Task #1789 재현 샘플

## exclusion_probe_line_spacing.hwpx
- 출처: 서울 정보소통광장 정보공개 결재문서(공개) 36385142 (동작소방서 재난관리과) —
  opengov 결재문서 계열, PII 방침 A(그대로 동결).
- 특성: 문단 기준(vert=문단, off=9391) 자리차지 표(603×309px) + 직전 문단 0.8 의 줄이
  표 위 공간에 잉크(th=16px)로는 들어가고 line_spacing(9.6px) 포함 시 ~2.8px 겹침.
- 결함(수정 전): HWPX 경로 전용 `overlaps_zone` 프로브(task 1510)가 lh+ls 로 겹침
  판정 → 문단 0.8 이 표 아래(≈875px)로 밀려 345px 변위.
- 기대(한글 정합): 문단 0.8 첫 줄 y≈529.9px — 저장 lineseg vpos=34925 및 HWP5 재파스
  렌더와 일치.
- 검증: `cargo test --test issue_1789_exclusion_probe_line_spacing` /
  `rhwp render-diff samples/task1789/exclusion_probe_line_spacing.hwpx --via hwp`
