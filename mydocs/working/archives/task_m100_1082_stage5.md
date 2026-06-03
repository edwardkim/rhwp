# Stage 5 보고서 — Task #1082: 미주 vpos-delta 수정 시도 → 부분개선/혼재, 잔존 블로커 특정

- 브랜치: `local/task1082` (시도 후 revert, 소스 clean)

## 시도 — 미주 누적을 vpos-delta(직전 미주 bottom 기준)로
typeset.rs 미주 다단 누적을 `current_height += px(this.bottom_vpos − prev_en.bottom_vpos)`
(컬럼 첫 미주는 자체 높이, advance 시 prev 리셋)로 변경 — 렌더 vpos 정규화 정합 목표.

## 결과 — 혼재 (net 부정 → revert)
| 파일 | 종전 | 시도 |
|------|------|------|
| 3-09'23 hwp | 626.9 | **193.1** (개선) |
| 3-09'23 hwpx | 626.9 | **193.1** (개선) |
| 3-09'22 hwp | 277.0 | **530.6** (악화) |
| 3-10'22 hwp | 158.5 | 240.3 (악화) |
| 3-11'22 hwp | 561 | 324.3 (개선) |

→ 메커니즘(미주 간 vpos 간격 누적)은 옳은 방향(3-09'23 627→193)이나, **본문(fmt 누적) ↔ 미주
(vpos 누적) 혼합 컬럼에서 base 불일치**로 일부 파일 악화. revert.

## 잔존 블로커 (정밀 특정)
- 본문 paragraph 는 typeset 에서 `fmt.total_height` 누적(vpos 와 ~trailing_ls 드리프트 보유).
  미주는 vpos-delta 누적. 한 컬럼에 본문 끝 + 미주 시작이 공존(body→endnote 전환)할 때, 미주의
  컬럼 기여 base 가 본문의 fmt-누적 current_height 와 어긋남 → 첫 미주 위치 오차가 컬럼 전체로
  전파.
- 즉 **본문·미주를 한 컬럼에서 일관된 측정공간(vpos-absolute)으로 통일**해야 클린 수정 가능.
  현 typeset 은 본문 fmt-누적/미주 vpos-누적 이원 → 부분 수정으로는 일부 회귀 불가피.

## 결론 / 권고
- C군 = 다단 미주 드리프트, 근본 메커니즘(미주 간 vpos 간격 누락) 확정. vpos-delta 수정이
  3-09'23 을 627→193 으로 크게 개선함을 실증(방향 정확).
- 클린 수정에는 **본문+미주 컬럼 누적의 vpos-absolute 통일**(또는 미주 컬럼 base 를 본문 vpos 와
  정합)이 필요 — typeset 컬럼 측정 모델 변경으로 회귀 검증 부담 큰 집중 과제.
- pi 인덱싱 정합(0b247163) 은 완료. C군 본 수정은 별도 집중 과제로 분리 권고
  (전수 sweep + 골든 + #1062 회귀 가드 동반).
