# Task M100 #2217 Stage 9 - 한컴형 글꼴 목록 분류

## 목표

로컬 글꼴을 감지한 뒤 Chrome의 `queryLocalFonts()`가 반환한 모든 CJK face가 기본 글꼴
드롭다운에 한 목록으로 노출되는 문제를 해소한다. 한컴처럼 범주를 먼저 고르고 해당 글꼴
목록을 보는 구조로 바꾼다.

## 분석

- 현재 `Toolbar`는 native `<select>`에 `getLocalFonts()` 전체 결과를 `로컬 글꼴`
  `optgroup`으로 추가한다.
- macOS Chrome의 Local Font Access API는 설치된 모든 face를 반환한다. SFNT name table의
  지역화 이름을 표시 이름으로 보존하므로, 중국어/일본어 face도 실제 이름 그대로 보인다.
- native `<select>`는 한컴 UI처럼 좌측 분류와 우측 목록을 동시에 제공할 수 없다.
- 현재 문서 API는 제목/본문/표 역할별 글꼴 목록을 별도로 제공하지 않는다. 이 정보를
  추정하면 잘못된 범주를 만들 수 있으므로, 이번 단계에서는 문서가 실제로 사용하는 글꼴을
  `문서 글꼴`로 정확하게 분리한다.

## 변경 계획

1. 기존 `#font-name` select는 현재 선택값과 글꼴 적용 이벤트를 보존하는 hidden state로
   유지한다.
2. 클릭/키보드 열기 시 전용 글꼴 메뉴를 띄워 `모든 글꼴`, `현재 글꼴`, `문서 글꼴`,
   `대표 글꼴`, `시스템 글꼴` 범주를 좌측에 표시한다.
3. `시스템 글꼴`에서만 전체 로컬 감지 결과를 지연 생성한다. 기본 진입 범주에서는 문서와
   현재 글꼴을 우선 보여 주어 수백 개 CJK face가 즉시 노출되거나 렌더를 막지 않게 한다.
4. mouse, keyboard, Escape, 외부 클릭으로 메뉴를 조작할 수 있게 하고, 선택 결과는 기존
   글꼴 적용 경로를 통해 반영한다.

## 검증 계획

- 글꼴 메뉴의 범주별 목록과 지연 시스템 목록 생성을 unit test로 확인한다.
- Studio build와 WASM build를 수행한다.
- `samples/issue2217/20200830.hwp`를 Chrome에서 열어 문서 글꼴과 시스템 글꼴이 분리되고,
  메뉴를 열어도 편집 가능 상태가 지연되지 않는지 확인한다.

## 변경 결과

1. native `#font-name` select는 기존 글꼴 적용 및 자동화 호환을 위한 선택 상태로 유지하고,
   열기 동작은 좌측 범주와 우측 목록을 갖는 글꼴 메뉴로 교체했다.
2. 기본 범주는 `문서 글꼴`이며, `현재 글꼴`, `대표 글꼴`, `시스템 글꼴`, `모든 글꼴`을 구분했다.
   전체 로컬 face는 `시스템 글꼴` 또는 `모든 글꼴`을 선택할 때만 조회·생성한다.
3. macOS legacy SFNT name record는 인코딩을 확정할 수 없으므로 표시/별칭 후보에서 제외했다.
   기존 저장 snapshot의 깨진 표시 이름은 browser family/full name 후보로 재정규화한다.
4. 메뉴는 고정 높이 안에서 우측 글꼴 목록만 스크롤하도록 만들었다.
5. 로컬 글꼴 재감지 뒤 드롭다운을 새로 구성할 때, 캐럿의 언어별 7개 글꼴이 아니라 최초 문서에서
   받은 전체 `fontsUsed` 목록을 계속 사용하도록 정정했다.

## 검증 결과

- `node --test tests/local-fonts.test.ts tests/toolbar-local-font-options.test.ts
  tests/document-initialization-order.test.ts`: 19 passed.
- `wasm-pack build --target web --out-dir pkg`: passed.
- `npm run build`: passed.
- `git diff --check`: passed.
- Chrome의 Vite 탭에서 키보드로 범주 메뉴가 열리고, 우측 목록이 350px 고정 높이의
  `overflow: auto` 컨테이너로 생성되는 것을 확인했다. 실제 설치 글꼴 목록의 사람 판정은
  최종 PR 준비 전에 작업지시자 화면에서 다시 확인한다.
