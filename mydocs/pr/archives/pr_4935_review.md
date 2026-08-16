---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4935 검토 - 검증 실패 자동 재시도 루프

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4935](https://github.com/edwardkim/rhwp/pull/4935) |
| 작성자 / source | @kevin9327 / `feat/repair-loop` |
| 원 source head | `c7cbe757cc9464c50af8e19c8ac105bbb33c0ea4` |
| 기준 devel | `82f28ae86441` |
| 가시성 검토 branch | `review/kevin9327-20260816` |
| local 적용 commit | `9a9a49ab9` |
| 메인터너 보정 | `2656276fc` |
| 원 PR 상태 참고값 | 작성 시점 `OPEN` / `MERGEABLE`; merge 직전에 재확인 필요 |

검증 실패에 대해 locate·diagnose·repair·re-verify를 순서대로 수행하고, 최대 횟수·진전 없음·루프를
안전하게 중단하는 자동화 루프를 추가한다.

## 메인터너 보정

회귀 시험의 가짜 `rhwp` 실행 파일이 Windows `.bat`로만 만들어져 Linux에서는 spawn 단계에서 실패했다.
`2656276fc`은 Windows는 기존 `.bat`를, Linux/macOS는 실행 가능한 POSIX shell 래퍼를 만들도록 바꿨다.
테스트 대상 루프의 정책은 바꾸지 않고, 동일한 orchestration 안전장치를 모든 개발·CI 환경에서 실행하게 한다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| repair-loop 및 자동화 계약 | `python3 -m unittest scripts.tests.test_automation_tool_contracts scripts.tests.test_repair_loop` | 33 passed |
| 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,479 passed, 38 skipped |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` | 통과 |

Python 오케스트레이션 변경으로 renderer·문서 바이트 출력은 바뀌지 않아 시각 대조는 적용하지 않았다.

## 판단

Linux에서 실제로 실행되지 않던 회귀 시험을 OS별 래퍼로 보정했고 33개 Python 계약 시험이 통과했다.
**메인터너 보정을 포함해 통합 수용 권고.**

구현·적용 순서는 [메인터너 보정 기록](pr_4935_review_impl.md)에 남긴다.
