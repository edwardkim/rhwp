---
kind: pr-review
status: pending-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4525 리뷰 - stableIndex 문서 경로 정렬 계약

## 라우팅과 접수

```text
base route: collaborator_external_pr.md
modifiers: intake_and_review.md, local_validation.md,
  visual_fixture_evidence.md, multi_pr_update_branch.md,
  rework_and_exceptions.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4525](https://github.com/edwardkim/rhwp/pull/4525) |
| 작성자 / source | @humdrum00001010 / `task_m100_4334_structural_node_id` |
| 원 code head | `dc42d10b5b2e52ce0bc455446ef3f2f72ba6dbdd` |
| 최신 `devel` 기준 | `b66e3d79a93c048478c4737443084f9e7149bbb2` |
| 기준선 merge | `bd980c24b871843ac1eda5fce1de370b8e4fcef3` |
| 최종 code candidate | `6edb700d16c790d5394d2919898f5d7c71e799ba` |
| 메인터너 보정 | `c4e3a08839fcee53b4e18308cc0b0a526d1cc7b6`, `6edb700d16c790d5394d2919898f5d7c71e799ba` |
| 로컬 가시성 브랜치 | `review/humdrum00001010-4525-20260811` |
| source 수정 권한 | `maintainerCanModify=true` |
| merge tree | 최신 `upstream/devel`을 source에 merge한 뒤 `git diff --check upstream/devel...HEAD` 통과 |

## 검토와 메인터너 보정

원 PR은 `paper_node_sort_key`와 Studio hit-test의 `stableIndex`를 스칼라에서 문서 경로 배열로
바꿨다. Rust와 TypeScript 본 구현은 배열 사전식 비교를 사용해 정합했지만,
`topmost-hittest.test.mjs`는 여전히 `typeof stableIndex === 'number'`를 요구했다. 이 수동 E2E를
실행하면 정상 WASM 응답도 실패하므로 merge 차단 결함이었다.

`c4e3a0883`은 `zOrder`의 숫자 계약을 유지하면서 `shapeStable`과 `imageStable`이 정수 배열임을
검증한다. 정렬 구현, 렌더 출력, fixture, CI workflow와 원 기여자 commit은 변경하지 않는다.

첫 code head의 Full CI는 최신 `devel` merge tree에서 `TableNode.cell_context` 초기화가 빠진
`collect_top_level_table_spans_domain` 테스트 때문에 실패했다. 이는 Git 텍스트 충돌이 아닌 기준선 API
호환 결함이다. contributor commit은 재작성하지 않고 `bd980c24b`에서 최신 `devel`을 source에 merge한 뒤,
`6edb700d1`에서 해당 테스트 literal에 `cell_context: None`을 추가했다.

## 완료한 검증

- `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --lib issue_4334`:
  정렬 경로 관련 3건 통과.
- `wasm-pack build --target web --dev`: 통과.
- `VITE_URL=http://127.0.0.1:7702 node e2e/topmost-hittest.test.mjs --mode=headless`:
  실제 WASM 응답의 `shapeStable=[0,0,2]`, `imageStable=[0,0,3]`와 겹침 클릭의 `shape` 선택을 확인.
- `cargo test --profile release-test --lib collect_top_level_table_spans_domain -- --nocapture`:
  최신 `devel` 호환 보정 대상 1건 통과.
- `cargo build --workspace`: 통과.
- 최신 `upstream/devel` merge와 `git diff --check upstream/devel...HEAD`: 통과.
- 최종 code candidate `6edb700d1`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/31480100040),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31480099750),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31480099759): 모두 성공.

공식 workflow에는 `MERGEABLE`만으로 기준선 호환을 단정하지 않고, latest merge tree의 실제 컴파일 오류가
확인된 경우에만 contributor source에 최신 `devel` merge와 별도 호환 보정을 허용하는 절차를 추가했다.

## 최종 권고

**수용.** 최종 code candidate의 Full CI, CodeQL, Render Diff와 로컬 보정 검증이 모두 통과했다.
이 문서와 오늘할일의 trailing docs-only fast-pass가 최신 head에서 성공하면 merge한다.
