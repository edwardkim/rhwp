# Task M100 #4135 Recovery R4 — 통합·실브라우저 검증 결과

- **기준**: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../plans/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r3.md`](task_m100_4135_recovery_r3.md)
- **승인**: 작업지시자가 R3 결과 보고 뒤 `진행해줘.`로 R3 결과와 R4 착수를 승인
- **판정**: 자동·실브라우저 범위 GREEN, macOS 한글 IME 물리 `S` 수동 확인과 R4 결과 승인 대기

## 1. R4에서 추가로 발견하고 보정한 결함

최신 WASM을 올린 첫 실브라우저 여정에서 다음 불일치를 재현했다.

| 입력 | 계산 엔진 결과 | 화면 표시 |
| --- | --- | --- |
| 첫 행 `10, 20, 30` | `60` | `6` |
| 둘째 행 `1, 2, 3` | `6` | `6` |

선택 범위 planner와 계산식 평가 결과는 옳았다. 문제는 `evaluate_table_formula(write_result=true)`가
결과 셀의 `Paragraph.text`와 `char_offsets`만 직접 덮고, 일반 셀 텍스트 편집 경로가 갱신하는
표 측정·문단 레이아웃·페이지 캐시 불변식을 건너뛴 데 있었다. 그 결과 저장된 텍스트는 `60`인데
기존 렌더 트리가 마지막 `0`을 그리지 않았다.

보정은 결과 기록을 `replace_text_in_cell_native_impl(..., paginate_immediately=true)`로 통합해
일반 셀 텍스트 교체와 같은 dirty/reflow/pagination 경로를 타게 했다. 기존
`evaluate_table_formula` API 동등성 source-side 테스트에 `10+20+30=60`, `1+2+3=6`과 SVG의
마지막 `0` 렌더링 계약을 결합했다. 별도 테스트를 하나 늘리는 초안은 unit-tier 정책의
`4226 > 4225` 차단을 확인한 뒤 제거해 정본 테스트 총량을 보존했다.

## 2. 자동 검증

### Studio

```text
npm test
1,242 tests / 1,241 pass / 1 skip / 0 fail

npm run build
PASS — TypeScript + Vite, 240 modules transformed
```

최종 focused 계약도 별도로 실행했다.

```text
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/issue-4135-block-calculation-plan.test.ts
17 pass / 0 fail
```

### Rust

```text
cargo test --locked --lib
rhwp: 3,893 pass / 13 ignored / 0 fail
rhwp-contracts: 15 pass
rhwp-ooxml-chart: 165 pass
rhwp-password-crypto: 2 pass

cargo test issue_4135 --lib
5 pass / 0 fail

cargo test document_core::table_calc --lib
33 pass / 0 fail
```

source-side 테스트 정책은 다음 정본 수치로 통과했다.

```text
node scripts/rust-unit-test-tiers.mjs --check
4225 tests / 299 modules / ready 0 / support 87 / white-box 4134 / cfg support items 28
```

### WASM·embed

```text
CARGO_TARGET_DIR=target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
PASS

VITE_URL=http://127.0.0.1:7715 npm run e2e:embed
17 PASS — public/legacy load, diagnostics, trace, export, forged peer, destroy
```

### 형식

```text
cargo fmt --all
cargo fmt --all -- --check
git diff --check
PASS
```

## 3. 실브라우저 결과

브라우저 제어 스킬로 `http://127.0.0.1:7715/`의 실제 Studio UI를 조작했다. 최신 WASM을
구분 URL의 새 탭에서 초기화하고, 복구본을 다시 열어 이전 런타임 캐시와 구분했다.

| 여정 | 결과 |
| --- | --- |
| `10,20,30,빈칸` / `1,2,3,빈칸` 2행을 F5 블록 선택 후 `Cmd+Shift+S` | 오른쪽 결과 `60`, `6` |
| 위 2행 + 빈 3행 전체를 선택 후 `Cmd+Shift+S` | 아래 결과 `11`, `22`, `33`, `66` |
| 셀 블록에서 영문 `S` | 셀 나누기 대화상자, 문자 미입력 |
| 셀 블록에서 영문 `M` | 선택 셀 합치기, Undo로 원복 |
| 셀 블록 선택을 끝낸 뒤 `Cmd+Shift+S` | 다른 이름으로 저장 대화상자 |
| embed E2E | Save As 소유권 없는 기존 차단 계약 유지 |

자동 브라우저 키 입력은 macOS 입력 소스를 실제 한글 IME로 전환한 `Process/KeyS` 이벤트를
신뢰성 있게 만들지 못한다. 순수 resolver와 InputHandler 순서 테스트에서 `ㄴ`,
`Process/KeyS`, `ㅡ`, `Process/KeyM` 계약은 GREEN이지만, 작업지시자가 실제 키보드 한글
상태에서 마지막 사용자 여정을 확인해야 R4를 확정한다.

## 4. 작업지시자 수동 확인 절차

개발 서버는 `http://127.0.0.1:7715/`에서 실행 중이다. 다음 한글 IME 확인이 필수다.

1. 표의 아무 셀을 클릭하고 `F5`를 두 번 누른 뒤 방향키로 둘 이상의 셀을 파랗게 선택한다.
2. macOS 입력 소스를 **한글**로 바꾼다.
3. 수정자 없이 물리 키보드의 `S` 키를 누른다. 한글 입력이라면 보통 `ㄴ`이 입력될 키다.
4. **셀 나누기 대화상자가 열리고 선택 셀에 `ㄴ`이 들어가지 않는지** 확인한 뒤 취소한다.
5. 같은 선택에서 입력 소스를 **영문**으로 바꾸고 `S`를 눌러 같은 대화상자가 열리는지 확인한다.
6. 선택을 `Esc`로 끝낸 뒤 `Cmd+Shift+S`를 눌러 **다른 이름으로 저장** 대화상자가 열리는지
   확인하고 취소한다.

선택적으로 한글 상태의 물리 `M`(`ㅡ`)도 선택 셀을 합치는지 확인하고 즉시 `Cmd+Z`로
되돌릴 수 있다.

수동 확인 결과와 R4 결과 승인을 받기 전에는 최종 보고서를 작성하지 않는다. 원격 push,
PR 생성, GitHub 코멘트도 별도 승인 전에는 수행하지 않는다.
