# PR #1344 검토 - 수식 집합연산자 확대 렌더 수정 (#1342)

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1344 |
| 작성자 | `planet6897` |
| 상태 | open / draft 아님 |
| base | `devel` |
| head | `fix/equation-set-operator-size-1342` (`3328a73d`) |
| 관련 이슈 | #1342 |
| 변경 규모 | 5 files, +242 / -7 |

## 2. 변경 요약

문제:

- `samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx` 16페이지 문24의
  `P(A∩B)`, `P(A∪B)` 등에서 ∩/∪가 본문보다 1.5배 크게 렌더됨.
- 큰 연산자 경로를 타면서 기호 뒤쪽에 불필요한 가로 간격도 생김.

근본 원인:

- 소형 이항 집합연산자 `UNION`, `SMALLUNION`, `CUP`, `INTER`, `SMALLINTER`, `CAP`이
  `BIG_OPERATORS`에 등록되어 있었다.
- 파서가 토큰을 대문자화한 뒤 `is_big_operator()`를 먼저 확인하므로, 해당 토큰이
  `parse_big_op()` → `layout_big_op()` 경로를 타고 `BIG_OP_SCALE = 1.5` 적용을 받았다.

수정:

- 위 6개 키를 `BIG_OPERATORS`에서 제거하고 `OPERATORS`로 이동.
- `BIGCUP`, `BIGCAP`은 큰 연산자로 유지.
- `test_set_operators_not_big` 회귀 테스트 추가.

## 3. GitHub 상태

- PR head: `3328a73dea4ef62d3278ffac7d5524574efa96a8`
- PR base SHA: `1b286dded93c63481b7ee6c736edca7fc8c1c41b`
- 현재 로컬 `devel`: `05fb73ca`
- PR base가 현재 `devel`보다 오래되어, 현재 `local/devel` 기준 검토 브랜치에서 PR 단일 커밋을 cherry-pick해 검토했다.

GitHub Actions:

| 체크 | 결과 |
|---|---|
| CI | success |
| CodeQL | success |
| Render Diff | success |

관련 이슈:

- #1342는 open 상태.
- PR 본문에 `closes #1342` 포함.

## 4. 로컬 적용 방식

검토 브랜치:

```text
local/pr1344-merge-test
```

적용 커밋:

```text
af3f87bc Task #1342: 수식 집합연산자(∩/∪) 1.5배 확대 렌더 수정
```

변경 파일:

- `src/renderer/equation/symbols.rs`
- `mydocs/plans/archives/task_m100_1342.md`
- `mydocs/plans/archives/task_m100_1342_impl.md`
- `mydocs/report/task_m100_1342_report.md`
- `mydocs/working/task_m100_1342_stage2.md`

## 5. 로컬 검증

실행 완료:

```bash
cargo fmt --all -- --check
cargo test --lib test_set_operators_not_big
cargo clippy --lib -- -D warnings
cargo run --bin rhwp -- export-svg 'samples/3-09월_교육_통합_2024-구분선아래20구분선위20.hwpx' -o output/poc/pr1344-set-operator -p 15 --debug-overlay
cargo test --lib
```

결과:

| 명령 | 결과 |
|---|---|
| `cargo fmt --all -- --check` | 통과 |
| `cargo test --lib test_set_operators_not_big` | 통과 |
| `cargo clippy --lib -- -D warnings` | 통과 |
| `cargo test --lib` | 1616 passed, 0 failed, 6 ignored |
| 16페이지 SVG export | 통과 |

검증 산출물:

- `output/poc/pr1344-set-operator/3-09월_교육_통합_2024-구분선아래20구분선위20_016.svg`

SVG 확인:

- ∩/∪ 6개 모두 `font-size="12.00"`으로 출력됨.
- `font-size="18.00"`로 남은 ∩/∪ 없음.

메인테이너 시각 판정:

- ∩/∪ 확대 렌더 수정은 통과.
- 다만 문24의 `rm P LEFT ( it A  B RIGHT )` 조건부 막대는 기대한 `P(A|B)` 형태로 출력되지 않음.
- 이 현상은 PR #1344 범위 밖으로 이미 별도 이슈 #1343에 등록된 `U+E04D(PUA) 조건부 막대 두부 렌더` 문제와 동일하다.
- 따라서 PR #1344의 수용 판정은 유지하고, 조건부 막대 렌더는 #1343 후속 처리로 분리한다.

## 6. 검토 의견

수정 방향은 타당하다.

- `lookup_symbol`이 case-sensitive 조회 후 대문자 fallback을 수행하므로, BIG에서 단순 제거하면
  `CAP`/`CUP` 같은 대문자 토큰이 기호로 매핑되지 않을 수 있다.
- PR처럼 `BIG_OPERATORS`에서 `OPERATORS`로 키를 이동하는 방식이 `is_big_operator()=false`와
  기호 매핑 보존을 동시에 만족한다.
- `BIGCUP`/`BIGCAP`을 큰 연산자로 유지한 것도 범위 분리가 명확하다.

주의점:

- `SQCUP`, `SQCAP`, `UPLUS`, `OPLUS`, `OTIMES` 등도 소형 연산자일 가능성이 있지만,
  PR 본문처럼 정답지 근거가 확보되지 않은 항목은 이번 범위에서 제외하는 것이 안전하다.
- `P(A|B)` 조건부 막대 `U+E04D(PUA)`가 두부/비정상 기호로 렌더되는 문제는 #1343으로 이미 분리되어 있으며,
  이번 PR의 ∩/∪ 확대 수정과는 별도 원인이다.
- `mydocs/report/task_m100_1342_report.md`, `mydocs/working/task_m100_1342_stage2.md`는 활성 폴더에
  추가되어 있으므로 최종 통합 시 archive 정책에 맞춰 이동하는 것이 좋다.

## 7. 권장 처리

권장: **수용 가능**.

권장 절차:

1. 작업지시자 시각 판정 요청.
2. 시각 판정 통과 시 현재 `devel` 기준 cherry-pick 커밋 통합.
3. 활성 `mydocs/report`, `mydocs/working` 문서를 archive로 이동.
4. 최종 검증 후 `devel` push.
5. PR #1344에 메인테이너 코멘트 작성.
6. PR #1344 종료 및 #1342 close 확인.

## 8. PR 코멘트 초안

```markdown
검토 완료했습니다.

소형 이항 집합연산자 `UNION`/`SMALLUNION`/`CUP`/`INTER`/`SMALLINTER`/`CAP`이 `BIG_OPERATORS`에 들어가 있어 `BIG_OP_SCALE=1.5` 경로를 타던 원인 분석과 수정 방향이 타당하다고 확인했습니다.

`lookup_symbol`의 case-sensitive 조회 특성상 단순 제거가 아니라 `OPERATORS`로 키를 이동한 것도 맞는 처리입니다. `BIGCUP`/`BIGCAP`을 큰 연산자로 유지한 범위 설정도 적절합니다.

로컬 검증:

- `cargo fmt --all -- --check`
- `cargo test --lib test_set_operators_not_big`
- `cargo clippy --lib -- -D warnings`
- `cargo test --lib`
- 16페이지 SVG export 및 ∩/∪ `font-size=12.00` 확인

GitHub Actions의 CI / CodeQL / Render Diff success도 확인했습니다.

메인테이너 시각 판정에서도 ∩/∪ 확대 렌더 수정은 통과했습니다. 문24의 `P(A|B)` 조건부 막대가 정상 출력되지 않는 문제는 이미 별도 이슈 #1343으로 분리된 `U+E04D(PUA)` 렌더 문제와 동일하므로 이번 PR 범위 밖 후속 처리로 보겠습니다.

수용 절차로 진행하겠습니다. 기여 감사합니다.
```
