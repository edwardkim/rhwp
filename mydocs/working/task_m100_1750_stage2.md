# Task #1750 Stage 2 완료보고서 — 분할 경로 저장 전체-리셋 가드 + 테스트

## 원인 규명 정정 (계획서 대비)
수행계획서는 "진입 가드 spacing_before 미반영"을 원인으로 잡았으나, 구현 중 정밀 재계산으로 정정:
- `first_line_h = fmt.line_heights[0] = 16.0px`(콘텐츠 높이, 조사 시 line_advance 25.6px 를
  오독). `remaining(29.1) < 16.0+9.3(sp_b)=25.3` 도 거짓 → **sp_b 반영만으로는 미해결**.
- 첫 줄 가시 하단(976.0+9.3+16.0=1001.3px)은 avail(1005.1px) **안** — 초과분은 트레일링
  줄간격뿐이라 콘텐츠-fit 시멘틱(#643)으로는 rhwp 분할이 자체 모순은 아님.
- **결정 신호는 저장 LINE_SEG**: pi22 의 ls[0] vpos=700(새 쪽 상단 near-top 리셋, 직전
  pi21 은 vpos 72680 페이지 하단부) — 두 줄 전체가 새 쪽. 단일 단 vpos-reset 가드
  (typeset.rs:2279)는 `cv == 0` 만 인정(HWP3-tolerance 예외 제외)해 cv=700 문서가 분할
  경로로 새어 들어옴.

## 수정
`src/renderer/typeset.rs` 분할 진입 가드에 `stored_whole_para_reset` 추가:
- 조건: 단일 단 + 첫 실줄 vpos ∈ (0, 2500] (near-top 리셋) + 직전 문단 마지막 줄
  vpos+lh > 60,000HU (페이지 하단부) → 분할 대신 `advance_column_or_new_page()`.
- 전체 배치 실패 후 분할 직전에만 적용 — 일반 흐름/vpos-reset 보수 기준(#321/#418) 불변.
  누적좌표 문서(cv 가 계속 증가)는 cv≤2500 이 성립하지 않아 자연 배제.
- 계획서의 sp_b 가드 변경은 재현 케이스에 무효과(위 정정)라 **미적용** (diff 최소화).

## 검증
- 통합테스트 `tests/issue_1750_split_guard_spacing_before.rs`: pi22 1쪽 미배치 + 2쪽 배치 — 통과.
- dump-pages: pi22 → 2쪽 상단 FullParagraph, 총 5쪽 불변, **2쪽 used 1003.5px = 저장 lineseg
  추정과 diff 0.0px** (수정 전 1쪽 1010.9px overfill).

## 상태
완료. Stage 3 (회귀 검증 + 최종보고) 진행. 이슈 #1750 에 규명 정정 코멘트 예정.
