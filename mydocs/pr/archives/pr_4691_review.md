---
kind: review
status: local-ci-complete
pr: 4691
issue: 4689
author: kevin9327
base: devel
---

# PR #4691 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4691](https://github.com/edwardkim/rhwp/pull/4691) |
| 작성자 | `kevin9327` |
| 관련 이슈 | [#4689](https://github.com/edwardkim/rhwp/issues/4689) |
| 원 code head | `2e1870144640330291074ddc0fd27290fb771f6d` |
| base / 문서 작성 시점 상태 | `devel` / `MERGEABLE`, `CLEAN` 참고값 |
| 원격 필수 CI | [Build & Test 성공](https://github.com/edwardkim/rhwp/actions/runs/31648522729/job/94290687324) |
| reviewer | `jangster77` 지정 완료 |

### 적용 절차

```text
base route: maintainer_general.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
maintainer_general.md, intake_and_review.md, local_validation.md,
multi_pr_update_branch.md, post_merge.md
```

최신 `upstream/devel` `f7a98ce04` 위의 `review/kevin9327-20260813`에 원 PR의
기능 커밋을 순서대로 체리픽했다. 적용 커밋은 `86594ea84`와 `2e18701446`이고,
로컬 적용본은 각각 `d13287690`, `adf2d0c4b`이다. 충돌은 없었다. 원 PR head는
현 `devel`의 조상이 아니었으므로, 오래된 기준선의 history는 병합하지 않고 PR 고유
기능 커밋만 적용했다.

## 변경 검토

`core-cli`의 14개 legacy 과제에 모두 `reference/` 기준 풀이를 추가하고, 기존의
예외를 제거해 모든 pack이 동일한 기준 풀이 완결성 계약을 따르게 한다. T13의 실제
판정 명령을 존재하지 않는 `harness status`에서 `harness-status`로 고치고,
pack 선언과 task가 부르는 명령의 정합을 검사한다. Windows 경로 비교도
`os.path.join` 기준으로 수정했다.

변경은 gym 기준 자료·Python 계약 테스트·작업 기록·참고용 HWP/PNG 증적에 한정된다.
Rust renderer, layout, WASM, CI workflow 및 `samples/` fixture는 바꾸지 않으므로
renderer visual sweep은 merge 판단 게이트로 선택하지 않았다. T08의 전/후 PNG는
참고 증적으로 직접 확인했으며, 표 `(0,0)`의 값이 `<신 설>`에서 `짐검증`으로 바뀐
주장과 일치하고 잘림은 보이지 않았다. 기준 PDF와의 fidelity 판정은 이 PR의 주장에
포함되지 않는다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| `cargo build --profile release-test --bin rhwp --target-dir target/pr-review` | 통과 |
| `rhwp capabilities --search harness-status --json` | `harness-status`의 `--keyring`, `--deep`, `--json` 표면 확인 |
| `python -m unittest scripts.tests.test_gym_packs scripts.tests.test_gym_score` | 32 passed |
| `python gym/tools/build_baseline.py --agent pr4691-review --pack core-cli --bin target/pr-review/release-test/rhwp.exe` | 성공 14 · 실패 0 |
| `python gym/score.py --agent pr4691-review --pack core-cli --bin target/pr-review/release-test/rhwp.exe` | 32/32, 14/14 과제 |
| 전 pack 기준 풀이 왕복 | 성공 100 · 실패 0 · 기준 풀이 없음 0 |
| 전 pack 채점 | 221/221, 12 pack |
| `git diff --check upstream/devel...HEAD` | 통과 |

동일 원 code head의 GitHub Build & Test도 성공했다. 위 로컬 실행은 체리픽한 최신
`upstream/devel` 기준의 누적 후보에서 수행했다.

## 최종 권고

**수용 및 merge 권고.** 발견한 blocker는 없다. 실제 merge 전에는 PR #4691의 최신
head, mergeability, 필수 GitHub Actions와 작업지시자 승인을 다시 확인한다. merge 후에는
이 문서를 archive로 옮기고 merge SHA·이슈 상태를 기록한 뒤 contributor comment를 게시한다.
