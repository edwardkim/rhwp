# Task M100 #4135 Recovery R1 — 누락된 RED 계약 결과

- **기준**: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../../plans/archives/task_m100_4135_impl.md)
- **승인**: 작업지시자가 Recovery 명칭 정정 뒤 `진행해줘.`로 Recovery 계획과 R1 착수를 승인
- **판정**: 의도한 RED 확정, Recovery R2 결과 승인 전 미착수

## 1. R1 범위

제품 코드는 변경하지 않았다. 다음 누락 계약만 테스트로 고정했다.

1. 오른쪽 빈 결과 열을 포함한 선택은 각 행의 선택 범위 합계 job을 만든다.
2. 아래 빈 결과 행을 포함한 선택은 각 열의 선택 범위 합계 job을 만든다.
3. Z 다음 열은 `AA`, `AB` 표기로 계산식과 코어 평가기 양쪽에서 표현한다.
4. 결과 가장자리 없음, 양축 모호, 단일, 불연속, 병합, 중첩 선택은 fail-closed한다.
5. 일부 dry-run 실패 시 write 없이 전체 job을 거절한다.
6. 영문·한글·`Process`의 물리 `KeyS`/`KeyM`은 IME 조기 반환 전에 셀 나누기/합치기로
   해석하고 Ctrl/Meta/Alt가 있으면 소유하지 않는다.

## 2. 변경 파일

| 파일 | 종류 | 내용 |
| --- | --- | --- |
| `rhwp-studio/tests/issue-4135-block-calculation-plan.test.ts` | 신규 테스트 | 가로/세로 job, AA/AB, 거절 조건, dry-run preflight |
| `rhwp-studio/tests/issue-4135-contextual-shortcut.test.ts` | 테스트 확장 | 한글/영문/Process `S/M`, 수정자, IME 이전 순서 |
| `src/document_core/table_calc/tokenizer.rs` | `#[cfg(test)]`만 | `AA1` 단일 셀 참조 RED |
| `src/document_core/table_calc/parser.rs` | `#[cfg(test)]`만 | `AA1` AST 셀 참조 RED |
| `src/document_core/table_calc/evaluator.rs` | `#[cfg(test)]`만 | `AA1`, `Z1:AA1` 평가 RED |

## 3. RED 결과

### Studio focused

```text
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/issue-4135-block-calculation-plan.test.ts \
  tests/undo-page-blockcalc.test.ts

tests 19 / pass 9 / fail 10 / skipped 0
```

통과한 9건은 기존 full/embed `Ctrl/Cmd+Shift+S` 문맥 라우팅과 snapshot undo 계약이다.
실패한 10건은 다음 두 원인으로만 묶인다.

- 6건: `src/command/block-calculation-plan.ts`가 아직 없어 `ERR_MODULE_NOT_FOUND`
- 4건: `resolveCellBlockLetterShortcut`과 IME 이전 dispatch가 아직 없음

기존 라우팅 회귀 때문에 생긴 실패는 없다.

### Rust focused

```text
cargo test issue_4135 --lib

running 4 tests
0 passed / 4 failed / 3902 filtered out
```

실패 원인은 현 계산식 모델이 열을 단일 `char`로만 보유하기 때문이다.

| 테스트 | 현재 관찰 | 기대 |
| --- | --- | --- |
| tokenizer `AA1` | `Function("AA1")` | 하나의 `CellRef` |
| parser `AA1` | `FuncCall { name: "AA1" }` | `CellRef` AST |
| evaluator `AA1` | 지원하지 않는 함수 오류 | 27번째 열 값 `27` |
| evaluator `SUM(Z1:AA1)` | `26` | `26 + 27 = 53` |

## 4. 정책·형식 검사

```text
node scripts/rust-unit-test-tiers.mjs --check
  PASS — 4225 tests / 299 modules / ready 0 / support 87 / white-box 4134

cargo fmt --all -- --check
  PASS

git diff --check
  PASS
```

## 5. R2 인계 조건

Recovery R2에서는 이 RED만 GREEN으로 만든다.

- 다중 문자 열 참조를 tokenizer → parser → evaluator에 일관되게 연결한다.
- 순수 planner와 dry-run preflight를 구현한다.
- 실제 `blockCalcCommand()`는 planner job 전체가 preflight를 통과한 뒤 한 snapshot에서만 쓴다.
- 중첩·병합·불연속·모호한 선택은 지원을 추측하지 않고 no-op으로 유지한다.

R1 결과를 작업지시자가 승인하기 전에는 Recovery R2 제품 코드 변경을 시작하지 않는다.
