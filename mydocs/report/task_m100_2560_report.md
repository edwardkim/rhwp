# task_m100_2560 처리결과 보고서 — 그리드 보기 "현재 쪽" 판정 정합

- **이슈**: [#2560](https://github.com/edwardkim/rhwp/issues/2560)
- **브랜치**: `task/m100-2560-grid-current-page` (base `devel` @ `3c54abfd`)
- **범위**: `rhwp-studio` view/engine 3파일 + 신규 테스트
- **분류**: 결함 수정 (PageUp 무동작, 쪽 표시기 오표시)

## 1. 문제

`getPageAtY` 가 그리드 보기에서 **행의 마지막 쪽**을 반환한다. `layoutGrid` 가 한 행의 모든
쪽에 같은 `rowTop` 을 넣으므로, 뒤에서부터 스캔하는 이 함수는 언제나 그 행의 최대 인덱스를
돌려준다.

이 의미 자체는 의도된 것이다 — `getPageAtPoint`(`virtual-scroll.ts:149`)가 반환값을
**`rowLastIdx`** 로 명명하고 X 로 좁히기 위한 스캔 끝점으로 쓴다. 문제는 **"현재 쪽" 이 필요한
소비처가 이 Y 전용 진입점을 직접 쓴 것**이다.

### 결함 1 — PageUp 무동작

`input-handler-keyboard.ts:1203-1209` 는 `currentPage ± 1` 로 목표를 정한다. 3열에서 현재 행이
(3,4,5)면 `currentPage = 5`, PageUp 목표는 4 — **같은 행**이라 `getPageOffset` 이 동일해
`setScrollTop` 이 제자리다. 아무리 눌러도 행을 벗어나지 못한다.

PageDown 은 `5+1 = 6` 이 실제로 다음 행이라 우연히 동작한다. **이 비대칭이 근본 원인을 가리킨다.**

### 결함 2 — 쪽 표시기 오표시

`canvas-view.ts:301` → `current-page-changed` → 상태바. 3열이면 첫 행에서 "3 / N" 으로 표시되고
1·2쪽은 현재 쪽으로 표시될 수 없다.

## 2. 분석 — 영향 없는 소비처 구분

| 소비처 | 판정 |
|---|---|
| `canvas-view.ts:512` `focusPage` | **영향 없음** — offset/height 만 쓰는데 한 행은 offset 이 같다 |
| `getPageAtPoint` | **영향 없음** — `rowLastIdx` 를 스캔 끝점으로 쓰는 현재 의미가 맞다 |
| `view/coordinate-system.ts:18` | 현재 `src/` 내 소비처 없음 — 잠재 위험만 |

`getPageAtY` 의 의미를 바꾸면 `getPageAtPoint` 가 깨지므로, **의미는 유지하고 별도 진입점을
추가**하는 방향을 택했다.

## 3. 변경

| 파일 | 변경 |
|---|---|
| `view/virtual-scroll.ts` | `getRowFirstPageAtY()` 추가(단일 컬럼에선 `getPageAtY` 와 동치), `pagesPerRow` 게터 추가, `getPageAtY` 에 의미 문서화 |
| `view/canvas-view.ts:301` | 쪽 표시기를 `getRowFirstPageAtY` 로 |
| `engine/input-handler-keyboard.ts` | PageUp/PageDown 을 행 첫 쪽 기준 **±열수** 로 |

`±1` → `±pagesPerRow` 가 핵심이다. 단일 컬럼에서는 `pagesPerRow = 1` 이라 **종전 동작과 동일**하다.

## 4. 검증

### 신규 테스트 — 이 클래스의 첫 테스트

`tests/virtual-scroll-grid-page.test.ts` (4건). `VirtualScroll` 은 타입만 import 하는 순수
모듈이라 **DOM/브라우저 없이** `node --test` 로 검증된다. 종전엔 `tests/virtual-scroll*` 파일이
**존재하지 않아 이 클래스 전체가 미테스트**였다.

1. `getPageAtY` 가 행의 마지막 쪽을 준다(의도된 의미 고정)
2. `getRowFirstPageAtY` 가 행의 첫 쪽을 준다 — 상태바가 1쪽을 표시 가능
3. PageUp 목표의 offset 이 현재와 **달라야** 한다 — 종전 ±1 이 무동작이던 지점
4. 단일 컬럼에서 두 진입점이 동치(회귀 없음)

전제 조건(`isGridMode()`, `pagesPerRow > 1`)을 테스트 안에서 단언해, 레이아웃이 바뀌어
그리드가 아니게 되면 조용히 통과하지 않고 실패한다.

### 회귀

`node --test tests/*.test.ts` — 신규 4건 포함 전부 통과, 기존 실패는 `cell-flow-boundary.test.ts`
하나로 이는 깨끗한 devel 에서도 실패하는 **사전 실패**다(이전 작업에서 stash 로 확인함).

### 미실행 항목 (투명 고지)

- **실제 키 입력 행위 검증 미실행** — PageUp 키다운→스크롤 이동까지는 jsdom/브라우저가 필요하다.
  본 PR 은 그 결정 요인인 좌표/스텝 계산을 순수 단위 테스트로 고정하는 데 그쳤다.
- 같은 스윕에서 나온 별건(`page:col-right` 가 `page:col-left` 와 바이트 동일, 머리말/꼬리말
  숨김 명령이 undo 라우팅 우회)은 **범위를 섞지 않기 위해 제외**했다. 필요하면 별도 이슈로 등록하겠다.
