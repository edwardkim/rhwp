---
kind: working
status: done
canonical: mydocs/plans/task_m100_6039.md
last_verified: 2026-08-25
---

# 작업 기록 — Task M100 #6039 Stage 1: 상태·순수 레이아웃 계약

- **이슈**: [#6039](https://github.com/edwardkim/rhwp/issues/6039)
- **브랜치**: `codex/issue-6039-page-arrangement`
- **기준 commit**: `upstream/devel` `385e93b2c`
- **기록일**: 2026-08-25 KST

## 구현 결과

### 쪽 배치 상태

`rhwp-studio/src/view/page-arrangement.ts`에 문서 내용과 독립적인 `PageArrangement` 판별 합집합을
추가했다.

- 기본값과 잘못된 저장값: `auto`
- 고정 배치: `single`, `double`, `facing`
- 여러 쪽: `multiple`, 가로·세로 각각 1~8 정수로 정규화
- 여러 쪽 맞춤: 외곽/쪽 사이 간격까지 포함해 가로·세로 제약 중 작은 배율 선택

`rhwp-settings.view.pageArrangement`에 정규화된 배치 상태만 저장한다. 이 경로는 문서 모델과
`document-changed`/`document-mutated` 이벤트를 사용하지 않는다.

### VirtualScroll 배치

`VirtualScroll.setPageDimensions()`의 네 번째 인자로 쪽 배치를 받는다. 기존 호출부는 인자를 생략하므로
자동 모드를 그대로 사용한다.

| 모드 | 구현 계약 |
| --- | --- |
| 자동 | 기존 `zoom <= 0.5` 임계값과 뷰포트 최대 열 계산 보존 |
| 한 쪽 | 배율과 무관하게 한 행 한 쪽, CSS 중앙 정렬 sentinel 보존 |
| 두 쪽 | `1-2`, `3-4` 연속 두 쪽 고정 |
| 맞쪽 | 1쪽 오른쪽 빈 슬롯, 이후 2쪽 왼쪽·3쪽 오른쪽 |
| 여러 쪽 | 지정 열 수 고정, 행 수는 맞춤 배율 계산에 사용 |

행별 실제 페이지 목록을 별도로 유지해 맞쪽 첫 행처럼 슬롯 수와 실제 페이지 수가 다른 경우에도 이전·
다음 행 프리페치가 정확한 페이지를 선택한다. 크기가 다른 페이지는 최대 슬롯 폭 안에서 가운데 정렬한다.

## Red 계약

구현 전 새 테스트를 실행해 다음 실패를 확인했다.

- `page-arrangement.ts` 미존재: `ERR_MODULE_NOT_FOUND`
- 한 쪽 요청이 기존 자동 5열로 배치
- 두 쪽·여러 쪽 요청이 단일 열로 배치
- 맞쪽 첫 1쪽이 오른쪽 슬롯에 놓이지 않음
- 서로 다른 폭의 페이지가 슬롯 왼쪽에 붙음

따라서 새 테스트는 기존 코드에서도 통과하는 사후 확인이 아니라 결여된 계약을 실제로 포착했다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `node --test tests/page-arrangement.test.ts tests/virtual-scroll-page-arrangement.test.ts tests/virtual-scroll-grid-page.test.ts tests/virtual-scroll-horizontal-pan.test.ts tests/user-settings.test.ts` | 32/32 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 1088 통과, 1 skip, 실패 0 |
| `git diff --check` | 통과 |

## 다음 단계

Stage 2에서 `CanvasView`가 저장된 배치 상태를 소비하도록 연결하고, 배치 전환 전후의 중심 쪽 앵커와
렌더 해제 경계를 구현한다. UI 대화상자와 메뉴/상태 표시줄 연결은 Stage 3까지 유보한다.
