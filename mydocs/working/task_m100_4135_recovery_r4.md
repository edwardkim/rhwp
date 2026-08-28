# Task M100 #4135 Recovery R4 — 통합·실브라우저 검증 결과

- **기준**: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../plans/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r3.md`](task_m100_4135_recovery_r3.md)
- **승인**: 작업지시자가 R3 결과 보고 뒤 `진행해줘.`로 R3 결과와 R4 착수를 승인
- **판정**: macOS 한글 IME 물리 `S` 수동 확인 실패를 보정하고 자동·영문 실브라우저 재검증 GREEN;
  작업지시자가 corrective 빌드의 한글 IME에서 대화상자만 열리고 `ㄴ`이 남지 않는 것을 확인해 R4 승인

## 0. 작업지시자 수동 확인 실패와 corrective RED

작업지시자가 macOS 한글 입력 상태에서 물리 `S`를 눌렀을 때 셀 나누기 대화상자는 열렸지만,
선택 셀에 `ㄴ`도 함께 입력됐다. 따라서 R3의 물리 키 라우팅은 성공했지만 `keydown` 뒤 이어지는
`compositionstart → input → compositionend` 스트림은 소비하지 못했다. 기존 R4 GREEN 판정은 자동화
가능 범위에 한정하며, R4 자체는 승인하지 않는다.

corrective RED는 다음 세 계약으로 고정했다.

1. 한글 `ㄴ/ㅡ` 셀 문자 단축키는 후속 composition/input 스트림 전체를 소비한다.
2. `Process`/`keyCode=229`에서 composition 이벤트가 없는 input-only 폴백은 첫 input만 소비한다.
3. 영문 `S/M`은 억제 상태를 만들지 않아 다음 정상 입력을 삼키지 않는다.

focused 실행 결과는 기존 11건 통과, 신규 3건 실패이며 실패 원인은 IME 후속 입력 guard 부재다.
corrective 구현과 전체 재검증, 작업지시자 재확인 전에는 아래 자동 GREEN 증적만으로 R4를 승인하지
않는다.

### 0.1 Corrective 구현과 재검증

`CellBlockLetterImeGuard`를 추가해 셀 문자 명령으로 소비한 한글 `ㄴ/ㅡ`, `Process`,
`isComposing`, `keyCode=229` keydown만 arm한다. dispatcher가 대화상자로 focus를 옮기기 전에
arm하고, `compositionstart`, `input`, `compositionend`에서 조합 상태와 hidden textarea를 비운다.
compositionend 뒤 동일 텍스트의 유령 input은 한 번 더 소비한다. 영문 `S/M`은 arm하지 않으며,
다음 물리 keydown과 문서 deactivate/destroy에서 guard를 초기화한다.

```text
focused issue tests: 22 pass / 0 fail
Studio 전체: 1,247 tests / 1,246 pass / 1 skip / 0 fail
production build: PASS — 240 modules transformed
embed E2E: 17 pass / 0 fail
```

최신 코드는 서비스 워커 범위를 분리한 `http://127.0.0.1:7716/`에 올렸다. 브라우저 제어로 빈
2×2 표를 만들고 F5 두 셀 블록에서 영문 `S`가 셀 나누기 대화상자를 열며 셀에 문자를 남기지
않는 것을 확인했다. 자동 브라우저 키 API는 한글 자모 키를 지원하지 않아 실제 macOS 한글 IME
`Process/KeyS → composition/input` 최종 판정은 작업지시자의 재확인으로 남긴다.

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
신뢰성 있게 만들지 못한다. 순수 resolver와 InputHandler 순서 테스트에서 명령 라우팅은
GREEN이었지만, 작업지시자 수동 확인에서 후속 `ㄴ` 입력이 함께 발생했다. corrective 구현 뒤
동일 사용자 여정을 다시 확인해야 R4를 확정한다.

## 4. 작업지시자 수동 확인 절차

corrective 구현이 반영된 개발 서버는 `http://127.0.0.1:7716/`에서 실행 중이다. 기존
`7715` 탭은 수동 확인 때 만든 미저장 문서를 보존하기 위해 강제 새로고침하지 않았다. 다음 한글
IME 재확인이 필수다.

1. 표의 아무 셀을 클릭하고 `F5`를 두 번 누른 뒤 방향키로 둘 이상의 셀을 파랗게 선택한다.
2. macOS 입력 소스를 **한글**로 바꾼다.
3. 수정자 없이 물리 키보드의 `S` 키를 누른다. 한글 입력이라면 보통 `ㄴ`이 입력될 키다.
4. **셀 나누기 대화상자가 열리고 선택 셀에 `ㄴ`이 들어가지 않는지** 확인한 뒤 취소한다.
5. 같은 선택에서 입력 소스를 **영문**으로 바꾸고 `S`를 눌러 같은 대화상자가 열리는지 확인한다.
6. 선택을 `Esc`로 끝낸 뒤 `Cmd+Shift+S`를 눌러 **다른 이름으로 저장** 대화상자가 열리는지
   확인하고 취소한다.

선택적으로 한글 상태의 물리 `M`(`ㅡ`)도 선택 셀을 합치는지 확인하고 즉시 `Cmd+Z`로
되돌릴 수 있다.

작업지시자가 `수정이 반영되었어.`라고 확인해 위 수동 게이트를 통과했다. 이 승인 뒤 제기된 F5 1·2회
단계 식별 UX는 기존 R4를 다시 여는 결함이 아니라 승인된 후속 Recovery R5로 분리한다. 원격 push,
PR 생성, GitHub 코멘트는 여전히 별도 승인 전에는 수행하지 않는다.
