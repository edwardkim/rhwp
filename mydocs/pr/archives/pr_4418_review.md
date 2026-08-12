---
kind: pr-review
status: local-accept-recommendation
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4418 리뷰 - HWPCTRL 저장 차분 관측·표/개체 조작 보강

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4418](https://github.com/edwardkim/rhwp/pull/4418) · @planet6897 |
| base / 원 head | `devel` / `2659b8eb0c6288080f77d7b06844b7b497f9e400` |
| 규모 | 95 files, `+9,508/-312`, 20 commits |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 범위와 검토

저장본 차분을 관측 가능한 HWPCTRL 오라클로 승격하고, 개체 z-order/flip, 표 크기 조절·셀 삭제,
캐럿·폰트 메트릭의 구현과 탐침을 추가한다. 문서화만으로 관측 가능성을 과장하지 않도록
`ir-sweep`, 저장 자취 시나리오, HWP5 직렬화 보정과 golden을 함께 검토했다.

누적 검토에서 `npm --prefix npm/hwpctrl-ocx run test:contract` 8건, Python harness contract 28건,
전체 HWPCTRL package gate가 성공했다. `ir-sweep`는 동일 `hwpx_sample2.hwpx` 입력에서
`identical=true`, `diffCount=0`을 반환했다.

## 판정

저장 자취를 실제 비교 대상으로 만든 구현·회귀 자료·문서 설명이 서로 일치한다. 별도 차단 결함은
발견하지 못했다. #4483이 이 PR을 선행으로 누적하므로 둘은 같은 통합 PR에서 순서를 보존해 반영한다.

**최종 권고: 최신 통합 head의 CI와 작업지시자 승인 후 수용.**
