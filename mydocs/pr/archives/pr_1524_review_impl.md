# PR #1524 검토 실행 계획서

- PR: https://github.com/edwardkim/rhwp/pull/1524
- 관련 이슈: #1523 `feat: 문서 개요/조문 구조 추출 명령 (export-structure)`
- 작성일: 2026-06-25
- 처리 경로: collaborator-mediated 외부 PR 처리 경로
- base/head: `edwardkim/rhwp:devel` <- `planet6897/rhwp:pr-task1523`
- 문서 작성 시점 참고 head: `c65b467fedc3eaad56e6b87a34e6139afa724c0e`
- 문서 작성 범위: 최신 PR head 기준 review 계획 및 검토 문서 작성

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 merge 전 최신 상태를 다시 확인한다.

## 1. 적용 경로 판단

PR #1524는 외부 contributor의 fork PR이며 `maintainerCanModify=true`이다. repository collaborator가 리뷰,
문서화, merge 준비를 담당하므로 `mydocs/manual/pr_review_workflow.md` 9장의 collaborator-mediated 외부 PR
처리 경로를 적용한다.

이 경로에서는 review 문서를 해당 PR head에 직접 포함할 수 있다. contributor의 원 코드 커밋은 rewrite하지
않고, 검토 기록은 별도 문서 커밋으로 분리한다.

이번 요청 범위에서는 오늘할일(`mydocs/orders/{yyyymmdd}.md`)을 갱신하지 않는다.

## 2. 현재까지의 사실 관계

| 항목 | 내용 |
|---|---|
| PR 제목 | `Task #1523: 문서 개요/조문 구조 추출 (export-structure)` |
| 작성자 | `planet6897` |
| 관련 이슈 | #1523 |
| label / milestone | `enhancement` / `v1.0.0` |
| 변경 규모 | 4 files, +394 / -0 |
| 커밋 수 | 2 commits |
| 문서 작성 시점 참고 mergeability | `MERGEABLE`, `BEHIND` |
| 문서 작성 시점 참고 CI | Build & Test, Canvas visual diff, CodeQL 계열 통과; WASM Build skipped |
| 자동 close 참조 | GitHub `closingIssuesReferences`는 비어 있음 |

커밋 구성:

1. `2d9370f5` - `Task #1523: 문서 개요/조문 구조 추출 (export-structure)`
2. `c65b467f` - `Task #1523: export-structure CLI 계약 정정 (help 정합 + -o 값 누락 오류)`

두 번째 커밋은 1차 리뷰에서 지적한 CLI help 정합성과 `-o` 값 누락 처리 문제를 정정한다.

## 3. 단계 계획

### Stage 1 - 메타 및 범위 재확인

- PR base가 `devel`인지 확인한다.
- head branch와 head SHA를 확인한다.
- `maintainerCanModify=true`인지 확인한다.
- label/milestone이 이슈 #1523과 일치하는지 확인한다.
- 변경 파일이 `document_core` 쿼리, CLI, 매뉴얼 범위를 벗어나지 않는지 확인한다.
- `closingIssuesReferences`가 비어 있으므로 merge 후 이슈 상태 확인 필요성을 기록한다.

### Stage 2 - 1차 리뷰 지적사항 재검증

1차 리뷰에서 발견한 문제:

- `rhwp --help`에서 `export-structure` 아래에 실제 없는 옵션이 노출됨.
- `-o` 뒤 출력 경로가 빠져도 오류 없이 stdout JSON 출력으로 진행됨.

재검증 항목:

- `src/main.rs` help 출력에서 `export-render-tree`와 `export-structure` 옵션 블록이 분리됐는지 확인한다.
- `export-structure` help가 `--mode`, `-o/--out`만 안내하는지 확인한다.
- `export-structure FILE -o`가 명시 오류를 출력하는지 확인한다.
- `export-structure FILE --output path`가 문서화되지 않은 옵션으로 거부되는지 확인한다.
- `mydocs/manual/cli_commands.md`가 실제 구현과 같은 `-o/--out` 계약을 설명하는지 확인한다.

### Stage 3 - 코드 구조 검토

- `StructureMode::parse()`가 허용 mode를 `auto|outline|clause`로 제한하는지 확인한다.
- `StructureDoc` / `StructureNode` JSON shape가 PR 설명과 일치하는지 확인한다.
- `outline` 모드가 `ParaShape.head_type`과 `para_level`을 이용하는지 확인한다.
- `clause` 모드가 조문형 heading 휴리스틱만 검사하는지 확인한다.
- `auto` 모드가 outline 존재 여부로 effective mode를 결정하는 정책을 검토한다.
- stack 기반 attach 로직이 같은 레벨/상위 레벨 전환 시 부모-자식 관계를 보존하는지 확인한다.
- 비제목 문단이 첫 heading 이전에는 `preamble`, 이후에는 직전 heading의 `body`에 들어가는지 확인한다.
- 함수가 `&Document`만 읽고 문서 모델을 mutation하지 않는지 확인한다.

### Stage 4 - 로컬 검증

실행한 검증:

```bash
cargo fmt --check
git diff --check upstream/devel...HEAD
cargo test --lib document_core::queries::structure -- --nocapture
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test --release --lib
```

CLI smoke 검증:

```bash
rhwp --help
rhwp export-structure samples/basic/KTX.hwp --mode clause -o /private/tmp/rhwp-pr1524-latest-ktx.json
rhwp export-structure samples/basic/KTX.hwp -o
rhwp export-structure samples/basic/KTX.hwp --output /private/tmp/rhwp-pr1524-latest-output.json
```

렌더러, 레이아웃, 파서, 직렬화 경로를 변경하지 않는 읽기 전용 질의이므로 `svg_snapshot`은 필수 검증에서
제외한다. GitHub Actions의 Canvas visual diff가 통과했는지는 최신 head 기준으로 별도 확인한다.

### Stage 5 - 잔여 위험 및 side effect 평가

다음 항목을 review 문서에 명시한다.

- `auto` 모드는 outline이 하나라도 있으면 전체 문서를 outline mode로 처리한다.
- `HeadType::Number`를 outline heading으로 포함하므로 일반 번호 문단을 제목으로 과검출할 수 있다.
- `clause` 모드는 법령형 텍스트 선두 패턴 휴리스틱이며, 한자 숫자/괄호형 번호/영문 목록 등은 제한적이다.
- `제1조의2` 같은 가지번호는 heading으로는 잡히지만 marker는 `제1조`까지만 저장된다.
- `1.` 일반 번호 목록은 `호`로 과검출될 수 있다.
- 여러 positional file argument를 넘기면 마지막 값을 사용한다.
- 오류 경로는 stderr 메시지 후 함수 return이며, process exit code는 기존 단순 CLI 스타일과 같이 0일 수 있다.
- `pub mod structure`와 JSON field는 신규 외부 계약이 될 수 있다.
- stdout mode에서는 JSON만 출력하고, file output mode에서만 완료 메시지를 출력한다.

위 항목은 현재 PR의 merge blocker가 아니라 후속 개선 또는 사용 문서상 한계 고지 항목으로 분류한다.

### Stage 6 - 문서 커밋 및 GitHub review 순서

권장 순서:

1. 최신 head 기준 review 문서 작성.
2. review 문서만 별도 커밋으로 묶기.
3. contributor fork의 PR head branch에 문서 커밋 push.
4. 새 head 기준 GitHub Actions 완료 대기.
5. 새 head 기준 검토 결과를 GitHub review `Approve`로 남기기.
6. merge 전 최신 `mergeable` / `mergeStateStatus` 재확인.
7. 작업지시자 승인 후 merge.

문서 커밋 전 또는 문서 커밋 push 전 approval을 남기면 approval이 이전 head에 걸리므로 피한다.

## 4. merge 전 최종 조건

- PR head 최신 커밋 기준 GitHub Actions 통과.
- review 문서가 PR diff에 포함됨.
- GitHub review 또는 PR comment로 검토 결과를 contributor에게 남김.
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인.
- `BEHIND` 상태를 update branch로 해소할지, admin merge로 처리할지 작업지시자 승인.
- 작업지시자 승인.

## 5. merge 후 확인 항목

- PR 본문과 첫 커밋 메시지에는 `closes #1523`가 있으나, GitHub `closingIssuesReferences`는 비어 있다.
- merge 후 #1523 상태를 확인한다.
- #1523이 open이면 작업지시자 승인 후 수동 close한다.
- contributor 감사 코멘트와 검증 요약을 남긴다.
- 필요 시 collaborator-mediated PR 처리 기록만 정리하고 별도 사후 문서 PR은 만들지 않는다.
