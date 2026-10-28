---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7298 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7298](https://github.com/edwardkim/rhwp/pull/7298), `planet6897`, head `8c9f9e925a863400b51f5480e1444c9c1caebca6`; 통합 적용 `04404a07b`.
- 범위: 저장 vpos가 본문 상단 띠로 되감길 때 줄 단위 분할도 쪽 경계로 처리한다. 같은 쪽 안의 부분 후퇴에는 적용하지 않는 반례를 함께 둔다.

## 검증

새 회귀는 시장구조조사 p4의 본문 밖 두 줄이 p5 첫 두 줄로 이동하는지와, 교육과정 지도 문서의 부분 후퇴가 새 쪽을 만들지 않는지를 고정한다. fresh macOS Native/WASM Visual Sweep의 p4·p5는 구조 이상 0쪽이다. PDF Creator는 Hwp 2022이고, p4·p5의 문단 순서와 쪽 경계를 직접 확인했다.

- [시장 p4 review](../assets/pr7310_review/market_review_004.png) · [시장 p4 overlay](../assets/pr7310_review/market_overlay_004.png)
- [시장 p5 review](../assets/pr7310_review/market_review_005.png) · [시장 p5 overlay](../assets/pr7310_review/market_overlay_005.png)
