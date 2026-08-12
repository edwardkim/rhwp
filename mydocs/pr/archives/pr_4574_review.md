---
kind: pr-review
status: pending-github-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4574 리뷰 - kevin9327 검증 사다리 18건 통합

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md,
  multi_pr_update_branch.md, post_merge.md
```

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4574](https://github.com/edwardkim/rhwp/pull/4574) |
| 작성자 / 원 기여자 | @jangster77 / @kevin9327 |
| base / source | `devel` / `pr/devel-kevin9327-agent-ladder` |
| code candidate | `4d43457af` |
| trailing 문서 이전 head | `24ed47acfc5a2f6bb42ac0f4d9c5cab58485cfaf` |
| 범위 | CI 성공 원 PR 18건 누적, 메인터너 정합 보정 1건, 원 PR별 archive review와 오늘할일 |
| reviewer | 작성자와 동일 계정에는 GitHub가 reviewer 요청을 거부한다. reviewer 미지정은 작성 시점 사실이며 merge 전 branch protection을 재확인한다. |

## 누적 범위와 메인터너 보정

#4465, #4510, #4511, #4529, #4534, #4536, #4538, #4540, #4542, #4544, #4546, #4548,
#4550, #4552, #4557, #4559, #4562, #4563을 오래된 PR 번호순으로 누적했다. 원 branch의
`devel` merge commit은 넣지 않았다. 원 PR별 최신 head, CI, 적용 SHA, 충돌과 수용 근거는
`pr_4465_4510_4511_4529_4534_4536_4538_4540_4542_4544_4546_4548_4550_4552_4557_4559_4562_4563_review_impl.md`와
각 `pr_<번호>_review.md`에 보존했다.

`4d43457af`는 생성 코덱스를 현재 CLI 83개 명령에 맞췄다. `harness-status`는 읽기 전용 명령으로,
`anchor`·`bundle`·`disclose`·`settle`은 실제 top-level 명령만 문서화한다. #4562 충돌에서는 새 온보딩
지침과 기존 roadmap 행을 모두 유지했다. 이 보정은 원 PR들을 독립 적용할 때는 드러나지 않는 누적 정합 문제만 다룬다.

## 완료한 검증

- `cargo fmt --all -- --check` 통과.
- `cargo build --bin rhwp --target-dir target/pr-review` 통과.
- `RHWP_BIN=/home/tsjang/rhwp/target/pr-review/debug/rhwp python3 tools/gen_agent_codex.py --check`:
  명령 83개, 실측 표본 18개, 변경 0.
- signing, harness, anchor, gate, bundle, disclose, settle, audit, provenance, agent-codex, skills focused nextest:
  37 passed.
- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  5,757 passed, 36 skipped, 7 slow, 448.545초.
- rebase 기준선의 추가분은 CI workflow, Python workflow 검사와 문서뿐이며 `src/` 및 `bindings/node/src/` tree가
  검증 완료 head와 동일함을 확인했다. 작업지시자 지시에 따라 전체 회귀를 다시 실행하지 않았다.
- 최신 head 이전 CI의 `생성 타입 최신 검사`가 capabilities 52개와 `envelopes.ts` 47개의 불일치를 차단했다.
  `envelopes.ts`를 재생성해 누락된 audit-report, conformance, disclose, recall-scope, settle 봉투 타입을 추가했고
  `RHWP_BIN=target/pr-review/debug/rhwp npm run gen:check`를 통과했다. 이 보정 후 최신 head CI를 다시 확인한다.

## 시각 검증과 merge 조건

renderer, layout, pagination, 기준 PDF 또는 fixture 변경이 없으므로 별도 visual sweep은 선택하지 않았다.
gym의 PNG는 과제 증적일 뿐 PDF fidelity 주장이나 merge 판단 근거가 아니다.

**최종 권고: 최신 trailing head의 GitHub Actions 통과, mergeable 및 branch protection 재확인, 작업지시자 승인 뒤 merge.**
merge 후 이 PR에 포함된 archive 기록을 유지한 채 원 PR 18건의 close/comment, 관련 issue 상태 확인,
`devel` sync, 정확한 local/remote review branch 정리를 순차 수행한다.
