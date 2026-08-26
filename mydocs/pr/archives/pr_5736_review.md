---
kind: pr-review
status: review-complete-pending-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-20
---

# PR #5736 검토 - 순수 흐름 자리차지 표의 body_bottom 클램프 보정

## 접수 메타데이터

| 항목 | 검토 기록 |
| --- | --- |
| PR / 작성자 | [#5736](https://github.com/edwardkim/rhwp/pull/5736) / `planet6897` |
| base / 원 PR head | `devel` / `adef36628c9c0738fc2242667cb3d16da7212e66` |
| source branch | `fix/5699-j3-bodybottom` |
| 변경 규모 | 4 files, +145 / -78 |
| 로컬 검토 기준 | `upstream/devel@b32113be61aefab049d03d6ab618c217104c080c` 위 체리픽 `cc99f3bae9249f4391a24a3c16897d86320d1b82` |
| 라우팅 | `collaborator_external_pr` + `intake_and_review` + `local_validation` + `visual_fixture_evidence` + `review_only_fast_pass` + `post_merge` |
| 작성 시점 상태 | 비 Draft, `MERGEABLE`, `CLEAN`; 원 PR source head의 required check는 모두 성공 |

원 PR은 최신 `upstream/devel`을 직접 병합하지 않고 원 contributor head를 유지했다. 로컬 검토 브랜치에서는
동일한 contributor commit을 최신 `devel` 위에 `-x` 체리픽해 전체 회귀를 수행했다. 이 문서는 원 source
commit을 재작성하지 않는 trailing review-only 기록이다.

## 변경 범위와 판정

`src/renderer/layout/table_layout.rs`에서 `VertRelTo::Para`인 `TopAndBottom` 표가
`body_bottom` 클램프 때문에 이미 페인트된 직전 본문 줄 위로 끌려 올라가는 경우를 제한적으로 해제한다.
해제 조건은 Top/Inside 정렬, 0 vertical offset, 클램프가 `y_start`보다 위로 이동한 경우, 흐름이 본문
영역 하단 안에 있는 경우, 표 전체가 용지 높이 안에 있는 경우로 한정된다. offset 고정 표와 과적 페이지의
기존 하단 클램프 동작은 유지한다.

신규 HWP fixture와 `issue_5699_table_bodybottom_clamp` 회귀 테스트는 37787 규제영향분석서의 p6에서
pi62 본문 줄과 pi63 자리차지 표의 세로 침범을 검증한다. `ir_field_sweep_baseline.tsv`의 77개 개선 행
정리와 신규 fixture의 `list_header_width_ref` 2개 행도 함께 확인했다.

차단 결함은 발견하지 못했다. 다만 신규 테스트는 하나의 fixture와 pi62/pi63 조합에 집중되어 있어
비영(非零) offset, 비대칭 여백, 각 해제 가드의 경계값을 독립적으로 모두 행렬화한 테스트는 아니다.
현재의 전체 회귀·Native Skia·Canvas Render Diff가 통과했고, 해당 공백은 차단 사유가 아닌 후속 테스트
보강 후보로 분류한다.

baseline 77개 삭제는 현재 재생성한 sweep 결과에서도 해당 행들이 사라진 사실과 일치한다. 비교 로직은
현재 값이 baseline보다 커질 때 회귀로 판정하므로, 해당 경로가 다시 나타나면 0 기준으로 실패한다.
따라서 이번 삭제가 회귀를 숨기는 것으로 판단하지 않았다. 다만 향후 baseline 정리에서는 삭제 근거를
같은 측정 원장에 계속 남기는 것이 바람직하다.

관련 issue [#5699](https://github.com/edwardkim/rhwp/issues/5699)는 PR 본문에 자동 종료 키워드가
확인되지 않았다. merge 후 실제 issue 상태를 재조회하고, 열려 있으면 merge SHA와 검증 결과를 담은
comment를 남긴 뒤 수동 close한다.

## 검증

### 로컬 검증

- `node scripts/rust-test-suite-manifest.mjs --prepare`: 통과
- `node scripts/rust-test-suite-manifest.mjs --check`: 통과
- `cargo fmt --all -- --check`: 통과
- `git diff --check upstream/devel...HEAD`: 통과
- `issue_5699_table_bodybottom_clamp`: 1 passed, 133 skipped
- 관련 focused test `issue_1858`, `issue_1891`, `issue_4514_overlay_table_flow`, `issue_5699_shape_flow_rewind`: 모두 통과
- `ir_field_sweep_baseline`: 4 passed, 전체 sweep 75.2초
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`: **8,009 passed, 39 skipped**, 204.296초

### 원 PR head의 GitHub 검증

원 source head `adef3662`에서 다음 결과를 확인했다.

- [CI run #32338150413](https://github.com/edwardkim/rhwp/actions/runs/32338150413): Build & Test, Lint,
  Native Skia, archive, regular/slow shard, Frontend package gates 성공; Rust-only 변경으로 Frontend unit
  gates는 skip
- [CodeQL run #32338150189](https://github.com/edwardkim/rhwp/actions/runs/32338150189): Rust, Python,
  JavaScript/TypeScript 분석과 CodeQL aggregate 성공
- [Render Diff run #32338150174](https://github.com/edwardkim/rhwp/actions/runs/32338150174): Canvas visual diff 성공
- Adapter inter-diff와 Proptest도 성공

별도 로컬 PDF 또는 review PNG를 merge 근거로 사용하지 않았다. HWP fixture에 대한 사용자-visible 판정은
원 PR의 Canvas Render Diff와 신규 fixture 회귀 테스트를 근거로 삼았다.

## 최종 권고

**수용 권고.** 차단 결함과 추가 메인터너 코드 보정은 발견하지 못했다. 이 문서와 오늘할일을 원 PR
source branch에 trailing docs-only commit으로 push한 뒤, 새 head의 preflight·Build & Test aggregate,
CodeQL, Render Diff와 `MERGEABLE/CLEAN`을 다시 확인한다. 모두 통과하면 작업지시자 승인에 따라 merge하고,
merge 후 [post_merge.md](../../manual/pr_review/post_merge.md) 순서로 #5699 comment/close, 최종 devel
동기화, review branch와 원격 source branch 정리를 수행한다.
