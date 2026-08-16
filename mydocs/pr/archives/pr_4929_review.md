---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4929 검토 - SWS 감사기의 strategist 게이트 연계

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4929](https://github.com/edwardkim/rhwp/pull/4929) |
| 작성자 / source | @kevin9327 / `feat/sws-strategist-gate` |
| 원 source head | `4af3d1a27c7ecf66c013e756e170f2bd2f35cfad` |
| 기준 devel | `d9f04c6eec1f` |
| 가시성 검토 branch | `review/kevin9327-20260816` |
| local 적용 commit | `c2c36e8bb` |
| 메인터너 보정 | `2d17db4c2` |
| 원 PR 상태 참고값 | 작성 시점 `OPEN` / `MERGEABLE`; merge 직전에 재확인 필요 |

`engagement.py --validate`에 SWS/1.0 감사기를 연결해 corpus·근거 대장·산출물 골격을 검증한다.

## 메인터너 보정

상대 corpus 출력 경로를 한 번 조합한 뒤 다시 상대 경로로 읽으면 base 경로가 중복 결합될 수 있었다.
`2d17db4c2`은 결합 직후 `resolve()`로 절대 경로를 고정하고, 재읽기 경로가 절대 경로임을 회귀로 검증한다.
이 보정은 SWS 판정 규칙을 바꾸지 않고 relative root 입력에서의 결과 파일 재사용만 결정적으로 만든다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| strategist 계약 | `python3 -m unittest scripts.tests.test_automation_tool_contracts` | 33 passed (공유 도구 계약 포함) |
| 자체 검사 | `python3 tools/strategist/sws_audit.py --self-check` | 문제 0건 |
| 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,479 passed, 38 skipped |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` | 통과 |

Python 자동화·문서 계약만 바꾸며 renderer 출력은 변경하지 않는다. 따라서 PDF/픽셀 대조는 적용하지 않았다.

## 판단

상대 경로 재읽기 결함을 차단했고, 자체 검사와 계약 시험이 모두 통과했다. **메인터너 보정을 포함해 통합 수용 권고.**

구현·적용 순서는 [메인터너 보정 기록](pr_4929_review_impl.md)에 남긴다.
