# PR #3132 검토 기록 — DocLang v0.6 내보내기

## 1. 메타

| 항목 | 값 |
| --- | --- |
| PR | [#3132](https://github.com/edwardkim/rhwp/pull/3132) |
| 작성자 | `myeolinmalchi` |
| base | `devel` |
| head | `myeolinmalchi:feat/export-doclang` |
| 관련 이슈 | [#3131](https://github.com/edwardkim/rhwp/issues/3131) |
| 규모 | 36 files, +9,548 / -0 lines |
| 성격 | 외부 contributor 기능 PR — HWP 5.0/HWPX → DocLang v0.6 XML 라이브러리·CLI 추가 |
| 최초 검증 기준 | `upstream/devel@9c387b83464172dc36a07796038b11dfb3831e3f` + PR head `46d9c27050707cf8a28129403c7a86961a827d08` |
| 보정 기준 | PR head `587e786d670c201ceb63854e378d7c3e6ba0eebf` + collaborator 보정 `0afa775b3` |
| 작성일 | 2026-07-23 |

`draft`, `mergeable`, head SHA와 CI 상태는 변동값이다. 최초 기록 시 PR은 OPEN, non-draft,
`MERGEABLE`, `mergeStateStatus: BLOCKED`였고 `jangster77`을 reviewer로 지정했다. 이 archive 기록을
PR head에 추가한 뒤에도 최종 merge 조건은 최신 PR head의 required checks 통과와 작업지시자 승인이다.

## 2. 목적과 변경 범위

#3131의 제안대로 별도 `hangulang` 크레이트의 HWP/HWPX → DocLang XML 변환기를 rhwp 내부에 이식한다.

- `src/doclang/`에 rhwp 모델을 Semantic IR로 낮추는 adapter, EqEdit→LaTeX pass, DocLang XML writer,
  loss report, 그림 resource policy를 추가한다.
- 공개 API로 `convert`, `ConvertOptions`, `ConvertOutcome`, `ConvertError`, `LossReport`,
  `ResourcePolicy`를 노출한다.
- `rhwp export-doclang <file.hwp|file.hwpx> [-o output.xml] [--assets-dir dir]` CLI와 매뉴얼을
  추가한다.
- 기존 renderer, layout, 저장 포맷, Studio 경로와 golden/fixture는 바꾸지 않는다.

9,548줄의 신규 모듈을 추가하는 대형 PR이므로 소형 PR의 즉시 merge 경로에는 해당하지 않는다.

## 3. 최신 devel 정렬과 merge simulation

최초 fetch·검증 뒤 PR head와 base가 모두 전진했다.

- 기존 head `46d9c270`에서 head `587e786d`로의 추가 커밋은 `Merge branch 'devel' into
  feat/export-doclang` 하나다.
- 이 merge가 가져온 `devel` 변경은 doccore table/shape 편집, HWP3 johab, serializer와 관련 보고서이며
  DocLang PR의 원 기능 파일은 변경하지 않는다.
- 최신 `upstream/devel@eb76c827` 위에 PR head를 `--no-commit --no-ff`로 합친 결과 충돌이 없었고
  `git diff --cached --check`도 통과했다.

## 4. 사전 검증과 보정 후 재검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | PASS |
| `cargo test --test doclang_export` | PASS — 원 PR 기준 HWP 문단·HWP 표 OTSL·HWPX 문단 3건, 보정 후 symlink/hard link 2건을 더해 5건 |
| `cargo test --profile release-test --tests --no-fail-fast` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo check --target wasm32-unknown-unknown --lib` | PASS |
| `cargo test --doc` | 원 PR head에서 EqEdit 예제 3건 FAIL 확인 후 보정 tree에서 PASS (4 passed, 2 ignored) |
| `cargo build --release` | PASS — 최초 검증 기준 |
| `git diff --check` | PASS |

보정 tree `review/pr3132-maintainer-20260723`에서도 위의 DocLang CLI·release-test·Clippy·WASM·doctest와
`git diff --check`를 다시 통과시켰다. 모든 Cargo 명령에는 `CARGO_INCREMENTAL=0`과 해당 worktree 전용
`CARGO_TARGET_DIR`를 사용했다.

CLI smoke도 임시 디렉터리에서 확인했다.

- `samples/para-001.hwp`는 `<doclang version="0.6">` XML로 변환되고 알려진 한글·한자 텍스트가 보존됐다.
- `samples/test-image.hwp --assets-dir <temp>/assets`는 `<picture>` 5개와 추출 asset 5개를 만들었다.
- renderer/layout/Studio Canvas 출력과 sample/golden을 변경하지 않으므로 visual sweep 대상이 아니다.

## 5. 발견된 차단 문제와 메인터너 보정

### 5.1 출력 경로가 원본을 덮어쓸 수 있음

`export-doclang`은 원본 보호를 표방하지만 `src/main.rs`에서 `output_path == input_path`만 비교했다.
따라서 output이 input을 가리키는 symlink인 경우 서로 다른 `Path`로 판단한 뒤 `fs::write`가 symlink를 따라
원본 HWP를 XML로 덮어쓴다.

격리된 임시 사본에서 다음을 재현했다.

```text
input.hwp  ← samples/para-001.hwp 복사본
output.xml → input.hwp (symlink)
rhwp export-doclang input.hwp -o output.xml
```

- 종료 코드: `0`
- 실행 전 SHA-256: `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`
- 실행 후 SHA-256: `08424c9555dd51c35968b39c36dc9571d21b3ac2617a2b26dee28da9f3ff07d4`
- 실행 후 `input.hwp`: DocLang XML UTF-8 text

기존 `paths_refer_to_same_file(input, output)`는 경로 문자열뿐 아니라 Unix device/inode와 canonical path도
확인한다. 메인터너 커밋 `0afa775b3`는 `export-doclang`도 이 helper를 사용하도록 바꾸고, 실제 CLI 통합
테스트로 symlink output과 Unix hard link output이 `EXIT_USAGE`(2)로 거부되며 입력 바이트가 변하지 않음을
검증했다.

### 5.2 EqEdit doctest가 존재하지 않는 crate를 import함

PR head의 `src/doclang/eqedit/mod.rs` 예제 3개는 `hangulang::eqedit`를 import한다. 현재 공개 API는
`rhwp::doclang::eqedit`이므로 `cargo test --doc`에서 세 예제가 모두 unresolved crate 오류로 실패했다.
`0afa775b3`는 세 예제 import를 현재 public API로 바꿨고, 보정 후 doctest는 4 passed / 2 ignored로 통과했다.

## 6. 권고와 후속

**권고: 메인터너 보정과 운영 문서 commit을 PR head에 push한 뒤 최신 CI를 확인한다. CI 확인 전까지 merge 보류.**

1. `0afa775b3`은 contributor head `587e786d` 위에만 추가된 code/test 보정 commit이다. contributor 원
   commit을 rewrite하거나 체리픽해 별도 통합 branch로 옮기지 않는다.
2. 이 review·implementation archive 기록과 `pr_review_workflow.md` 9.3.1절의 직접 보정 push 절차를
   별도 운영 문서 commit으로 PR head에 추가한다.
3. push 직전 source fork SHA와 PR `headRefOid`를 다시 확인하고, 변경에 LFS 객체가 없음을 확인한다.
   승인 뒤 `GIT_LFS_SKIP_PUSH=1 git push https://github.com/myeolinmalchi/rhwp.git HEAD:feat/export-doclang`
   으로 contributor branch에 직접 push한다.
4. code/test commit이 포함되므로 문서-only fast-pass는 쓰지 않는다. push 뒤 최신 code SHA의 required
   GitHub Actions, `mergeable`, `mergeStateStatus`를 다시 확인한다.
5. 작업지시자 승인 후 GitHub review 또는 contributor 코멘트와 merge 후속 처리를 수행한다.

### 6.1 collaborator-mediated remote push 가능 여부

`maintainerCanModify: true`이고 source는 `myeolinmalchi/rhwp:feat/export-doclang`이다. 과거 #1376,
#1429, #1447도 외부 contributor PR head에 collaborator의 보정 또는 review 문서 commit을 추가해 처리했다.

source fork의 일반 repository API permission은 `push: false`로 표시됐지만, 기본 Git LFS pre-push hook의
`locks:verify` 인증 오류와 Git ref write 권한은 별개다. PR head에서 만든 빈 probe commit으로
`GIT_LFS_SKIP_PUSH=1 git push --dry-run`이 성공해, LFS 객체를 바꾸지 않는 보정의 PR별 ref write 가능 여부를
비파괴적으로 확인했다. 실제 push에서 `core.hooksPath`를 무력화하지 않는다.

이 권한은 `upstream/devel` 직접 push 권한을 뜻하지 않는다. 실제 push와 GitHub review approval은
작업지시자 승인 뒤에만 수행한다.
