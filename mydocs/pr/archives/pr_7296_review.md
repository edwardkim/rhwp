---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7296 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7296](https://github.com/edwardkim/rhwp/pull/7296), `planet6897`, head `c7e20b28b64dd41aaf516e473e1d41bc0d2bdf2d`; 통합 적용 `263ec3613`.
- 범위: tab leader 시작점을 앞 글자 뒤 0.25em으로 제한하고, 오른쪽 정렬된 쪽번호와의 뒤 간격은 기존 규칙을 유지한다.

## 검증

새 회귀는 KTX 목차의 첫 점끌이 시작 x를 PDF 기준 219.07px에 고정한다. fresh macOS Native/WASM의 KTX p2에서 항목 순서·점끌이·쪽번호·테두리를 직접 확인했고 구조 이상은 0쪽이다. 비교 PDF Creator는 Hwp 2022다.

- [KTX p2 review](../assets/pr7310_review/ktx_review_002.png) · [KTX p2 overlay](../assets/pr7310_review/ktx_overlay_002.png)
