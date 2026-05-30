# Task M100 #1139 Stage 31

## 목적

한컴오피스 도움말의 `미주` 및 `주석 모양: 미주 모양` 동작을 분석하고, rhwp-studio에 미주 삽입과 미주 모양 설정 기능을 추가한다.

## 시작 기준

- 기준 커밋: `46b5be3c` (`task 1139: Stage30 머릿말 표 기준 보정`)
- Stage30 변경은 rebase 이후 `46b5be3c`로 정리했다.
- `upstream/devel`은 `a5a3e0e5`까지 동기화했고, Task #1139 커밋 28개는 그 위에 linear rebase 했다.
- 임시 merge 결과 `ca4d27bb`는 `backup/task_m100_1139-merge-ca4d27bb` 브랜치에 보존했다.

## 배경 분석

작업지시자가 rhwp-studio 콘솔에서 메뉴 각주를 누를 때 다음 로그가 보인다고 보고했다.

```text
[footnote] pos: {sectionIndex: 0, paragraphIndex: 0, charOffset: 0}
insert.ts:137 [footnote] result: {ok: true, paraIdx: 0, controlIdx: 2, footnoteNumber: 1}
insert.ts:134 [footnote] pos: {sectionIndex: 0, paragraphIndex: 0, charOffset: 0}
insert.ts:137 [footnote] result: {ok: true, paraIdx: 0, controlIdx: 3, footnoteNumber: 2}
```

이 분석 결과, 보고된 각주 로그와 새 문서 각주 마커 미표시 문제는 Task #1139 회귀가 아니라 `upstream/devel`에도 존재하는 기존 결함으로 판별했다. 이후 작업지시자가 Stage31 목적을 미주 기능 추가로 변경했다.

## 한컴 기능 기준

- 참고 도움말:
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fendnote_format.htm`
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fendnotes.htm`
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fannotations(format).htm`
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fannotations.htm`
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=format%2Fnew_number.htm`
  - `https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Ffootnote_format.htm`
- 한컴 도움말 `미주`: 본문 커서 위치에 미주 번호를 넣고, 미주 내용은 현재 구역의 끝 또는 문서의 끝에 모아 배치한다.
- 한컴 도움말 `미주`: 미주 실행 직후 커서는 미주 내용을 입력할 수 있는 문서 끝 위치로 이동하고, `주석` 탭이 나타난다. 본문 복귀는 `<Shift+Esc>` 또는 `주석` 탭의 `닫기`로 수행한다.
- 한컴 도움말 `주석 모양: 미주 모양`: 미주 번호 서식, 앞/뒤 장식 문자, 본문과 미주 사이 구분선, 구분선/미주 사이 여백, 번호 매기기 방식, 미주 내용 번호 속성, 미주 위치를 구역 속성으로 설정한다.
- 한컴 도움말 `주석 모양: 미주 모양`: 기본 미주 번호는 `1)`이며, 번호 모양은 `1,2,3`, 앞 장식 문자는 없음, 뒤 장식 문자는 `)`이다.
- 한컴 도움말 `각주/미주 모양`: 각주는 본문 쪽 아래에, 미주는 본문 위치와 무관하게 장/문서 끝에 모인다는 차이를 명시한다.
- 한컴 도움말 `새 번호로 시작`: 각주/미주 번호도 특정 위치 이후부터 사용자가 지정한 시작 번호로 다시 매길 수 있다.

## 진행 계획

1. 기존 `Endnote` IR, 미주 렌더링, `endnote_shape` 파싱/직렬화 경로를 확인한다.
2. native/WASM에 미주 삽입 API를 추가한다.
3. native/WASM에 미주 모양 조회/적용 API를 추가한다.
4. rhwp-studio 입력 메뉴/툴바의 `미주` 스텁을 실제 명령으로 연결한다.
5. rhwp-studio에 `미주 모양` 대화상자를 추가해 번호 서식, 장식 문자, 구분선, 여백, 번호 매기기, 미주 위치를 설정한다.
6. 미주 삽입 직후 미주 내용 편집 모드로 진입하고, 주석 전용 도구상자와 닫기 버튼을 표시한다.
7. 미주 삽입과 미주 모양 적용 회귀 테스트를 추가하고 Rust/WASM/Studio 빌드 검증을 수행한다.

## 현재 상태

- 2026-05-30: 작업지시자가 Stage30 시각 검증 완료 후 커밋, `upstream/devel` 동기화, 각주 메뉴 로그의 회귀 여부 확인을 지시했다.
- 2026-05-30: Stage30 커밋과 `upstream/devel` 동기화를 완료하고 Stage31을 시작했다.
- 2026-05-30: 작업지시자가 merge 대신 rebase가 맞는지 확인했고, 기존 커밋 기록이 사라지지 않았음을 확인한 뒤 임시 merge 커밋은 백업 브랜치에 보존하고 `upstream/devel` 위로 rebase 했다.
- 2026-05-30: `rhwp-studio/src/command/commands/insert.ts`의 `[footnote] pos/result` 로그는 `git blame`과 `upstream/devel` 비교 결과 초기 커밋부터 있던 `console.log`이며 Task #1139 변경이 추가한 에러 로그가 아님을 확인했다.
- 2026-05-30: `insertFootnote` 메뉴 경로는 현재도 `result.ok` 성공 시 `document-changed`만 emit한다. 보고된 로그는 `{ok: true}` 성공 응답이다.
- 2026-05-30: 현재 HEAD에서 각주 관련 기존 회귀 테스트를 확인했다.
  - `cargo test --test issue_1058_textbox_list_header issue_1058_new_footnote_inner_para_contract -- --nocapture`
  - `cargo test --test issue_598_footnote_marker_nav -- --nocapture`
- 2026-05-30: 1차 판단은 "보고된 콘솔 로그 자체는 Task #1139 회귀가 아니라 기존 디버그 로그"이다. 실제 UI 오류가 따로 있다면 브라우저 콘솔의 `console.warn('[insert:footnote] 각주 삽입 실패:', err)` 또는 stack trace를 별도로 확인해야 한다.
- 2026-05-30: 작업지시자가 새 문서에서 각주 메뉴 1회 클릭 로그를 다시 제공했다. `content.js:1 Uncaught (in promise) The message port closed...`는 저장소 소스에서 검색되지 않는 Chrome extension content script 계열 로그로 판단하고, rhwp-studio 각주 경로와 분리했다.
- 2026-05-30: 임시 재현 테스트로 `HwpDocument::create_empty()` → `create_blank_document_native()` → `insert_footnote_native(0,0,0)` 2회 → `build_page_render_tree(0)`를 확인했다. 현재 HEAD에서 API 응답은 `{ok:true, controlIdx:2/3, footnoteNumber:1/2}`지만 렌더 트리의 `FootnoteMarker`가 `[]`로 비어 실패했다.
- 2026-05-30: 같은 임시 재현 테스트를 별도 worktree의 `upstream/devel`(`a5a3e0e5`)에서도 실행했고 동일하게 실패했다. 따라서 "새 문서 각주 삽입 후 마커/각주 영역이 보이지 않는 문제"는 Task #1139 회귀가 아니라 upstream에도 이미 존재하는 기존 결함이다.
- 2026-05-30: 원인 형태: 삽입 후 본문 문단은 `text=""`, `char_offsets=[]`, `controls=4`, `char_count=33` 상태가 된다. 각주 컨트롤은 추가되지만 visible text/control anchor가 없어 `composer`의 `footnote_positions`와 `paragraph_layout`의 `FootnoteMarker` 생성 경로에 도달하지 못한다.
- 2026-05-30: 작업지시자가 Stage31 목적을 미주 및 미주 모양 기능 추가로 변경했다. 한컴 도움말 기준으로 미주는 문서/구역 끝에 모이는 주석이며, 미주 모양은 구역 속성으로 처리한다.
- 2026-05-30: `insert_endnote_native`/`insertEndnote`를 추가해 본문 위치에 `Control::Endnote`를 삽입하고, 미주 내용 문단에는 `AutoNumberType::Endnote` anchor를 생성하도록 했다.
- 2026-05-30: `get_endnote_shape_native`/`apply_endnote_shape_native` 및 WASM bridge를 추가해 구역의 `endnote_shape` 번호 서식, 장식 문자, 구분선, 여백, 번호 매기기, 위치를 조회/적용하도록 했다.
- 2026-05-30: rhwp-studio 입력 메뉴와 툴바의 `미주`를 실제 명령으로 연결하고, 입력 메뉴에 `미주 모양` 대화상자를 추가했다.
- 2026-05-30: 미주 모양의 앞 장식 문자가 기존 미주에 반영되지 않는 문제를 확인하고, `apply_endnote_shape_native`에서 기존 `Control::Endnote`와 내부 `AutoNumberType::Endnote`를 함께 재번호/재동기화하도록 보정했다.
- 2026-05-30: 한컴 도움말 기준으로 미주 삽입 직후 커서가 미주 내용 위치로 이동해야 하므로, `getNoteEditInfo`/`getCursorRectInNote`를 추가하고 rhwp-studio의 기존 각주 편집 모드를 미주에도 재사용하도록 연결했다.
- 2026-05-30: 미주/각주 편집 중 한컴처럼 `주석` 전용 도구상자를 표시하고, `각주`, `미주`, `각주/미주 모양`, `닫기` 버튼을 배치했다.
- 2026-05-30: `닫기` 버튼은 `insert:note-close` 명령으로 연결하고, `<Shift+Esc>`/`Escape`와 동일하게 주석 편집 모드를 종료해 본문 위치로 복귀하도록 했다.
- 2026-05-30: `tests/issue_1139_inline_picture_duplicate.rs`에 Stage31 회귀 테스트 4개를 추가했다.
  - 새 빈 문서에서 미주 삽입 후 본문/미주 기본 `1)` 마커 렌더 확인
  - 미주 삽입 직후 편집 대상 조회, 미주 내부 캐럿 좌표 계산, `test` 입력 렌더 확인
  - 미주 모양 적용 후 기존 미주 앞/뒤 장식 문자와 시작 번호 재반영 확인
  - 미주 모양 API 적용 후 `endnote_shape` 필드와 JSON round-trip 확인
- 2026-05-30: 검증 완료.
  - `cargo fmt --all --check`
  - `cargo test --test issue_1139_inline_picture_duplicate issue_1139_stage31 -- --nocapture`
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo test --test issue_598_footnote_marker_nav -- --nocapture`
  - `cargo build`
  - `npm --prefix rhwp-studio run build`
  - `git diff --check`
  - `wasm-pack build --target web --out-dir pkg`
  - 로컬 Chrome headless DOM 검증: `미주` 클릭 후 주석 도구상자 표시, `121212` 입력, `닫기` 클릭 후 주석 모드 종료 확인
- 2026-05-30: Stage31 변경분을 커밋했고, 내부 절차에 따라 PR 생성은 작업지시자 승인 대기 상태로 둔다.
- 2026-05-30: 작업지시자 승인 후 PR #1178을 생성했다. URL: `https://github.com/edwardkim/rhwp/pull/1178`
