# PR #2397 검토 — 그림 속성 적용 파이프라인 책임 분리 (postmelee, #2392)

- collaborator 의 정식 타스크 산출물 — 계획서·단계 4·보고서 동봉 (하이퍼-
  워터폴 전 과정), Closes #2392 / Related #2022
- 본질: PicturePropsDialog.handleOk (CC 348/381줄) 를 순수 모델
  (picture-props-apply-model — runtime import 0, 정책 fixture 20)과
  orchestration 으로 분리. **CC 348→2.**

## 계량 (기여자, #2130 산식 준수)

Total CC −276 / Top20 합 −301 / CC>25 합 −348 / CC>100 7→6 — 산식 교훈
(통이동≠감소, 총량 지표)을 정확히 반영했고, #2124 스냅샷 누적치와의 분리
귀속까지 명시. stable changed function = handleOk 단 1.

## 행동 보존 증거

setter 5회 호출·인자 순서 핀 + 정책 fixture 20(diff/always-send/Through 보존/
TakePlace→TopAndBottom) + mutation surface 가드 + snapshot/fallback 계약 유지.
구 코드 대면 시 신규 구조 전용 핀 3건만 실패(모델 참조 가드 — 예상 형태).

## 재실증

devel 충돌 0(orders 자동 병합) / tsc OK / npm test 403/403 / CI 전 항목 green.

## 판단

**merge 권고.** 리팩토링 거버넌스 2원칙(SOLID+복잡도)의 프론트 적용 모범 —
maintainer 결산 교훈을 인용·준수한 계량 보고가 특히 좋다.

## 처리 결과 (2026-07-19)

merge 완료(admin) + 감사 코멘트. #2392 는 close-issues 워크플로 자동 close.
