---
kind: pr-review
status: code-ci-running
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4914 검토 — 중첩 표 조각이 쪽 위에서 사라지는 렌더 회귀

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4914](https://github.com/edwardkim/rhwp/pull/4914), @planet6897 |
| 원 head | `a6e7a1434465f5f767aecfc96640ce7f769bd202` |
| 통합 적용 | `8b644a870`, `a8f0cc20d` |
| 기준 | `upstream/devel@ae5f2a345` |

첫 가시 조각 보정이 flow의 가시 내용을 전부 소비한 경우에는 origin 보정을 적용하지 않도록 경계 조건을
추가했고, 음수 y 영역 노드를 `dump-extents`가 보고하도록 진단을 보완했다. 회귀 테스트는 두 번째 페이지의
표 조각이 양의 영역에 남고 100개를 넘는 가시 run을 갖는지 확인한다.

## 시각 증빙

- 입력: `samples/issue4889/18098267_nested_fragment_origin.hwp`
  (`sha256: ed1c104ee32fc19bec82efdb6c31b1e2b65a634f80bfc9408fdd930b37cc7f3c`)
- 기준: [Hancom Office 2020 PDF](../../../pdf/issue4889/18098267_nested_fragment_origin-2020.pdf), 3쪽
  (`sha256: 25e545419bbaeb8f17d73a3c923b2548e8852b6e00694142e82f998825594718`;
  MCP 변환 `status=success`, `server.run_status=0`, `server.validation=ok`)
- rhwp와 기준 PDF의 2쪽 대조 결과: [대표 비교 PNG](../assets/pr_4914_issue4889_p2_fidelity.png), pixel diff 20.4%.
  폰트·텍스트 레이어 차이 때문에 완전 동등 판정으로 쓰지 않으며, 이 PR의 수용 근거는 표 조각과 가시 내용이
  통째로 사라지지 않는다는 회귀 계약이다.

통합 PR [#4936](https://github.com/edwardkim/rhwp/pull/4936)의 최초 코드 후보 CI는 녹색이었다. 최신 devel
동기화 뒤 docs head의 필수 CI와 head 동일성을 다시 확인하면 **수용 가능**이다.
