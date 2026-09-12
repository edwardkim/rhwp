# #6040 새 Stage 2 — animation preview 중복 제거

- 일자: 2026-09-07 KST
- 제품 기준: `56706247f4950286117496c41f5b2c4b1cdbddc5`
- 기준선 진단 head: `9762ea755`, 사용자 승인·이슈 연결 기록: `387fd9c86`
- 선행 계획 commit: `6e8d2d6c4`
- 상태: **최소 보정·자동 검증 완료, 사용자 기존 동작 유지 확인**
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md)

## 결정과 변경

Stage 1.3의 사용자 검증을 수용했다. 가로 스크롤 눈금자 지연과 세로 재현 조사는
[#6821](https://github.com/edwardkim/rhwp/issues/6821)로 등록하고
[#6040 본문·코멘트](https://github.com/edwardkim/rhwp/issues/6040#issuecomment-5561975759)에 연결했다.
눈금자 예약은 이번 Stage에서 수정하지 않았다.

제품 변경은 `CanvasView.onZoomChanged()`의 **두 번째 `updateRenderedPageZoomPreview()` 호출
한 곳 제거**다. `recalcLayout()` 안의 첫 preview를 유지한다. 그 사이의 앵커 복원은 viewport
scroll만 바꾸고 preview는 콘텐츠 내부의 VirtualScroll 좌표를 읽으므로, 두 번째 호출은 같은 요소에
동일한 top/left/transform/transformOrigin을 다시 썼다.

- 최신 geometry 계산, 첫 preview, scroll 앵커 복원, 배율 표시, pending 작업 취소 순서는 유지했다.
- 자동 열 전환·히스테리시스, pointer/중앙 앵커, ruler/caret/selection 좌표 원천은 바꾸지 않았다.
- 전체 dimensions/layout은 zoom event당 1회 그대로다. Stage 1.3에서 전체 geometry 재설계의
  실익을 입증하지 못했으므로 배열/인덱스 재구축 개편은 이번 Stage에서 보류한다.
- 줌 정착의 전면 release와 visible 동기 raster, DPR·예산·LRU·scheduler는 바꾸지 않았다.
  Stage 3에서 별도 검토할 문제를 이 변경의 개선 효과로 계산하지 않는다.

## 결정적 검증

새 `canvas-view-zoom-preview.test.ts`는 실제 CanvasView 메서드와 VirtualScroll을 실행한다.
DOM·viewport·raster 경계만 fake이며, 위치/scale 계산을 복제한 대체 구현은 실행하지 않는다.

1. 수정 전: 13건 중 animation 10건에서 `2 !== 1`로 중복 호출을 검출했고 나머지 3건은 통과했다.
2. 수정 후: 13/13 통과. 다섯 배치 × 두 이동 방향에서 여섯 배율 왕복을 검사했다.
3. 서로 다른 renderedZoom의 main과 세 overlay 종류가 최신 geometry 및 bitmap별 scale을 사용한다.
4. 활성 세 페이지 × 네 요소에서 style 쓰기는 event당 96회에서 48회로 줄어든다. 추가 preview를
   다시 실행해도 최종 요소 좌표가 같다는 비교를 포함한다.
5. animation 중 직접 resize 재계산의 preview는 유지한다. settled/direct zoom의 release → cancel →
   visible 동기 호출 순서와 빈 문서의 무작업 계약도 확인했다.

이는 **활성 요소의 중복 순회·style 쓰기 제거** 증거다. 브라우저가 동일 style 대입을 내부적으로
최적화할 수도 있으므로 실제 paint 비용이나 전체 줌 시간의 50% 감소로 해석하지 않는다.

## 실문서 호출 수

Stage 1.3과 같은 1280×720 CSS px, 실제 DPR 2, Chromium 152, 자동 배치, Canvas2D,
같은 fixture bytes와 native `--no-opt` WASM을 사용했다. 패널에서 문서 로딩 완료를 확인하고
100%부터 `50→34→100%`를 실행했다. 각 전환 전에 기록을 초기화했으며 결과 12건을 모두 남겼다.

숫자 세 개는 `100→50 / 50→34 / 34→100%` 순서다. N은 마지막 settled를 포함한 zoom event 수다.

| 문서 | 이전 N / preview | 이번 N / preview | 열 수 | 이번 main raster |
| --- | --- | --- | --- | --- |
| 4쪽 실문서 | 12·9·12 / 22·16·22 | 12·9·12 / 11·8·11 | 3·4·1 | 4·4·3 |
| exam_kor (20쪽) | 12·9·12 / 22·16·22 | 11·9·12 / 10·8·11 | 2·3·1 | 4·9·4 |
| kps-ai (77쪽) | 7·9·12 / 12·16·22 | 12·9·12 / 11·8·11 | 3·4·1 | 6·12·3 |
| basic/KTX (1쪽, 실제 4-layer) | 12·9·12 / 22·16·22 | 12·9·12 / 11·8·11 | 1·1·1 | 1·1·1 |

- 이전 `2×(N−1)`, 이번 `N−1`: **모든 animation event의 preview 순회가 2회에서 1회로 감소**했다.
- 양쪽 모두 layout/dimensions는 N회이고, 이번 12건 모두 정착 release 1회·complete·오류 0건이다.
- N은 rAF cadence/장치 부하에 따라 달라진다. kps-ai의 총 preview 12→11만 보고 절감률을 계산하거나,
  N의 증가를 줌 시간 회귀로 단정하지 않는다.
- KTX는 세 전환 모두 main/background/behind/front 네 surface를 확인했다.
- kps-ai·KTX의 34% 화면에서 문서·레이어 합성을 확인했다. 4쪽의 첫 캡처는 JSON 패널이 상단을
  가려 전체 화면 정합성 증거로 채택하지 않았다. 최종 트랙패드·슬라이더 판정은 사용자 확인이 남는다.

[최소 호출 JSON — 로컬 보존 안내](assets/issue6040-post-stack/README.md)은 모든 전환의 호출 수·완료 상태·
오류·누락 여부·layer 수만 보존한다. 시간 span과 중간 PNG/log는 추가하지 않았다. 재집계 절차는
[증거 색인](assets/issue6040-post-stack/README.md)을 따른다.

## 실행한 검증과 한계

| 검증 | 결과 |
| --- | --- |
| 신규 actual CanvasView + 기존 줌·배치·눈금자·관찰기 focused | 85/85 통과 |
| Studio 전체 `npm test` | 1,503건: 1,502 pass, 1 skip, 0 fail |
| `npm run build` | TypeScript·Vite 253 modules·PWA 성공 |
| 실문서 Canvas2D | 12/12 complete, 오류·span/frame buffer 누락 0 |
| production 관찰 코드 검색 | `scrollProbe`·`geometry.preview` 문자열 없음 |
| `git diff --check` 및 최소 JSON 재집계 | 통과 |

WASM SHA-256은 `24d3d2ffe0c1b43d4f53c23762820112b8298c2a081c9dd4842406819f4130fc`로
기준선과 같다. Rust/WASM source를 변경하지 않아 다시 빌드하거나 전체 Rust gate를 실행하지 않았다.
이번 CanvasView source SHA-256은
`d42b846764819ce158a1c8ceaaf87d1c69fad20df4dd00d7359b4b7d0e585595`다.

이번 표본은 시간 개선을 판단하는 교차 순서·warm-up·통계 반복 A/B가 아니다. screenshot·JSON의
명시적 DOM readback은 시간 성능 자료로 쓰지 않는다. CanvasKit 및 모든 배치의 실제 browser matrix는
아직 수행하지 않았으며 Stage 4의 전체 수용 gate가 남는다. 자동 테스트의 통과로 사용자의 연속
트랙패드 입력이나 compositor 프레임 정합성을 검증했다고 주장하지 않는다.

## 다음 gate

2026-09-07 후속 확인: 사용자는 이전 기준선보다 악화되지 않았지만 다중 페이지 문서에서 핀치 도중과
뗀 직후 끊김·페이지 지연 등장이 남는다고 보고했다. 이는 이번 중복 제거의 시간 개선 증명이 아니다.
연속 핀치 계측 → 기존 화면 유지·페이지별 교체 → 최신 배율 작업 취소 순서로 진행을 요청했으며,
저해상도 선렌더와 별도 미리보기 cache는 보류했다. 새 Stage 3.1부터 진행한다.

- 4200 서버를 새로고침하면 이 최소 보정이 반영된다. 계측 없는 exam_kor에서 기존과 같은 줌 왕복,
  빠른 방향 전환, 정착 후 화면·클릭 정합성을 확인한다. #6821 지연의 해결을 기대하는 검증은 아니다.
- Stage 2 결과 승인 후에만 Stage 3의 정착 surface 수명과 visible raster 경로를 구체화한다.
- 원격은 #6821 생성과 #6040 연결까지 완료했다. 이번 소스의 push·PR 생성·이슈 종료는 수행하지 않았다.
