# #6040 Stage 3.3 — 오래된 렌더 작업의 취소 수명 검증

- 일자: 2026-09-09 KST
- Issue: #6040
- 상태: **집중 자동 검증·최소 보정 완료, Stage 4 통합 수용은 미완료**
- 기준 제품: `3992866fa`, 사용자 확인 기록 `39fba4d0b`, 단계 계획 `b4cb79209`
- [수행 계획](../plans/task_m100_6040.md), [구현 계획](../plans/task_m100_6040_impl.md),
  [B1 및 폭 resize 결과](task_m100_6040_post_stack_stage3_2b.md)

## 판정과 변경

기존 scheduler는 새 입력 중 구 visible/prefetch dispatch를 거부하고, PageRenderer는
job identity로 구 decode 완료의 repaint를 막고 있었다. 그러나 `scheduleReRender`가
등록한 microtask는 직후 취소·교체해도 실행되어 WASM source-image key·layer tree 조회와
decode를 시작할 수 있었다. 완료 후 화면 반영 차단과 작업 시작 차단은 다른 계약이다.

microtask 진입부에 기존 job의 `completed`, 현재 page job identity, prefetch token 검사를
추가했다. 새 세대 구조나 추가 scan 없이 현재 요청일 때만 prefetch를 시작한다.
동일 Canvas에 20회 연속 예약한 결정적 테스트에서 시작 횟수는 20→1로 줄었다.
이는 **그 테스트 조건의 호출 수**이며 실제 핀치 성능이 20배 좋아졌다는 의미가 아니다.

제품 변경은 PageRenderer의 4줄뿐이다. 화질·raw DPR 보호·surface 예산·quiet 시간·
페이지 scheduler budget은 바꾸지 않았다. 이미 시작한 WASM이나 decode를 선점하지 않으며,
유효한 작업의 decode 실패 시 fallback과 취소 후 새 요청의 재시도도 유지한다.
별도 cold metadata 경량화, Worker, 타일, staging, 저화질 placeholder는 추가하지 않았다.

## 집중 검증

실제 CanvasView prototype·PageRenderer·PageRenderScheduler·ViewportManager를 사용하되
WASM/Canvas는 통제 가능한 fixture로 대체했다. event loop와 완료 순서를 고정해 재현했다.

| 경계 | 확인한 계약 |
| --- | --- |
| 빠른 축소·확대 반복 | 부분 완료 surface는 취소만으로 지우지 않으며 마지막 zoom key로 수렴 |
| strict 응답 | quiet flush가 비동기 예약을 만들더라도 visible은 응답 전에 최신 상태로 완료 |
| 문서 교체 | `prepareDocumentLoad`에서 구 큐·surface·이미지 작업을 취소, 같은 page 번호의 새 문서 렌더 가능 |
| renderer 선택 역전 | 새 선택·문서 epoch 변경·dispose 뒤 늦은 선택을 적용하지 않음 |
| 이미지 시작 전 취소 | page/all/revision/backend/dispose에서 구 WASM 조회 0회 |
| 동일 surface 재사용 | 연속 20회 중 최신 작업만 시작·repaint |
| decode·fallback·reject 역전 | 구 callback은 새 job/timer/surface를 건드리지 않으며 최신 fallback은 완료 |
| 실패·예약 정리 | 기존 scheduler 예외 전파·다음 쪽 진행, prefetch 예약 반환 검사 유지 |

- 새 시작 차단 테스트를 보정 전 실행: **25 total / 19 pass / 6 fail**.
- 보정 후 확대 집중 묶음 6개 파일: **125 pass / 0 fail**.
- Studio 전체: **1,597 total / 1,596 pass / 1 skip / 0 fail**.
- `npm run build`(tsc 포함), `git diff --check` 통과. 기존 bundle 크기 경고 유지.
- Rust/WASM source 변경 없음. 기존 진단용 no-opt WASM을 재사용했다.

## 실제 브라우저 smoke

Chrome 152 인앱, viewport 1280×720, DPR 2, Canvas2D, 4200 개발 서버에서 확인했다.
KTX 실제 4-layer 문서의 자동 배치에서 34→200→50% 버튼 줌을 연속 요청했다.
관찰 trace는 complete/superseded/complete였고 최종 50%에서 main/background/behind/front
4개 surface, DPR 2, pending image 0, visible/prefetch 큐 0, 관찰 오류 0을 확인했다.
지도·색상 표·본문이 표시된 최종 화면을 직접 확인했다. 패널의 image 상태 `none`만으로
모든 브라우저 decode/paint 완료를 증명하지 않는다.

이후 KTX 200% 요청 뒤 exam_kor를 열어 100%로 복귀했다. 문서 20쪽, visible 첫 쪽
DPR 2, pending image·visible/prefetch 큐·관찰 오류 모두 0을 확인했다. 이 역시 실제 문서
교체의 정상 완료 smoke이며, 로딩과 구 callback의 정확한 경합 시점은 단위 테스트 근거다.

버튼 호출 사이의 도구 지연이 있으므로 모든 요청이 렌더 중간에 끼어들었다고 주장하지 않는다.
정확한 취소 시점·완료 역전은 위 결정적 테스트가 검증하며, 이 smoke는 정상 이미지 표시가
사라지거나 최종 대기 작업이 남는지 확인하는 보조 증거다. 실제 Firefox 핀치 전후 지표가 아니다.

## 남은 수용 범위

Stage 4에서 동일 조건 A/B, CanvasKit 실문서, 실제 입력·편집·문서/renderer 전환 조합,
메모리·warm 스크롤 및 읽기 최종 화질을 검증해야 한다. animation/quiet 종료 뒤 늦은
resize의 idle 전역 갱신 가능성과 미생성 visible의 초기 공백도 별도 잔여로 유지한다.
이번 집중 검사 통과를 전체 #6040 완료나 PR 제출 준비 완료로 확대하지 않는다.
원격 push·PR 생성·이슈 종료는 수행하지 않았다.
