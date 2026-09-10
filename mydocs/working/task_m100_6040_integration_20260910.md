# #6040 채택 후보의 최신 devel 로컬 통합 검증

- 날짜: 2026-09-10 KST
- 단계: Stage 4 후속 통합 검증. 최종 수용·제출 단계로 승격하지 않는다.
- 최신 fetch 기준: `37bd46a72f9fd9ffd709e35244df79c00e789780`
- 채택 후보: `3d2a7620f119a69c5729267602164f6a531584a8` (제품은 Stage 4.2 `a54d97575`와 동일)
- 로컬 브랜치: `codex/issue-6040-integration-20260910`
- 이전 판정: [채택 후보·종료 조건](task_m100_6040_candidate_acceptance_audit.md)

## 통합 경계

최신 devel에서 독립 worktree를 만들고 후보를 병합했다. 원 작업 트리의 미커밋 Stage 4.3
실험과 별도 구조 이슈 초안은 그대로 보존했으며 통합에 포함하지 않았다. 원격 게시·push·PR
생성은 수행하지 않았다. 병합은 로컬 이력 보존과 검증을 위한 것이며 제출 방식 확정이 아니다.

제품 충돌은 `canvas-view.ts`의 `viewport-scroll` 구독 한 곳이었다. devel의 #6902는 문서 교체
직후 resize 앵커를 억제하고 사용자 scroll에서 해제한다. #6040은 animation뿐 아니라 quiet
정착 대기 중에도 scroll의 동기 렌더 우회를 막는다. 억제 플래그 해제와 `isZoomPreviewActive()`
검사를 함께 유지했다. 자동 병합된 #6902 load/reset/resize 경로도 확인했다.

`mydocs/orders/20260907.md`, `20260908.md`, `20260909.md`의 충돌은 devel의 다른 작업 기록과
후보의 #6040/#6821 기록을 모두 유지해 해결했다. todo 형식을 새 작업 표에 적용하고, 관련 없는
기존 운영 기록의 상태·내용을 재작성하지 않았다.

## 검증 기록

- 통합의 새 package-lock에 맞춰 해당 worktree에서 `npm ci --no-audit --no-fund` 실행.
- 첫 전체 테스트: 1,633 pass / 2 skip / 0 fail.
- 추가 통합 회귀 2개: animation 중 및 quiet 대기 중 문서 교체 resize가 이전 앵커를 사용하지
  않고 동기 raster를 피하며, 다음 resize에서는 앵커 사용을 다시 허용함을 실제 prototype으로 검사.
- 추가 후 전체 테스트: **1,635 pass / 2 skip / 0 fail**, 6,472.6215ms.
- `git diff --check` 및 충돌 marker 검사 통과. 제품 Rust/Cargo 변경은 devel 대비 없다.
- Docker daemon 미실행을 확인해 가이드의 native locked `--no-opt` 진단 WASM 빌드 경로 사용.
  #6040 전용 Cargo cache만 재사용하고 기존 서버의 `pkg/`는 바꾸지 않는다.
- native locked `--no-opt` WASM 빌드 통과(3분 9초). SHA-256:
  `6ee336d08591621b5e5da087014f5a3eebc999325eba726c1ddab8a2d943e719`.
- `npm run build`: TypeScript·Vite·PWA 통과. 기존 CanvasKit 외부 모듈/chunk 크기 경고는 남는다.

### 실제 브라우저 smoke

loopback 4205, 인앱 Chromium, 1280×720, DPR 2, 새 통합 WASM, `exam_kor.hwp` 20쪽을 사용했다.
관찰 패널 버튼을 사용한 기능 검사이며 Firefox 연속 트랙패드 성능 비교는 아니다.

| 경로 | 조작 | 관찰 |
| --- | --- | --- |
| Canvas2D | 자동 100→34→50→100% | 각각 3열/6쪽, 2열/2쪽, 1열/1쪽. visible DPR 모두 2 |
| Canvas2D | 100% 다음 행 → 200% | focus=0을 유지하며 visible=1, 클릭 없이 DPR 2·3-layer 정착 |
| CanvasKit | 100→34→200% | 34% visible 6쪽, 200% visible 1쪽, DPR 2. DOM surface cache key에서 실제 `backend:canvaskit` 확인 |
| Canvas2D | 마지막 100% 화면 | 정상 문서 표시 스크린샷 확인. 계측/CanvasKit 테스트 탭은 닫고 통합 탭만 후속 검증용 유지 |

수집한 위 정착 관찰점에서 errors 없음, pending queue 0, trace complete를 확인했다.
Canvas2D 줌 trace의 `page.releaseAll`은 0이었다. 이 표는 모든 배치·편집·숨김/복귀·fallback
시나리오를 전수 확인했다는 뜻이 아니다. 한 origin의 두 탭을 사용했으므로 탭 전환 이후의 상태를
직전 표본의 연속 성능/메모리 trace로 해석하지 않는다.

### 메모리 경보와 계측 경계

Canvas2D 200% 비편집 쪽 정착 관찰의 `totalAllocatedPixels`는 **221,018,100**이었다.
이는 active+idle pool+detached cache의 합계 표본이며, 단순 RGBA 환산은 884,072,400 bytes다.
실제 프로세스/GPU 메모리 또는 교체 중 peak 측정이 아니다. 정착 뒤 표본 하나만으로
소유권 누수·예산 실패 또는 기준선 대비 증가를 확정하지 않지만, 최종 화질 복원이 자원 비용을
수반한다는 경보로 남긴다. 시간축을 고정한 단일 탭에서 기존 기준선과 함께 peak·해제 수명·예약
비용을 확인하기 전 메모리 게이트를 통과로 표시하지 않는다. 이번 통합에서 예산이나 DPR 정책을
임의로 바꾸지 않았다.

## 남은 수용 조건

후속 [메모리 검사](task_m100_6040_memory_gate.md)에서 동일 환경 2회씩 비교한 결과 offscreen
retained-transition의 비용 증가를 확인했다. 메모리 게이트는 미확인에서 **보정 필요**로 변경한다.
DEV 관찰과 보정 제안만 작성했으며 제품 수정과 Firefox 수용 요청은 아직 진행하지 않는다.

통합 자동 검사는 Firefox 실제 핀치 무회귀, 교체 중 peak 메모리, strict 편집·선택·이미지·backend
통합 매트릭스의 대체가 아니다. `--no-opt` WASM 결과도 최적화 배포판과의 성능 비교로 사용하지
않는다. 최종 수용 전 조건은 이전 종료 조건 보고서를 유지한다.
