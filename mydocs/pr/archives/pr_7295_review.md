---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7295 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7295](https://github.com/edwardkim/rhwp/pull/7295), `planet6897`, head `b0c19d4f959846b9554bc4dad4b0d3dfe08f5b88`; 통합 적용 `4121aaf4f`.
- 범위: `body_overflow_baseline.tsv`의 기존 발생값만 현재 실측으로 낮춘다. 신규 발생이나 상한 증가는 없다.

## 검증

Windows `cargo test --profile release-test --test regression_suite_013 body_overflow_does_not_grow_partition -- --nocapture`가 16/16 통과했다(69.85초). 이 gate는 문서 로드/렌더 실패를 0으로 숨기지 않으며, baseline 없는 신규 발생 또는 증가를 실패로 처리한다. renderer 변경이 아닌 검증 원장 변경이므로 Visual Sweep은 비해당이다.
