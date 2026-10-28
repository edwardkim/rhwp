# Task M100 #6040 Stage 3.2B 보정 — 클릭 없는 읽기 화질 보호

- 사용자 승인: 2026-09-09, “최소한 사용자가 편집할 때나 문서를 읽는 것에 영향이 없게”.
- 이전 제품: `efb9823c0`, 승인 반영 계획: `15c21b04b`.
- 상태: 로컬 구현·테스트·실문서 확인 완료, 사용자 체감 확인 대기. B 전체 완료 아님.
- 연관 원인: #6467 비편집 쪽 예산과 #6042 정착 visible 64M gate. 현재 #6040 수용 보정으로 진행.

## 원인과 변경한 판단

제보 이미지의 배율은 211%였다. 실제 exam_kor 200%에서도 편집 focus 0을 유지한 채 1번 쪽으로
이동하면 스크롤·렌더가 모두 끝나도 DPR 1.5였고, 클릭해 focus를 옮겨야 DPR 2가 됐다.
정착 승격의 64M gate는 페이지 전체×레이어 비용으로 계산하므로 높은 배율의 읽기 페이지를
의도적으로 낮은 화질에 남겼다. 기존 #6042 보고에서도 200%의 이 예외를 수용했으나, 이번 사용자
요구와 충돌한다. 100%에서 복원됨을 확인한 것만으로 읽기 전체의 화질을 보장했다고 볼 수 없다.

새 정책은 **정착한 visible 전체와 편집 쪽의 원래 화면 DPR 보호**다. 64M 한도를 높이는 것이 아니라
그 한도로 읽기 화질을 낮추던 결정을 폐기한다. 일반 plan(default: zoom·resize·편집 포함)에서도
visible raw DPR lock을 유지해 다른 이벤트가 직전 승격을 다시 낮추지 않게 한다. 화면 밖 쪽은 기존
예산·히스테리시스·LRU·prefetch admission을 사용한다. 스크롤 중에는 기존 surface DPR을 유지하고
150ms 정착 뒤 최신 visible을 페이지별 scheduler에서 갱신한다. 편집 focus를 자동으로 옮기지 않는다.

이전 material-visible 8px 중복 교차 계산과 64M/raw→1.5→fallback resolver는 제거했다. 기존
visibility 집합을 재사용하며 Worker나 전체 문서 선렌더를 추가하지 않았다. 기존 Canvas당
67,108,864px 크기 clamp와 출력 profile 계약은 그대로다. 따라서 raw 보호는 기존 렌더러의 안전
상한까지 없애거나 모든 크기·장치에서 무제한 해상도를 보장한다는 뜻이 아니다.

## 재현 테스트와 검증

- 변경 전 새 테스트가 `canvas2d/DPR 1.5/100%/default`에서 visible 일부를 DPR 1로 낮춰 실패했다.
- 변경 후 Canvas2D/CanvasKit × DPR 1/1.5/2/3 × zoom 34/100/200/211/300%의 40조합에서
  scrolling 유지, settled/default raw 복원, 편집 focus 불변, 클릭 뒤 다른 visible 보호,
  예산 초과 offscreen 쪽 demotion을 통과했다. 각 조합은 둘 이상의 visible을 포함한다.
- 집중 51개 통과. 최종 전체 Studio **1,556 total / 1,555 pass / 1 skip / 0 fail**.
- `npm run build`(TypeScript 포함), `git diff --check` 통과. 기존 bundle 경고 외 실패 없음.
- Rust/WASM 변경 없음. 실제 브라우저는 기존 진단용 WASM이며 배포판 성능 비교 자료가 아니다.

## 실제 브라우저 확인

Canvas2D, DPR 2, viewport 1232×863에서 진단 패널로 측정했다. 전체 원시 대신
[최소 JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md)을 남긴다.

| 상황 | 결과 |
| --- | --- |
| exam 200%, focus 0, visible 1, 클릭 전 | 기존 DPR 1.5 → 후보 DPR 2 |
| 후보 동일 페이지 클릭 후 | DPR 2/scale 4 그대로, page box·scroll 좌표 동일 |
| exam 211%, focus 1, visible 2 | 클릭 없이 DPR 2/scale 4.22, queue·timer·pending image 0 |
| 두 쪽 100%, focus 1, visible 4·5 | 두 페이지 모두 DPR 2 |
| 두 쪽 200% 설정, 실제 visible 6 한 쪽 | DPR 2, offscreen 4번 쪽은 DPR 1. 두 visible 실측으로 과장하지 않음 |
| 자동 34%, 실제 3열, visible 3~8 | 여섯 페이지 모두 DPR 2, pending image 0 |
| KTX 4-layer 100% | DPR 2, 네 surface, pending image 0, 지도·표 표시 직접 확인 |

200% 클릭 전후 테스트에서 후보 page box는 x=-504, y=-199.203125, width=2245,
height=3174.796875였고 scroll=(784,3554)를 유지했다. 과거 원인 조사와 후보 사이의 scroll 위치는
다르므로 그 둘을 정합 이미지 A/B로 해석하지 않는다. 후보 내부 클릭 전후의 geometry만 대조했다.
211%는 실제 배율 대화상자로 설정했고 버튼 기반 scroll로 이동했다. 손가락 핀치 성능 표본이 아니다.

## 비용과 남은 수용 조건

200%의 visible 1번 쪽은 **48,125,352 → 85,553,550 surface pixels**, +37,428,198px다.
단순 RGBA 환산 약 **149.7MB 증가**이며 GPU/RSS 실측이 아니다. 같은 상황에서 화면 밖의 편집
focus 0번 쪽은 기존 보호 때문에 114,071,400px가 남아 후보 active 합계는 199,624,950px였다.
이 비용을 40M 이내라고 숨기지 않는다. 원장은 `overBudgetMandatory=true`, cachedPixels=0이며
선택 prefetch가 거절된다. 전체 문서를 raw로 보존하는 변경은 아니지만 visible와 편집 보호 자체의
필수 비용은 높아질 수 있다.

이는 화질 복원이며 성능 개선으로 발표하지 않는다. 입력 중 기존 surface 재사용과 page slicing은
유지되지만 개별 고배율 raster는 여전히 긴 main-thread 작업일 수 있다. 모든 장치에서 끊김·메모리
문제가 없다는 보장은 할 수 없다. 사용자에게 클릭 없는 읽기 화질과 정착 후 끊김을 함께 확인받고,
문제가 남으면 화질을 다시 낮춰 숨기기보다 렌더/보존 범위를 좁히는 별도 설계를 검토한다.

이번 보정은 아직 실제 폭 resize 전체 갱신 예외·staging 비교·통제 성능 검증을 해결한 것이 아니다.
기존 B1 잔여 게이트를 유지한다. 원격 push·PR·코멘트는 하지 않았다.
