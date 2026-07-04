# Task #1750 최종 보고서 — 분할 경로가 저장 LINE_SEG 의 전체-리셋 신호를 무시하는 결함 수정

## 요약
hwpdocs 페이지·PI 대조 S2(overfill) 원인 규명에서 확정한 결함 수정. 쪽 경계에서 저장
LINE_SEG 가 문단 **전체**를 새 쪽 상단으로 인코딩(첫 줄 near-top vpos 리셋)했는데도 rhwp 가
문단을 분할해 첫 줄을 이전 쪽 말미에 남기던 문제. 대표 `수소전기차 별표5`(3024019)
PI_MISMATCH(pi22·pi40) → **MATCH**, 2쪽 사용고가 저장 lineseg 와 diff 0.0px.

## 원인 (이슈 #1750 + 규명 정정 코멘트)
- 초기 가설(진입 가드 spacing_before 미반영)은 구현 중 정정 — first_line_h 는 16.0px 라
  sp_b 를 더해도 가드 미발동이며, 첫 줄 가시 하단(1001.3px)은 avail(1005.1px) 안.
- **확정 원인**: pi22 저장 ls[0] vpos=700(새 쪽 상단 리셋, 직전 문단은 vpos 72680 하단부).
  단일 단 vpos-reset 가드(typeset.rs)는 `cv == 0` 만 인정(HWP3-tolerance 예외 제외)해
  near-top(700HU) 리셋 문서가 분할 경로로 누수 → 분할 루프가 첫 줄을 무조건 배치.

## 수정
`src/renderer/typeset.rs` 분할 진입 가드에 `stored_whole_para_reset` 추가:
단일 단 + 첫 실줄 vpos ∈ (0, 2500] + 직전 문단 끝 vpos+lh > 60,000HU(페이지 하단부) 이면
분할 대신 페이지 넘김. 전체 배치 실패 후 분할 직전에만 적용 — 일반 흐름/기존 vpos-reset
보수 기준(#321/#418) 불변, 누적좌표 문서는 조건 불성립으로 자연 배제.

## 검증
| 항목 | 결과 |
|------|------|
| 재현·코퍼스 원본 (한글 OLE) | PI_MISMATCH → **MATCH** (5=5쪽, 연쇄 pi40 포함) |
| cargo test --lib | 2050 passed / 0 failed |
| 통합 (1417/546/1486/diag1042 + 신규 issue_1750) | 11 passed / 0 failed |
| byeolpyo1/4 · 승강기 · task1700 · task1745 | 4/26/42/1/3쪽 무회귀 |
| 코퍼스 mismatch 39건 | 대상 1건(2 mismatch) MATCH 전환, 악화 0 |
| 코퍼스 MATCH 표본 150건 | 150/150 유지 |
| rustfmt / clippy --lib | 통과 |

## 한계 / 후속
- near-top 임계 2500HU·하단부 임계 60,000HU 는 본 케이스+기존 코드 선례(#1086 의 2500) 기준 —
  다른 리셋 오프셋 문서 발견 시 조정.
- 결함 A(#1749, saved_single_line_bottom_fits 저장 vpos 신뢰 overfill)는 별도 타스크.
- S1/S2 잔여(메트릭 누적 차 razor-thin)는 개체 단위 오라클(#1720 계열) 영역.

## 산출물
- 소스: `src/renderer/typeset.rs` (+ 통합테스트 `tests/issue_1750_split_guard_spacing_before.rs`)
- 재현: `samples/task1750/split_guard_spacing_before.hwp` + README
- 검증 TSV: `output/poc/hwpdocs_pipage/{bad39,match150}_recheck_1750.tsv`
