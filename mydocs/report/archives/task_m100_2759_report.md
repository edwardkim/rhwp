# Task #2759 처리 결과 보고서 — 드래그·클릭 문서 뮤테이션 3건 히스토리 라우팅

- 일자: 2026-07-21
- 이슈: [#2759](https://github.com/edwardkim/rhwp/issues/2759)
- 브랜치: `task/m100-2759-drag-undo-routing` (base `origin/devel` = `bd442b2a`)
- 범위: `rhwp-studio/` TypeScript 만. `.rs` 무수정.

## 1. 무엇을 고쳤나

`mutation-method-registry.ts:1-19` 가 명명한 재발 계급(#2027/#2037/#2053/#2077 — 미기록 뮤테이션의
①undo 불가 ②redo 미무효화 ③스냅샷 undo 동반 파괴)에 속하는 **엔진 층 진입점 3곳**을 라우팅했다.

| # | 진입점 | 종전 | 수정 |
|---|---|---|---|
| 1 | 회전 핸들 드래그 종료 `finishPictureRotateDrag` | 상태만 정리, 미기록 | `computeRotationRecord` 로 각 변화 판정 → `executeOperation({kind:'record', ResizeObjectCommand([{rotationAngle before/after}])})` |
| 2 | 직선 끝점 드래그 종료 (onMouseUp 인라인) | 상태만 정리, 미기록 | `finishLineEndpointDrag` 신설: 드래그 시작 시 원래 끝점 캡처 → `computeLineEndpointRecord` 판정 → `executeOperation({kind:'record', MoveLineEndpointCommand})` |
| 3 | 클릭 시 z순서 변경 `bringShapeToFront` | `this.wasm.changeShapeZOrder` 직접 호출 | 메뉴 정렬(insert.ts:427)과 동형 `executeOperation({kind:'snapshot', operationType:'changeZOrder'})` |

세 수정 모두 기존 라우팅된 형제(리사이즈/이동 드래그, 메뉴 회전/정렬)와 동형 패턴을 그대로 미러했다.
새 커맨드는 `MoveLineEndpointCommand`(전/후 글로벌 끝점 좌표) 하나뿐이고, 회전은 기존
`ResizeObjectCommand` 의 임의 속성 가방(`{rotationAngle}`)을 재사용해 새 클래스가 없다.

## 2. 변경 파일

| 파일 | 내용 |
|---|---|
| `src/engine/object-drag-record.ts` (신규) | 순수 결정 함수 `computeRotationRecord`, `computeLineEndpointRecord`(변화 없으면 null). picture-resize.ts 패턴 |
| `src/engine/command.ts` | `MoveLineEndpointCommand` 추가(execute=after 재적용, undo=before 복원). `LineEndpoints` 타입 import |
| `src/engine/input-handler-picture.ts` | `finishPictureRotateDrag` 에 record 기록 + `computeRotationRecord` import |
| `src/engine/input-handler-mouse.ts` | 끝점 드래그 시작 시 `orig` 캡처, `finishLineEndpointDrag` 신설, onMouseUp 위임, `bringShapeToFront` snapshot 라우팅 |
| `src/engine/input-handler.ts` | `lineEndpointState.orig` 필드, `finishLineEndpointDrag` 위임 메서드 |
| `tests/object-drag-record.test.ts` (신규) | 순수 함수 단위 테스트 8건 |
| `tests/undo-drag-command-behaviour.test.ts` (+`tests/support/*.mjs`, 신규) | 커맨드 클래스 execute/undo 행위 테스트(자식 프로세스 로드) |
| `tests/undo-drag-click-routing.test.ts` (신규) | 3 진입점 라우팅 소스 가드 4건 |

## 3. 검증

### 3-1. 행위 테스트 vs 소스 가드 (무엇을 택했나)

두 층 모두 사용했다.

- **행위(순수 모듈)**: `object-drag-record.ts` 를 외부 import 없는 순수 함수로 분리해 기본 test 러너로
  직접 검증(picture-resize.ts ↔ picture-resize.test.ts 선례). 기록 결정 로직의 진짜 red→green.
- **행위(커맨드 클래스)**: 이 저장소는 `cursor.ts:85` TS 파라미터 프로퍼티 때문에 기본(strip-only)
  러너로 엔진 클래스를 import 못 한다. PR #2720 방식(`--experimental-transform-types` +
  `module.registerHooks` 별칭)을 자식 프로세스로 spawn 해 `command.ts` 를 실제 로드하고 mock
  WasmBridge 로 `MoveLineEndpointCommand`·`ResizeObjectCommand(회전)` 의 execute/undo 를 검증했다.
  **로컬(Node 24.17)에서 실제 green** 이며, 구버전 Node(플래그/registerHooks 미지원)에서는 skip 해
  CI 를 깨지 않는다.
- **소스 가드**: 배선(어느 종료 핸들러가 executeOperation 을 호출하는가)은 `undo-menu-object-ops`
  모델의 정적 가드로 핀했다 — 이것이 라우팅 red→green 의 주 구동체.

### 3-2. red→green (각 수정 개별 되돌림, 실제 출력)

`git stash` 는 워크트리 간 공유되어 위험하므로 사용하지 않고, 각 수정을 Edit 로 개별 되돌려 실행했다.

- **RED 1 (회전 되돌림)**: `undo-drag-click-routing` 4건 중 1건만 실패, 나머지 12건(순수+행위 포함) GREEN.
  ```
  AssertionError: origAngle→finalAngle 기록 결정을 순수 함수로 위임
    expected: /computeRotationRecord\(/
  ```
- **RED 2 (끝점 onMouseUp 위임 되돌림)**: `onMouseUp ... finishLineEndpointDrag 로 위임` 1건만 실패.
  ```
  AssertionError: onMouseUp 은 상태 인라인 초기화가 아니라 finishLineEndpointDrag 로 위임해야 함(기록 경로 확보)
  ```
- **RED 3 (z순서 되돌림)**: `bringShapeToFront ... snapshot` 1건만 실패.
  ```
  AssertionError: this.wasm.changeShapeZOrder 직접 호출 금지 — executeOperation 경유여야 함
  ```

**RED 에서도 통과하는 것**: 순수 함수 테스트(`object-drag-record`)와 커맨드 행위 테스트
(`undo-drag-command-behaviour`)는 배선이 아니라 메커니즘을 보므로 각 RED 에서도 GREEN. 라우팅 gap 은
소스 가드가 잡는다는 설계 의도 그대로다.

### 3-3. CI 게이트

- `npx tsc --noEmit`: **2 errors**, 모두 사전 존재(`@wasm/rhwp.js` 모듈 미해석 — WASM 산출물 미빌드,
  환경 요인). 수정 전 baseline 과 diff **완전 동일 = 신규 타입 오류 0**.
- `npm test`: **478 tests / 477 pass / 1 fail**. 유일 실패는 `cell-flow-boundary.test.ts`(깨끗한 devel
  에서도 실패하는 기존 실패, 본 작업 무관). baseline 465→478(+13 신규), skipped 0(행위 테스트 실제 실행).
  - (참고) 머신 고부하 시 1회 다른 기존 테스트가 타임아웃으로 함께 실패한 적 있으나, 부하 완화 후
    재실행 시 `cell-flow-boundary` 단일 실패로 재현. 신규 테스트는 격리 실행에서 13/13 GREEN.

### 3-4. 뮤테이션 표면 원장

세 수정 모두 스캔 대상 파일의 `wasm.*` 뮤테이터 텍스트 수를 늘리지 않는다(호출을 콜백 안으로 이동
또는 유지, 신규 `moveLineEndpoint` 호출은 스캔 비대상 `command.ts` 에만). `mutation-routing-guard` 원장
테스트 GREEN — BASELINE 갱신 불필요.

## 4. 범위 밖 (잔여)

- **`lineEndpointState.ref` 의 cellPath/headerFooter 미포함**: 형제 상태와 달리 경로축이 없다. 선택
  진입부도 직선에 cellPath 를 안 넘기고 `moveLineEndpoint` 에 경로 인자가 없어, 셀 내 직선 끝점 드래그가
  도달 가능한지 미확인. 본문 직선의 undo 누락은 이와 무관하게 성립하므로 본 PR 에서 확장하지 않음.
- **선택만으로 z순서가 바뀌는 설계**: 한컴은 선택 시 재정렬하지 않는다. 유지 여부는 메인테이너 판단
  요청(이슈 4절). 유지 시 최소한 기록은 되어야 하며(본 PR 이 그 부분 처리), no-op 재클릭이 히스토리에
  쌓이는 부작용은 사전 판별이 TS 만으로 불가(변경 플래그/ z순서 setter 부재 = Rust 변경 필요)라 별건.
- **머리말/꼬리말 개체의 `ResizeObjectCommand` 경로**: `setProps` 에 headerFooter 분기 부재는 기존
  리사이즈 드래그도 동일한 선행 한계라 확장하지 않음.

## 5. 주의 기록 (환경)

작업 중 `git stash` 가 **워크트리 간 공유 스택**임을 확인(다른 워크트리의 #2751 `body.rs` 변경이
워킹트리에 누출됨). 해당 `body.rs` 는 본 작업과 무관하며 스테이징하지 않았다. 커밋은 파일 명시
스테이징으로만 수행(`git add -A` 미사용).
