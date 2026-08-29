# Task M100 #6040 Stage 1 완료 보고 — 자동 열과 실제 점유 중앙 정렬

- **이슈**: [#6040](https://github.com/edwardkim/rhwp/issues/6040)
- **브랜치**: `codex/issue-6040-zoom-topology`
- **기준 commit**: `upstream/devel` `2deb3dd61`
- **계획 commit**: `5b98fb684`
- **결과 승인**: 2026-08-30 작업지시자 승인, Stage 2 진행
- **Stage 범위**: 자동 열 순수 계산·page count cap·중앙 정렬·히스테리시스 계약. 줌 preview와
  topology commit 분리는 Stage 2에 유지

## 구현 결과

- 자동 모드의 `zoom <= 0.5` 전역 gate를 제거했다.
- 표시된 최대 페이지 폭, 현재 CSS page gap, 편집 영역 폭만으로 들어가는 열 수를 계산한다.
- 후보 열 수를 `1..pageCount`로 제한해 존재하지 않는 빈 열이 그리드 중앙 정렬 폭에 포함되지 않게 했다.
- candidate가 1열이면 기존 단일 열 CSS 중앙 정렬을, 2열 이상이면 기존 그리드 좌표 계약을 사용한다.
- 이전 commit 열 수를 선택적으로 받는 순수 `resolveAutoPageColumns()`를 추가했다. 열 경계 양쪽에 8 CSS
  px dead band를 두어 Stage 2의 zoom commit과 resize가 같은 계산·히스테리시스를 재사용할 수 있다.
- `single`, `double`, `facing`, `multiple`, 가로 쪽 이동과 마지막 미완성 행의 시작 열 규칙은 바꾸지 않았다.

## 문제 시나리오 측정

800×1000 쪽, CSS gap 6px 조건에서 확인한 결과다.

| 문서·편집 영역 | 배율 | 이전 핵심 증상 | Stage 1 결과 |
| --- | ---: | --- | --- |
| 6쪽·폭 900px | 51% | 전역 gate로 1열 | 2열, 묶음 중심 450px = viewport 중심 450px |
| 6쪽·폭 900px | 50% | 폭 계산이 갑자기 활성화 | 2열 유지 |
| 6쪽·폭 900px | 49% | 경계 직후 열 topology 변화 | 2열 유지 |
| 3쪽·폭 2000px | 27% | 빈 열 때문에 왼쪽 치우침 | 3열, 좌우 670~1330px, 중심 오차 0px |
| 3쪽·폭 2000px | 17% | 축소할수록 왼쪽 치우침 확대 | 3열, 좌우 790~1210px, 중심 오차 0px |

## focused 검증

다음 여섯 suite에서 자동 배치와 기존 좌표·행 이동·가로 팬 계약 46건이 모두 통과했다.

```text
node --test \
  rhwp-studio/tests/virtual-scroll-page-arrangement.test.ts \
  rhwp-studio/tests/virtual-scroll-grid-page.test.ts \
  rhwp-studio/tests/virtual-scroll-horizontal-pan.test.ts \
  rhwp-studio/tests/active-page-integration.test.ts \
  rhwp-studio/tests/page-scroll-step.test.ts \
  rhwp-studio/tests/canvas-view-page-arrangement.test.ts

tests 46, pass 46, fail 0
```

추가한 회귀 범위는 다음과 같다.

- 51%·50%·49%에서 2쪽 폭이면 모두 2열
- 27%·17% 3쪽 문서의 열 수 cap과 실제 묶음 중심 오차 ≤1 CSS px
- invalid/0 geometry의 1열 fallback
- 1↔2열 경계의 ±8px 히스테리시스
- 고정 쪽 배치와 가로 쪽 이동 불변

## 전체·정적 검증

- `(cd rhwp-studio && npx tsc --noEmit)`: 통과
  - 격리 worktree에는 생성형 `pkg/rhwp.js`가 없어서 원본 작업공간의 동일 WASM 산출물을 로컬 symlink로
    연결해 검사·build한 뒤 즉시 제거했다. source와 Git 변경에는 포함되지 않는다.
- `npm --prefix rhwp-studio test`: 1,246건 중 1,245 pass·1 skip·0 fail
- `npm --prefix rhwp-studio run build`: 239 modules production build 통과
- `git diff --check`: 통과

`npm ci`는 잠금파일 기준 382 packages를 설치했고 기존 lock 기준 3 vulnerabilities(1 low·2 high)를
보고했다. 의존성이나 lockfile은 변경하지 않았다.

## 범위 감사

- 수정한 source는 `rhwp-studio/src/view/virtual-scroll.ts` 한 파일이다.
- 테스트는 자동 배치 계약과 기존 임계값을 설명하던 주석만 변경했다.
- `CanvasView`, `CanvasPool`, `PageRenderer`는 아직 변경하지 않았다.
- render scale tier·surface 예산(#6041), 행 인덱스·LRU·scheduler(#6042)는 추가하지 않았다.

## 다음 게이트

작업지시자가 Stage 1 결과를 승인하면 이 source·test·보고 문서를 commit하고 Stage 2에서만 다음 작업을
진행한다.

1. 줌 제스처 시작 시 commit topology와 기준 페이지/정규화 앵커 snapshot
2. animation frame의 전체 `recalcLayout()` 제거와 active element CSS preview
3. settled event의 최종 후보 단일 commit과 앵커 복원

Stage 2 승인 전에는 활성 Canvas 점진 교체(Stage 3)를 구현하지 않는다.
