# Task M100 #4135 Recovery R2 — 선택 범위 블록 합계 결과

- **기준**: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../../plans/archives/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r1.md`](task_m100_4135_recovery_r1.md)
- **승인**: 작업지시자가 R1 결과 보고 뒤 `진행해줘.`로 R1 결과와 R2 착수를 승인
- **판정**: R2 GREEN, Recovery R3 결과 승인 전 미착수

## 1. 구현 결과

### 선택 범위 planner

`rhwp-studio/src/command/block-calculation-plan.ts`를 추가했다.

- 선택 범위 오른쪽 끝 열이 모두 비면 각 행의 왼쪽 선택 범위를 계산한다.
- 선택 범위 아래 끝 행이 모두 비면 각 열의 위쪽 선택 범위를 계산한다.
- 오른쪽과 아래가 모두 비어 방향이 모호하거나 어느 쪽도 비지 않으면 no-op이다.
- 단일·불연속·병합·중첩 선택은 no-op이다.
- 열 이름은 base-26으로 `A..Z, AA, AB...`를 만든다.
- 모든 job을 `writeResult=false`로 preflight하고 하나라도 실패하면 쓰기를 시작하지 않는다.

### Studio 명령 배선

`blockCalcCommand()`가 앵커 셀의 `=SUM(above)`를 실행하던 경로를 제거했다.

1. F5 선택 범위와 표 컨텍스트를 읽는다.
2. 선택한 각 좌표의 실제 셀 index, 공백, `rowSpan`/`colSpan`을 수집한다.
3. planner가 만든 모든 job을 dry-run한다.
4. 전부 성공할 때만 한 `tableBlockCalc` snapshot에서 결과를 기록한다.
5. write 중 예상 밖 오류가 나면 `SnapshotCommand`의 before snapshot으로 전체 rollback한다.

이 명령 함수는 블록 합계·평균·곱이 공유하지만 #4135의 브라우저 최종 수용 판정은 블록 합계에
한정한다.

### 계산식 코어

계산식의 셀 열 참조를 단일 `char`에서 문자열로 확장했다.

- `AA1`을 하나의 셀 참조로 tokenize·parse한다.
- evaluator는 base-26 열 이름을 0-based index로 변환한다.
- `SUM(Z1:AA1)`처럼 Z 경계를 넘는 범위를 평가한다.
- `LOG10(100)`처럼 숫자가 포함된 함수명 뒤에 `(`가 있으면 셀 참조로 오인하지 않는다.
- 기존 `?` 열·행 와일드카드와 `A0` 거절 계약을 보존한다.

## 2. 변경 파일

| 파일 | 변경 |
| --- | --- |
| `rhwp-studio/src/command/block-calculation-plan.ts` | 순수 planner·preflight |
| `rhwp-studio/src/command/commands/table.ts` | 선택 셀 수집·plan·snapshot 일괄 쓰기 |
| `rhwp-studio/tests/undo-page-blockcalc.test.ts` | 선택 범위·preflight·단일 snapshot 배선 가드 |
| `src/document_core/table_calc/tokenizer.rs` | 다중 문자 열 token |
| `src/document_core/table_calc/parser.rs` | 문자열 열 AST |
| `src/document_core/table_calc/evaluator.rs` | base-26 열 평가 |

## 3. 검증 결과

### R2 Studio focused

```text
node --test \
  tests/issue-4135-block-calculation-plan.test.ts \
  tests/undo-page-blockcalc.test.ts

9 pass / 0 fail
```

가로/세로 job, Z→AA, 모호·불연속·병합·중첩 거절, dry-run preflight, undo snapshot 배선이
모두 통과했다.

### 계산식 코어

```text
cargo test issue_4135 --lib
4 pass / 0 fail

cargo test document_core::table_calc --lib
33 pass / 0 fail
```

R1의 `AA1`, `Z1:AA1` RED가 GREEN이 됐고 기존 계산식·함수·방향·와일드카드 회귀도 통과했다.

### R3 경계 보존

```text
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/issue-4135-block-calculation-plan.test.ts \
  tests/undo-page-blockcalc.test.ts

20 tests / 16 pass / 4 fail
```

남은 4건은 모두 아직 승인받지 않은 Recovery R3의
`resolveCellBlockLetterShortcut`·IME 이전 dispatch RED다. R2 planner·계산식·기존
`Ctrl/Cmd+Shift+S` 라우팅 실패는 0건이다.

### build·정책·형식

```text
npm run build
PASS — TypeScript + Vite, 240 modules transformed

node scripts/rust-unit-test-tiers.mjs --check
PASS — 4225 tests / 299 modules / ready 0 / support 87 / white-box 4134

cargo fmt --all -- --check
PASS

git diff --check
PASS
```

다중 문자 열의 `LOG10` 보호용 Rust 테스트 함수를 별도로 두었을 때 source-side 총량 정책이
`4227 > 4225`로 거절했다. 해당 단언을 기존 R1 테스트 함수 안으로 합쳐 보호 계약은 유지하고
inventory를 4,225건으로 복원했다.

## 4. 미실행·후속 경계

- Rust 변경을 반영한 새 WASM 산출과 실제 브라우저 수동 검증은 통합 단계 R4에서 수행한다.
- 한글 IME의 수정자 없는 물리 `S/M` 라우팅은 R3 승인 전에는 구현하지 않는다.
- 계산식 메타데이터 저장·원본 값 변경 시 자동 재계산, 병합·중첩 표 지원은 이번 범위 밖이다.

R2 결과를 작업지시자가 승인하기 전에는 Recovery R3 제품 코드 변경을 시작하지 않는다.
