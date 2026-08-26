---
kind: pr-review
status: accepted-pending-integration-pr-approval
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6011 review - RowBreak 1x1 선형 셀 저장 vpos 스냅 복원 (#5995)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6011](https://github.com/edwardkim/rhwp/pull/6011) |
| 작성자 | [@planet6897](https://github.com/planet6897) |
| 원 head | `0c60b7a85294909210f3d34ae1edb7954aeeb112` |
| 통합 commit | `1b8d29da3` |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, CI·CodeQL 성공 |
| 판정 | **수용 권고** |

## 변경 검토

RowBreak 1x1 선형 셀에서 저장 vpos 스냅을 켜는 기존 호환 경로가 `vertical_offset == 0`에만
한정되어, 저자가 남긴 미세 오프셋 `-98HU`에서 27개 문단이 재흐름되던 문제를 완화한다. 셀 내부
vpos는 셀 콘텐츠 기준 좌표라 표 자신의 미세 세로 오프셋과 독립이라는 설명은 코드 주석과 일치한다.

또한 `table_partial.rs`에서 저장 vpos가 이미 포함한 `spacing_before`를 다시 더하던 경로를
`layout_composed_paragraph`의 재가산 규약에 맞춰 보정한다. 표 host 문단은 재가산 경로를 타지 않으므로
텍스트 문단에만 제한한 점도 적절하다.

## 로컬 검증

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --all -- --check`: 통과
- `cargo clippy --locked --target-dir target/pr-review --lib --bins --tests -- -D warnings`: 통과
- 전체 nextest: `8306 tests run: 8306 passed (4 slow), 42 skipped`, 211.485s

## 증적

원 PR은 저장소 외부 코퍼스 `30269_붙임1)제도개선권고안.hwp`에 대한 한글 2020 오라클 PDF 실측값을
본문과 작업 문서 `mydocs/working/issue_5995_nested_cell_stored_ladder.md`에 기록했다. 해당 원본과
비교 PNG는 저장소에 포함되지 않았으므로 이번 통합 검토에서는 코드 제한 조건, PR 자체 CI, 통합 후보
전체 회귀로 수용성을 판단했다.

본문 기준 정량 변화는 내부표 상단 135.11mm(오라클 135.66mm), 내부표 아래 간격 7.7mm(오라클 7.6mm),
총 쪽수 22쪽 유지다. 잔여 0.8px fit margin 문제는 PR 본문에서 후속 분리 대상으로 명시되어 있고,
이번 변경의 렌더/조판 self-consistency를 차단하지 않는다.

## 권고

원 PR CI와 통합 후보 전체 회귀가 모두 통과했다. 변경 조건이 RowBreak 1x1 선형 셀의 기존 호환 경로에
좁게 한정되어 있어 이번 통합 PR에 포함해 수용 가능하다.
