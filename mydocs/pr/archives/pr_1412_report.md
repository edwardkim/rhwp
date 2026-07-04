# PR #1412 처리 보고서 — task 1411 공식 미주 모양 모델 잔여 검증

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1412 |
| 제목 | `task 1411: 공식 미주 모양 모델 잔여 검증` |
| 작성자 | `jangster77` |
| 관련 이슈 | #1411 |
| PR base | `devel` `a2a1b383` |
| 원 PR head | `c338a1d6` |
| 처리 기준 | `local/devel` |
| 통합 방식 | PR 커밋 7개 cherry-pick + contributor 문서 archive 정리 |
| 리뷰 문서 커밋 | `c0005b83` |
| PR 커밋 반영 | `38d8ff7d`..`07d876a6` |
| 처리 상태 | 완료 |
| devel merge | `ca2d214f` |
| PR close | `2026-06-14T23:54:24Z` |
| Issue #1411 close | `2026-06-14T23:54:32Z` |

## 2. 처리 내용

작업지시자 리뷰 보고서 승인 후 PR #1412의 원 커밋 7개를 현재 `local/devel` 위에
cherry-pick했다. 충돌은 없었고 author 정보는 보존됐다.

원 PR 커밋:

```text
1660aa5a task 1411: baseline 잔여 후보 재현
905c96c9 task 1411: 2022-10 p14 후보 분류
80c1c322 task 1411: 2022-10 p14 미주 수식 tail gap 보정
13b85752 task 1411: 2024-09 잔여 후보 분류
a65b8326 task 1411: 2024-11 잔여 후보 분류
2e3ab703 task 1411: 최종 검증 보고서 정리
c338a1d6 task 1411: PR 생성 상태 기록
```

`local/devel` 반영 커밋:

```text
38d8ff7d task 1411: baseline 잔여 후보 재현
ca2b7000 task 1411: 2022-10 p14 후보 분류
161d8e08 task 1411: 2022-10 p14 미주 수식 tail gap 보정
5ed09c5b task 1411: 2024-09 잔여 후보 분류
0b621c87 task 1411: 2024-11 잔여 후보 분류
8e5682bd task 1411: 최종 검증 보고서 정리
07d876a6 task 1411: PR 생성 상태 기록
```

처리 전 `local/devel`에 남아 있던 stale `PR #1371` active 리뷰 문서 잔여는 제거했다.
현재 `devel`에는 이미 `mydocs/pr/archives/pr_1371_review.md` 최신 처리본이 있으므로,
오래된 active 문서를 다시 push하지 않기 위한 정리다.

완료된 Task #1411 산출물은 archive로 이동했다.

- `mydocs/plans/archives/task_m100_1411.md`
- `mydocs/plans/archives/task_m100_1411_impl.md`
- `mydocs/working/archives/task_m100_1411_stage1.md`
- `mydocs/working/archives/task_m100_1411_stage2.md`
- `mydocs/working/archives/task_m100_1411_stage3.md`
- `mydocs/working/archives/task_m100_1411_stage4.md`
- `mydocs/working/archives/task_m100_1411_stage5.md`
- `mydocs/report/archives/task_m100_1411_report.md`

## 3. 변경 내용

`src/renderer/layout.rs`:

- textless tall equation tail 뒤 새 미주 문항 제목에서 visible content 기준 gap이 이미 확보된 경우,
  layout 단계의 logical note title gap 중복 보존을 생략한다.
- 생략한 gap은 같은 미주 본문까지 유지하고, 다음 미주 제목에서만 vpos base에 지연 복원한다.
- 보정 조건을 미주 흐름, 문항 제목, textless TAC equation tail, 단 하단부, continued partial 제외로 좁혔다.

문서:

- #1411 수행/구현/단계/최종 보고서 추가 후 archive 이동
- 2026-06-15 오늘할일 갱신
- PR #1412 리뷰 보고서 추가

## 4. 검증

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| Canvas visual diff | pass |
| Analyze rust | pass |
| Analyze python | pass |
| Analyze javascript-typescript | pass |
| CodeQL | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo test --lib compact_endnote_question_title_after_tall_tail_limited_backtrack -- --nocapture` | 통과, 1 passed |
| `cargo build --bin rhwp` | 통과 |
| PR 커밋 7개 cherry-pick | 통과, 충돌 없음 |

## 5. 판정

**수용 가능**.

PR #1412는 #1411의 목표인 PR #1410 후속 잔여 후보 재분류와 `2022-10` p14 실제 layout 결함
보정에 부합한다. `2024-09`, `2024-11` 잔여는 공식 미주 모양값 계산식 문제가 아니라
TAC shape/equation/table continuation split 계열로 분류되어 후속 범위가 명확해졌다.

새 단위 테스트가 `layout.rs` 변경을 직접 추가로 덮지는 않지만, contributor의 targeted sweep,
GitHub Canvas visual diff, 로컬 focused test와 빌드 확인을 종합하면 수용 조건을 만족한다.

## 6. 완료 절차

처리 보고서 승인 후 다음 절차를 완료했다.

1. `local/devel` 상태 재확인
2. `local/devel`을 `devel`에 no-ff merge — `ca2d214f`
3. `git push origin devel` — `a2a1b383..ca2d214f`
4. PR #1412에 cherry-pick 반영 코멘트 작성 후 close
5. Issue #1411 수동 close
6. PR #1412 리뷰/처리 문서 archive 이동
