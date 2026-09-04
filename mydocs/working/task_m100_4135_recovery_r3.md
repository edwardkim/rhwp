# Task M100 #4135 Recovery R3 — 한글 IME 셀 나누기·합치기 결과

- **기준**: `upstream/devel@94ff48d2b81dee5241110db9d2417dffbfb7f9ec`
- **브랜치**: `codex/issue-4135-contextual-shortcut`
- **계획**: [`task_m100_4135_impl.md`](../plans/archives/task_m100_4135_impl.md)
- **선행 결과**: [`task_m100_4135_recovery_r2.md`](task_m100_4135_recovery_r2.md)
- **승인**: 작업지시자가 R2 결과 보고 뒤 `진행해줘.`로 R2 결과와 R3 착수를 승인
- **판정**: R3 GREEN, Recovery R4 결과 승인 전 미착수

## 1. 구현 결과

### 순수 물리 키 resolver

`contextual-shortcut.ts`에 `resolveCellBlockLetterShortcut()`을 추가했다.

| 입력 | 결과 |
| --- | --- |
| `s/S`, `ㄴ`, `Process/KeyS` | `table:cell-split` |
| `m/M`, `ㅡ`, `Process/KeyM` | `table:cell-merge` |
| 셀 블록 밖 | 소유하지 않음 |
| Ctrl/Meta/Alt가 있는 `S/M` | 소유하지 않음 |

대문자 입력을 위해 Shift는 허용한다. `Process` 자체만으로는 소유하지 않고 물리 `code`가
`KeyS` 또는 `KeyM`일 때만 명령으로 해석한다.

### InputHandler 순서

키보드 입력 순서를 다음처럼 고정했다.

1. F5 셀 블록 `Ctrl/Cmd+Shift+S` 문맥 resolver
2. 수정자 없는 F5 셀 블록 `S/M` 물리 키 resolver
3. 일반 IME 조기 반환
4. 나머지 키보드 처리

따라서 한글 IME가 `key='Process', keyCode=229`를 보내도 2번에서 먼저 `preventDefault()`하고
셀 명령을 dispatch한다. 기존 F5 분기 아래의 영문 전용 `M/S` 코드는 제거해 소유자를 한 곳으로
통합했다.

## 2. 변경 파일

| 파일 | 변경 |
| --- | --- |
| `rhwp-studio/src/command/contextual-shortcut.ts` | 영문·한글·Process `S/M` 순수 resolver |
| `rhwp-studio/src/engine/input-handler-keyboard.ts` | IME 이전 dispatch·기존 중복 분기 제거 |
| `rhwp-studio/tests/issue-4135-contextual-shortcut.test.ts` | 조기 resolver 순서 계약 갱신 |

## 3. 검증 결과

### R1~R3 focused

```text
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/issue-4135-block-calculation-plan.test.ts \
  tests/undo-page-blockcalc.test.ts

20 pass / 0 fail
```

R1에서 남겨 둔 한글/영문/Process `S/M` 4 RED가 GREEN이 됐다. 기존 full/embed
`Ctrl/Cmd+Shift+S`, 선택 범위 planner, preflight·snapshot 계약도 함께 통과했다.

### 주변 shortcut·프로파일 회귀

```text
node --test \
  tests/issue-4135-contextual-shortcut.test.ts \
  tests/ime-shortcut-routing.test.ts \
  tests/shortcut-map.test.ts \
  tests/chrome-mode.test.ts

36 pass / 0 fail
```

IME의 Ctrl/Cmd 단축키, Ctrl+M chord, Save As `KeyS`, embed 파일 수명주기 차단 계약의 회귀는
없다.

### build·형식

```text
npm run build
PASS — TypeScript + Vite, 240 modules transformed

git diff --check
PASS
```

## 4. 미실행·R4 인계

- Studio 전체 Node 테스트와 embed E2E는 R4에서 실행한다.
- R2 Rust 변경이 포함된 새 WASM을 만들고 로컬 서버에 반영하는 작업도 R4에서 수행한다.
- 실제 macOS 한글/영문 전환, 가로/세로 블록 합계, 셀 블록 밖 Save As는 R4 실브라우저에서
  확인한다.

R3 결과를 작업지시자가 승인하기 전에는 Recovery R4 통합·실브라우저 검증을 시작하지 않는다.
