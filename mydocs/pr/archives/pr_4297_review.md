---
kind: pr-review
status: review-ready
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-09
---

# PR #4297 검토 - Python output 위치 인자 정합

| 항목 | 값 |
| --- | --- |
| PR / 작성자 | [#4297](https://github.com/edwardkim/rhwp/pull/4297) / `kevin9327` |
| 관련 roadmap | #3907 R61 D-1 |
| base / source head | `devel` / `39be46faf2af298022eb4d2e5df76971ca8d03a0` |
| 누적 적용 | `34d9b36ce` |
| 접수 참고 상태 | MERGEABLE / CLEAN, 2 files, +46 / -4 |

## 판정

**수용 권고.** Python `export_hwpx`와 `convert`의 output은 CLI 계약상 위치 인자이며 기존
`-o` 조립은 실제 호출을 usage error로 끝냈다. Node의 기존 argv 규칙과 대조해 Python wrapper를
맞췄고, output을 포함한 전체 argv regression을 추가했다. Python 전체 251 passed와 누적 Rust
회귀가 통과했다.

공개 binding API 변경이므로 실제 merge 전 최신 source·checks를 재확인한다. 공통 검증은
[통합 검토 계획](pr_4282_review_impl.md)에 있다.
