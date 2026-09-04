---
kind: working
status: done
canonical: mydocs/plans/archives/task_m100_6039.md
last_verified: 2026-08-25
---

# 작업 기록 — Task M100 #6039 Stage 2: CanvasView 전환 계약

- **이슈**: [#6039](https://github.com/edwardkim/rhwp/issues/6039)
- **브랜치**: `codex/issue-6039-page-arrangement`
- **기준 commit**: `upstream/devel` `385e93b2c`
- **기록일**: 2026-08-25 KST

## 구현 결과

### CanvasView 배치 전환

`CanvasView`가 `rhwp-settings.view.pageArrangement`의 정규화된 값을 초기 상태로 읽고
`page-arrangement-changed` 보기 전용 이벤트를 처리한다. 레이아웃 재계산은 현재 배치를
`VirtualScroll.setPageDimensions()`에 전달하며 문서 변경 이벤트나 HWP/HWPX 모델 변경을 일으키지 않는다.

배치 변경 전 뷰포트 중심에 가장 가까운 쪽과 그 쪽 안의 상대 위치를 기록하고, 변경 후 같은 쪽의 새 좌표에
중심 앵커를 복원한다. 따라서 자동·한 쪽·두 쪽·맞쪽·여러 쪽 사이를 전환해도 사용자가 보고 있던 쪽을
잃지 않는다.

### Canvas 재사용 경계

`VirtualScroll`은 각 페이지의 실제 행·열 슬롯을 조합한 토폴로지 키를 제공한다. 배율과 절대 좌표는 키에서
제외한다.

- 같은 2열 그룹인 `두 쪽`과 `여러 쪽 2열` 사이에서는 기존 Canvas를 새 좌표로 재배치한다.
- `맞쪽`은 첫 쪽의 왼쪽 빈 슬롯 때문에 일반 2열과 다른 토폴로지로 판정한다.
- 행·열 슬롯 토폴로지가 바뀔 때만 대기 중인 프리페치와 렌더를 취소하고 Canvas를 해제한다.

이 경계는 배치 설정 자체의 정확성에 필요한 최소 작업만 포함한다. 줌 중 Canvas 교체 성능은 계획대로
후속 이슈 #6040의 범위로 유지한다.

### PageUp/PageDown 실제 행 이동

페이지 이동은 `pagesPerRow` 산술 대신 `VirtualScroll.getRowStartPages()`가 제공하는 실제 행 시작 목록을
사용한다. 맞쪽 6쪽 문서는 `1 / 2-3 / 4-5 / 6`의 행 시작 페이지 `[0, 1, 3, 5]`를 가지므로 첫 빈 슬롯과
마지막 단독 행이 있어도 모든 행을 빠짐없이 지난다.

## Red 계약

구현 전에 새 테스트를 실행해 다음 실패를 확인했다.

- `VirtualScroll.getLayoutTopologyKey()` 미존재
- `CanvasView`가 저장된 배치 상태와 보기 전용 변경 이벤트를 소비하지 않음
- 배치 전환의 중심 앵커 복원 및 토폴로지별 Canvas 해제 경계 미존재
- 맞쪽 PageDown 테스트가 실제 행 목록 API 미존재로 실패

따라서 새 테스트는 기존 코드에서도 통과하는 사후 확인이 아니라 Stage 2에서 추가한 연결 계약을 실제로
포착했다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `node --test tests/canvas-view-page-arrangement.test.ts tests/page-scroll-step.test.ts tests/virtual-scroll-page-arrangement.test.ts tests/virtual-scroll-grid-page.test.ts tests/virtual-scroll-horizontal-pan.test.ts tests/zoom-anchor.test.ts` | 38/38 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1,094 통과, 1 skip, 실패 0 |
| `git diff --check` | 통과 |

## 다음 단계

Stage 3에서 상태 표시줄 배율 표시와 `보기 > 화면 확대/축소`가 공유하는 설정 대화상자를 구현하고,
배율·쪽 모양 선택의 저장·복원과 문서 dirty 비영향 계약을 검증한다.
