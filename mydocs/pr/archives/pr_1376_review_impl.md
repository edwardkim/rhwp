# PR #1376 리뷰 처리 기록

## PR 커밋

| 항목 | 값 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1376 |
| 작성자 | `mrshinds` (`wpresign`) |
| 작성자 상태 | first-time contributor |
| 원본 커밋 | `e6bbe2bf59796874fc13b38b4fbc44c7478e24fa` |
| 로컬 검토 커밋 | `3e35c471c8940b3c6aca52f88f637f59783683a2` |
| 상태 | merge 보류 |

## Stage 1 - PR 상태 확인 - 완료

- base는 `devel`이다.
- draft가 아니다.
- 작성자는 first-time contributor이다.
- `maintainer_can_modify=true`이다.
- PR 본문과 metadata에 연결 이슈가 없다.
- GitHub 상태는 `mergeable=true`, `mergeStateStatus=BLOCKED`이다.

## Stage 2 - 이슈 확인 - 완료

다음 검색을 수행했다.

```bash
gh pr view 1376 --repo edwardkim/rhwp --json closingIssuesReferences
gh issue list --repo edwardkim/rhwp --state open --search 'TAC host line spacing OR tac-host-spacing OR line-seg OR line segment spacing'
gh search issues --repo edwardkim/rhwp --state open 'treat_as_char table line spacing'
gh search issues --repo edwardkim/rhwp --state open 'lineSegArray'
gh search issues --repo edwardkim/rhwp --state open 'TAC table'
gh search issues --repo edwardkim/rhwp --state open 'host line spacing'
```

결과:

- #1376과 직접 연결된 open issue 없음
- 열린 #1352는 TAC 표 셀 내부 이미지/텍스트 세로 정렬 문제라 별개
- 닫힌 #770은 유사 spacing 이력으로 참고 가능

## Stage 3 - 로컬 검토 브랜치 구성 - 완료

`local/devel` 기준으로 검토 브랜치를 만들고 원본 contributor 커밋만 cherry-pick 했다.

```bash
git switch -C local/pr1376-review local/devel
git cherry-pick -x e6bbe2bf59796874fc13b38b4fbc44c7478e24fa
```

이후 PR #1374 처리 기록 EOF 정리 커밋도 포함했다.

```bash
git log --oneline --decorate -3
```

결과:

- `316e0b78 docs: PR #1374 처리 기록 EOF 정리`
- `3e35c471 fix(layout): apply TAC table host-line spacing when control index differs from line-seg index`
- `6fec954d docs: PR #1374 처리 기록`

## Stage 4 - 로컬 검증 - 완료

| 명령 | 결과 |
|---|---|
| `git diff --check HEAD` | 통과 |
| `cargo test --test issue_1139_inline_picture_duplicate issue_1139_page9_endnote_table_does_not_overlap_header` | 통과 |
| `cargo test --test svg_snapshot` | 통과, 8 passed |
| `cargo test --lib test_tac_host_line_spacing_with_preceding_invisible_controls` | 통과 |
| `cargo build --lib` | 통과 |
| `cargo test --lib` | 실패 |

`cargo test --lib` 실패 상세:

```text
failures:
    renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2

test result: FAILED. 1724 passed; 1 failed; 6 ignored
```

실패 메시지 핵심:

```text
박스 bottom y=531.68 -> ① y=556.53 gap=24.85 가 PDF 기대값 20.00 (±2 px) 와 일치해야 함.
```

## Stage 5 - GitHub Actions 확인 - 완료

다음 명령으로 실패 로그를 확인했다.

```bash
python /Users/tsjang/.codex/plugins/cache/openai-curated/github/c6ea566d/skills/gh-fix-ci/scripts/inspect_pr_checks.py --repo . --pr 1376 --json
```

결과:

- `Build & Test`: 실패
- 실패 테스트: `renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2`
- 실패 값: 로컬과 동일하게 gap `24.85`
- `CodeQL`, `Analyze`, `Canvas visual diff`: 통과
- `WASM Build`: skipped

## Stage 6 - 판단 - 완료

GitHub Actions와 로컬 검증이 같은 기존 회귀 테스트에서 실패하므로 현재 PR은 merge할 수 없다.

구현 방향 자체는 의미가 있으나, `line_segs.last()` 폴백이 너무 넓게 적용되어 기존
`issue_521`의 TAC 표 outer margin bottom PDF 정합을 깨뜨린다. contributor에게는 다음을
정중하게 요청한다.

- 관련 이슈 번호를 PR 본문에 연결
- `test_521_tac_table_outer_margin_bottom_p2`가 통과하도록 폴백 조건을 좁힘
- 수정 후 GitHub Actions 재실행

## PR 코멘트 초안

아래 문구는 작업지시자 승인 전까지 게시하지 않는다.

```markdown
@mrshinds Thank you very much for the detailed analysis and for adding a focused synthetic fixture. The explanation around `control_index` versus `lineSegArray` is very helpful.

I checked the PR locally and in GitHub Actions. Unfortunately, the current version cannot be merged yet because the existing regression test below fails both locally and in CI:

- `renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2`
- CI: https://github.com/edwardkim/rhwp/actions/runs/27415506755/job/81033135957

The failing case expects the PDF-based gap after the TAC table to stay at `20.00 ±2px`, but with this PR it becomes `24.85px`. So the `line_segs.last()` fallback appears to fix the new fixture, but it also applies too broadly and changes an existing TAC table outer-margin case.

Could you please narrow the fallback condition so that:

1. `test_tac_host_line_spacing_with_preceding_invisible_controls` still passes, and
2. `test_521_tac_table_outer_margin_bottom_p2` also keeps passing?

Also, could you link the related issue number in the PR description, or open a small issue for this specific TAC host-line spacing problem and link it here?

Again, thank you for the careful repro case. This is a useful fix direction, and I think it just needs the fallback scope tightened before we can merge it.

---

한국어로도 함께 남깁니다.

자세한 분석과 재현 fixture를 추가해 주셔서 감사합니다. `control_index`와 `lineSegArray`가 서로 다른 인덱스라는 설명은 문제를 이해하는 데 큰 도움이 되었습니다.

로컬과 GitHub Actions에서 확인한 결과, 현재 PR은 아직 바로 머지하기 어렵습니다. 기존 회귀 테스트인 `renderer::layout::integration_tests::tests::test_521_tac_table_outer_margin_bottom_p2`가 로컬과 CI에서 동일하게 실패하고 있습니다.

이 테스트는 TAC 표 뒤 PDF 기준 간격이 `20.00 ±2px`로 유지되기를 기대하지만, 현재 PR 적용 후에는 `24.85px`로 커집니다. 따라서 새 fixture는 해결하지만 `line_segs.last()` 폴백이 너무 넓게 적용되어 기존 TAC 표 outer margin 케이스를 변경하는 것으로 보입니다.

가능하시다면 다음 두 테스트가 모두 통과하도록 폴백 조건을 조금 더 좁혀 주실 수 있을까요?

1. `test_tac_host_line_spacing_with_preceding_invisible_controls`
2. `test_521_tac_table_outer_margin_bottom_p2`

또한 이 TAC host-line spacing 문제에 대응되는 이슈 번호를 PR 본문에 연결해 주시거나, 작은 이슈를 새로 열어 이 PR에 연결해 주시면 좋겠습니다.

좋은 재현 케이스와 분석을 제공해 주셔서 다시 한 번 감사합니다. 방향은 유효해 보이며, merge 전에는 폴백 적용 범위만 조금 더 좁히면 될 것 같습니다.
```
