---
kind: pr-review
status: pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6393
author: jangster77
---

# PR #6393 review - PR 전 Rust lint gate 명확화

## 라우팅과 metadata

- PR: [#6393](https://github.com/edwardkim/rhwp/pull/6393), base: `devel`.
- base route: `collaborator_self_merge.md`; modifiers: `intake_and_review.md`,
  `local_validation.md`, `review_only_fast_pass.md`.
- 작성자·self-review: `jangster77`; collaborator 본인 PR이므로 reviewer request는 등록하지 않았다.
- 작성 시점 참고값: code candidate `4baba2a50eb21a71fc04ac36e846cd90647f110e`, Open non-draft,
  `MERGEABLE`, 4 files, `+70/-18`. CI 결과와 mergeability는 merge 직전에 다시 확인한다.

## 변경 범위와 원인

- 이전 절차는 Rust parser/model/CLI 행에만 `clippy`를 명시하고, Rust test/baseline 변경 행에는
  focused test와 최신 CI만 기록했다. 따라서 test/baseline helper 변경에서 format만 확인하고
  Clippy를 GitHub CI에 맡길 수 있는 공백이 있었다.
- `AGENTS.md`, `CONTRIBUTING.md`, `local_validation.md`, `pr_review_workflow.md`를 같은 규칙으로
  정렬했다. Rust source 또는 Rust test/baseline helper 변경은 PR 생성 전에 format, native Clippy,
  WASM32 Clippy, workspace all-target Clippy를 순차로 통과해야 한다.
- HWP/HWPX/PDF 같은 fixture data만 바꾼 경우와 Rust helper 변경을 분리했다. data-only 변경에는
  불필요한 Cargo lint를 강제하지 않지만, helper가 같이 바뀌면 Rust lint gate를 적용한다.
- 이 PR에는 Rust source, test, baseline, fixture data, workflow가 포함되지 않아 review-only fast-pass
  범위다.

## 로컬 검증

- `git diff --check`: 통과.
- `python3 scripts/check_markdown_links.py AGENTS.md CONTRIBUTING.md
  mydocs/manual/pr_review/local_validation.md mydocs/manual/pr_review_workflow.md`: 4개 문서의 내부
  Markdown 상대 링크 이상 없음.
- 현재 `ci.yml` Lint job의 format, native Clippy, WASM32 Clippy, workspace all-target Clippy 명령을
  대조했다. 문서에 적은 `--locked`, `target/pr-review`은 로컬 검토 target·lockfile 보호를 위한
  동등한 제약이며, 검사 범위를 줄이지 않는다.
- 변경 파일 목록에 classifier를 적용한 결과:

  ```json
  {"rust_required":"false","frontend_mode":"none","render_required":"false",+  "native_skia_required":"false","codeql_languages":"none",+  "classification_status":"classified","reason":"classified:review-only"}
  ```

- 문서 전용 변경이므로 Cargo lint·nextest는 실행하지 않았다. 새 규칙의 실제 Cargo 실행은 이후 Rust
  변경 PR에서 해당 변경 범위의 필수 gate로 수행한다.

## 최종 권고와 후속 조건

**조건부 수용.** 원인인 범위 표의 빈틈과 CI lint와의 대응 관계가 모두 문서화됐다.

- trailing 문서 commit 뒤 최신 head의 preflight와 Build & Test aggregate가 review-only fast-pass로
  성공하는지 확인한다.
- 최신 head가 `MERGEABLE/CLEAN`이고 required check가 성공하면 작업지시자 승인에 따라 squash merge한다.
