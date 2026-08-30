# Task M100 #6041 Stage 1 보고 — budget-first 순수 정책

- **상태**: 구현·검증 완료
- **승인 출처**: 2026-08-30 작업지시자 대화·#6041 본문
- **절차 주의**: 이 보고 파일은 최초 구현 뒤 누락을 발견해 작성했으며 사전 생성됐다고 주장하지 않는다.

## 결과

- 열 수와 무관한 visible 32M/retained 40M surface pixel 예산을 순수 planner로 만들었다.
- 예산 이내에서는 raw DPR을 유지하고 초과분만 raw→2→1.5→1로 낮춘다.
- retained는 offscreen을 먼저, 같은 단계에서는 포커스에서 먼 페이지를 먼저 조정한다.
- 편집 페이지와 print/highQuality는 항상 raw DPR을 유지한다.
- 88% release hysteresis로 예산 경계의 품질 왕복을 막는다.

## 검증

- planner focused test 13건 통과
- 단일 Canvas, 4-layer, offscreen, focus 이동, DPR 3, export와 hysteresis 계약 포함
- `pagesPerRow`·자동 열 수 상수에 의존하지 않음을 정적 회귀로 확인

