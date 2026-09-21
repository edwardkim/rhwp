---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7286 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7286](https://github.com/edwardkim/rhwp/pull/7286), `planet6897`, head `c656f32cb5b1d8b9390f91fdc68e7b940160ad40`.
- 통합: 최신 `upstream/devel` `a74733edb` 위에 `git cherry-pick -x`로 적용한 `a62720de2`.
- 범위: 저장 vpos 사다리가 표의 **외곽 여백 상자 전체**를 비운다는 증거가 있을 때만 표 윗변을 외곽 상자 기준으로 둔다. vertical offset·불완전 사다리·본문 밖 좌표는 기존 흐름 배치를 유지한다.

## 검증

- manifest 준비는 48/48, Windows `release-test` focused는 통과했다.
- fresh macOS Native와 fresh WASM package로 `samples/hwpctl_API_v2.4.hwp` p17을 한컴 PDF와 비교했다. 이 HWP는 저장 메타데이터상 Hancom Office 2018이며, 비교 PDF의 Creator는 Hwp 2022다. PDF 라벨의 `2020`만으로 생성 엔진을 단정하지 않았다.
- Visual Sweep은 p17/64/94 전체에서 구조 이상 0쪽, 평균 pixel match 95.98682%였다. p17의 표 윗변·행 구조를 직접 판독했다. 글꼴 raster 차이로 잉크 보조 지표는 25.32002%이며, 이를 전체 시각 일치 판정으로 쓰지 않는다.

- [p17 review](../assets/pr7310_review/hwpctl_review_017.png) · [p17 overlay](../assets/pr7310_review/hwpctl_overlay_017.png)
