# Green CI external PR batch - implementation and review plan

## Routing

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_external_pr.md, intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md
```

## Selection

The 2026-09-04 open-PR scan excluded requested PR #6713, drafts #6702, #6685,
#6670, and #5953, plus #6637 because its re-query result was
`CONFLICTING/DIRTY`. The selected source heads were `MERGEABLE/CLEAN` with
latest CI, CodeQL, Render Diff, Adapter inter-diff, and Proptest success or
policy-expected skips: #6683, #6690, #6698, #6703, #6705, #6709, and #6710.

## Integration history

Base: `upstream/devel` `009e30fe1`.

| Source PR | Source commits | Integration commits |
| --- | --- | --- |
| #6683 | `e5f385c17`, `e5dde4373` | `dd8ca73a2`, `4c333ab94` |
| #6690 | `f9d76b11c`, `c37925771` | `eb84bbbc7`, `9f0455b6f` |
| #6698 | `7e47ef691` | `c0145ec66` |
| #6703 | `0bb2f2b08`, `219868e86` | `ceadaf94a`, `2dd41febf` |
| #6705 | `0bfcd04cd`, `05325df7c` | `7b15d9582`, `dda7902e5` |
| #6709 | `df9c6c612`, `36b550089` | `e6b9a3ed5`, `ffd47191e` |
| #6710 | `ceb649a2d`, `4a1eb7c27` | `c340bd7a8`, `61cd71fb9` |

All source commits were applied with `git cherry-pick -x` in their original
PR order. The first attempted single-head import was abandoned before use
because #6683 and #6690 have trailing commits; the final branch contains the
complete source commit series. The final import completed without manual
conflict resolution.

## Maintainer correction

`872f3d4c5` changes only regression tests for #6202 and #5057. It preserves
the allowed missing-private-fixture skip but replaces post-fixture `return`
paths with `expect`, preventing false green results on picture-edit, HWPX
export, ZIP read, or ZIP write failure.

## Remaining gates

1. Run the prescribed integration-head Rust lint/test gates sequentially in a
   dedicated review target.
2. Run external-fixture cases with `RHWP_ISSUE5941_SAMPLE`,
   `RHWP_ISSUE6202_SAMPLE`, and `RHWP_ISSUE5057_SAMPLE` set to the identified
   private corpus inputs.
3. Produce and inspect integration-head visual evidence for each page-visible
   claim. Copy only representative current-head PNGs to stable
   `mydocs/pr/assets/` paths and record hashes, pages, metrics, and limitations.
4. Add a review/today trailing commit only after the above evidence is factual,
   then obtain explicit approval before any push or PR creation.

No GitHub source comment, close, push, PR creation, approval, or merge was
performed in this stage.
