# #7952 — 전체 회귀에서 확인한 CLI 등록 계약 보정

Issue: [#7952](https://github.com/edwardkim/rhwp/issues/7952)

## 첫 정식 검증 결과

대상 `fcb6526a3644462e4d12a685e960ce450909a330`, 기준 devel
`552514f423ec91f1b7171f6e6b0882a4efe034f2`의 별도
`/home/tsjang/rhwp-7952-verify` worktree에서 공용 target을 사용했다.

- focused CLI 125/125 PASS, format PASS.
- Native Clippy, WASM32 Clippy, workspace build, workspace all-target Clippy PASS.
- suite manifest의 base 비교 PASS.
- 전체 release-test: **10,450 PASS / 8 FAIL / 45 skipped**. 컴파일 10분 52초, 실행 727.744초.
  이 실패 상태로 push·PR을 진행하지 않았다.

## 실패 원인과 독립 기대값

실패 8개는 모두 스킬 계약의 `Cargo.toml` **명시적 `[[bin]]` 개수를 2로 고정한 검사**다.
기존 manifest는 명시 선언 2개 외에 자동 발견으로 총 26개의 루트 실행 타깃을 제공했다.
이번 승인된 설계는 자동 발견을 끄고 기존 역할을 다음 4개로 명시한다. 논리 실행 파일 수가
늘어난 것이 아니며 문서 조판·렌더링 실패도 아니다.

| 승인된 이름 | 진입점 |
| --- | --- |
| `rhwp` | `src/main.rs` |
| `rhwp-q` | `src/bin/rhwp-q/main.rs` |
| `rhwp-agent` | `src/bin/rhwp-agent/main.rs` |
| `font-metric-gen` | `src/tools/font_metric_gen.rs` |

이 기대값은 [승인된 구현 계획](../plans/task_m100_7952.md)의 역할·타깃 계약이다.
실패한 현재 값이나 구현 결과의 개수를 그대로 기준값으로 채택하지 않았다.

## 보정

다음 8개 case의 과거 개수 핀을 공용 `tests/support/root_binary_contract.rs` 검사로 교체한다.
기존 스킬 내용·참조 파일·이웃 스킬 보존 검사는 유지한다.

- `agent_security_sweep_skill_contract`
- `agent_codex_skill_contract`
- `agent_provenance_skill_contract`
- `agent_handoff_skill_contract`
- `agent_work_receipt_skill_contract`
- `agent_cli_skill_contract`
- `agent_contributor_skill_contract`
- `agent_strategist_skill_contract`

공용 검사는 승인된 **이름과 진입점의 정확한 집합** 및 `autobins=false`를 함께 확인한다.
개수가 같아도 진입점을 바꾸거나 자동 발견을 켜면 거부한다. 새 root binary를 등록해도 거부한다.
`rhwp_q_dispatch_contract`에 이 세 반례와 설명용 `[[bin]]` 주석을 허용하는 대조군을 추가했다.
검사 삭제·skip·전역 count 허용치 완화로 실패를 가리지 않는다.

## 다음 검증

보정을 일반 commit으로 고정한 뒤 같은 review worktree를 그 commit으로 전환하고,
CLI 및 8개 스킬 case의 focused, format·필수 세 Clippy·workspace build·suite 정책,
전체 release-test를 다시 수행한다. 제품 Rust 원본과 Cargo 설정은 첫 정식 검증 이후 바꾸지 않았다.
재검증 완료 전에는 이 단계의 보정을 통과로 기록하거나 PR을 만들지 않는다.

첫 실행의 원본 로그·종료 코드·명령은 ignored
`output/pr-review/cli-build-scope-20261016/`의 attempt 1 기록으로 보존한다.
