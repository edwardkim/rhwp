# PR #1344 완료 보고서 — 수식 집합연산자 확대 렌더 수정

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1344 |
| 작성자 | `planet6897` |
| 관련 이슈 | #1342 |
| 검토 브랜치 | `local/pr1344-merge-test` |
| 통합 방식 | 현재 `devel` 기준 cherry-pick |
| PR 커밋 | `3328a73d` |

## 2. 처리 내용

PR #1344는 수식의 소형 집합연산자 `∩`/`∪`가 큰 연산자 경로를 타면서 본문보다 1.5배 크게
렌더되는 문제를 수정한다.

처리 내용:

- `src/renderer/equation/symbols.rs`에서 `UNION`, `SMALLUNION`, `CUP`, `INTER`, `SMALLINTER`, `CAP`을
  `BIG_OPERATORS`에서 `OPERATORS`로 이동
- `BIGCUP`, `BIGCAP`은 큰 연산자로 유지
- `test_set_operators_not_big` 회귀 테스트 추가
- contributor report/working 문서는 archive 정책에 맞춰 이동

## 3. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| CI | 통과 |
| CodeQL | 통과 |
| Render Diff | 통과 |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib test_set_operators_not_big` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --lib` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과 |

시각 판정:

| 항목 | 결과 |
|---|---|
| 16페이지 SVG export | 통과 |
| ∩/∪ 6개 `font-size="12.00"` 확인 | 통과 |
| rhwp-studio WASM 시각 판정 | 통과 |

## 4. 범위 외 확인

문24의 `rm P LEFT ( it A  B RIGHT )` 조건부 막대는 기대한 `P(A|B)` 형태로 출력되지 않는다.

판정:

- 이 문제는 PR #1344의 ∩/∪ 확대 수정과 별도 원인이다.
- 이미 #1343으로 등록된 `U+E04D(PUA) 조건부 막대 렌더` 문제와 동일하다.
- 이번 PR은 수용하고, 조건부 막대 렌더는 #1343 후속 처리로 분리한다.

## 5. 판정

**수용 가능**.

- `lookup_symbol`의 대소문자 조회 특성을 고려하면 단순 제거가 아니라 `OPERATORS`로 키를 이동한 방식이 맞다.
- ∩/∪가 본문 크기로 렌더되는 것을 SVG와 WASM에서 확인했다.
- 큰 연산자 `BIGCUP`/`BIGCAP`은 유지되어 범위 분리가 명확하다.

## 6. 남은 절차

1. `local/devel`에 PR 커밋과 문서 정리 커밋 통합
2. `origin/devel` push
3. PR #1344에 메인테이너 코멘트 작성
4. PR #1344 종료
5. 이슈 #1342 close 확인
