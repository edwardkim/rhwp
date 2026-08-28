# Task M100 #6108 Stage 2 완료 보고 — 맞춤 진입점 단일화와 무사용 경로 정리

- **이슈**: [#6108](https://github.com/edwardkim/rhwp/issues/6108)
- **브랜치**: `codex/issue-6108-zoom-fit`
- **선행 Stage**: `d5a4cfc77` — 쪽 배치별 맞춤 계산 계약 단일화
- **Stage 범위**: 명령·대화상자 진입점, 여러 쪽 비율 UI, 무사용 보기 경로

## 구현 결과

### 맞춤 metrics와 resolver 공유

- `view.ts`에 현재 뷰포트·첫 쪽·쪽 배치·page gap을 한 번에 만드는 `getZoomFitMetrics()`를 두었다.
- 보기 메뉴와 상태 표시줄이 호출하는 `applyZoomFit()`은 이 helper와 `resolveZoomFitZoom()`만 사용한다.
- 확대/축소 대화상자의 초기 폭/쪽 맞춤 판별과 확인 시 최종 계산도 같은 helper snapshot을 사용한다.
- 저장된 맞춤 복원은 `CanvasView` 공개 getter 대신 정규화·저장된 `userSettings` 배치를 직접
  `resolveZoomFitZoom()`에 전달한다.
- 명령 파일에 남아 있던 `calculateFitPageZoom()`·`calculateArrangementFitWidthZoom()` 직접 호출은
  제거했다.

### 여러 쪽 UI

- 여러 쪽을 선택하면 실제 계산에 적용되지 않는 프리셋·폭 맞춤·쪽 맞춤·사용자 정의 비율 radio와 사용자
  입력을 비활성화한다.
- 자동·한 쪽·두 쪽·맞쪽으로 돌아오면 기존 선택 상태를 보존한 채 다시 활성화한다.
- 여러 쪽의 가로×세로 입력과 가로 이동 모드의 한 쪽 강제 계약은 유지한다.

### 무사용 경로 감사·정리

저장소 전체 `rg`와 TypeScript 검사로 다음 항목의 실제 소비처가 없음을 확인한 뒤 제거했다.

- 발행자가 0인 `page-arrangement-changed` CanvasView 구독
- 통합 `setPageViewSettings()`로만 위임하던 `setPageArrangement()`·`setPageMovement()` wrapper
- 외부 소비처가 없던 `getPageArrangement()`·`getPageMovement()` wrapper
- CSS 소비처가 없던 배율 슬라이더 `is-neutral` class 토글 두 곳과 DOM lookup

`setPageViewSettings()`는 `page-view-settings-changed` 구독에서만 쓰이므로 `private`으로 좁혔다. 배치 전환의
중심 앵커 복원, topology 비교, 필요한 경우의 Canvas 해제 로직은 변경하지 않았다.

## 테스트 우선 확인

구현 전 Stage 2 계약을 추가해 다음 RED를 확인했다.

- 여러 쪽 비율 선택 disabled 처리 부재
- 명령의 공통 fit metrics helper 부재와 직접 calculator 호출
- legacy `page-arrangement-changed` 구독 및 공개 wrapper 잔존
- `is-neutral` class toggle 잔존

구현 후 focused suite는 42/42 통과했다.

```text
node --test \
  tests/zoom-fit.test.ts \
  tests/zoom-dialog.test.ts \
  tests/zoom-dialog-integration.test.ts \
  tests/zoom-fit-mode-persistence.test.ts \
  tests/page-arrangement.test.ts \
  tests/canvas-view-page-arrangement.test.ts

tests 42, pass 42, fail 0
```

## 정적 검증

- `npm exec tsc -- --noEmit`: 통과
  - Stage 1과 같은 방식으로 생성형 WASM `pkg`를 로컬에서만 연결한 뒤 제거했다.
- 삭제 대상 이벤트·CanvasView wrapper·class toggle 잔여 소비처: 0건
- `git diff --check`: 통과

## 다음 게이트

작업지시자가 Stage 2 결과를 승인하면 Stage 3에서 다음 통합 검증만 진행한다.

1. Studio 전체 TypeScript unit test와 production build
2. 실제 Chrome에서 자동·한 쪽·두 쪽·맞쪽·여러 쪽의 폭/쪽 맞춤 및 메뉴·상태 표시줄·대화상자 비교
3. 필수 format·diff gate와 최종 보고서 작성

#6109의 사용자 입력 오류 표시와 원자 view-settings transaction은 분리 상태를 유지한다.

PR review에서 HTML에 남은 `is-neutral` literal과 여러 쪽 맞춤 규칙 저장 누락을 확인해 Stage 3에서
추가 보정했다.
