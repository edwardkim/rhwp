---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
---

# PR #5774 - WASM 검증의 Cargo lockfile 오염을 막는다

## 라우팅과 메타데이터

```text
base route: collaborator self-merge
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  docs_and_git_workflow.md, github_operations.md
code candidate: 5feb13948dee6a5f1fb2051dca3d0a71619fe614
```

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5774](https://github.com/edwardkim/rhwp/pull/5774) |
| Issue | [#5773](https://github.com/edwardkim/rhwp/issues/5773) |
| 작성자 | `jangster77` self-review |
| base / head | `devel` / `codex/wasm-pack-locked-metadata` |
| code candidate 상태 | Open, non-draft. trailing 문서 head CI 대기 |

최종 merge 전에는 trailing 문서를 포함한 최신 head의 SHA, mergeability, preflight와 `Build & Test`
aggregate를 다시 확인한다.

## 변경 범위와 판정

- raw `wasm-pack build --locked`의 사전 `cargo metadata`는 lockfile을 갱신할 수 있다. POSIX wrapper는
  shim `cargo`로 metadata에만 `--locked`를 더하고 나머지 호출은 원래 인수대로 위임한다.
- Windows PowerShell wrapper는 임시 `cargo.exe` proxy와 sibling `wasm-pack.exe`를 사용해 Windows의 실행 파일
  탐색 순서에서도 같은 metadata 보호를 적용한다. `cmd.exe` wrapper는 이를 native PowerShell로 위임한다.
- `CONTRIBUTING.md`, 개발·PR 검토 가이드와 로컬 source bind-mount Docker Compose 경로를 wrapper로 통일했다.
  GitHub Actions workflow와 의존성 갱신 정책은 변경하지 않았다.
- trailing 문서는 renderer/layout 결과를 바꾸지 않는다. PR review에서 문서 비교가 필요한 경우의 공식
  절차가 [PDF/SVG visual sweep 가이드](../../manual/verification/visual_sweep_guide.md#github-merge-comment)임을
  라우터·fixture 증적·merge 후속 안내에 명시한다.

## 로컬 검증

- `node --test scripts/tests/rust-test-suite-manifest.test.mjs`: 18 passed.
- suite `--prepare`/`--check`, `node scripts/rust-unit-test-tiers.mjs --check`, focused
  `issue_1035_alignment`: 통과. `--prepare`는 ignored 파생 suite만 만들었고 `Cargo.toml`·`Cargo.lock`은
  변경하지 않았다.
- `python3 -m unittest scripts/tests/test_docker_wasm_compose.py`: 4 passed.
- `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check`: 통과.
- `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg`: 성공 후
  `git diff --exit-code -- Cargo.toml Cargo.lock` 통과.
- Windows `win10-ted`에서 PowerShell wrapper와 `cmd.exe` wrapper의 실제 WASM build 뒤에도 동일 Cargo
  파일 diff 검사가 통과했다.
- 변경 문서 6개의 `scripts/check_markdown_links.py` 상대 링크 검사가 통과했다. 전체
  `check_document_metadata.py`는 이 PR과 무관한 기존 `mydocs/tech` 4개 파일의 front matter 누락 16건으로
  실패했으며, 새 review 문서는 오류 목록에 없었다.
- Visual Sweep 문서 규칙은 경로·comment template 정리만 다룬다. 이 PR 자체는 renderer 출력 변경이 없어
  visual sweep 실행 대상이 아니다.

## 최종 권고

**병합 권고, trailing 문서 head 검증 대기.** wrapper의 macOS와 Windows 실측이 root Cargo 파일 무변경을
확인했고, 문서 변경은 검토 시 문서 비교 절차와 PNG 증적 표시를 혼동하지 않도록 정본 경로를 고정한다.
최신 PR head의 CI와 mergeability가 통과하면 merge할 수 있다.
