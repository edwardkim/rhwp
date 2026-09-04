# PR #1337 완료 보고서 — 미주 다단 초과 바운드 회귀 가드

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1337 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1336 |
| 검토 브랜치 | `local/pr1337-merge-test` |
| 통합 방식 | 현재 `devel` 기준 cherry-pick |
| PR 커밋 | `33018447` |

## 2. 처리 내용

PR #1337은 `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwp` 22쪽 미주 2단 영역의
잔여 body 초과 현상에 대해, 미주 다단 fit/accumulation 로직을 직접 수정하지 않고 기존
`issue_1082` 회귀 메트릭에 2024 변형 샘플을 추가한다.

처리 내용:

- `tests/issue_1082_endnote_multicolumn_drift.rs`에 2024 구분선 상/하 20mm 변형 케이스 추가
- 기존 `REG_LIMIT_PX = 60.0` 바운드 안에서 잔여 초과를 추적
- 미주 다단 경로의 exam 샘플군 하드튜닝은 이번 PR에서 변경하지 않음
- contributor 작업 문서 4개는 archive 정책에 맞춰 이동

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| CI | 통과 |
| CodeQL | 통과 |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --test issue_1082_endnote_multicolumn_drift` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --release --test issue_1082_endnote_multicolumn_drift` | 통과 |

## 4. 판정

**수용 가능**.

- 이번 PR은 #1336의 근본 레이아웃 정정이 아니라, 현재 프로젝트 허용 한계 내 잔여 초과를
  회귀 가드로 고정하는 PR이다.
- 미주 다단 fit/accumulation 경로가 fragility와 exam 샘플군 하드튜닝을 포함하므로, 이번
  범위에서 소스 로직을 건드리지 않는 판단은 보수적으로 타당하다.
- 향후 수백 px 단위의 재회귀는 기존 `issue_1082` 계열 테스트에서 감지된다.

## 5. 남은 절차

1. `local/devel`에 PR 커밋과 문서 정리 커밋 통합
2. `origin/devel` push
3. PR #1337에 메인테이너 코멘트 작성
4. PR #1337 종료
5. #1336 close 상태 재확인
