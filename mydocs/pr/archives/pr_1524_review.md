# PR #1524 검토 — 문서 개요/조문 구조 추출 (`export-structure`)

- PR: https://github.com/edwardkim/rhwp/pull/1524
- 관련 이슈: #1523 `feat: 문서 개요/조문 구조 추출 명령 (export-structure)`
- 작성자: `planet6897` (Jaeuk Ryu)
- 작성일: 2026-06-25
- 처리 경로: collaborator-mediated 외부 PR
- base/head: `edwardkim/rhwp:devel` <- `planet6897/rhwp:pr-task1523`
- 문서 작성 시점 참고 head: `c65b467fedc3eaad56e6b87a34e6139afa724c0e`
- 규모: 4 files, +394 / -0
- label/milestone: `enhancement` / `v1.0.0`

`draft`, `mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 최종 merge 전 최신 상태를 다시 확인한다.

## 1. 요약 판단

**코드 변경은 수용 가능. 문서 커밋을 PR head에 포함하고, 새 head 기준 CI 통과 후 approve 및 merge 진행 권고.**

PR #1524는 파서, 렌더러, 레이아웃, 직렬화 경로를 변경하지 않고 `Document` IR을 읽어 문서 개요/조문 구조를
JSON 트리로 추출하는 조회 기능을 추가한다. 변경 범위는 `document_core::queries`의 신규 structure query,
CLI 서브커맨드, 매뉴얼에 한정된다.

1차 리뷰에서 발견한 CLI 계약 문제는 `c65b467f`에서 해소됐다. 최신 head 기준으로 help 문구, `-o` 값 누락
오류, 미지원 `--output` 거부 동작을 재확인했고, 로컬 검증과 GitHub Actions도 통과했다.

남는 위험은 주로 구조 추출 휴리스틱의 범위와 JSON/API 계약의 향후 호환성이다. 현재 PR 목적이 "읽기 전용
초기 구조 추출 질의"라는 점을 고려하면 blocker는 아니다.

## 2. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/document_core/queries/structure.rs` | `StructureMode`, `StructureDoc`, `StructureNode`, `build_structure()` 추가. `outline`/`clause`/`auto` 모드 지원 |
| `src/document_core/queries/mod.rs` | `structure` 모듈 공개 |
| `src/main.rs` | `export-structure` 서브커맨드와 CLI 옵션 처리 추가 |
| `mydocs/manual/cli_commands.md` | `export-structure` 매뉴얼 추가 |

커밋별 역할:

| 커밋 | 내용 |
|---|---|
| `2d9370f5` | 구조 추출 쿼리, CLI, 매뉴얼, 단위 테스트 추가 |
| `c65b467f` | `export-structure` help 블록 분리, `-o` 값 누락 오류 처리 추가 |

## 3. 관련 이슈와 메타 판단

이 PR은 #1523의 요구인 "문서 개요/조문 구조 추출 명령"을 직접 구현한다. 기능 추가 성격이므로 label은
`enhancement`, milestone은 `v1.0.0`이 적절하다. 코드 변경은 파서/렌더 품질 개선이라기보다 사용자와 분석
도구가 사용할 신규 조회 기능에 가깝다.

PR 본문과 첫 커밋 메시지에는 `closes #1523`가 있으나, 문서 작성 시점 GitHub `closingIssuesReferences`는
비어 있다. `devel` 대상 PR에서 자동 close가 실패할 수 있으므로 merge 후 #1523 상태 확인이 필요하다.

## 4. 1차 리뷰 지적사항 재검증

### 4.1 해결됨 - `rhwp --help` 옵션 블록 정합

기존 문제는 `export-structure` help가 `export-render-tree`의 옵션 블록과 붙어 있어 `--output`, `-p`,
`--show-para-marks` 같은 미지원 옵션을 안내하는 것이었다.

최신 head에서는 `src/main.rs:85`부터 `export-render-tree` 옵션 블록이 끝난 뒤, `src/main.rs:94`부터
`export-structure` 전용 블록이 분리되어 있다. 전용 안내는 다음 두 옵션만 포함한다.

- `--mode <방식>`
- `-o, --out <파일>`

로컬 `rhwp --help` 출력에서도 `export-structure` 아래에 `--mode`, `-o/--out`만 표시되는 것을 확인했다.

### 4.2 해결됨 - `-o` 값 누락 오류 처리

기존 문제는 `export-structure FILE -o`가 오류 없이 stdout JSON 출력으로 진행되는 것이었다.

최신 head에서는 `src/main.rs:657`부터 `-o | --out` 처리 시 다음 인자가 없으면
`오류: -o 뒤에 출력 파일 경로가 필요합니다.`를 출력하고 return한다. 로컬 smoke에서도 동일 메시지를
확인했다.

### 4.3 의도된 동작 - `--output` 미지원

`export-structure`는 파일 하나를 직접 쓰는 명령이고, 기존 `export-svg`/`export-render-tree`의 `--output`은
출력 폴더 의미다. 최신 help와 매뉴얼은 `-o/--out`으로 통일되어 있다. 따라서 `--output`이
`알 수 없는 옵션: --output`으로 거부되는 현재 동작은 문서와 구현이 일치한다.

다만 명령군 전체 UX 관점에서는 장기적으로 `-o`의 long option 명명 규칙을 통일할지 별도 논의할 수 있다.
이번 PR의 blocker는 아니다.

## 5. 코드 검토

### 5.1 구조 추출 모델

`StructureDoc`은 `mode`, `node_count`, `preamble`, `roots`를 제공한다. `StructureNode`는 `level`, `kind`,
`marker`, `heading`, `section`, `paragraph`, `body`, `children`을 제공한다. 비어 있는 `marker`, `body`,
`children`, `preamble`은 `serde(skip_serializing_if = ...)`로 생략된다.

이 JSON shape는 PR 설명과 매뉴얼의 `{mode, node_count, preamble, roots:[...]}` 계약과 맞다. 새 기능이 CLI로
노출되므로 field 이름과 생략 정책은 사실상 외부 계약으로 간주해야 한다.

### 5.2 outline 모드

`classify_outline()`은 `ParaShape.head_type`이 `Outline` 또는 `Number`이면 heading으로 분류하고,
`para_level + 1`을 JSON level로 내보낸다. `para_shape_id`가 범위를 벗어나면 `None`으로 처리한다.

범위를 벗어난 `para_shape_id`를 오류로 만들지 않는 것은 조회 명령에서 안전하지만, 깨진 문서를 진단하려는
사용자에게는 누락으로 보일 수 있다. 현재 PR 범위에서는 허용 가능한 선택이다.

### 5.3 clause 모드

`classify_clause()`는 문단 선두에서 다음 패턴을 인식한다.

- 원문자 `①`부터 `⑳`
- `제` + ASCII 숫자 + `편|장|절|관|조`
- ASCII 숫자 + `.`
- `가~하` + `.`

초기 법령형 문서 구조 추출에는 충분한 최소 휴리스틱이다. 다만 한자 숫자, `1)`, `가)`, 로마자, 영문 목록,
다단계 가지번호를 넓게 처리하지는 않는다. `제1조의2`는 heading으로는 검출되지만 marker는 `제1조`까지만
저장된다.

### 5.4 auto 모드

`has_outline()`은 문서 전체에서 `HeadType::Outline | HeadType::Number`가 하나라도 있으면 effective mode를
`outline`으로 선택한다. 이 정책은 문서가 한컴 개요/문단번호를 구조 제목으로 일관되게 사용하는 경우
효율적이다.

잔여 위험은 있다. 본문 중 일반 번호 문단이 `HeadType::Number`로 들어온 문서에서는 `auto`가 clause 검출을
포기하고 outline mode로 전체 문서를 처리할 수 있다. 이는 기능의 의미상 한계로 문서화 또는 후속 개선 대상이지,
읽기 전용 신규 명령의 merge blocker는 아니다.

### 5.5 트리 attach 로직

`build_structure()`는 heading을 만나면 stack top의 level이 새 heading level 이상인 동안 pop하여 부모에 붙이고,
그 뒤 새 heading을 push한다. 문서 순서 기반 outline tree를 구성하는 일반적인 방식이다.

비제목 문단은 첫 heading 이전이면 `preamble`, 이후면 현재 stack top의 `body`에 붙는다. "본문을 직전 제목에
귀속"한다는 PR 설명과 일치한다. 다만 하위 heading 이후에 다시 상위 heading으로 돌아가기 전까지의 본문은
가장 깊은 현재 heading에 붙으므로, 사용자가 기대하는 법령 본문 소속과 문서 패턴에 따라 다르게 느낄 수 있다.

### 5.6 CLI 처리

`export_structure()`는 `--mode`, `-o|--out`, positional file만 처리한다. 알 수 없는 option은 stderr 메시지 후
return한다. 출력 파일을 지정하면 JSON을 파일에 쓰고 완료 메시지를 stdout에 출력한다. 출력 파일을 지정하지
않으면 stdout에는 JSON만 출력한다.

주의할 점:

- positional file을 여러 개 넘기면 마지막 값이 사용된다.
- 오류 경로에서 process exit code는 별도 설정하지 않는다. 기존 단순 CLI 스타일과 크게 다르지 않지만,
  automation 사용자는 stderr 메시지를 확인해야 한다.

## 6. 로컬 검증

검증 worktree: `/private/tmp/rhwp-pr1524-latest`

| 항목 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo test --lib document_core::queries::structure -- --nocapture` | 통과, 3 passed |
| `cargo build --release` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --release --lib` | 통과, 1937 passed / 6 ignored |
| `rhwp --help` | `export-structure` 전용 옵션 블록 확인 |
| `export-structure samples/basic/KTX.hwp --mode clause -o ...` | 통과, JSON 파일 생성 |
| `export-structure samples/basic/KTX.hwp -o` | 명시 오류 확인 |
| `export-structure samples/basic/KTX.hwp --output ...` | `알 수 없는 옵션: --output` 확인 |

GitHub Actions는 문서 작성 시점 최신 PR head 기준으로 Build & Test, Canvas visual diff, CodeQL 계열이
성공했다. WASM Build는 workflow 조건상 skipped 상태다. 문서 커밋을 PR head에 push하면 새 head 기준 CI를 다시
기다려야 한다.

## 7. Side Effect 평가

- 파서, 렌더러, 레이아웃, 직렬화 경로는 변경하지 않는다.
- `build_structure()`는 `&Document`를 읽기만 하며 문서 상태를 mutation하지 않는다.
- 새 모듈은 `src/document_core/queries/mod.rs`에서 `pub mod structure`로 공개된다. crate API surface와 JSON
  output schema가 새 계약이 된다.
- stdout JSON 출력 경로와 file output 완료 메시지 경로는 분리되어 있다.
- `serde_json::to_string_pretty()` 실패 가능성은 낮고, 실패 시 명시 오류를 출력한다.
- 렌더 영향이 없으므로 golden snapshot 재생성은 필요하지 않다.
- 변경량은 신규 파일 중심이라 기존 기능 회귀 가능성은 낮다.

## 8. 잔여 위험 분류

### Non-blocking

- `auto` mode가 outline 하나만 발견해도 전체 문서를 outline으로 처리하는 정책.
- `HeadType::Number`를 heading으로 간주해 일반 번호 문단을 과검출할 가능성.
- `clause` 휴리스틱이 일부 법령/목록 표기 변형을 놓칠 가능성.
- `1.` 일반 번호 목록을 `호`로 과검출할 가능성.
- `제1조의2` marker가 `제1조`까지만 저장되는 제한.
- 여러 positional input을 넘겼을 때 마지막 파일만 사용하는 단순 parser 정책.
- 오류 시 exit code가 0일 수 있는 CLI 일관성 문제.

### Follow-up 후보

- 실제 법령 샘플 기반 `build_structure()` 통합 테스트 추가.
- `auto` mode의 outline confidence 개선.
- `--mode clause`에서 `제1조의2`, `1)`, `가)` 등 패턴 확장.
- JSON schema를 문서화하고 backward compatibility 규칙 정리.
- CLI 오류 경로의 exit code 정책 정리.

## 9. 권고

현재 코드 변경은 merge 수용 가능하다. 단, collaborator-mediated 외부 PR 처리 경로상 review 문서를 PR head에
포함하는 문서 커밋이 아직 필요하다.

권장 진행:

1. 이 review 문서와 실행 계획서를 PR head에 별도 문서 커밋으로 추가한다.
2. 문서 커밋 push 후 새 head 기준 GitHub Actions를 기다린다.
3. CI 통과 후 최신 head 기준 GitHub review `Approve`를 남긴다.
4. merge 전 `mergeable` / `mergeStateStatus`를 재확인한다.
5. `BEHIND` 상태 처리 방식과 최종 merge는 작업지시자 승인 후 수행한다.

## 10. merge 전 조건

- PR head 최신 커밋 기준 GitHub Actions 통과.
- review 문서가 PR diff에 포함됨.
- GitHub review 또는 PR comment로 검토 결과를 contributor에게 남김.
- merge 전 최신 `mergeable` / `mergeStateStatus` 재확인.
- 작업지시자 승인.

## 11. merge 후 확인

PR 본문과 커밋 메시지에는 `closes #1523`가 있지만 GitHub `closingIssuesReferences`는 비어 있다.
`devel` 대상 PR에서 자동 close가 실패할 수 있으므로, merge 후 #1523 상태를 확인하고 open이면 작업지시자
승인 후 수동 close한다.
