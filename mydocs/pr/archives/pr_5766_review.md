---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
---

# PR #5766 - rhwp-q-pack volume probe 공통화와 CodeQL 분석량 축소

## 라우팅과 메타데이터

```text
base route: collaborator self-merge
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md, docs_and_git_workflow.md, github_operations.md
code candidate: 17ec31ab5615dee3daf78da09a23ba546fd864de
```

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR | [#5766](https://github.com/edwardkim/rhwp/pull/5766) |
| Issue | [#5764](https://github.com/edwardkim/rhwp/issues/5764) |
| 작성자 | `jangster77` self-review |
| base / head | `devel` / `task_m100_5764` |
| 규모 | 52 files, +197 / -750,389 |
| code candidate 상태 | `MERGEABLE` / `CLEAN`, Open, non-draft |

최종 merge 전에 trailing 문서 head의 SHA, `MERGEABLE/CLEAN`, preflight와 `Build & Test`
aggregate를 다시 확인한다.

## 변경 범위와 판정

- `src/bin/rhwp-q-pack/gen/s00.rs`부터 `s49.rs`까지의 생성 shard 50개와 약 750k LOC를
  제거했다.
- 새 `probe.rs`는 문서를 한 번 순회해 공통값과 18개 Control 특징별 누적값을 수집하고,
  기존 slot seed, 280 probe, 3배 특징 가중치, wrapping 산술을 적용한다.
- renderer, layout, fixture, 시각 출력 경로는 바꾸지 않았다. 따라서 visual sweep은 적용 대상이
  아니다.
- 추적 소스를 다시 조사한 결과 `rhwp-q-more`는 이미 같은 공통 probe 구조이며, 다른 대량
  `gen/`/연속 slot Rust 또는 TypeScript 묶음은 남아 있지 않았다. `rhwp-q-kit`은 52개 명령별
  모듈이나 총 5,361 LOC, 파일당 최대 351 LOC로 이 개선의 반복 생성 후보가 아니다.

## 로컬 검증

- `cargo fmt --all`과 `cargo fmt --all -- --check`가 통과했다.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast`가
  `8,008 passed, 39 skipped, 3 slow`로 통과했다. 콜드 빌드 포함 총 소요 시간은 `8분 31초`였다.
- 기존 `agent_q_pack_contract`가 전체 회귀에 포함되어 통과했다. 작업지시자 지시에 따라
  slot 결과 동일성을 고정하는 새 golden 계약은 추가하지 않았다.

## 원격 CI 실측 비교

비교 기준은 동일한 `devel` 대상 공통 probe 개선 PR [#5761](https://github.com/edwardkim/rhwp/pull/5761)의
code candidate `b9a56a11d288a9d4737827c10210131577878342`다. 두 실행은 모두 GitHub-hosted runner의
단일 관측값이므로 cache, registry, runner 부하 차이를 제거한 벤치마크가 아니다.

| 경로 | #5761 | #5766 | 차이 |
| --- | ---: | ---: | ---: |
| Rust CodeQL job 전체 | 20분 02초 | 13분 46초 | -6분 16초 (-31.3%) |
| `Perform CodeQL Analysis` | 19분 05초 | 12분 46초 | -6분 19초 (-33.1%) |
| CodeQL workflow wall-clock | 20분 21초 | 14분 03초 | -6분 18초 (-30.9%) |
| CI workflow wall-clock | 14분 00초 | 12분 21초 | -1분 39초 (-11.8%) |
| Lint | 9분 39초 | 6분 33초 | -3분 06초 (-32.1%) |
| Archive A builder | 3분 54초 | 3분 39초 | -15초 (-6.4%) |
| Archive B builder | 9분 11초 | 5분 21초 | -3분 50초 (-41.8%) |
| Archive C builder | 7분 58초 | 8분 19초 | +21초 (+4.4%) |

- [#5766 CI](https://github.com/edwardkim/rhwp/actions/runs/32357829802)는 preflight, lint,
  Archive A/B/C builder와 네 worker, property roundtrip, adapter inter-diff 및 aggregate가 모두
  성공했다.
- [#5766 CodeQL](https://github.com/edwardkim/rhwp/actions/runs/32357829623)의 Rust analyze job도
  성공했다.
- Archive C의 작은 역행처럼 개별 builder 시간에는 runner 변동이 있으므로, archive 시간의 변화만으로
  공통 probe의 효과를 단정하지 않는다. 대량 생성 source 제거와 Rust CodeQL 분석 단계 단축이 이 PR의
  직접 관찰 근거다.

## 최종 권고

**병합 권고, trailing 문서 head 검증 대기.** 이 문서와 오늘할일만 추가한 뒤 review-only fast-pass가
code candidate `17ec31ab5`의 Full CI를 올바르게 재사용하고 최신 `Build & Test` aggregate가 성공하면,
작업지시자 승인으로 merge할 수 있다.
