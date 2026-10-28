# 구현 계획 — Task M100 #6107

- **이슈**: [#6107](https://github.com/edwardkim/rhwp/issues/6107)
- **브랜치**: `codex/issue-6107-active-page-ruler`
- **기준 commit**: `upstream/devel` `70ebacc4c9589e8c778907e179a6dab18cce8eb0`
- **문서 성격**: 구현 전 파일 단위 설계

## 활성 페이지 상태 계약

`편집 페이지`와 `뷰포트 현재 페이지`를 구분한 뒤 하나의 snapshot으로 해석한다.

```ts
type ActivePageSource = 'editing' | 'viewport';

interface ActivePageSnapshot {
  pageIndex: number;
  source: ActivePageSource;
}
```

- 캐럿·텍스트 선택·선택 개체의 focus 페이지가 현재 가시 집합에 있으면 `editing`이 우선한다.
- 편집 페이지가 없거나 화면 밖이면 뷰포트 중심점의 실제 페이지를 `viewport` fallback으로 사용한다.
- 빈 맞쪽 슬롯은 페이지가 아니므로 fallback 후보에 포함하지 않는다.
- 상태 표시줄과 눈금자는 동일 snapshot을 사용한다. 눈금자가 별도로 첫 가시 페이지를 고르지 않는다.
- viewport fallback에서는 편집 문단에 종속된 들여쓰기 핀을 다른 페이지의 속성처럼 표시하지 않는다.

## 파일별 구현

### `rhwp-studio/src/view/active-page.ts` (신규)

- 가시 페이지, 편집 페이지, 뷰포트 기준 페이지를 입력으로 `ActivePageSnapshot`을 반환한다.
- 범위 밖 인덱스와 빈 문서는 명시적으로 처리한다.
- DOM·WASM 의존성을 두지 않아 Node focused test로 우선 계약을 고정한다.

### `rhwp-studio/src/view/page-scroll.ts`

- `PageScrollResult.delta` 단일값을 `deltaX`, `deltaY`로 확장한다.
- 세로 이동은 기존 행 시작·페이지 높이·뷰포트 높이 계약을 유지한다.
- 가로 이동은 실제 페이지 X 경계, 전체 너비, 뷰포트 너비와 `scrollLeft` 한계를 사용한다.
- 큰 페이지는 한 뷰포트씩 이동하되 페이지 시작 경계를 건너뛰지 않는 기존 규칙을 X축에도 적용한다.

### `rhwp-studio/src/engine/input-handler-keyboard.ts`

- PageUp/PageDown 뒤 캐럿 보정이 `deltaX`와 `deltaY`를 모두 사용하게 한다.
- 가로 이동에서도 스크롤 전 캐럿의 화면상 위치를 새 페이지 hit-test 기준으로 유지한다.
- 머리말/꼬리말·각주·개체/셀 선택 등 기존 화면 전용 분기는 보존한다.

### `rhwp-studio/src/engine/input-handler.ts`

- `cursor-rect-updated` payload에 `pageIndex`를 포함한다.
- 일반 캐럿, 조합 중 캐럿, 드래그 selection focus가 같은 payload 계약을 사용한다.
- 선택 개체 경로가 기존 cursor rect를 갱신하지 않는 경우 선택 페이지를 전달하는 최소 이벤트 경계를 추가한다.

### `rhwp-studio/src/view/canvas-view.ts`

- 최근 편집 페이지와 현재 가시 집합을 보관하고 순수 resolver로 활성 페이지를 계산한다.
- 클릭·캐럿 이동과 스크롤 모두 같은 `ActivePageSnapshot` 발행 경로를 사용한다.
- 기존 `current-page-changed` 상태 표시줄 계약은 snapshot의 `pageIndex`에서 파생한다.
- `refreshDocumentAgentMutation()`은 `scrollY`, `viewportHeight`, `scrollX`, `viewportWidth`를 모두 전달한다.

### `rhwp-studio/src/view/ruler.ts`

- `current-page-changed` 또는 활성 페이지 snapshot을 구독해 기준 페이지를 보관한다.
- 가로 좌표는 `getPageLeftResolved(activePageIndex, totalWidth)`에서, 세로 좌표는
  `getPageOffset(activePageIndex)`에서 계산한다.
- 가로·세로 눈금은 활성 페이지 한 쪽만 그리고 `getPageInfo`, 표시 좌표, 핀 commit의 page index를
  동일 snapshot에서 가져온다.
- 활성 페이지가 분명하므로 그리드에서 핀을 전부 숨기는 기존 임시 제한을 제거하되, viewport fallback에서
  문단 핀은 숨겨 잘못된 편집 문맥을 만들지 않는다.

### 사용되지 않는 페이지 이동 API

- `page-arrangement-changed`, `setPageMovement`, `getPageMovement`의 실제 발행·구독·외부 노출을 다시 검색한다.
- 소비처가 0인 내부 API만 관련 파일과 테스트에서 제거한다. 확장 기능·공개 전역 진입점이면 이번 범위에서
  삭제하지 않고 근거를 보고서에 남긴다.

## 테스트

### `rhwp-studio/tests/active-page.test.ts` (신규)

- 보이는 편집 페이지 우선, 화면 밖 편집 페이지의 viewport fallback, 빈 문서와 범위 밖 입력
- 한 행 여러 쪽에서 중심점 페이지와 편집 페이지가 다른 경우

### `rhwp-studio/tests/page-scroll-step.test.ts`

- 가로 한 쪽 이동의 이전/다음 페이지 경계
- 페이지가 뷰포트보다 넓을 때 한 화면씩 이동하고 모든 페이지 시작을 지나는지 확인
- 문서 처음/끝 clamp와 X/Y 실제 delta
- 기존 세로 단일 열·자동 그리드·맞쪽 행 이동 무회귀

### CanvasView·Ruler 계약 테스트

- cursor rect의 `pageIndex`가 활성 페이지 변경으로 이어지는지 확인
- ruler가 활성 페이지의 X/Y/용지 속성만 사용하고 첫 페이지 상수나 보이는 전체 페이지 루프를 쓰지 않는지 확인
- document-agent 갱신이 2D 가시 영역을 검사하는지 확인
- 페이지별 여백과 용지 크기가 다른 fixture에서 핀 commit page가 일치하는지 확인

## 시각 검증

- 세로 이동: 1쪽과 2쪽을 번갈아 클릭해 세로 눈금이 선택 쪽 Y 위치를 따르는지 확인
- 가로 이동: 이전/다음 쪽을 클릭·PageUp/PageDown으로 전환해 가로 눈금이 선택 쪽 X 위치를 따르는지 확인
- 두 쪽·맞쪽·여러 쪽: 같은 화면의 다른 페이지를 선택해 두 눈금자와 상태 표시줄이 같은 쪽을 가리키는지 확인
- 서로 다른 구역/용지/여백: 눈금자 핀 드래그가 선택 쪽 구역만 변경하는지 확인
- 휠 좌우 변환 켜짐/꺼짐: #6039 축 잠금 계약이 유지되는지 확인

## 예상 커밋 경계

1. `test/fix(studio): 활성 페이지와 2D PageUp/PageDown 계약`
2. `fix(studio): CanvasView와 document-agent의 2D 가시 페이지 통합`
3. `fix(studio): 눈금자를 선택 페이지 snapshot에 정렬`
4. `docs/test(studio): #6107 통합 회귀와 시각 검증 기록`
