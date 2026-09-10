# 단계 6 — PR 제출 전 통합 검증

- Issue: [#6963](https://github.com/edwardkim/rhwp/issues/6963)
- 실행일: 2026-09-10
- 최초 검증 후보: `1a3d72617`; 분류 가드 보정 후보: `b05cadb0e` (제품 코드 불변)
- 상태: 보정 후보의 필수 로컬 통합 검증·PR 본문 준비 완료. 원격 push·PR 생성 승인 대기.

## 적용 절차와 환경

base route: `collaborator_self_merge.md`.
modifiers: `local_validation.md`, `visual_fixture_evidence.md`.
loaded documents: `pr_review_workflow.md`, `pr_review/README.md` 및 위 자식 문서,
`dev_environment_guide.md`, `edit_command_review_checklist.md`.
PR 번호가 없어 번호 기반 review·오늘할일은 생성하지 않았다.

macOS Apple Silicon, 논리 CPU 12개, 메모리 24GiB에서 nextest 기본 동시성을 사용한다.
Cargo 작업은 전용 review worktree의 고정 `target/pr-review`에서 순차 실행한다.
검증 worktree에 있던 17개 변경이 커밋된 source와 바이트 단위로 일치함을 확인한 뒤
후보 HEAD로 동기화했다. source checkout에는 파생 suite를 만들지 않았다.

최신 `upstream/devel` `37bd46a72f9fd9ffd709e35244df79c00e789780`을 fetch하고
`git merge-tree --write-tree HEAD upstream/devel`을 실행했다. 보정 후보 `b05cadb0e`에서도
충돌 없이 생성된 tree는 `eb3022285b13ae96bf0f498512a6c2469736528e`다. 이 결과는 병합 충돌 검사이며 최신 merge
tree 전체를 별도로 컴파일한 증거는 아니다. GitHub required checks는 게시 후 별도로 확인한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| review suite prepare·fmt·fmt check | 통과 |
| native root Clippy `-D warnings` | 통과 |
| WASM32 library Clippy `-D warnings` | 통과 |
| workspace build·workspace all-target Clippy | 통과 |
| review suite manifest check | 통과 |
| Studio TypeScript `--noEmit` | 통과 |
| Studio `npm test` | 1,501 통과, 2 skip, 0 실패 |
| `release-test` 전체 nextest | 보정 후 9,401 통과 / 0 실패 / 46 skip (실행 249.146초) |
| Native Skia library·placeholder·direct PDF | library 4,112 통과 / 13 ignored, placeholder 2/2, direct PDF 4/4 통과 |
| 하이퍼링크 Skia focused 회귀 | 9/9 통과 |
| 실제 WASM 재빌드 | 같은 review worktree에서 wrapper `--no-opt` 통과 |
| fresh WASM의 Studio package·편집 runner | TypeScript·1,501 unit·production bundle·WASM 시나리오 10종 통과 |
| 브라우저 저장·PDF 회귀 | 최종 후보에서 저장·재열기·PDF 81개 주석·실제 뷰어 클릭 통과 |
| 새 integration 등록 정책 | Node 정책 테스트 23/23 통과 |
| 문서·공백 검사 | 변경 Markdown 9개 링크 및 diff check 통과 |

최초 실행의 원시 로그·시간은 `/tmp/issue6963-gates/`, 보정 후 최종 결과는 그 아래 `final/`에
보관하며 PR에 커밋하지 않는다.
Rust source-side `#[cfg(test)]`, 신규/변경 sample, npm/editor API, CI workflow는
변경하지 않아 각 전용 정책·신규 sample 보안 입력·package/CI 검증은 해당하지 않는다.

Docker CLI는 설치되어 있지만 `docker info`가 데몬 연결 실패를 반환했다.
WASM 검증에는 매뉴얼이 허용한 native wrapper `--no-opt`를 사용하며,
Docker 표준 빌드나 최적화 배포 산출물 검증으로 표시하지 않는다.

최종 혼합 변경의 frontend 검증은 Rust와 동일한 `b05cadb0e` review worktree에서
`npm ci` 후 해당 트리의 `pkg/`를 직접 생성하고 실행했다. 다른 checkout의 pkg를 복사하지
않았다. WASM SHA-256은
`7d35c463aa5dd890252f0f2cdc96d95af67e7fe6c2b48f797f602fcda21d8f07`이며 단계 5 바이너리와 같다.
Skia CLI 실행 파일의 SHA-256은
`6443efedecabb98ee1c0f346cd788b25769cc7f2a05f7efe872994cf5e50c29c`다.

주요 재현 명령(모든 Cargo 명령은 같은 review worktree에서 순차 실행):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
node scripts/run-rust-test.mjs issue_2724_passthrough_invalidation_guard -- --cargo-profile release-test --target-dir target/pr-review
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --no-fail-fast
cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs issue_6963_hyperlink_pdf -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt
```

같은 트리의 `rhwp-studio/`에서 `npm exec tsc -- --noEmit`, `npm test`, `npm run build`,
`node --experimental-transform-types --no-warnings tests/support/hyperlink-wasm.runner.mjs`를
순차 실행했다. 브라우저 재현은 단계 5의 명령에서 해당 review Vite URL을 사용한다.

## 전체 회귀에서 발견한 분류 누락

최초 전체 nextest는 9,401개 중 9,400개를 통과했고,
`issue_2724_passthrough_invalidation_guard::classification_drift_is_blocked` 1개가 실패했다.
새 `insert_hyperlink_native`, `update_hyperlink_native`, `remove_hyperlink_native`가 모두
`commit_hyperlink_paragraph`에 변경 적용을 위임하는데 가드의 분류 목록에는 없었다.
공통 helper에서 `section.raw_stream = None`, 구역 dirty·페이지 캐시 무효화를 실제로
수행함을 확인하고 세 API를 `Exempt::DelegatesTo("commit_hyperlink_paragraph")`로 등록했다.
검사 자체를 끄거나 Pending 상한·baseline을 늘리지 않았다. 가드가 위임 대상의 실재와
무효화 도달을 계속 검사하며, 실제 저장 왕복 검증도 그대로 유지한다.
보정은 `b05cadb0e`의 테스트 분류 18줄이며 제품 코드 변경은 없다.
보정 후 동일 가드 5개와 필수 Rust lint 묶음을 모두 통과했고, 전체 nextest를 다시 실행해
9,401개가 모두 통과했다. 최초 실패 실행을 통과 증적으로 대체하지 않고 두 결과를 구분했다.

## 편집 계약과 시각 검증

하이퍼링크 삽입·수정·해제는 `executeOperation()` snapshot을 통과한다. 무선택 삽입의
글자 추가와 필드 추가를 하나로 묶어 실패 시 함께 복원한다. 동일 주소·취소는 history를
추가하지 않고, 성공 시 기존 full refresh가 dirty·caret·페이지 캐시를 갱신한다.
외부 편집·문서 세대·읽기 전용 전환으로 모달 컨텍스트가 달라지면 적용을 차단한다.
선택·caret·undo/redo·저장 왕복은 단계 4의 실제 WASM runner와 브라우저에서 확인했다.

시각 범위는 주석이 실제 링크 글자에 대응하고 잉크를 바꾸지 않는지다.
단계 3의 CLI SVG/Skia PDF와 단계 5의 Chrome PDF를 독립 파서로 검사했다.
Chrome 5종 PDF의 81개 주석, 두 페이지 긴 링크, 한글 query/fragment와 뷰어 클릭을
확인했다. 인쇄 DOM 대비 최대 영역 오차는 0.381pt 미만, 링크 유무의 두 대표 PDF는
Poppler 144dpi에서 변경 픽셀 0이다. 한컴과의 기존 위치 차이는 그대로 보고한다.
최종 review worktree의 fresh WASM에서도 같은 전체 브라우저 여정과 독립 PDF 파서를
다시 실행해 통과했고, 새 문서·Textmail의 픽셀 비교를 다시 수행해 두 경우 모두 0을 확인했다.
최종 PDF 뷰어 스크린샷을 직접 열어 글자가 정상 표시됨을 확인했다.

PR 증적 정책에 맞춰 이 작업에서 만든 원시 JSON·DOM 기록·중간 contact sheet 7개를
ignored `output/issue6963-pr-preparation/intermediate/`로 옮겼다. 기존 단계 보고서의
수치·결론과 대표 링크 영역 비교·모달·PDF 뷰어 이미지는 보존했다. 테스트가 생성하는
원시 산출물은 재현 명령으로 다시 만들 수 있다.

## 제출 범위와 잔여 조건

HTTP/HTTPS 텍스트 링크 편집, HWP/HWPX 저장·재열기, Studio 및 CLI SVG/Skia PDF의
URI·영역 보존이 이번 범위다. 신규 mailto·파일·내부 책갈피 UI, 편집 화면에서 외부 URL로
이동, 회전·세로쓰기 링크 등 미지원 컨텍스트는 확장 범위로 구분한다.
bare `createEmpty()`의 최소 IR 저장 문제는 실제 Studio의 `createBlankDocument()` 경로와
분리한 잔여 제약이다. 한컴과 전체 조판이 일치한다고 주장하지 않는다.

push·PR 생성·merge는 아직 실행하지 않았다. 본문과 제목은 ignored
`output/issue6963-pr-preparation/pr-body.md`, `pr-title.txt`에 준비했다.
검증한 제품·테스트 후보는 `b05cadb0e`이며, 후속 커밋은 문서·중간 증적 정리만 포함한다.
원격 push·PR 생성의 승인을 받은 뒤 게시한다. 사용자 원래 checkout의 `samples/exam_eng.pdf` 변경은
그대로 보존한다.
