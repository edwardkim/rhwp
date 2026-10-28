---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6155 검토 - 어울림 문단의 표 하단 흐름 전진 보정

## 접수 메타데이터

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 작성자 | [#6155](https://github.com/edwardkim/rhwp/pull/6155) / [@planet6897](https://github.com/planet6897) |
| 원 PR head | `3083225beb2b9344716049156a8c595d6130c042` |
| 관련 issue | [#6128](https://github.com/edwardkim/rhwp/issues/6128) |
| 통합 PR | [#6159](https://github.com/edwardkim/rhwp/pull/6159) |
| 코드 후보 | `a6616441b21c6127999039419625a622f86db982` |
| 작성 시점 상태 | 코드 후보 CI 성공, 이 trailing 기록 head의 CI 대기 |

## 변경과 판단

- 어울림 문단이 표 하단보다 더 내려간 경우, 표 기준 advance가 아니라 실제 문단 하단까지 흐름 전진을 반영한다.
- `issue_6128_wraparound_para_flow_advance`가 7쪽 fixture의 4쪽에서 wraparound 뒤 본문이 표와 겹치지 않고 계속 아래로 흐르는지 검증한다.

## 시각 증적

- 기준 문서: `samples/issue6128/156653004_privacy_day_ceremony.hwpx`
- HWP 2024 MCP client, `--engine 2020`, Hancom `12.0.0.4605`으로 생성한 [기준 PDF](../assets/pr_6159/baseline/issue6128/issue6128-hwp2020.pdf)를 보관했다.
- RHWP와 PDF는 모두 7쪽이며, 쟁점 4쪽의 [review 이미지](../assets/pr_6159/visual-sweep/issue6128-p4/issue6128-p4/review/review_004.png), SVG, render tree, overlay를 보관했다.
- 자동 분석은 flagged page가 없었다. proxy는 참고값으로만 두고, wraparound 이후 본문과 표 흐름을 수동 확인했다. 세부 해시는 [증적 매니페스트](../assets/pr_6159/EVIDENCE.md)와 [SHA-256 목록](../assets/pr_6159/SHA256SUMS)에 있다.

## 검증

- 관련 회귀 네 건, 전체 integration 8,401건, Native Skia lib test, WASM build를 통과했다.
- 코드 후보 CI는 [run 32986377307](https://github.com/edwardkim/rhwp/actions/runs/32986377307)에서 성공했다. PR 이벤트 run이 생성되지 않아 동일 head의 workflow_dispatch 회귀 모드를 사용했음을 구분해 기록한다.

## 최종 판정

**수용 권고, trailing CI 대기.** 최신 문서와 증적 head가 통과하면 #6159 merge 후 원 PR과 #6128에 통합 결과를 남긴다.
