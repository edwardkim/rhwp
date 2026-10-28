# PR #6942 검토: Markdown 중첩 표 내용 보존

## 판정: 승인

- 대상: [PR #6942](https://github.com/edwardkim/rhwp/pull/6942), 작성자 `salgum1114`.
- 검증한 코드 head: `8f468c849e0b9fa3266830becb3b8a92323e5e99`.
- 검토일: 2026-09-09. 로컬 브랜치: `review/pr6942-markdown-20260909`.
- PR 본문의 중첩 표 Markdown 내보내기 범위에서 차단 결함을 발견하지 않았다. 메인터너 코드 보정은 불필요하다.
- 이 판정은 위 코드 head에 대한 검토 결과다. 실제 병합 및 후속 CI 결과는 아래에 별도로 기록한다.

## 변경 내용 검토

- `src/document_core/queries/rendering.rs`에서 셀 안의 하위 표를 재귀적으로 추출한다. 하위 표는 부모 표 뒤에 별도 Markdown 표로 출력한다.
- 부모 셀에는 하위 표 참조를 원래 컨트롤 위치에 삽입하고, 하위 표에는 부모 표 번호와 셀의 행·열을 표시한다.
- 중첩 표가 없는 셀은 기존 텍스트·수식 추출 경로를 유지한다. 표 내부 이미지 수집 경로도 유지한다.
- CLI `export-markdown`은 해당 공통 추출 함수를 호출한다. MCP `hwp_export_markdown`은 같은 CLI에 위임하며, 이번에는 실제 MCP 호출도 확인했다.
- 기존 테스트 `tests/cases/markdown_nested_tables.rs`의 합성 HWPX를 검증 입력으로 사용했다. 비공개 문서를 새로 수집하거나 기준 PDF를 생성하지 않았다.

## 실제 집중 검증 결과

macOS에서 위 head를 전용 `CARGO_TARGET_DIR=target/pr6942-markdown-20260909`로 빌드했다. 공유 target은 변경하거나 제거하지 않았다.

```sh
CARGO_TARGET_DIR=target/pr6942-markdown-20260909 cargo build -p rhwp --bin rhwp -j 8
target/pr6942-markdown-20260909/debug/rhwp export-markdown /tmp/pr6942-markdown-20260909/nested.hwpx -o /tmp/pr6942-markdown-20260909/cli --json
target/pr6942-markdown-20260909/debug/rhwp mcp-serve
```

- 빌드: 성공, 1분 46초. 마지막 명령은 stdio JSON-RPC 초기화 후 `hwp_export_markdown`을 호출했으며, 같은 입력과 별도 출력 폴더를 사용했다.
- CLI: 종료 코드 0, 1페이지 중 1페이지 출력. MCP: `isError=false`, 1페이지 출력.
- 두 경로의 Markdown 파일은 바이트 단위로 동일했다: 268바이트, SHA-256 `a545f9d4e593b8e05e7bc33210adf6dc87bde2b1b0847604c5f68fd395ba21b7`.
- `BEFORE`, `AFTER`, `INNER A/B/C/D`, `DEEP VALUE`, `OUTER RIGHT`는 각각 정확히 한 번 나타났다.
- 부모 셀의 `BEFORE [하위 표 1-1 참조] AFTER`, 2단계 참조 `1-1-1`, 부모 셀의 1행 1열 표시를 확인했다.
- HTML 표 태그 없이 Markdown 표로 출력됨을 확인했다. 실제 출력 중 핵심 부분은 아래와 같다.

```markdown
| BEFORE [하위 표 1-1 참조] AFTER | OUTER RIGHT |
| --- | --- |

**하위 표 1-1 — 표 1의 1행 1열**

| INNER A [하위 표 1-1-1 참조] | INNER B |
| --- | --- |
| INNER C | INNER D |

**하위 표 1-1-1 — 표 1-1의 1행 1열**

| DEEP VALUE |
| --- |
```

## CI와 생략한 검증

- [CI](https://github.com/edwardkim/rhwp/actions/runs/34325016284): lint, Native Skia, Archive A/B/C/D 및 Build & Test 성공. Archive C는 같은 head의 재실행에서 성공했다. 최초 실패 원인을 이번 검토에서 확정하지 않았다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34325016225), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34325016316), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34325016305), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34325015958): 성공.
- CI Impact Policy 성공과 `mergeStateStatus=CLEAN`을 확인했다.
- 사용자 지시에 따라 CI와 중복되는 전체 회귀 테스트, 기존 집중 테스트, lint 및 WASM 검사를 로컬에서 반복하지 않았다. 이번 로컬 검증은 실제 CLI/MCP 진입점의 동작 확인이다.
- 한컴 PDF와 페이지 외관 대조, 실문서 코퍼스 전체, 중첩 깊이·성능 한계는 이번에 검증하지 않았다. 렌더링 외관 개선이나 전체 코퍼스 보장을 주장하지 않는다.
- 임시 HWPX·Markdown·로그는 커밋에 포함하지 않는다. 재현 입력은 기존 테스트에 포함되어 있고, 댓글에 필요한 텍스트 증적은 이 문서에 기록했다.

## 병합 및 옵션 2 문서 기록

- 사용자 승인으로 원 PR을 일반 merge commit `9bd4ac23d6901b98811c4934ef5c619b8e977f98`에 병합했다. 병합 시각: 2026-09-09 11:51:25 UTC.
- 병합 직전 코드 head가 유지되고 `MERGEABLE/CLEAN`이며 모든 필수 검사가 성공한 것을 확인했다. 로컬 devel은 해당 upstream/devel로 fast-forward했다.
- 원 PR은 `maintainerCanModify=false`였으므로 원 fork에 문서 push를 시도하지 않았다. 사용자 승인으로 병합 후 최신 devel 기반의 `docs/pr6942-review-20260909`에서 이 리뷰와 오늘할일만 별도 PR로 제출한다.
- 병합 후 [CI](https://github.com/edwardkim/rhwp/actions/runs/34347721972): lint, Native Skia, Archive A/B/C/D builder 및 실행 worker, Build & Test, duration refresh 성공. 독립 WASM/Frontend/promotion 작업은 skip이었다.
- 병합 후 [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34347722814): Rust·JavaScript/TypeScript·Python 분석 성공.
- 병합 후 [Adapter](https://github.com/edwardkim/rhwp/actions/runs/34347721911), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/34347721931), [Close Issues](https://github.com/edwardkim/rhwp/actions/runs/34347721929): 성공. merge SHA의 push run 목록에 별도 Render Diff run은 없었다. PR 단계의 Render Diff 성공과 구분한다.
- [오늘할일](../../orders/20260909.md)은 최신 devel의 기존 항목을 보존하고 이번 항목만 추가했다. 이 문서 PR 자체의 CI·병합·후속 CI는 생성 이후 확인할 별도 gate다.

## 후속 댓글 및 정리 계획

- 문서 PR 병합과 해당 merge SHA의 devel CI 성공 후 `post_merge.md`에 따라 원 PR에 처리 댓글을 한 번 게시한다. 기존 처리 댓글이 있으면 수정하고 중복 등록하지 않는다.
- 작성자는 GitHub의 `FIRST_TIME_CONTRIBUTOR` 표시가 있으나 [PR #6933](https://github.com/edwardkim/rhwp/pull/6933)의 병합 이력이 있다. 첫 기여자 안내의 환영·감사·구체적인 검증 설명을 준수하되, 이 PR이 첫 번째 병합이라고 단정하지 않는다.
- 기여자에게 감사하고, 다음 PR부터 `Allow edits by maintainers`를 켜서 `maintainerCanModify=true`가 되도록 요청한다. 메인터너가 승인된 보정과 리뷰 기록을 원 PR에 추가할 수 있도록 하기 위한 안내이며, 이번 기여의 수용을 취소하는 조건은 아니다.
- 댓글에는 실제 원 PR merge SHA, 실제 PR/devel CI 결과, CLI/MCP 출력 일치와 위 Markdown 핵심 발췌를 직접 표시한다. 문서 병합 SHA에 고정한 리뷰 링크도 포함한다. PNG/PDF 시각 검증을 수행했다고 표현하지 않는다.
- closing issue references가 비어 있음을 API로 확인했다. 후속 처리 시 다시 확인하며 관련 이슈를 추정하여 닫지 않는다.
- 최종 devel 동기화·clean 및 실행 중 Cargo/Rust 작업 부재를 확인한 뒤 이번 로컬 review/docs 브랜치와 문서 PR의 upstream 임시 브랜치, 전용 target만 정리한다. 기여자 fork 브랜치·기본 작업공간·공유 target 및 다른 작업 worktree는 보존한다.
