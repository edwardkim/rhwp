# 구현 계획 — Task M100 #6039

- **이슈**: [#6039](https://github.com/edwardkim/rhwp/issues/6039)
- **브랜치**: `codex/issue-6039-page-arrangement`
- **기준 commit**: `upstream/devel` `385e93b2c`
- **문서 성격**: 구현 전 파일 단위 설계

## 상태 모델

문서 내용이 아닌 사용자 보기 상태로 다음 판별 합집합을 둔다.

```ts
type PageArrangement =
  | { kind: 'auto' }
  | { kind: 'single' }
  | { kind: 'double' }
  | { kind: 'facing' }
  | { kind: 'multiple'; columns: number; rows: number };
```

- 저장값이 없거나 잘못됐으면 `auto`로 정규화한다.
- `multiple.columns`와 `multiple.rows`는 각각 1~8 범위로 정규화한다.
- 배율은 `ViewportManager`의 수치 상태로 유지하고 배치 상태와 합치지 않는다.
- 배치 상태만 `rhwp-settings.view`에 저장하며 문서 dirty 이벤트를 발행하지 않는다.

## 파일별 구현

### `rhwp-studio/src/view/page-arrangement.ts`

- `PageArrangement` 타입, 기본값, 저장값 정규화와 동등성 비교를 제공한다.
- 여러 쪽 가로×세로와 페이지 간격을 기준으로 뷰포트 맞춤 배율을 계산한다.
- 계산 결과는 Studio의 허용 배율 범위로 제한한다.

### `rhwp-studio/src/core/user-settings.ts`

- `ViewSettings`에 `pageArrangement`를 추가하고 기존 저장값에 대한 하위 호환 기본값을 둔다.
- 정규화된 getter/setter만 외부에 노출한다.

### `rhwp-studio/src/view/virtual-scroll.ts`

- `setPageDimensions`가 명시적 쪽 배치 상태를 받도록 확장한다.
- `auto`는 기존 `zoom <= 0.5` 임계값과 최대 열 계산을 그대로 유지한다.
- `single`은 한 행 한 쪽과 행 중앙 정렬을 유지한다.
- `double`은 `1-2`, `3-4`처럼 연속 두 쪽을 같은 행에 둔다.
- `facing`은 첫 홀수 쪽의 왼쪽을 빈 슬롯으로 두고 이후 짝수/홀수를 좌/우에 둔다.
- `multiple`은 지정한 열 수로 행을 구성하며 마지막 행도 전체 그리드 폭 기준으로 중앙 정렬한다.
- 페이지 크기가 서로 달라도 열 너비와 행 높이는 각 행/열의 최댓값을 사용하고 페이지는 슬롯 안에서 중앙 정렬한다.

### `rhwp-studio/src/view/canvas-view.ts`

- 저장된 배치 상태로 초기화하고 보기 상태 변경 이벤트를 구독한다.
- 배치 변경 전 뷰포트 중심 쪽을 찾고, 재배치 뒤 같은 쪽을 중심 앵커로 복원한다.
- 토폴로지가 바뀐 경우에만 현재 렌더 페이지를 해제하고 다시 그린다.
- 기존 자동 그리드의 줌 앵커, 현재 쪽, 클릭 좌표와 눈금자 좌표 소비 방식은 유지한다.

### `rhwp-studio/src/ui/zoom-settings-dialog.ts`

- 한컴 화면 확대 대화상자에 대응하는 `비율`과 `쪽 모양` 그룹을 제공한다.
- 비율은 고정값, 폭 맞춤, 쪽 맞춤, 사용자 값을 선택할 수 있다.
- 쪽 모양은 `자동`, `한 쪽`, `두 쪽`, `맞쪽`, `여러 쪽`과 1×1~8×8 입력을 제공한다.
- `여러 쪽` 선택 시 가로×세로 쪽 수가 들어오는 배율을 계산해 적용한다.
- 입력값 검증 실패 시 대화상자를 닫지 않고 오류 상태를 표시한다.

### `rhwp-studio/src/command/commands/view.ts`, `rhwp-studio/index.html`, `rhwp-studio/src/main.ts`

- `view:zoom-settings` 명령을 등록하고 `opensDialog: true`를 선언한다.
- 보기 메뉴에 `화면 확대/축소…` 진입점을 추가한다.
- 상태 표시줄의 배율 값을 버튼으로 바꾸고 클릭 시 같은 명령을 dispatch한다.
- 확인 시 사용자 설정 저장, 배치 변경 이벤트, 수치 배율 변경을 각각 명시적으로 적용한다.

### 테스트와 문서

- `rhwp-studio/tests/page-arrangement.test.ts`: 정규화와 여러 쪽 맞춤 배율
- `rhwp-studio/tests/virtual-scroll-page-arrangement.test.ts`: 다섯 배치 모드의 행/열/좌표
- `rhwp-studio/tests/user-settings.test.ts`: 기본 `auto`, 저장·복원, 1~8 정규화
- 기존 줌 앵커·자동 그리드·PageUp/PageDown 테스트: 회귀 없음 확인
- 실제 브라우저: 메뉴/상태 표시줄 동일 대화상자, 맞쪽 첫 빈 슬롯, 문서 dirty 비영향 확인
- 새 UI 전용 CSS 접두어가 필요하면 `mydocs/manual/rhwp_studio_ui_conventions.md`에 함께 등록한다.

## 위험과 완화

| 위험 | 완화 |
| --- | --- |
| 자동 모드 회귀 | 기존 임계값·열 계산을 `auto` 전용 경로로 보존하고 현재 테스트를 그대로 통과시킨다. |
| 맞쪽 빈 슬롯 때문에 현재 쪽 판정이 어긋남 | 실제 페이지 인덱스만 offset/left 배열에 유지하고 빈 슬롯은 좌표 계산에만 반영한다. |
| 배치 전환이 문서 변경으로 기록됨 | `document-changed`/`document-mutated`가 아닌 보기 전용 이벤트만 사용한다. |
| 여러 쪽 배율과 수동 배율이 충돌 | `multiple` 확인 시 맞춤 계산을 최종 유효 배율로 명시하고 상태 모델 자체는 분리한다. |
| UI 변경이 자동화 호출을 대기시킴 | 명령에 `opensDialog: true`를 선언하고 다이얼로그 정책 원장 테스트를 통과시킨다. |
