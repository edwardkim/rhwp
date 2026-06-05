# PR #1288 리뷰 — 조사/판정: 미주 trailing A 정규화 중단(defer)

- PR: https://github.com/edwardkim/rhwp/pull/1288
- 관련 이슈: #1258
- 작성자: `planet6897`
- base: `devel`
- head: `feature/issue-1258-trailing-base-flow-normalize`
- 상태: open, draft 아님
- 성격: 코드 변경 없음, 조사/판정 문서 추가

## 1. PR 요약

PR #1288은 #1258의 A 정규화(typeset 미주 base-flow trailing IR 명시) 시도 결과를 문서화한다.

핵심 결론:

- 단일줄/다줄 미주 trailing 처리 비대칭 위치를 특정했다.
- option B 단순 적용 시 `#1246` 계열 보정과 결합되어 between-notes gap 이중가산 회귀가 발생함을 실측했다.
- 정규화는 구조적 단순화라기보다 gap 제공 위치를 옮기는 측면 이동에 가까우며, 순 이득이 불확실하므로 중단(defer)한다.
- 코드 변경은 없다.

## 2. 변경 파일

PR 변경 파일은 6개이며 모두 문서다.

```text
mydocs/plans/task_m100_1258.md
mydocs/plans/task_m100_1258_impl.md
mydocs/report/task_m100_1258_report.md
mydocs/working/task_m100_1258_stage1.md
mydocs/working/task_m100_1258_stage2.md
mydocs/working/task_m100_1258_stage3.md
```

통계:

```text
6 files changed, 411 insertions(+)
```

## 3. 내용 검토

### Stage 1

기존 동작 baseline 을 수치로 고정했다.

- `cargo test --lib height_cursor`: 34 passed
- `issue_1082_endnote_multicolumn_drift`: 4 passed
- `issue_505`: 46 passed
- `issue_1139_inline_picture_duplicate`: 9 passed
- 전체 `cargo test`: 2004 passed, 0 failed, 22 ignored

문22 및 미주사이20 렌더 y baseline 도 기록되어 있다.

### Stage 2

단일줄/다줄 비대칭 위치를 다음처럼 특정했다.

- 단일줄: `paragraph_layout.rs:4523`에서 마지막 줄 trailing 포함
- 다줄: `paragraph_layout.rs:4509` 경로에서 문제 경계 trailing 0

결론은 option B(render 정규화) 채택이었다.

### Stage 3

option B 단순 적용 시 다음 회귀가 발생했다고 기록되어 있다.

- `issue_1139_inline_picture_duplicate`: 46 -> 38 passed, 8 failed
- 문22 y: 484.3 -> 510.77px
- between-notes gap 이 26.5 -> 52.9px로 이중가산

원인은 `#1246`의 `stored_gap_px = result - y_offset` 조건이 상대값이라, render y를 내리면 result도 같이 내려가 `#1246`이 계속 발화한다는 점으로 정리되어 있다.

### 최종 보고서

최종 결론은 중단(defer)이다.

- `paragraph_layout.rs:4509` 정규화와 `#1246` 제거, 다줄 base-shift 가드를 한 커밋에서 결합해야 의미가 있다.
- 결합 변경 후에도 특례 제거 효과가 불확실하다.
- 현재 S8 계열(#1246/#1256/#1261)을 유지하는 것이 비용 대비 타당하다.

## 4. 로컬/브랜치 상태

PR은 현재 base 대비 `BEHIND` 상태다. 변경이 문서뿐이라 충돌 가능성은 낮지만, 현 `devel`에서는 `mydocs/plans`, `mydocs/report`, `mydocs/working` 루트 문서를 archive로 정리하는 정책을 적용 중이다.

따라서 그대로 merge 하기보다 maintainer integration에서 다음 이동을 적용하는 것이 적절하다.

```text
mydocs/plans/task_m100_1258.md -> mydocs/plans/archives/task_m100_1258.md
mydocs/plans/task_m100_1258_impl.md -> mydocs/plans/archives/task_m100_1258_impl.md
mydocs/report/task_m100_1258_report.md -> mydocs/report/archives/task_m100_1258_report.md
mydocs/working/task_m100_1258_stage1.md -> mydocs/working/archives/task_m100_1258_stage1.md
mydocs/working/task_m100_1258_stage2.md -> mydocs/working/archives/task_m100_1258_stage2.md
mydocs/working/task_m100_1258_stage3.md -> mydocs/working/archives/task_m100_1258_stage3.md
```

## 5. 검증 관점

이 PR 자체는 코드 0줄 변경이므로 Rust 빌드/테스트를 다시 요구하는 성격은 낮다.

다만 수용 전 확인할 항목:

- 문서만 변경되었는지 확인
- archive 이동 후 `git diff --stat` 확인
- 필요하면 `cargo fmt --all -- --check` 정도만 smoke로 수행

GitHub status check rollup은 비어 있다. 변경 경로가 `mydocs/**`라 CI paths-ignore에 의해 일반 CI가 트리거되지 않는 것이 자연스럽다.

## 6. 권장 처리

권장: **수용하되 archive 이동을 포함한 maintainer integration으로 처리**.

이유:

- #1258의 조사 결과를 남기는 가치가 있다.
- 코드 변경이 없어 회귀 위험이 없다.
- 결론이 “중단(defer)”라 후속 작업 경계가 명확하다.
- 단, 현 문서 정리 정책과 맞추기 위해 루트 문서 위치는 archive로 이동해야 한다.

권장 절차:

1. `local/devel`에서 `local/pr1288-integration` 생성
2. PR #1288 커밋 또는 파일 내용을 수용
3. 6개 문서를 각 archive 폴더로 이동
4. 리뷰 문서와 함께 커밋
5. maintainer 승인 후 `local/devel` 병합/push
6. PR #1288에는 cherry-pick/integration 수용 완료 코멘트 후 close

## 7. 코멘트 초안

```text
조사/판정 문서의 결론을 확인했습니다.

코드 변경 없이 #1258 A 정규화 시도 결과와 중단(defer) 근거를 남기는 PR로 판단했습니다.
현재 devel의 mydocs 정리 정책에 맞춰 maintainer integration 과정에서 plans/report/working 문서는 archive 경로로 이동해 반영하겠습니다.

기여 감사합니다.
```

## 8. Integration 결과

integration 브랜치:

```text
local/pr1288-integration
```

적용:

```text
git fetch origin pull/1288/head:local/pr1288-upstream
git cherry-pick 6657554408f15f0c867c6867ac9a1bf8bbeeecf2
git cherry-pick 3407c02d3434621b79dd887cb7e205922bd0642f
git cherry-pick 1989f71914876b444e252519ff30dbecf7f6e355
git cherry-pick a79abb3c501a20c72edffe3ea46cf437498345bc
```

archive 이동:

```text
mydocs/plans/task_m100_1258.md -> mydocs/plans/archives/task_m100_1258.md
mydocs/plans/task_m100_1258_impl.md -> mydocs/plans/archives/task_m100_1258_impl.md
mydocs/report/task_m100_1258_report.md -> mydocs/report/archives/task_m100_1258_report.md
mydocs/working/task_m100_1258_stage1.md -> mydocs/working/archives/task_m100_1258_stage1.md
mydocs/working/task_m100_1258_stage2.md -> mydocs/working/archives/task_m100_1258_stage2.md
mydocs/working/task_m100_1258_stage3.md -> mydocs/working/archives/task_m100_1258_stage3.md
```

검증:

```text
cargo fmt --all -- --check
통과
```

판정:

- 코드 변경 없음
- 문서 결론 일관성 확인
- archive 정리 완료
- 수용 가능
