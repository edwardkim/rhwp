# Task M100 #1470 stage2 착수 문서

- 이슈: #1470 `스타일 적용/편집 불일치: 왼쪽 여백 배율, 줄간격 미반영, 표 캡션/생성 위치 문제`
- 기준 커밋: `a0c7d0bc task 1470: 스타일 적용 및 표 캡션 보정`
- 작업 브랜치: `task_m100_1470`
- 작성일: 2026-06-22
- 상태: 구현 진행 중. 작업지시자 승인 후 TAC 표 중복 렌더 보정 및 focused 회귀 테스트 추가.

## 1. Stage 2 배경

Stage 1 커밋 후 CDP로 `http://localhost:7700/`의 실제 Studio/WASM 런타임을 검증했다.
스타일 값과 표 속성 API는 기대값을 만족했지만, `createTableEx(... treatAsChar: true)`로 만든 표가
렌더 트리와 화면에서 같은 `controlIdx`로 두 번 그려지는 문제가 남았다.

확인된 정상 항목:

- 스타일 왼쪽 여백 raw `3000`은 UI 조회값 `20px`로 표시되어 15pt 대응값을 유지했다.
- 본문과 셀 문단 모두 `lineSpacing=300`, `lineSpacingType=Percent`로 반영됐다.
- 스타일 수정 후 본문 `LineInfo`가 변경되어 reflow가 실행됨을 확인했다.
- 표 생성 옵션은 `tableWidth=10000`, `tableHeight=8000`, 셀 크기 `[4000x3000, 6000x3000, 4000x5000, 6000x5000]`로 반영됐다.
- 캡션 on/off 속성은 `hasCaption=true/false`로 토글됐다.
- 검증 문서는 1쪽 유지, 표 bbox의 `pageIndex=0`으로 다음 페이지 생성 문제는 재현되지 않았다.

## 2. 남은 문제

`createTableEx(... treatAsChar: true)` 표가 동일 문단/동일 컨트롤 번호로 두 번 렌더된다.

CDP 진단 결과:

- `table-only` 케이스에서도 `layoutTables.length = 2`
- 두 렌더 노드 모두 `paraIdx=0`, `controlIdx=2`
- 첫 노드: 대략 `y=136`
- 두 번째 노드: 대략 `y=254.5` 또는 스타일 적용 후 `y=273.1`
- 캡션 생성 시 캡션 번호도 두 위치에 함께 표시됨
- 캡션 삭제 후에도 중복 표 자체는 남음

즉 Stage 2의 주된 결함은 캡션 생성이 아니라, TAC 표 렌더 경로가 같은 표를 두 번 emit하는 문제다.

## 3. 원인 후보

현재 코드상 다음 두 경로가 같은 TAC 표를 모두 그릴 가능성이 있다.

1. `src/renderer/layout/paragraph_layout.rs`
   - 인라인 TAC 표를 텍스트 흐름 위치에 직접 `layout_table`로 렌더한다.
   - 렌더 후 `tree.set_inline_shape_position(...)`으로 중복 방지용 위치를 등록한다.

2. `src/renderer/layout.rs`
   - 이후 같은 `Table` PageItem 처리 경로에서 `tree.get_inline_shape_position(...)`을 조회하지만,
     위치가 있어도 `layout_table` 호출 자체는 계속 수행한다.
   - 결과적으로 paragraph layout에서 한 번, Table PageItem 경로에서 한 번 렌더될 수 있다.

셀 내부 중첩 TAC 표 경로인 `src/renderer/layout/table_layout.rs`에는
`already_rendered_inline` 가드가 있어 이미 등록된 inline 위치를 만나면 별도 렌더를 건너뛰는 패턴이 있다.
Stage 2는 top-level TAC 표에도 같은 의도의 가드가 필요한지 확인한다.

## 4. 작업 범위

1. `createTableEx(... treatAsChar: true)`로 생성한 본문 TAC 표가 렌더 트리에 한 번만 나타나도록 보정한다.
2. 이미 paragraph layout에서 직접 렌더한 TAC 표와 Table PageItem 경로의 중복 렌더를 분리한다.
3. 중복 렌더를 제거하더라도 다음 문단 위치 계산, TAC 표 줄간격, outer margin advance는 유지한다.
4. 캡션 on/off 후에도 동일 표가 한 번만 렌더되는지 확인한다.
5. 기존 한컴 원본 TAC 표 E2E와 다중 TAC 표/선행 텍스트 케이스의 회귀를 확인한다.

## 5. 제외 범위

- Stage 1에서 통과한 스타일 여백/줄간격/셀 전파 로직 재설계
- 표 캡션 AutoNumber 모델 재설계
- 전체 TAC 표 pagination 재설계
- 모든 floating object 중복 emit 경로 일괄 정리
- 한컴 도움말 스타일 UI 전체 재현

## 6. 구현 방향

우선 후보:

- top-level table PageItem 처리에서 `is_tac && inline_pos.is_some()`인 경우,
  paragraph layout이 이미 렌더한 inline TAC 표로 보고 두 번째 `layout_table` 호출을 건너뛰는 방향을 검토한다.
- 단순 skip만 하면 흐름 y advance가 달라질 수 있으므로, 기존 `layout_table` 반환값으로 진행하던 높이/줄간격 처리를
  `inline_pos + 측정 표 높이 또는 LINE_SEG 기준`으로 대체할 수 있는지 확인한다.
- 셀 내부 중첩 TAC 표의 `already_rendered_inline` 가드를 참고하되, 본문 top-level 표의 pagination/PageItem 흐름과
  충돌하지 않도록 범위를 좁힌다.

대안:

- `paragraph_layout.rs`에서 top-level TAC 표 직접 렌더를 하지 않고 위치 등록만 하도록 바꾸는 방법도 가능하지만,
  기존 한컴 방식 입력 E2E가 이 경로에 의존하므로 우선순위는 낮다.

## 7. 테스트 계획

Focused Rust 테스트:

- `issue_1470_create_table_ex_tac_renders_once`
  - 빈 문서에 텍스트와 `create_table_ex_native(... treat_as_char=true)` 표 1개 생성
  - `get_page_control_layout_native(0)`에서 같은 `(secIdx, paraIdx, controlIdx)` 표 노드가 1개만 나오는지 확인

- `issue_1470_create_table_ex_tac_caption_renders_once`
  - 같은 표에 `hasCaption=true` 적용
  - 표 노드가 1개만 나오고 1쪽에 남는지 확인

기존 회귀 테스트:

- `cargo test --release issue_1470 --lib`
- TAC/표 관련 기존 테스트 중 최소 범위:
  - `tac-inline-create` 또는 동등 CDP 시나리오
  - `tac-inline-table` 또는 렌더 트리 기반 좌표 검증

브라우저 검증:

- 7700 서버에서 CDP 또는 IAB로 다음을 확인한다.
  - 앱 로드, 새 문서 생성, console error 없음
  - `createTableEx(... treatAsChar: true)` 직후 `layoutTables.length = 1`
  - 캡션 on/off 후에도 `layoutTables.length = 1`
  - 스크린샷에서 표와 캡션 번호가 한 위치에만 표시됨

빌드/포맷:

- `cargo fmt --check`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`

`cargo clippy --all-targets -- -D warnings`는 이전 사용자 중지 지시가 있었으므로, 별도 지시가 있을 때만 실행한다.

## 8. 승인 게이트

이 문서 승인 전에는 소스 파일을 수정하지 않는다.
승인 후 Stage 2 구현은 TAC 표 중복 렌더 제거와 focused 회귀 테스트 추가로 제한한다.

## 9. 구현 결과

승인 후 실제 재현 로그를 추가로 확인한 결과, 중복 순서는 단순히 `FullParagraph -> Table`만이 아니라
`Table PageItem -> PartialParagraph` 순서도 존재했다.

확인된 실제 실패 흐름:

- `PageItem::Table { para_index: 0, control_index: 0 }`가 먼저 표를 렌더한다.
- 뒤이어 같은 문단의 `PartialParagraph`가 본문 텍스트를 렌더하면서 오브젝트 마커 위치의 TAC 표를 다시 렌더한다.
- 결과적으로 같은 `paraIdx/controlIdx` 표 노드가 2개 수집된다.

적용한 보정:

- `src/renderer/layout.rs`
  - `segment_width=0`인 신규 문서/API 생성 문단에서는 현재 단 폭을 fallback으로 사용해 TAC 인라인 판정이 0폭 기준으로 실패하지 않도록 했다.
  - TAC Table PageItem이 표를 렌더한 뒤 `inline_shape_position`을 등록하도록 했다.
  - 반대로 `paragraph_layout`이 먼저 등록한 TAC 표는 Table PageItem에서 다시 `layout_table`을 호출하지 않고 흐름 advance만 보존하도록 했다.

- `src/renderer/layout/paragraph_layout.rs`
  - TAC 표 직접 렌더 전 `inline_shape_position` 등록 여부를 확인해 이미 Table PageItem이 그린 표는 다시 렌더하지 않도록 했다.
  - 문단 내부 TAC 표도 `is_tac_table_inline(...)` 판정이 참인 경우에만 직접 렌더하도록 제한했다.

- `src/renderer/pagination/engine.rs`
  - legacy pagination 경로도 `segment_width=0`일 때 단 폭 fallback을 사용하도록 맞췄다.

- `src/wasm_api/tests.rs`
  - `issue_1470_create_table_ex_tac_renders_once`
  - `issue_1470_create_table_ex_tac_caption_renders_once`
  - 두 회귀 테스트를 추가해 같은 `(paraIdx, controlIdx)` 표 렌더 노드가 1개만 나오는지 고정했다.

검증 결과:

- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo test --release issue_1470_create_table_ex_tac --lib`: 통과
- `cargo test --release issue_1470 --lib`: 통과, 5 passed
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`: 통과
- IAB `http://localhost:7700/` 앱 로드: 성공, 콘솔 error 없음
- CDP Studio/WASM 런타임 검증:
  - 새 문서 생성 후 `createTableEx(... treatAsChar: true)` 실행
  - `created.controlIdx = 2`, `pageCount = 1`
  - `getPageControlLayout(0)` 기준 표 노드 수:
    - 생성 직후 `1`
    - `hasCaption=true` 후 `1`
    - `hasCaption=false` 후 `1`
  - 캡션 on 상태 직접 SVG 렌더도 표 노드 `1`

검증 산출물:

- CDP 앱 캡처: `/tmp/rhwp-cdp-1470-stage2-caption-on.png`
- 직접 렌더 SVG: `/tmp/rhwp-cdp-1470-stage2.svg`
- 직접 렌더 PNG: `/tmp/rhwp-cdp-1470-stage2-render.png`

추가 E2E 참고 결과:

- `CHROME_CDP=http://127.0.0.1:19222 VITE_URL=http://localhost:7700 node e2e/tac-inline-create.test.mjs --mode=host`
  - 실패.
  - 표 중복 렌더가 아니라 표 뒤 커서 이동/키보드 입력 순서 문제를 드러냈다.
  - 최종 텍스트가 `표 다음`으로 이어지지 않고 `다음`과 `표`가 서로 다른 문단에 갈라졌다.
  - Stage 2 범위 밖의 다음 스테이지 후보로 분리한다.

- `CHROME_CDP=http://127.0.0.1:19222 VITE_URL=http://localhost:7700 node e2e/tac-inline-table.test.mjs --mode=host`
  - 실패.
  - 테스트가 현재 Studio API에 없는 `w.getParaText`를 호출해 초기 검증 단계에서 중단됐다.
  - 테스트 스크립트 갱신이 필요하므로 Stage 2 통과/실패 판단 근거로 쓰지 않는다.
