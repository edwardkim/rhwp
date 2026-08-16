---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-16
---

# PR #4937 검토 - 단일 장 레이아웃 이상탐지

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4937](https://github.com/edwardkim/rhwp/pull/4937) |
| 작성자 / source | @kevin9327 / `feat/render-anomaly-detector` |
| 원 source head | `12fe6e1e6c27a2503392ba1f8853c71b1f2c3968` |
| 기준 devel | `d9f04c6eec1f` |
| 가시성 검토 branch | `review/kevin9327-20260816` |
| local 적용 commits | `101b6d729`, `76bb06027`, `59ebfb1f7` |
| 메인터너 보정 | `2d17db4c2`, `9c139b118` |
| 원 PR 상태 참고값 | 작성 시점 `OPEN` / `MERGEABLE`; merge 직전에 재확인 필요 |

`layout-anomaly`는 단일 RenderTree에서 overflow·overlap·빈 쪽 가능성을 JSON으로 보고하고, strict 모드에서
확정 이상 신호를 exit 3으로 처리한다.

## 메인터너 보정

첫 보정 `2d17db4c2`은 agent quality profile·생성 매뉴얼에 명령을 등록하고, 구현 파일의 단위 시험을
하위 파일로 옮겨 1,000줄 제한을 지켰다. 두 번째 보정 `9c139b118`은 공개 JSON 명령으로서 누락된
provenance map, provenance 계약 스윕, agent knowledge map의 필드 사전과 MCP 도구 표기를 추가했다.
따라서 발견 가능성·비신뢰 콘텐츠 경계·문서 계약이 구현과 같은 명령 집합을 광고한다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 이상탐지 단위 | `cargo test --profile release-test --target-dir target/pr-review diagnostics::layout_anomaly --lib` | 13 passed |
| agent 계약 | `cargo test --profile release-test --target-dir target/pr-review --test agent_profile_router_contract --test agent_codex_contract` | 8 passed, 2 passed |
| provenance 계약 | `cargo test --profile release-test --target-dir target/pr-review --test provenance_contract provenance_map_covers_every_json_command -- --nocapture` 및 text-bearing 계약 | 각 1 passed |
| 지식 맵 계약 | `cargo test --profile release-test --target-dir target/pr-review --test knowledge_map_field_dictionary_contract -- --nocapture` | 2 passed |
| 생성 문서 | `python3 tools/gen_agent_codex.py --check` | 변경 0 |
| 전체 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 6,479 passed, 38 skipped, 7 slow, 323.542초 |
| 품질 | `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `git diff --check` | 통과 |

이 명령은 기존 RenderTree를 읽어 진단할 뿐 조판·renderer·paint 경로를 변경하지 않는다. 따라서 별도
PDF pixel sweep은 적용하지 않았고, 전체 회귀의 기존 renderer baseline으로 출력 회귀를 감시했다.

## 판단

CLI·MCP·agent 문서·provenance 계약을 함께 보정해 공개 명령의 발견·검증 경계를 완결했다.
**메인터너 보정을 포함해 통합 수용 권고.**

구현·적용 순서는 [메인터너 보정 기록](pr_4937_review_impl.md)에 남긴다.
