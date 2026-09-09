# 단계 2 — 하이퍼링크 코어 편집과 저장 왕복

- Issue: [#6963](https://github.com/edwardkim/rhwp/issues/6963)
- 실행일: 2026-09-10
- 기준: `a3cd825c23d550e0c5f46b4e2eb3735a537f23f2`
- 상태: 코어 구현 및 focused 검증 완료. Studio UI와 PDF 출력은 후속 단계.

## 변경 내용

`src/model/hyperlink.rs`에 한컴 Command의 첫 주소 성분을 읽는 `command_uri`와
HTTP/HTTPS 주소를 `;1;0;0;` Command로 만드는 `web_command`를 추가했다.
콜론·세미콜론·fragment 구분자를 이스케이프하며, 한글·query·percent encoding은
원문 그대로 둔다. 기존 doclang 내보내기도 같은 decoder를 사용한다.
읽기는 기존 mail/file/internal 표현을 그대로 지원하고, 신규 웹 주소 입력은 scheme,
빈 authority, 공백·제어문자·역슬래시·사용자정보, HWP5의 u16 문자열 길이를 검사한다.
이 검사는 전체 URL 정규화나 DNS/접속 가능성 검사가 아니다.

`DocumentCore`에 다음 API를 추가했다. 모든 offset은 Unicode scalar 인덱스이며
반열린 범위 `[start, end)`다. `HyperlinkTarget`은 section/본문 para와 선택적인 중첩
cell_path를 갖는다. 빈 경로는 본문, 경로 항목은 `(컨트롤, 셀, 내부 문단)`이다.

- `hyperlinks_native`: ID·주소·표시 글자·범위 조회.
- `insert_hyperlink_native`: 선택 문자열에 필드 시작/끝 마커 추가.
- `update_hyperlink_native`: 주소만 수정. 같은 주소는 `false`를 반환하고 문서·이벤트를 유지.
- `remove_hyperlink_native`: 표시 문자열과 글자 서식을 남기고 필드 마커만 제거.

본문·표 셀·글상자의 단일 문단을 지원한다. 캡션·HWP3·다단락 필드·개체가 들어 있는
문단·겹치는 필드는 명시적으로 거부한다. 다단락 필드의 중간 문단도 그 문단 목록 안의
열린 필드를 검사해 거부한다. 지원하지 않는 경로를 본문 경로로 바꾸지 않는다.
무선택 상태에서 표시 글자를 새로 만드는 기능은 host가 텍스트 삽입과 링크 추가를 한
snapshot으로 묶도록 했다. WASM bridge와 Studio operation 연결은 단계 4에서 수행한다.

변경 전에 URL·경로·범위를 검사하고 문단 복제본을 수정한 후 반영한다. 필드 마커를
넣고 뺄 때 글자 offset·글자 서식·range tag의 UTF-16 축도 함께 옮긴다. 같은 주소의
수정은 캐시와 이벤트에 영향을 주지 않는다. 변경 이벤트는 `HyperlinkChanged`이며
중첩 경로를 포함한다. ID 검색은 각주·머리말·글상자·그룹 도형·캡션 등까지 순회하고,
새 하이퍼링크 ID 공간이 소진되면 오류를 반환한다.

HWPX 주소 수정은 `Field.command`, named `Command`/`Path`를 함께 갱신하고
`raw_parameters_xml`을 무효화한다. 나머지 named parameter와 HWP5의 해석하지 않는
CTRL_DATA는 보존한다. HWPX 전용 추가 파라미터가 HWP5 변환에서 일부 손실되는 기존
제약(#4396)은 그대로이며, 주소 보존과 추가 파라미터 보존을 별도로 검사했다.

## 검증

회귀 테스트 원본은 `tests/cases/issue_6963_hyperlink_edit.rs`이다.
파생 suite는 별도 review worktree에서만 준비했으며 커밋 대상에 포함하지 않는다.

최종 focused 회귀 테스트 **11/11 통과**, native/WASM library Clippy(`-D warnings`),
해당 integration suite Clippy, 전체 fmt check와 파생 suite manifest check 통과.
최종 파생 배정은 `regression_suite_025`이며, 재배정 가능하므로 아래 case runner를 사용한다.

11개 테스트가 다음 계약을 확인한다.

- 한글·query·fragment·세미콜론·역슬래시 decoder와 잘못된 입력 거부.
- 이모지·탭이 있는 선택 범위의 추가/수정/해제 및 HWP/HWPX 저장 왕복.
- 인접 링크 세 개 중 첫째·가운데·마지막을 각각 해제한 뒤 다른 범위 보존.
- 오류와 동일 주소 수정 시 문서·이벤트 불변.
- 한컴 `samples/hwpx_sample2.hwpx`의 실제 중첩 fragment 링크 주소 수정 후,
  HWP/HWPX 주소 보존 및 HWPX의 나머지 named parameter 보존.
- 한컴 `samples/basic/Textmail.hwp`의 실제 중첩 링크 해제/재삽입 후,
  HWP/HWPX의 주소와 전체 본문·셀·글상자 문자열 보존.
- snapshot 복원과 표시 문자 기준 글자 서식 경계 보존.
- 개체 문단·잘못된 중첩 경로 거부, 각주 필드와 새 ID의 충돌 및 ID 초과 방지.
- 링크 앞/안에 글자를 추가한 뒤 저장된 범위, 표시 글자 전부 삭제 후 링크 해제.
- 기존 mailto 옆의 웹 링크 수정 시 mailto 주소 보존.

재실행 명령(review worktree):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs --cargo-test issue_6963_hyperlink_edit -- -p rhwp --target-dir target/pr-review
cargo clippy --locked -p rhwp --lib --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --test regression_suite_025 --target-dir target/pr-review -- -D warnings
cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
```

## 남은 작업

페이지·줄별 링크 영역과 PDF `/Link`·`/URI` 주석은 단계 3이다.
Studio 버튼·대화상자·단축키·WASM bridge·undo/redo의 실제 UI 연결은 단계 4,
브라우저 PDF 저장과 PDF 뷰어 클릭 검증은 단계 5다. 이 단계의 코어 snapshot 시험을
Studio undo/redo나 PDF 보존 완료의 증거로 확대하지 않는다.

PR/push 직전의 전체 workspace lint 및 해당 범위의 최종 검증 게이트는 아직 실행하지
않았다. 원격 push·PR 생성·merge도 수행하지 않았다. 사용자 변경
`samples/exam_eng.pdf`는 이 작업과 분리된 원래 checkout에 그대로 두었다.
