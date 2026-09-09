# PR #1337 검토 - 미주 다단 초과 바운드 회귀 가드 (#1336)

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1337 |
| 작성자 | `planet6897` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `fix/1336-endnote-overflow-guard` (`33018447`) |
| 관련 이슈 | #1336 |
| 변경 규모 | 5 files, +233 / -0 |

## 2. 변경 요약

PR #1337은 `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` 22쪽 미주 2단 영역에서
콘텐츠가 body 하단을 초과하는 현상(#1336)에 대해, 렌더링 로직을 직접 수정하지 않고 기존
`issue_1082` 미주 다단 드리프트 테스트에 2024 변형 샘플을 추가한다.

핵심 판단:

- 대상 문서의 초과량은 `issue_1082` 회귀 메트릭 기준 약 50.1px.
- 기존 허용 한계 `REG_LIMIT_PX = 60.0` 안에 있다.
- 미주 다단 fit/accumulation 경로는 exam 샘플군에 대한 하드튜닝이 많아, 현재 PR에서 일반화 수정을
  시도하면 2022/2023 미주 샘플 회귀 위험이 크다.
- 따라서 이번 PR은 근본 정정보다 바운드 회귀 가드를 추가해 추후 대형 회귀를 감지하는 방향이다.

## 3. GitHub 상태

- PR head: `33018447ad7778f9fd1ff0575826016d2198002f`
- PR base SHA: `1b286dded93c63481b7ee6c736edca7fc8c1c41b`
- 현재 로컬 `devel`: `d73f795b`
- PR base가 현재 `devel`보다 오래되어, 검토는 현재 `local/devel`에서 검토 브랜치를 만들고 PR 단일 커밋을 cherry-pick하여 진행했다.

GitHub Actions:

| 체크 | 결과 |
|---|---|
| CI | success |
| CodeQL | success |

관련 이슈:

- #1336은 이미 `completed`로 close됨.
- 이슈 코멘트에도 "근본 정정 보류 + 바운드 회귀 테스트" 결정이 기록되어 있다.

## 4. 로컬 적용 방식

검토 브랜치:

```text
local/pr1337-merge-test
```

적용 커밋:

```text
89a75133 test(endnote): 미주 다단 초과 바운드 회귀 가드 추가 (#1336)
```

변경 파일:

- `tests/issue_1082_endnote_multicolumn_drift.rs`
- `mydocs/plans/task_m100_1336.md`
- `mydocs/plans/task_m100_1336_impl.md`
- `mydocs/report/task_m100_1336_report.md`
- `mydocs/working/task_m100_1336_stage1.md`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --test issue_1082_endnote_multicolumn_drift
cargo clippy --all-targets -- -D warnings
cargo test --release --test issue_1082_endnote_multicolumn_drift
```

결과:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 5 passed |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --release --test issue_1082_endnote_multicolumn_drift` | 5 passed |

## 6. 검토 의견

수용 가능한 PR이다.

장점:

- 실제 미주 다단 로직을 건드리지 않아 회귀 위험이 낮다.
- 기존 `issue_1082` 회귀 메트릭에 동일 계열 2024 변형을 추가하므로 테스트 위치가 적절하다.
- 50.1px 수준의 잔여 초과를 즉시 고치지 않고 바운드로 추적한다는 결정이 코드 상태의 fragility와 맞다.

주의점:

- 이 PR은 #1336의 근본 레이아웃 문제를 해결하지 않는다.
- `REG_LIMIT_PX = 60.0` 안에서 현상을 허용하는 정책을 명시적으로 받아들이는 PR이다.
- contributor 작업 문서가 `mydocs/plans`, `mydocs/report`, `mydocs/working` 활성 폴더에 추가된다. 최종 통합 시 archive 정책에 맞게 이동하는 것이 좋다.

후속 권장:

- 미주 다단 fit/accumulation 캡의 일반화는 별도 대형 이슈/마일스톤으로 분리.
- 이번 PR은 바운드 회귀 가드로만 수용.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 승인 후 현재 `devel` 기준 cherry-pick 커밋을 통합.
2. contributor 작업 문서 4개는 archive 정책에 맞춰 이동.
3. 최종 검증 후 `devel` push.
4. PR #1337에 "근본 정정 보류 + 바운드 회귀 가드"로 수용했다는 메인테이너 코멘트 작성.
5. #1336은 이미 close 상태이므로 상태만 재확인.

## 8. PR 코멘트 초안

```markdown
검토 완료했습니다.

이번 PR은 미주 다단 fit/accumulation 로직을 직접 수정하지 않고, `issue_1082` 회귀 메트릭에 2024 변형 샘플을 추가해 60px 바운드 안에서 추적하는 방향으로 확인했습니다.

현재 대상 초과량이 기존 `REG_LIMIT_PX = 60.0` 이내이고, 미주 다단 경로가 exam 샘플군에 대한 하드튜닝을 많이 포함하고 있어 근본 수정을 별도 리팩터링 과제로 분리하는 판단은 타당하다고 봅니다.

로컬 검증:

- `cargo fmt --all -- --check`
- `cargo test --test issue_1082_endnote_multicolumn_drift`
- `cargo clippy --all-targets -- -D warnings`
- `cargo test --release --test issue_1082_endnote_multicolumn_drift`

GitHub Actions의 CI / CodeQL success도 확인했습니다.

메인테이너 측에서는 이 PR을 "근본 정정 보류 + 바운드 회귀 가드 추가"로 수용하는 방향으로 처리하겠습니다. 기여 감사합니다.
```
