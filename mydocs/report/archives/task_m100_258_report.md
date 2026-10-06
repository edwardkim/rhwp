# Task M100-258 최종 보고서 — 한글 누름틀 + 양식 모드 구현

- 이슈: https://github.com/edwardkim/rhwp/issues/258
- 브랜치: `local/task_m100_258`
- 작성일: 2026-06-15
- 기준 브랜치: `upstream/devel`

## 1. 완료 범위

- rhwp-studio에 `normal`/`form` 편집 모드를 추가했다.
- 양식 모드에서 `editable=true`인 ClickHere 누름틀 내부 텍스트 입력/삭제만 허용하고,
  일반 본문 입력, 삭제, 붙여넣기, 구조 삽입, 서식 변경, 누름틀 삭제를 차단했다.
- `getFieldInfoAt*`, `getFieldList()` JSON에 양식 모드 판단과 이동에 필요한
  `editableInForm`, `startCharIdx`, `endCharIdx`를 노출했다.
- 양식 모드에서 Tab/Shift+Tab으로 다음/이전 editable ClickHere로 이동하도록 했다.
- `insert:field` 스텁을 누름틀 삽입 대화상자로 교체했다.
- 본문/셀/중첩 cellPath 위치에 ClickHere field range와 command/CTRL_DATA를 생성하는
  Rust/WASM API를 추가했다.
- HWPX 직렬화에서 새로 생성한 ClickHere `Field.command`를 `hp:parameters`로 저장해
  HWPX 재파싱 시 안내문/메모가 보존되도록 했다.
- 누름틀 입력/고치기 대화상자는 바깥 클릭으로 닫히지 않게 했고, 누름틀 삽입 직후에는
  한컴처럼 안내문이 표시되도록 active field를 즉시 잡지 않게 했다.
- 누름틀 끝에서 오른쪽 이동 후 이어 입력하면 field range 밖 본문으로 들어가도록 했고,
  누름틀 경계 삭제는 한컴처럼 `[누름틀]을 지울까요?` 확인을 거치게 했다.
- 빈 누름틀 안내문 클릭 후 첫 입력 위치를 field start로 정규화해 `입력하세요` 클릭 뒤
  바로 `123` 같은 값을 입력할 수 있게 했다.
- 빈 누름틀 첫 입력 직후 active field와 마커를 새 field value 기준으로 다시 계산해,
  Enter 같은 추가 편집 없이도 입력값과 누름틀 마커가 즉시 표시되도록 보정했다.
- 빈 누름틀 첫 입력 뒤 `getCursorRect()`가 0폭 placeholder가 아니라 실제 입력값 기준
  caret x를 반환하도록 해 field end 마커가 입력값 끝을 따라가게 했다.
- 기존 HWP 샘플에서 `SectionDef`/`ColumnDef`/`Field` 같은 비가시 컨트롤 gap이
  본문 TextRun source offset을 밀지 않도록 placeholder 합성 대상을 좁혔다.
- 누름틀 삭제 확인 대화상자의 `확인`이 대화상자를 띄운 원래 커서 위치의 field를
  제거하도록 고정했다.
- 누름틀 삭제 확인 시 한컴 동작에 맞춰 `FieldRange`, `Control::Field`,
  대응 `ctrl_data_records`, field range 내부 본문 텍스트까지 함께 제거하도록 고정했다.
- 누름틀 삭제 후 커서가 삭제된 field 끝 offset에 남지 않고 삭제 전 field 시작 위치로
  돌아가도록 고정했다.
- 누름틀 전체 선택 복사 시 구조 컨트롤 제거 후 `FieldRange.control_idx`와
  `ctrl_data_records`가 어긋나 ClickHere 속성이 사라지던 문제를 보정했다.
- 문단 중간 누름틀 전체 선택 복사 시 `split_at(start)`로 값 텍스트와 `Control::Field`가
  분리되어 빈 누름틀처럼 붙여넣어지던 문제를 보정했다.
- 누름틀 시작/끝 경계의 바깥 상태를 분리해 방향키와 Home/End가 한컴처럼 누름틀 이전/이후
  위치를 만들 수 있게 했다.
- 누름틀 시작 이전 위치에서 입력한 텍스트가 누름틀 값으로 들어가지 않고 일반 본문으로
  남도록 field start 삽입 경계를 보정했다.

## 2. 검증

- `cargo fmt --check`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --test issue_838_field_set_value`
- `wasm-pack build --target web --out-dir pkg`
- `npm run build`
- `git diff --check`

Stage8 추가 검증:

- `cargo fmt`
- `cargo test --test issue_258_clickhere_form_mode`
- `npm run build`
- `git diff --check`

Stage9 추가 검증:

- `cargo fmt`
- `cargo test --test issue_258_clickhere_form_mode`
- `npm run build`

Stage10 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode`
- 루트 `npm run build`는 스크립트가 없어 실행 불가 확인
- `cd rhwp-studio && npm run build`
- `git diff --check`

Stage11 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode first_input_into_empty_clickhere_is_rendered -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --lib rebuild_`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` Playwright 검증 통과
  (`value=123`, field range `0..3`, cursor x `113.4→135.4`, `「123」` 표시)

Stage12 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode clickhere_hwp_sample_cursor_rects_follow_visible_value -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo fmt --check`
- `git diff --check`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` Playwright 검증 통과
  (`누름틀-2024.hwp` range `0..8`, cursor x `113.4→172.1`, 삭제 확인 후 첫 field 제거)

Stage13 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode removing_clickhere_keeps_text_but_removes_field_control -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --lib rebuild_`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` Playwright 검증 통과
  (`확인` 클릭 후 첫 field 제거, `getFieldInfoAt(0,0,8)={"inField":false}`, 텍스트는 본문으로 유지)

Stage14 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode removing_clickhere_removes_field_text_and_control -- --exact --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cargo test --lib rebuild_`
- `cargo fmt --check`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` Playwright 검증 통과
  (`Delete` 확인 후 첫 문단 text 빈 문자열, field 목록에는 두 번째 `222212212`만 유지,
  렌더 SVG에 `11223344` 미존재)

Stage15 추가 검증:

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/` Playwright 검증 통과
  (`Delete` 확인 후 첫 문단 text 빈 문자열, 커서 `charOffset=8→0`, 삭제 후
  `getFieldInfoAt(0,0,0)={"inField":false}`)

Stage16 추가 검증:

- `upstream/devel` fetch/rebase 완료 (`upstream/devel` `df4f4a83` 기준)
- `cargo build --release`: 통과
- `cargo test --release --lib`: 최초 실패 후 테스트 fixture 기대값 정정
  - 실패 테스트: `document_core::queries::field_query::tests::set_cell_field_text_updates_text_metadata`
  - 원인: `Paragraph.char_count`는 문단 끝 마커 포함 기준인데 테스트 기대값이 텍스트 길이만 반영
  - 수정 후 `cargo test --release --lib set_cell_field_text_updates_text_metadata`: 통과
  - 수정 후 `cargo test --release --lib`: 통과

Stage17 추가 검증:

- `cargo fmt`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode copying_clickhere_preserves_field_control_after_structural_controls_are_stripped -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`

Stage18 추가 검증:

- `cargo fmt --check`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode copying_clickhere_after_prefix_preserves_field_value -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `wasm-pack build --target web --out-dir pkg`

Stage19 추가 검증:

- `cargo fmt`
- `git diff --check`
- `cargo test --test issue_258_clickhere_form_mode clickhere_start_boundary_insert_respects_active_field_state -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `wasm-pack build --target web --out-dir pkg`

Stage20 추가 검증:

- `cd rhwp-studio && npm run build`
- Chrome 자동 검증은 현재 Codex 세션에서 Chrome 확장 백엔드가 노출되지 않아 보류
  - Chrome 실행/확장 설치/Native host manifest는 정상 확인

Stage21 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `git diff --check`
- `wasm-pack build --target web --out-dir pkg`
- `http://localhost:7700/` Playwright 검증 통과
  (`ArrowLeft` 후 `a`, `b` 입력 시 첫 문단 text `ab11223344`,
  첫 field value `11223344`, field range `2..10`, 커서 `fieldStartExitKey` 유지)

Stage22 추가 검증:

- `cd rhwp-studio && npm run build`
- `cargo test --test issue_258_clickhere_form_mode`
- `git diff --check`
- `http://localhost:7700/` Playwright 검증 통과
  (새 누름틀에 `123` 입력 후 왼쪽 방향키 4회, `abcd` 연속 입력 시 본문 text `abcd123`,
  field value `123`, field range `4..7`)
- `http://localhost:7700/` Playwright 추가 검증 통과
  (`123` 입력 후 왼쪽 방향키 4회, `abc`, 오른쪽 방향키 1회:
  `abc`는 field 밖 prefix, 오른쪽 방향키 후 같은 `charOffset=3`에서 field 내부 시작으로 진입)

Stage23 추가 검증:

- `cd rhwp-studio && npm run build`
- `git diff --check`
- `http://localhost:7700/` Playwright 검증 통과
  (`123` 입력 후 왼쪽 방향키 4회, `abc`, 오른쪽 방향키 1회:
  본문 text `abc123`, field value `123`, field range `3..6`,
  시작 낫표가 field 시작 경계 기준에 표시)

Stage24 추가 검증:

- `cd rhwp-studio && npm run build`
- `http://localhost:7700/` Playwright 검증 통과
  (새 누름틀 삽입 직후 guide `사용자 이름`과 caret이 오른쪽 바깥에 표시,
  `←` 후 guide가 사라진 빈 입력칸으로 진입,
  입력 없이 `→` 후 guide와 오른쪽 바깥 caret 복원,
  `abc|123` start-exit 상태에서 `←`는 `ab|c123`으로 이동,
  start-exit 상태 Backspace는 확인창 없이 `ab|123`으로 앞 글자 삭제)
- `git diff --check`

Stage25 추가 검증:

- `cargo test --test issue_258_clickhere_form_mode adjacent_clickhere_input_prefers_new_empty_field_at_shared_boundary -- --nocapture`
- `cargo test --test issue_258_clickhere_form_mode`
- `wasm-pack build --target web --out-dir pkg`
- `cd rhwp-studio && npm run build`
- `cargo fmt --check`
- `git diff --check`
- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음
- `http://localhost:7700/` Playwright 검증 통과
  (`abc[123][123]` 구성 후 두 field value가 각각 `123`, field range가 `3..6`, `6..9`로 분리,
  `Shift+ArrowRight` 6회 후 selection range `3..9`, selection rect 폭 `44px`로 `123123` 전체 선택)

Stage26 추가 검증:

- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음
  - Browser 탭에서는 개발용 `__wasm/__inputHandler` 전역이 없어 내부 상태 검증은 Playwright로 수행
- `http://localhost:7700/` Playwright 실제 마우스 드래그 검증 통과
  (`abc[123][123]` 구성 후 field range `3..6`, `6..9`, selection range `3..9`,
  selection rect 폭 `44px`, selection layer `mix-blend-mode: difference`,
  field marker `display:none`, `123123` 전체가 검은 배경/흰 글자로 표시)
- `cargo test --test issue_258_clickhere_form_mode`
- `cd rhwp-studio && npm run build`
- `cargo fmt --check`
- `git diff --check`

Stage27 추가 검증:

- Stage26의 검은 반전 선택색은 작업지시자 기준과 달라 폐기하고, 선택 highlight를 기존 파란 반투명
  `rgba(51,144,255,0.35)`로 복원
- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, console error/warn 없음
- `http://localhost:7700/` Playwright 일반 본문 선택 검증 통과
  (`dddddddddddd` 범위 `2..12`, selection layer `mix-blend-mode: normal`,
  highlight background `rgba(51, 144, 255, 0.35)`)
- `http://localhost:7700/` Playwright 샘플 누름틀 선택 검증 통과
  (`samples/누름틀-2024.hwp` 첫 누름틀 `11223344`, selection layer `mix-blend-mode: normal`,
  highlight background `rgba(51, 144, 255, 0.35)`)
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (11 passed)
- `cd rhwp-studio && npm run build`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

Stage28 추가 검증:

- `[123][123]` 인접 누름틀 복사/붙여넣기 후 두 번째 누름틀이 숨는 문제를 재현하고 보정
- 내부 클립보드 붙여넣기 시 복사본 field ID를 현재 문서 최대 ID 이후로 재부여
- 붙여넣기/문단 분리 후 누름틀 `field_ranges`가 남은 문단은 `rebuild_char_offsets`로
  FIELD_BEGIN/FIELD_END gap과 `char_count`를 재정규화
- `copying_adjacent_clickheres_preserves_separate_pasted_fields` 회귀 테스트 추가
  (원본 2개 + 붙여넣기 2개 field ID 분리, range `0..3`/`3..6`, SVG 숫자 4회 렌더, 삭제 후 남은 필드 보존)
- 작업지시자 시각 확인: 복사/붙여넣기 표시 정상
- `cargo test --test issue_258_clickhere_form_mode`: 통과 (12 passed)
- `wasm-pack build --target web --out-dir pkg`: 통과
- Browser plugin: `http://localhost:7700/` 로드, title `rhwp-studio`, toolbar 표시, console error/warn 없음
- `cd rhwp-studio && npm run build`: 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과

Stage29 추가 검증:

- `Home`/`End`가 일반 본문 키 처리 switch 전에 `handleNavigationShortcut`에서 먼저 처리되어
  누름틀 경계 밖 상태(`fieldStartExitKey`/`fieldEndExitKey`)가 설정되지 않던 문제를 보정
- `executeNavigationAction`의 `lineStart`/`lineEnd` 경로에도 각각
  `markCurrentFieldStartOutside`/`markCurrentFieldEndOutside`를 호출
- 작업지시자 시각 검증 완료: 인접 누름틀에서 Home/End 후 누름틀 바깥 줄 시작/끝 이동 정상
- `cd rhwp-studio && npm run build`: 통과
- `git diff --check`: 통과

Stage30 PR 준비 검증:

- 원격 `upstream/devel`과 로컬 추적 `upstream/devel`이 모두
  `0ae7fe1a04525cc16da98e85a2aaf43cd102f53c`로 동일함을 확인
- 작업 브랜치 HEAD: `6ec70a8171e34480c576e295bb7d3da290d5a4b0`
- `local/task_m100_258`은 `upstream/devel` 대비 29 commits ahead
- `git diff --check`: 통과
- `cargo build --release`: 통과
- `cargo test --release --lib`: 통과 (1824 passed, 6 ignored)
- `cargo test --profile release-test --tests`: 통과
- `cargo fmt --check`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `cd rhwp-studio && npm run build`: 통과
  - Vite chunk size 경고만 발생, 빌드는 성공
- 작업트리 깨끗함

## 3. 남은 후속

- 사용자 정보, 문서 요약, 작성한 날짜, 파일 이름/경로 등 누름틀 외 필드 탭은 후속 이슈로 분리한다.
- 양식 개체 전체(Edit/CheckBox/RadioButton/ComboBox/PushButton)의 완전 상호작용은 기존
  FormObject 작업과 이어서 별도 처리한다.
- PR용 원격 브랜치 push와 PR 생성은 작업지시자 승인 후 진행한다.
- GitHub Actions CI 확인은 PR 생성 후 진행한다.
