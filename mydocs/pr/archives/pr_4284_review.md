---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4284 검토 - HWPX resource id 할당 상한

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4284](https://github.com/edwardkim/rhwp/pull/4284) / `kevin9327` |
| 관련 이슈 | #4281 |
| base / source head | `devel` / `fd5ca343a3550db8bf4e2ee7742914dc117791f5` |
| 누적 적용 | `2e69e1321` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 1 file, +50 / -12 |

## 판정

**수용 권고.** `charPr`와 `paraPr` id가 `resize_with`의 길이를 결정하던 입력 증폭 경로에
HML과 같은 65,535 상한을 적용했다. oversized id는 기존 resource 처리 관례대로 경고 후
무시하며, 작은 조작 HWPX가 대량 할당으로 이어지지 않는다. huge id regression과 누적 Rust
5,499건이 통과했다.

시각 출력이나 fixture를 갱신하지 않았으므로 visual sweep은 대상이 아니다. merge 전 원격 상태를
다시 확인한다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 기록했다.
