---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6153 검토 - 중첩 표 host 문단 줄 간격의 셀 컷 회계 보정

## 접수 메타데이터

| 항목 | 확인값 |
| --- | --- |
| 원 PR / 작성자 | [#6153](https://github.com/edwardkim/rhwp/pull/6153) / [@planet6897](https://github.com/planet6897) |
| 원 PR head | `1d15b56258e2330163d2e790aa80f2b1254aa582` |
| 관련 issue | [#6126](https://github.com/edwardkim/rhwp/issues/6126) |
| 통합 PR | [#6159](https://github.com/edwardkim/rhwp/pull/6159) |
| 코드 후보 | `a6616441b21c6127999039419625a622f86db982` |
| 작성 시점 상태 | 코드 후보 CI 성공, 이 trailing 기록 head의 CI 대기 |

## 변경과 판단

- HWP5 저장 ladder에서 실제 delta가 있는 nested host에만 `line_spacing`을 셀 컷 회계에 반영한다.
- native stored ladder에 대한 광범위한 wrapper fallback은 유지하지 않으며, 기존 HWPX reset/uniform 판정도 보존한다.
- `issue_6126_fragment_cut_overshoots_one_line`은 7쪽 문서의 3쪽에서 셀 소유 경계를 넘는 텍스트가 없음을 회귀로 고정한다.

## 시각 증적

- 기준 문서: `samples/issue6126/3171199_design_capability_criteria.hwp`
- HWP 2024 MCP client, `--engine 2020`, Hancom `12.0.0.4605`으로 생성한 [기준 PDF](../assets/pr_6159/baseline/issue6126/issue6126-hwp2020.pdf)를 보관했다.
- RHWP와 PDF는 모두 7쪽이며, 쟁점 3쪽의 [review 이미지](../assets/pr_6159/visual-sweep/issue6126-p3/issue6126-p3/review/review_003.png), SVG, render tree, overlay를 보관했다.
- 글꼴과 래스터 차이의 pixel/ink proxy를 합격 기준으로 사용하지 않았고, 표 셀 경계와 본문 흐름을 수동 확인했다. 세부 해시는 [증적 매니페스트](../assets/pr_6159/EVIDENCE.md)와 [SHA-256 목록](../assets/pr_6159/SHA256SUMS)에 있다.

## 검증

- 관련 회귀 네 건, 전체 integration 8,401건, Native Skia lib test, WASM build를 통과했다.
- 코드 후보 CI는 [run 32986377307](https://github.com/edwardkim/rhwp/actions/runs/32986377307)에서 성공했다. PR 이벤트 run이 생성되지 않아 동일 head의 workflow_dispatch 회귀 모드를 사용했음을 구분해 기록한다.

## 최종 판정

**수용 권고, trailing CI 대기.** 최신 문서와 증적 head가 통과하면 #6159 merge 후 원 PR과 #6126에 통합 결과를 남긴다.
