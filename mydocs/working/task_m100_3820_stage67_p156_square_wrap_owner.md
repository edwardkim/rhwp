---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-08
---

# Task #3820 Stage 67 — production HWP p156 Square-wrap owner

## 범위

Stage 66의 p144 HWPX BehindText placeholder 정렬은 완료했다. 이 Stage는 #3820 원장의
초기 215쪽 기준인 정책연구 문서 p156 그림 64의 Square-wrap 본문 침범을 독립적으로
재현·분석한다. 다른 page owner, p94/p106/p107--108, 자동 후보 원장은 이 Stage에서
함께 변경하지 않는다.

| 항목 | 경로 |
| --- | --- |
| HWP 입력 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` |
| PDF oracle | `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf` |
| 검사 범위 | PDF p155--p157 |

편람 p155--p157은 이 Stage의 대상이 아니다. #3820의 최신 comment가 별도 production
fixture를 나열하지만, p156의 `pi=1692/ci=1`, 215쪽 기준 계약은 정책연구 HWP/PDF 세트에
속한다. 따라서 편람 sweep은 결함의 증적·판정에 사용하지 않는다.

## 검증 계획

1. 현 commit의 release-test binary로 p155--p157 direct PDF visual sweep을 만든다.
2. 그림 64 및 같은 y-band의 paragraph source `cs/sw`, render-tree text right edge와
   Square-wrap anchor lifetime을 대조한다.
3. PDF와 다른 source owner/paint가 재현될 때만 최소 보정을 다음 구현 Stage에 기록한다.
   page count나 raster aggregate만으로 결함 여부를 판단하지 않는다.

## 결과 — 현 head에서는 비재현

`target/task-3820-stage65-hwpx-noninline-tac/release-test/rhwp`로 정책연구 HWP와
직접 PDF를 p155--p157 범위에서 다시 raster 비교했다. sweep은 요청 3쪽을 모두 완료했고,
검토용 p156 패널은 다음에 보관한다.

- [review_156.png](../pr/assets/task_m100_3820_stage67_policy_p156_square_wrap_owner/review_156.png)
- [summary.json](../pr/assets/task_m100_3820_stage67_policy_p156_square_wrap_owner/summary.json)

그림 64(`pi=1692`, `ci=1`)의 현재 render-tree 경계는
`x=436.5..661.4`, `y=90.1..392.9`이다. 같은 Square-wrap 문단 `pi=1697`은 9개
`TextLine`으로 분할되며 오른쪽 끝이 `x=430.5` 이하여서 그림 왼쪽과 최소 6 px의
간격을 유지한다. 패널도 PDF처럼 그림 왼쪽에서 본문이 줄바꿈되고 그림 내부를
침범하지 않는 것을 확인했다.

따라서 #3820에 기록된 p156 본문-그림 교차는 현재 `upstream/devel` 기반 head에서
재현되지 않는다. 폰트 glyph/line-width의 미세 차이는 남지만, 이 Stage는 Square-wrap
침범에 대해 코드나 baseline을 변경하지 않는다. 다음 Stage는 아직 직접 대조가 남은
독립 page owner를 대상으로 한다.
