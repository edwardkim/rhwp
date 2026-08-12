---
kind: report
status: active
last_verified: 2026-08-11
---

# PR #4598 검토 — gym T12 HWPX 형식·판정 계약 보정

## 라우팅

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md, pr_review/review_only_fast_pass.md
current head: 4f24c73e97b39d577a398c4e06d535060f5f35de (접수 시점 참고)
```

## Metadata

| 항목 | 접수 시점 참고값 |
| --- | --- |
| PR | [#4598](https://github.com/edwardkim/rhwp/pull/4598) |
| 작성자·self-review 담당 | `edwardkim` |
| base / head | `devel` / `task/4586-gym-t12-hwpx` |
| 관련 이슈 | `Closes #4586` |
| 규모 | 24개 파일, +937/-26, 6 commits |
| 상태 | Open, non-draft, `MERGEABLE/BLOCKED` |
| 1차 트리야지 | assignee `edwardkim`, milestone `v1.0.0`, labels `bug`·`documentation`·`rust`·`hwpx`·`ci`·`github_actions`·`harness`·`cli`·`test`·`python` |

작성자 본인은 GitHub에서 자기 PR의 requested reviewer나 `APPROVE` 대상이 될 수 없다. 작업지시자의
self-review 지시에 따라 별도 reviewer request 없이 최신 head에 `COMMENTED` review로 검토 결과를
게시한다.

## 변경 범위

이 PR은 gym T12가 실제 HWPX 대신 HWP5 위장 파일을 만점 처리한 두 계약 결함을 함께 닫는다.

1. `convert`는 HWP5 전용 명령으로 유지하고 `.hwp` 이외 출력 경로를 입력 IO 전에 exit 2로 거부한다.
2. T12는 `export-hwpx`를 안내하고 `info`의 실제 `format == hwpx`를 먼저 검사한다.
3. scorer는 기존 `expect_exit`을 보존하면서 복수 `expect_exits`를 지원한다. 허용된 exit 3의 JSON
   봉투도 `answer_eq`·`value_eq` 판정까지 진행한다.
4. 실제 HWPX focused 결과 `identical:false`를 기준 답안과 검증 메타데이터에 고정한다.
5. 생성 Codex 교본, 판단 트리, CLI·gym 문서와 GitHub Actions의 Python 계약 테스트를 정합화한다.

renderer, layout, paint, WASM API와 rhwp-studio는 바뀌지 않았다. HWP/HWPX/PDF fixture도 추가·교체하지
않았으므로 시각·fixture 증적 보조 경로는 적용하지 않는다.

## Self-review finding

blocking finding은 없다.

- `convert`가 출력 확장자를 보고 명령을 자동 전환하지 않아 HWP5/HWPX 책임 경계가 유지된다.
- 출력 계약을 입력 IO보다 먼저 검사하고, exit 2·빈 stdout·산출물 없음·대체 명령 안내를 회귀 테스트가
  함께 고정한다.
- `expect_exits`는 비어 있지 않은 정수 배열만 허용하며, 없으면 기존 `expect_exit` 단일 값 또는 기본값
  0으로 되돌아가 기존 과제와 호환된다.
- T12는 형식 검사를 IR 비교보다 먼저 수행하므로 HWP5 위장 제출은 동등성 값과 무관하게 실패한다.
- 기준선은 1호 선수 14과제 전체를 다시 실행했다고 가장하지 않고 T12 focused 실행의 version, commit,
  capabilities·입력·산출 해시를 별도 `verification.json`에 기록한다.
- 생성기의 plan 출력 경로를 저장소 상대경로로 바꾼 것은 checkout 절대경로가 `planSha256`을 바꾸던 기존
  비결정성을 제거하며, 연속 `--check` 변경 0으로 확인했다.

전체 회귀의 첫 실행에서 기존 `cli_exit_codes::unreadable_input_reports_runtime_failure`가 한 건 실패했다.
입력 파일 없음 계약을 검증하면서 `convert` 출력에 `.hwpx`를 주던 잘못된 전제가 원인이었다. 제품 코드를
완화하지 않고 `convert`에는 실제 `.hwp`, `export-hwpx`에는 실제 `.hwpx` 임시 경로를 분리했으며, 단일
재검증과 전체 재실행으로 닫았다.

## 검증

- T12 실제 focused 판정: 2/2 checks passed
- `issue_4586_gym_t12_contract`: 3 passed / 0 failed
- `scripts/tests/test_gym_score.py`: 5 passed / 0 failed
- 기존 convert output axis와 #1638 verify 계약: 각 1 passed / 0 failed
- workflow contract wiring: 3 passed / 0 failed
- agent Codex contract: 2 passed / 0 failed
- 생성 교본 검사: 명령 83 · 실측 18 · 계약만 65 · 변경 0
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --all -- --check`, `git diff --check`: 통과
- release-test 전체 재실행: 5,763 passed / 0 failed / 36 skipped / 6 slow, 221.286초
- 계획·단계·최종 보고서·오늘할일 7개 문서의 내부 Markdown 상대 링크: 이상 없음
- 최신 `upstream/devel` `a70797db4`와 접수 head의 merge tree `42d1ac45a647eaf08936cf876d651acf7105aa64`:
  충돌 없음, merge tree `git diff --check` 통과
- initial code candidate `4f24c73e9`의 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/31487755505),
  [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/31487755332),
  [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/31487755346): 모두 성공. CodeQL은
  JavaScript/TypeScript·Rust·Python 분석을 통과했고 Render Diff는 Canvas visual diff를 통과했다.

전체 회귀는 `cargo-nextest 0.9.137`에서 성공했다. 저장소 권장 `0.9.140`보다 낮다는 경고가 있었지만
실행과 집계는 정상 완료됐다. 로컬 `actionlint`는 설치되어 있지 않아 실행하지 못했으며, workflow 계약
테스트와 initial code candidate의 GitHub lint·CI 결과로 보완한다.

렌더 영향이 없어 Native Skia·Docker WASM·브라우저·시각 검증은 적용하지 않았다.

## Review-only 후속과 최종 권고

이 문서와 오늘할일만 initial code candidate 뒤의 single-parent trailing review-only commit으로 추가한다.
initial candidate의 Full CI가 성공한 뒤 push해 review-only fast-pass A가 같은 PR의 녹색 code candidate를
재사용할 수 있게 한다.

최신 review-only head의 preflight·required aggregate와 mergeability가 성공하면 blocking finding 없이
merge를 권고한다. self-review 결과는 `COMMENTED` review로 게시하며 실제 merge는 작업지시자의 별도
승인을 조건으로 한다.
