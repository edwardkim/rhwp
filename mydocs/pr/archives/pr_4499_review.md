---
kind: pr_review
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4499 검토 - HWPX 차트의 HWP5 fallback OLE 변환

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `visual_fixture_evidence.md`, `rework_and_exceptions.md`

누적 체리픽과 검토 순서는
[`pr_4366_4499_review_impl.md`](pr_4366_4499_review_impl.md)에 기록한다.

## 범위와 기준점

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4499](https://github.com/edwardkim/rhwp/pull/4499) / @johndoekim |
| 원 source head | `e34e6d8b1e34dc364165fc82140884f8fd182e7a` |
| 기준 `upstream/devel` | `8dbe982e89e780fe0612a1bc66aa417bbd6356b2` |
| 누적 검토 브랜치 | `review/johndoekim-20260811` |
| 원 변경 | 7파일, `+1957/-3`, 11 commits |
| 원 PR 상태 확인 | open, mergeable, 원 source head의 required checks 성공 |

HWPX의 `<hp:switch>` 차트 branch가 만드는 가상 `BinData` 참조를 HWP5에서 직접 쓸 수 있는
`<hp:default>` fallback OLE로 접는다. 이 변경은 dangling reference와 차트 소실을 동시에
제거하고, wasm HWP export가 라이브 IR을 변형하지 않도록 snapshot 경로도 사용한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| 원 source head와 GitHub required checks 대조 | `e34e6d8`에서 성공, mergeable 확인 |
| 차트 HWPX -> HWP 변환 | `samples/chart/세로막대형/묶은세로막대형.hwpx`, `rhwp convert --verify` 성공, IR 차이 없음 |
| HWP 2020 PDF - 차트 원본 | 1쪽, 5,860 bytes, 본문 검증 성공 |
| HWP 2020 PDF - rhwp HWP 변환본 | 1쪽, 5,861 bytes, 본문 검증 성공 |
| 144 DPI 1쪽 래스터 SHA-256 | 원본/변환본 동일: `6ff074…c67` |
| 누적 전체 | `cargo nextest run --cargo-profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --tests --test-threads 12 --no-fail-fast`: **5,730 passed, 7 slow, 36 skipped**, 437.285s |

한컴 2020에서 원본과 변환본을 각각 PDF로 출력한 뒤 첫 쪽 144 DPI 래스터를 비교했다. 두 PNG의
SHA-256이 같고, 묶은 세로 막대 3계열, 4개 범주, 제목, 범례, 축 눈금이 모두 존재한다. 증적은
[원본](../assets/pr_4499_chart_source_p001_hancom2020.png)과
[변환본](../assets/pr_4499_chart_rhwp_p001_hancom2020.png)에 보존했다.

## 권고

차트 fallback OLE 채택, BinData 참조 정합, wasm snapshot 저장, 실제 한컴 렌더까지 일관되게
검증됐다. PR #4366의 메인터너 보정과 누적 검토에서 충돌이 없었으며, 최신 source head의 required
checks가 계속 성공 상태인 것을 merge 직전에 다시 확인하는 조건으로 **merge 권고**한다. 현재
단계에서는 GitHub push, comment, merge를 수행하지 않았다.
