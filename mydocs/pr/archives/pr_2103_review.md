# PR #2103 리뷰 문서

## 1. 메타

| 항목 | 내용 |
|---|---|
| PR | #2103 |
| 제목 | task 2069: OLE 개체 선택/캡션/붙여넣기 한컴 호환 보정 |
| 작성자 | jangster77 |
| base | devel |
| head | task/m100-2069-ole-object-selection |
| 관련 이슈 | Closes #2069 |
| 작성 시점 참고값 | mergeable=MERGEABLE, 코드 검증 기준 SHA=200f64d858a443ee7981901d889e7229851900f9 |
| 규모 | 37 files, +3259 / -361 (문서/asset 추가 전 GitHub 참고값) |

최종 merge 조건은 PR head 최신 커밋 기준 GitHub Actions 통과와 작업지시자 승인이다.
위 mergeable, SHA, 규모 값은 작성 시점 참고값이며 merge 전 재확인한다.

## 2. 관련 이슈 요약

#2069는 `samples/한셀OLE.hwp`에서 한셀 OLE 미리보기가 보이지만 한컴처럼 개체 선택, 개체 속성 진입,
우측 캐럿 표시가 되지 않고, 우측 클릭/하단 클릭 시 `CursorState updateRect` 경고가 반복되는 문제다.

한컴 기준 기대 동작은 다음과 같다.

- OLE 오른쪽 가장자리에 캐럿이 표시된다.
- OLE 클릭 시 파란 선택 핸들이 표시된다.
- 우클릭 메뉴에서 `개체 속성(P)...`로 진입할 수 있다.
- OLE 내부를 표 셀처럼 편집하지 않는다.

## 3. 변경 범위

핵심 변경은 다음과 같다.

- OLE RawSvg/preview render node에 원본 control 참조 메타를 보존한다.
- render tree, control layout, cursor rect, mouse/input handling에서 OLE를 선택 가능한 개체로 취급한다.
- 빈 문단 + non-TAC OLE에서 한컴처럼 OLE 오른쪽 wrap-zone 캐럿 rect를 합성한다.
- Enter/Backspace 후 다시 Enter를 누르는 흐름에서 OLE anchor/wrap line이 깨지지 않게 보정한다.
- OLE/그림/표 캡션 설정, 해제, 자동번호 재계산, caption path cursor를 정리한다.
- 그림 선택 후 다른 캐럿 위치로 이동한 뒤 붙여넣기할 때 내부 그림 control copy가 유지되도록 보강한다.
- `samples/한셀OLE.hwpx`와 issue 2069 전용 Rust/E2E 회귀 테스트를 추가한다.

## 4. 렌더 영향 및 시각 검증

이 PR은 PDF pagination/layout visual sweep보다는 editor interaction, render tree metadata, cursor rect,
object selection UI를 바꾸는 PR이다. 따라서 기준 PDF/MCP visual sweep은 수행하지 않았다.

대신 `localhost:7700` Vite 앱과 headless Chrome E2E로 다음 사용자-visible 동작을 검증했다.

- OLE 개체 속성 대화상자 진입
- OLE 캡션 설정 및 제거
- OLE 선택 캐럿 위치
- Enter -> Backspace -> Enter 재진입 동작
- OLE 내부 클릭이 표 셀 편집으로 들어가지 않는지
- 그림 선택 후 캐럿 이동 뒤 markerless paste가 그림 control을 복사하는지

보존한 대표 증적 asset은 다음과 같다.

| asset | SHA-256 |
|---|---|
| `mydocs/pr/assets/pr_2103_ole_click_object_selection.png` | `3578b3b89959b8d4fac0e99d8c82a19a4bf222f30ef7098ba2ac5b369d3d1950` |
| `mydocs/pr/assets/pr_2103_ole_enter_backspace_reenter.png` | `29a3fa4bbd3dfee7c03c2f7978797f66dadfd78f93be4ea81dd67c1010d09394` |
| `mydocs/pr/assets/pr_2103_ole_caption_right_bottom.png` | `2aff349fb2f6ddc18aaa891b470f05a350fdfbcf27e400655ce3fda14c1a0121` |
| `mydocs/pr/assets/pr_2103_ole_caption_removed.png` | `a768dfff44c77a2eb856427c2869ec3a52e2c70456aff5846ab771df918e72c2` |

추가 샘플:

- `samples/한셀OLE.hwpx`
- SHA-256: `e9d1b0b79021f47743f56e1e162c0226fbc4e83865d4443cb9e0ce8af3e4cc8a`

## 5. 검증 결과

최종 확인한 로컬 검증은 다음과 같다.

- `cargo fmt --check` 통과
- `cargo test --lib issue_1470_picture_caption_path_cursor_and_control_paste` 통과
- `cargo test --lib issue_1470_picture_caption_can_be_removed_and_renumbers` 통과
- `cargo test --lib issue_1470_table_caption` 통과
- `cargo test --test issue_2069_ole_object_selection` 통과
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 통과
- `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` 통과
- `wasm-pack build --target web --out-dir pkg` 통과
- `cd rhwp-studio && npm run build` 통과
- `CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" VITE_URL=http://localhost:7700 node e2e/issue-2069-ole-object-selection.test.mjs --mode=headless` 통과
- `git diff --check` 통과

`cargo test --profile release-test --tests` 첫 실행에서 `RawSvgNode` 테스트 fixture가 신규 `control_ref` 필드를
채우지 않아 컴파일 실패했으나, synthetic test fixture를 `RawSvgNode::new(...)` 생성자 사용으로 보정한 뒤
전체 통합 테스트를 재실행해 통과했다.

## 6. 리스크 및 잔여 확인

- OLE 미리보기 자체의 한컴셀 내부 편집은 지원 범위가 아니다. 이번 PR은 한컴처럼 OLE를 문서 개체로 선택하고
  개체 속성/캡션/캐럿 흐름을 맞추는 범위다.
- PDF 기준 visual sweep은 수행하지 않았다. 이번 변경의 핵심 검증축은 editor interaction과 render tree/cursor
  metadata다.
- merge 전 GitHub Actions 최신 head 결과를 재확인한다.

## 7. 최종 권고

로컬 전체 통합 테스트, clippy, WASM build, studio build, E2E 검증을 통과했고, #2069의 핵심 완료 기준을
충족한다. PR #2103은 옵션 1 경로로 review 문서, E2E asset, 오늘할일 갱신을 같은 PR head에 포함한 뒤
GitHub Actions 통과와 작업지시자 승인 조건으로 merge 가능 후보로 본다.
