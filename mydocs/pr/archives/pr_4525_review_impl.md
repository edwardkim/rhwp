---
kind: review-implementation
status: pending-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4525 메인터너 보정 이행 기록

## 고정 기준

- 원 PR: [#4525](https://github.com/edwardkim/rhwp/pull/4525)
- 원 기여자 / source: @humdrum00001010 / `task_m100_4334_structural_node_id`
- 원 code head: `dc42d10b5b2e52ce0bc455446ef3f2f72ba6dbdd`
- 최신 `devel` 기준: `b66e3d79a93c048478c4737443084f9e7149bbb2`
- 기준선 merge: `bd980c24b871843ac1eda5fce1de370b8e4fcef3`
- 최종 code candidate: `6edb700d16c790d5394d2919898f5d7c71e799ba`
- 로컬 가시성 브랜치: `review/humdrum00001010-4525-20260811`
- 보정 권한: PR metadata의 `maintainerCanModify=true`

## 보정 범위

1. `stableIndex`가 숫자에서 문서 경로 배열로 바뀐 #4334 계약에 맞춰
   `rhwp-studio/e2e/topmost-hittest.test.mjs`의 E2E 단언을 갱신한다.
2. `zOrder`는 숫자, `shapeStable`과 `imageStable`은 정수 배열임을 함께 검증한다.
   단순 배열 존재만 확인해 잘못된 JSON 값을 수용하지 않는다.
3. Rust 정렬 구현, 문서 경로 생성, fixture, CI workflow, 원 기여자 commit은 변경하지 않는다.

## 완료한 보정과 검증

1. `c4e3a0883`에서 E2E assertion을 배열 계약으로 바꿨다. `stableIndex`가 존재만 하는
   비정상 배열이 통과하지 않도록 양쪽 배열의 정수 원소도 함께 확인한다.
2. `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --lib issue_4334`를
   실행해 정렬 경로 관련 3건을 통과했다.
3. `wasm-pack build --target web --dev`를 실행해 Studio용 웹 WASM을 재생성했다.
4. `VITE_URL=http://127.0.0.1:7702 node e2e/topmost-hittest.test.mjs --mode=headless`를
   실행해 `shapeStable=[0,0,2]`, `imageStable=[0,0,3]`, 겹침 클릭의 `shape` 선택을 확인했다.
5. 첫 Full CI는 최신 `devel`이 추가한 `TableNode` literal에 `cell_context`가 없어서 실패했다. Git
   텍스트 충돌은 없었지만 source의 새 struct field와 최신 base test가 함께 컴파일되지 않는 current-base
   호환 결함이었다.
6. contributor commit을 재작성하지 않고 `bd980c24b`에서 최신 `devel`을 source에 merge한 뒤,
   `6edb700d1`에서 해당 literal에 `cell_context: None`을 추가했다.
7. `cargo test --profile release-test --lib collect_top_level_table_spans_domain -- --nocapture`와
   `cargo build --workspace`를 실행해 모두 통과했다.
8. 최종 candidate `6edb700d1`의 CI, CodeQL, Render Diff가 모두 성공했다.
9. `dc42d10..c4e3a0883`의 E2E 보정은 LFS 대상이 아닌
   `rhwp-studio/e2e/topmost-hittest.test.mjs` 한 파일뿐임을 확인했다. 원격 source head와
   PR head는 code candidate `6edb700d1`으로 일치한다.

## 원격 반영 단계

1. `c4e3a0883`, `bd980c24b`, `6edb700d1`을 contributor source branch에 push했고,
   LFS 대상이 없음을 확인했다.
2. code/test 보정이므로 `6edb700d1`에서 Full CI를 실행해 성공을 확인했다.
3. 이 review 기록·오늘할일·공식 workflow 보완만 별도 trailing docs-only commit으로 push해 fast-pass를
   확인한다.

## 롤백 경계

- 보정은 E2E assertion과 최신 base의 `TableNode` 테스트 초기화 한 곳에 한정한다.
- 원 기여자 commit은 rebase, amend, reset, force-push하지 않는다.
