# PR #7988 검토 — Unicode 검색·치환의 원문 문자 범위

## 최종 판정

**승인 — 원 기여와 정식 회귀를 반영한 통합 head의 로컬 검증과 실제 code CI 완료.**
code CI head는 `fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`이며 로컬 제품·테스트 producer와의 차이는 검토 문서뿐이다.
원 contributor head의 직접 merge 승인과 구분한다. 사용자 병합·후속 처리 승인은 받았고
문서 trailing head의 최신 CI·mergeability가 남은 병합 조건이다.

## 접수 정보

- 원 PR: https://github.com/edwardkim/rhwp/pull/7988 · 작성자 `donggyun112`(기존 기여자).
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/7984.
- 원 head `a7c884a79efed47ad59d0fbf78227cc5dbe46624` → 마지막 적용 `d3a8bddda06344b64dffe4fb130fbdec5a761bb3`; 6개 기능·검사 commit의 author와 원 SHA 연결 보존.
- reviewer `jangster77` 선행 요청 및 API 확인.
- [적용 기록](pr_7988_review_impl.md) · [계획](../../plans/pr_review_donggyun112_20261017.md).

## 구현 주장과 실제 소비 경로

원문의 `İX`를 `i\u0307`로, `i\u0307X`를 `İ`로 검색할 때 원문 매치 길이는 각각 1·2다.
치환 결과의 독립 기대값은 모두 `ZX`이며 후속 X를 보존해야 한다.
기존 소비자는 검색어의 문자 수를 삭제 길이로 사용해 각각 X를 지우거나 결합 점을 남겼다.

`find_in_text/find_matches`가 소문자 버퍼 각 문자를 원문 scalar 인덱스에 연결하고 첫 인덱스와
마지막 인덱스+1에서 `(offset,length)`를 반환한다. `search_first_body`와 본문·셀·글상자·수식의
`push_container_hits`는 이 공통 결과를 `SearchHit`에 전달한다. grep의 본문·셀·글상자·중첩 표·수식
각 분기도 동일 tuple을 `GrepMatch.length`와 `make_context`에 전달한다. 기존 replace one/all/nth가
이 원문 길이로 삭제·치환한다. 공개 JSON 키와 `GrepMatch.length`의 public 가시성을 유지한다.

Rust의 문자별 `to_lowercase` 계약이며 완전한 언어별 case folding·정규화 지원을 새로 추가하지 않는다.
확장 일부 매치·기존 겹치는 매치 정책은 이번 변경 범위 밖이다.

## 반례와 정식 회귀

`tests/cases/issue_7984_casefold_source_span.rs`의 정식 4개 검사는 본문 one/all/nth,
원문 검색·grep 길이, 두 번째 매치와 대소문자 구분 대조군, 셀·글상자·수식의 suffix를 검사한다.
base의 검색 두 파일만 source overlay에서 복원해 실제 컴파일한 음성 대조는 4건 모두 assertion FAIL,
수정 후는 4 PASS다. 실패 지점에서 각 검사 함수가 중단되므로 모든 내부 조합의 수정 전 실행을 주장하지 않는다.
primary source와 공유 target은 보존했으며 별도 overlay wrapper만 음성 실행에 적용했다.

메인터너 보강 `2dee7730306dcc7f57615c87c25cb09af3c90f63`는 기존 `undo-contracts.test.mjs`에 양방향 Unicode 및 ASCII 대조군의
실제 Ctrl+F2 모두 바꾸기·Ctrl+Z·Ctrl+Shift+Z를 추가한다. 수정 전 실제 브라우저에서도
`Z`/`Z\u0307X` 및 redo의 같은 오류 등 의도한 4개 assertion FAIL을 확인했다.
최종 fresh WASM에서 API 검색·one/all·HWP 저장 재열기 6조합과 실제 UI 3조합이 PASS다.
WASM에 없는 nth API를 실행한 것으로 보고하지 않으며 nth는 Native 정식 회귀로 보호한다.

## 공통 로컬 검증

최종 제품·테스트 검증 head는 `2dee7730306dcc7f57615c87c25cb09af3c90f63`, 최신 기준은 `7d01ef610f378157b79d36ef23e8ed4a3c8ab051`다.
기존 producer `92e952295a161cb153f562d3e988e5da629f2f3e`에서 완료한 focused·lint·workspace 검사는 리베이스 후에도
`src/tests/crates/vendor/bindings/tools/scripts/Cargo.toml/Cargo.lock/build.rs`의 Git object가 같음을
확인했다. 전체 회귀는 사용자 리베이스 요청으로 컴파일 중 중단한 실행을 완료로 계산하지 않고,
리베이스 후 다시 실행했다. 새 기준 manifest·unit-tier와 frontend도 재확인했다.

| 검사 | 실제 결과 |
| --- | --- |
| 신규 Native 검색 회귀 | 수정 전 4건 assertion FAIL / 수정 후 4 PASS |
| Rust lint 묶음 | fmt, Native clippy, WASM clippy, workspace build·all-targets clippy 모두 exit 0 |
| 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast` | Summary [ 751.818s] 10820 tests run: 10820 passed (11 slow), 45 skipped |
| Rust manifest·unit-tier | 최신 base 고정, 모두 exit 0 |
| TypeScript | `npx --prefix rhwp-studio tsc --noEmit -p rhwp-studio/tsconfig.json`, exit 0 |
| 전체 Studio 단위 | 2,320건 중 2,318 PASS / 0 FAIL / 2 SKIP |
| E2E manifest | 파일 158 / 행 158, 누락 0 |
| production build | `npm --prefix rhwp-studio run build`, exit 0 |
| fresh WASM | 공용 target, jobs 8, codegen-units 16, wrapper `--target web --out-dir pkg --no-opt`, exit 0 |
| 실제 Chrome 검색·치환 | API 6조합의 원문 길이·one/all·HWP 저장/재열기, 실제 UI 3조합 replace-all·undo/redo PASS |
| 실제 Chrome 교차 구역 | 8개 입력 진입점, 비어 있지 않은 undo/redo, 빈 누름틀, 정상 대조군 PASS |
| 실제 Chrome undo | 기존 계약 + HF/FN 9사례 + Unicode 치환 3사례 PASS |

Chrome for Testing 154.0.8037.57, Linux x86_64, Node v24.15.0, npm 11.12.1, Rust 1.93.1에서 실행했다.
기존 SKIP 2건은 `pkg-node` 전용 혼합 모양/서식 왕복 검사이며 신규 검사는 실행했다.
실제 OS 클립보드·마우스 드래그·IME 후보창은 미검증이다. DOM clipboard/composition과 실제 키보드를 구분한다.
Docker 최적화 배포 검증이 아닌 host release `--no-opt` fresh WASM UI 검증이다.
브라우저 실행 시 `pkg/`와 Studio public의 JS/WASM 바이트가 같으며 이전 검색 수정 전 WASM과는 다르다.

| 검증 산출물 | SHA-256 | 바이트 |
| --- | --- | ---: |
| JS | `7dcd8775b5ff2e6d6d46f6811d03cd3e0f0db9d7d0f0d1a0606741e762bce06e` | 475795 |
| WASM | `3daf35122ef497f05e4927a85ee449a564d7f615b0eccfdffbf6c838f0674229` | 22343456 |

원장은 ignored `output/pr-review/donggyun112-20261017/`의 `rebased-rust-gates.json`,
`three-pr-rust-gates.json`, `three-pr-frontend-early-gates.json`, `three-pr-browser-gates.json`,
`three-pr-fresh-wasm-identity.json`, `rebase-input-equivalence.json`과 `logs/`다.
공유 `target/pr-review`는 보존하며 검증용 생성 glue는 제출 코드에 섞지 않는다.

## 조판 원칙과 검증 범위

| 항목 | 판정과 근거 |
| --- | --- |
| 일반성·공통 결과 | 충족: 문서 ID 예외 없이 원문 범위를 검색·grep·치환이 공유 |
| 실제 소비 경로·반례 | 충족: 위 호출 경로, 양방향 길이 확장과 ASCII 대조군, 수정 전 FAIL/후 PASS |
| 측정·배치·분할·이어받기 | 비해당: 줄 메트릭·좌표·높이·pagination·paint 변경 없음 |
| 저장 조판 정보·serializer | 비해당: 저장 정보 수용 조건·속성 변경 없음; 저장 재열기는 내용 보존 검사 |
| Visual Sweep·픽셀 회귀 선행 조건 | 비해당: 검색 범위와 편집 내용 관계 검사이며 렌더링 위치·형태 회귀가 아님 |
| 기준값 변경 | 비해당: golden·baseline·허용치 완화 없음 |
| 독립 검증 | 충족: 명시한 Unicode 원문 범위와 suffix 보존; 실제 WASM과 단위 대역 구분 |

수식 script도 검색·치환의 텍스트 보존만 검사했다. 수식 모양이나 한컴 피델리티 개선을 주장하지 않으며
Native/Print Visual Sweep·수식 확대 판독을 수행한 것으로 표시하지 않는다.
합성 입력은 정식 테스트 코드가 직접 생성하며 외부 PDF를 정상 출력 증거로 대용하지 않는다.

## Merge 후 comment 계획

통합 PR의 실제 merge SHA·최종 CI·개별 기록을 확인한 뒤 원 기여와 메인터너 검사 보강을 구분한
한국어 존댓말 comment에 통합 PR·기록·CI 링크를 남기고 원 PR을 close한다. fork는 보존한다.

## 통합 PR code CI와 남은 후속 처리

[통합 PR #7997](https://github.com/edwardkim/rhwp/pull/7997)의 정확한 code head
`fd3d97b0b9effa2d0d4a1f65f47e3475ba4ec5c8`에서 [CI](https://github.com/edwardkim/rhwp/actions/runs/39041517840) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39041517836) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39041517829) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/39041517834) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/39041517854) · [Controller](https://github.com/edwardkim/rhwp/actions/runs/39042954892)를 확인했다.
집계 Build & Test·언어별 CodeQL·Canvas·Adapter·Proptest와 정책 status가 완료됐고
실패·진행 중 check는 없다. GHAS CodeQL의 neutral과 영향 정책상 skip을 실제 성공 worker와 구분한다.
PR 생성 전 source PR CI를 재사용한 결과가 아닌 이 통합 PR의 실제 실행이다.

사용자는 CI 모니터링 이후 병합·후속 처리를 승인했다. 이 기록은 동일 PR의 문서-only trailing이며
제품·테스트·workflow·fixture·baseline은 변경하지 않는다. 최신 base와 merge tree의 문서 링크·
오늘할일 보존을 확인하고 push한 뒤, trailing head의 실제 fast-pass·required aggregate·mergeability를
다시 확인한다. 아직 실행 전인 trailing CI·merge·이슈 종료·정리는 완료로 표시하지 않는다.

원본 로컬 영수증·로그의 종료 후 보존 경로는 `/home/tsjang/rhwp-pr-evidence/pr-7997/`다.
저장소의 ignored `output/pr-review/donggyun112-20261017/`는 실행 당시 경로이며,
최종 보존 파일을 해시 확인한 뒤 `post_merge.md`에 따라 root `output/` 전체를 정리한다.
공용 `target/pr-review`와 contributor fork는 보존한다.
