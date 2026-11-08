# PR #8348 리뷰 — 문서 조판·그림·PDF 처리와 성능 개선 통합

## 최종 판정

**승인 — 사용자 지정 기준의 로컬 검증과 정확한 head CI를 확인하고 머지했다.** 기존 미달과 새 MCP 출력 차이는 아래에 구분한다. 리뷰와 오늘할일은 사용자 지정 옵션 2의 별도 기록 PR로 반영한다.

## 접수와 통합 범위

- [PR #8348](https://github.com/edwardkim/rhwp/pull/8348), 작성자 `jangster77`, base `devel`.
- 검증 head `a35f3754ca8a72f1ff345a15bdf1caaa47301658`, 기준 `db5cd5bcc60a69e42f419f18a7f57a33b6fb7fdc`, merge `c76d87cfda742a4624e0a588aa09eb9d89b87d49`.
- planet6897의 조판·그림·PDF·성능 PR 20건을 체리픽해 통합했다. 편람 PR #8268과 #8161은 제외했다. 실제 순서·원 head와 보정은 [통합 계획](../../working/review_integration_20261026_document_updates_plan.md)과 원 PR별 리뷰에 기록한다.
- [#7723](https://github.com/edwardkim/rhwp/issues/7723)은 기본 이름과 바이트가 같은 2010 이름 원본이다. 사용자가 선택한 기존 한컴 2024 MCP PDF의 기존 래스터 기준 Native·fresh WASM 각 64쪽 raw 100%와 별칭의 내용·소유 회귀를 확인했다. 새 MCP 출력은 100%가 아니며 완료 결과로 대체하지 않는다. [Native](../../working/assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-native.json), [WASM](../../working/assets/review_integration_20261026/validation/post-rebase-8328/issue7723/selected-reference-latest-wasm.json), [별칭 검사](../../working/assets/review_integration_20261026/validation/post-rebase-8328/issue7723/existing-alias-regression-status.json). 현재 PDF 재래스터 기준 Native 세 쪽은 99.98~99.99%다.

## 검증과 남은 차이

- 글꼴 검색을 제한한 26개 변경 회귀 파일에서 110개 검사와 별도 후속 검사 24개가 통과했다. Rust fmt·Native/WASM/workspace Clippy·workspace 빌드와 suite 정책 검사도 통과했다. 전체 Rust 회귀는 사용자 지시에 따라 최종 head에서 재실행하지 않았다. [글꼴 격리 검사](../../working/assets/review_integration_20261026/validation/post-rebase-8328/fontless-branch-regression.json), [후속 검사](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-after-targeted-nextest.json), [lint·빌드](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr-local-rust-gates-20261029.json).
- Studio TypeScript와 관련 선택 검사 41개, 최신 WASM 브라우저 AI 검사 34개·PDF 배치 검사 6개, 글꼴 검색을 제한한 CanvasKit 선 검사를 통과했다. [Studio](../../working/assets/review_integration_20261026/validation/post-rebase-8328/studio-targeted-20261029.json), [브라우저](../../working/assets/review_integration_20261026/validation/post-rebase-8328/studio-fresh-browser-20261029.json).
- #8319 보존 대상 197개 입력·256쪽은 같은 기존 PDF 래스터에서 Native·WASM 모두 0.1%p 이상 하락이 없다. #8328 base와 후보에 공통으로 존재한 90% 미만 여섯 쪽은 통과로 표시하지 않는다. [Native](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-native-same-raster-comparison.json), [WASM](../../working/assets/review_integration_20261026/validation/post-rebase-8328/pr8184-incoming-wasm-same-raster-comparison.json), [기존 미달](../../working/assets/review_integration_20261026/validation/post-rebase-8328/upstream-existing-low-pages-audit.json).
- #8219의 사용자 지정 `hwp3-sample4.hwp` 1쪽은 글꼴 제약으로 90% 미만이며, 작업지시자는 추가 보정 없이 진행하도록 했다. base와 후보의 전쪽 PNG는 각 경로에서 같다. [개별 리뷰](pr_8219_review.md). #8227의 별도 편집 후 저장·재열기 실험은 원 PR의 편집 전 수용 결과와 구분하며 차이가 남는다. [실험](../assets/pr8227-editable-after-save/evidence.json).
- 새로 설치한 글꼴은 내부 검증 전용이다. 변경 파일과 변경된 HWPX·ZIP·EPUB에서 글꼴 확장자 파일을 발견하지 못했다. 이 검사는 PDF 글꼴 자원을 확인하지 않았다. [검사 범위](../../working/assets/review_integration_20261026/validation/post-rebase-8328/font-distribution-audit.json).

## CI와 보안 경고

정확한 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/40748531466)는 체크아웃 지연으로 취소된 C·D를 재실행한 2차에서 성공했다. [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/40748531214)도 체크아웃 지연으로 취소된 JavaScript 작업의 2차가 성공했다. [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/40748530375), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/40748530331), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/40748530650), [CI 영향 정책](https://github.com/edwardkim/rhwp/actions/runs/40748528970)은 성공했다.

[CodeQL 경고 #284](https://github.com/edwardkim/rhwp/security/code-scanning/284)는 기존 devel의 `src/model/native_font_program.rs`에서 DocInfo/header 바이트 일치 확인용 SHA-256에 테스트 암호 입력을 연결한 오탐으로 확인해 이유를 기록하고 `false positive`로 처리했다. 해당 파일의 통합 PR diff는 없다. 이후 PR의 필수 검사와 mergeability를 재확인하고 같은 head를 머지했다.

## 후속 처리

`upstream/devel`에서 merge SHA의 포함을 확인했다. #7723은 자동 종료되지 않아 수동 종료했고 #7445의 해당 행을 취소선과 #8348 링크로 갱신했다. 원 기여 PR의 후속 상태는 별도로 확인한다. 이 문서와 오늘할일을 별도 기록 PR로 반영하며 그 PR의 문서-only CI를 확인한다.
