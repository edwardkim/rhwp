---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4527 리뷰 - 셀 밖 가운데 정렬 말미 공백

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4527](https://github.com/edwardkim/rhwp/pull/4527) · @planet6897 |
| base / 원 head | `devel` / `92267cb942a8e0adb233f71bf8ee0299d3865ba4` |
| 규모 | 5 files, `+170/-83`, 1 commit |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 범위와 충돌 해소

셀 밖 Center 문단의 마지막 시각 공백을 정렬 폭에서 제외하고, TAC·셀 내부·밑줄 공백 경계를 보존한다.
`ir_field_sweep_baseline.tsv`은 #4520의 #4490 행과 #4527의 #4491 행이 충돌했으므로 양쪽 행을
모두 유지했다.

`center_trailing_ws_alignment` focused test와 golden/renderer focused test가 통과했다. HWP 2020 기준
38쪽 비교의 p9에서 표·서명 도식의 owner 쪽이 유지됐고, 구조 candidate는 없었다. 기준과 rhwp의
글꼴 굵기·줄폭 차이는 asset에서 보이지만 마지막 서명 줄의 잘못된 왼쪽 이동을 재현하는 흐름 회귀는 없다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
