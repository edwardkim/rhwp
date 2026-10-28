---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7297 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7297](https://github.com/edwardkim/rhwp/pull/7297), `planet6897`, head `9b012006322cfa38cfca4ab419297aae1957b550`; 통합 적용 `efb35e357`.
- 범위: `신명 중명조`의 **실측된 라틴 문자만** metric overlay에 추가한다. 한글, 공백, 미측정 문자는 기존 fallback을 유지한다.
- 기준 HWP는 private corpus 경로에서 검증에만 사용했고 저장소에 추가하지 않았다. 커밋된 PDF는 p4 기준 증적만 보존한다.

## 검증

새 검사로 E/U·숫자·점의 전진폭, 한글 불변, 미측정 문자 fallback, 다른 face 불변을 확인했다. fresh macOS Native/WASM으로 private source p4와 PDF p4를 비교한 결과 구조 이상 0쪽, pixel match 94.08621%였다. 1쪽 PDF를 4쪽 인덱스에 맞추기 위한 빈 앞쪽 wrapper는 `/tmp`에만 만들었고 커밋하지 않는다.

- [p4 review](../assets/pr7310_review/sinmyeong_review_004.png) · [p4 overlay](../assets/pr7310_review/sinmyeong_overlay_004.png)
