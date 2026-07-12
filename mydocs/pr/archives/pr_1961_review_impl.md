# PR #1961 리뷰 구현 로그

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1961
- 제목: `Issue #1943: write_line 완전 재작성 — connectLine 변질 + 컴포넌트 블록/좌표 소실 수정 (최대 625px)`
- 작성자: `planet6897`
- base: `devel`
- head: `fix/1943-line-serialize`
- 검토 시점 head SHA: `6c846710e1365c5ba597ef14692676f951cd80f5`
- merge commit: `bb2df91c63de36606a0bd642cb56fcfe137a031c`
- mergedAt: `2026-07-05T15:13:42Z`
- 상태: merged, draft 아님, merge 전 `MERGEABLE` / `CLEAN`
- 변경 규모: 1 file, +172 / -35
- reviewer assign: `jangster77`
- `closingIssuesReferences`: 비어 있음. 본문 `closes #1943`가 있었지만 merge 직후 #1943은 open 상태로 남음.

## Stage 2. 로컬 fetch 및 merge 확인

완료.

```bash
git fetch upstream devel pull/1961/head:local/pr1961
git switch -C review/pr1961 local/pr1961
git merge upstream/devel --no-commit --no-ff
```

결과:

- `review/pr1961`에서 검토했다.
- `upstream/devel` 기준 `Already up to date`.
- 충돌 없음.

## Stage 3. 변경 내용 검토

완료.

확인한 변경 범위:

- `src/serializer/hwpx/shape.rs`

중점 확인:

- connector 보유 `LineShape`는 `<hp:connectLine>`으로 직렬화한다.
- `LinkLineType`은 OWPML `ConnectLineType.type` enum 문자열로 방출한다.
- 좌표는 parser가 실제로 읽는 `hp:startPt`/`hp:endPt` 자식 요소로 방출한다.
- connector의 `subjectIDRef`/`subjectIdx`, `controlPoints`/`point`를 보존한다.
- `ShapeComponentAttr` 기반 component block, line style, fill, shadow, `groupLevel`, `instid`를 보존한다.
- OWPML schema `ConnectLineType`과 parser `parse_shape_object` 경로를 함께 확인했다.
- 샘플명/페이지 번호/임의 계수 기반 하드코딩은 보이지 않는다.

## Stage 4. 실파일/시각 검증 범위 확인

완료.

대표 파일 검색:

```bash
rg --files /Users/tsjang/rhwp /Users/tsjang/hwpdocs /Users/tsjang/Cloud/Devel/hwpdocs 2>/dev/null | rg '2654625|3180687|1079960|21264499|156715276'
find /Users/tsjang/rhwp /Users/tsjang/Downloads /Users/tsjang/Documents /Users/tsjang/Cloud/Devel \
  -path '*/target' -prune -o -path '*/node_modules' -prune -o -path '*/.git' -prune -o \
  \( -iname '*2654625*' -o -iname '*3180687*' -o -iname '*1079960*' -o -iname '*21264499*' -o -iname '*156715276*' \) -print
find output -path '*/target' -prune -o \
  \( -iname '*2654625*' -o -iname '*3180687*' -o -iname '*1079960*' -o -iname '*21264499*' -o -iname '*156715276*' \) -print
```

결과:

- 현재 검토 환경에서 PR 본문/이슈 본문 대표 실파일을 찾지 못했다.
- 기준 PDF도 PR에 첨부되어 있지 않아 visual sweep은 수행하지 않았다.
- 작성자 PR 본문에 실파일 `render-diff --via hwpx` 결과가 기록되어 있으므로, 리뷰어 로컬은 serializer/parser/schema 계약과 테스트 게이트 중심으로 검증했다.

## Stage 5. 로컬 검증

완료.

검토 시작 시 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다. 이후 명령은 순차 실행했다.

```bash
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx::shape
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `cargo test --lib serializer::hwpx::shape`: 18 passed
- `cargo test --profile release-test --tests`: 통과
  - lib tests: 2126 passed, 6 ignored
  - `tests/svg_snapshot.rs`: 8 passed
  - command exit 0
- `cargo clippy --all-targets -- -D warnings`: 통과

## Stage 6. GitHub CI 확인

완료.

최신 head `6c846710e1365c5ba597ef14692676f951cd80f5` 기준:

- CI preflight: success
- CodeQL preflight: success
- Native Skia tests: success
- Build default-feature tests: success
- Build & Test: success
- CodeQL: success
- WASM Build: skipped

## Stage 7. 결론

merge 완료로 정리한다.

근거:

- 변경 범위가 HWPX line serializer로 제한되어 있다.
- parser/schema 계약에 맞는 구조 보존 수정이다.
- 신규 게이트가 PR 핵심 결손을 직접 확인한다.
- 로컬 전체 테스트와 clippy, GitHub CI가 통과했다.

옵션 2 후속 필요:

- 원 PR merge commit: `bb2df91c63de36606a0bd642cb56fcfe137a031c`
- review 문서와 오늘할일은 문서-only PR로 처리한다.
- merge 직후 #1943은 open 상태이므로 문서-only PR merge 후 후속 코멘트와 close 처리를 수행한다.
