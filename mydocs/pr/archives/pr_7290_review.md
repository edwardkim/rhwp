---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7290 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7290](https://github.com/edwardkim/rhwp/pull/7290), `planet6897`, head `7b70158f7e153ed71c30b7a5bb4eeaa719a076a6`.
- 적용: `7f4e76be0`. 원 history에 #7286 변경이 포함되어 있어 해당 선행 커밋은 중복 체리픽하지 않았다.
- 범위: 빈 host의 Para-relative·left/inside-aligned Square 표에만 위쪽 `outMargin`을 반영한다. 가시 host와 다른 wrap·정렬은 제외해 기존 흐름 소유를 보존한다.

## 검증

- 새 회귀는 다섯 hwpctl 표의 정본 괘선 좌표와 가시-host 반례를 함께 고정한다.
- fresh macOS Native/WASM Visual Sweep의 hwpctl p64·p94에서 표 구조와 주변 문단 흐름을 확인했다. 세 선택 쪽 모두 구조 이상 0쪽이다.
- PDF Creator는 Hwp 2022이며, 문서 저장 메타데이터와 PDF 파일명으로 엔진을 혼동하지 않았다.

- [p64 review](../assets/pr7310_review/hwpctl_review_064.png) · [p64 overlay](../assets/pr7310_review/hwpctl_overlay_064.png)
- [p94 review](../assets/pr7310_review/hwpctl_review_094.png) · [p94 overlay](../assets/pr7310_review/hwpctl_overlay_094.png)
