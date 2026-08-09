---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4286 검토 - HWP3 drawing 재귀 깊이 상한

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4286](https://github.com/edwardkim/rhwp/pull/4286) / `kevin9327` |
| 관련 이슈 | #4285 |
| base / source head | `devel` / `3dc09023a779c69e6881d5ed4dc1c2512f3a88bb` |
| 누적 적용 | `278d1b297` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 1 file, +80 / -0 |

## 판정

**수용 권고.** 파일의 child bit가 만드는 HWP3 GSO 재귀에 명시적 depth bound가 없던 문제를
닫았다. cap 경계 test는 깊은 chain이 stack overflow 전에 parse error로 끝남을 확인한다. 전체
Rust 회귀 5,499건도 통과했다.

HWP3 parser 안전성 변경으로 renderer/fixture 출력은 직접 바꾸지 않는다. 실제 merge 전 최신
remote head와 required checks를 재확인한다. 공통 검증은 [통합 검토 계획](pr_4282_review_impl.md)에 있다.
