# 최종 결과보고서 — Task #1612

**제목**: 잔여 −1쪽 정밀 조사 — dump-pages `hwp_used` 다페이지 혼동 수정 + razor-thin 본문 갭 특성화
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1612 · **브랜치**: `local/task1612` (base: `local/task1611`)

## 1. 배경
요인 A(#1608)·요인 B footer(#1611) 해소 후 통제셋 잔여 −1쪽 12건의 "별 요인" 조사.

## 2. 핵심 발견
1. **dump-pages `hwp_used` 메트릭 다페이지 혼동(진단 버그)**: `compute_hwp_used_height` 가
   누적 vpos 를 per-page `used` 와 비교 → diff 가 페이지마다 ~800px 누적 증가, "대형 tac 표
   과소측정(−3300px)" 으로 **오판 유발**. (실제 과소측정 아님 = 메트릭 아티팩트.)
2. **잔여 12건 실제 특성**: 단일페이지 footer 8건(본문 누적 ~20~43px 부족, razor-thin) +
   다페이지 4건(메트릭 혼동 과대 표시). 본문 per-line 은 저장 LINE_SEG(한글 동일) →
   inter-paragraph gap/spacing 미세차 = Task #1600 하드코어, **단일 surgical fix 부재**.

## 3. 수정 (진단 메트릭 정정, 저위험)
`src/document_core/queries/rendering.rs` `compute_hwp_used_height`: 페이지 시작 vpos
오프셋(`base_top`) 차감으로 per-page 화. dump-pages 출력 전용 함수라 **페이지수 불변**.

검증: 36398709 diff −3300px 아티팩트 → per-page 수십~166px 정상화. 단위 테스트 추가.

## 4. 결과
| 게이트 | 결과 |
|--------|------|
| 페이지수 (render_page_gate) | task1611 대비 **변동 0** (일치 72 유지) |
| cargo test --lib | 1976 passed / 0 failed |
| clippy | 무경고 |

## 5. 한계·후속
- **razor-thin 본문 누적 갭(8건 ~20~43px)은 코드 수정 안 함** — 고위험·저마진. 통제셋 net>0
  보장 어려움(대량 +1 회귀 우려). 특성 보존(tech 문서). 향후 한글 권위 PDF 기반 inter-para
  gap 정밀 정합 시 재시도 대상.
- 누적 효과: 통제셋 일치 60(#1600 시작)→66(#1608)→**72(#1611)** = +12. #1612 는 진단 인프라
  개선(페이지수 불변).

## 6. 산출물
- 소스: `src/document_core/queries/rendering.rs` (compute_hwp_used_height)
- 테스트: `task1612_hwp_used_height_is_per_page_not_cumulative`
- 문서: `_stage1~3`, 본 보고서, `mydocs/tech/render_minus1_page_gap.md` 갱신
