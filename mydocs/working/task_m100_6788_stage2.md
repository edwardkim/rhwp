# Task M100 #6788 — 2단계 구간별 Undo/Redo 복원 완료보고서

- Issue: [#6788](https://github.com/edwardkim/rhwp/issues/6788)
- 작성일: 2026-09-06
- 상태: **2단계 focused 검증 완료 — 3단계 승인 대기**
- 계획: [수행계획서](../plans/task_m100_6788.md), [구현계획서](../plans/task_m100_6788_impl.md)
- 이전 결과: [1단계](task_m100_6788_stage1.md), commit `0efd0195a`
- 브랜치: `codex/6788-preserve-mixed-char-format`
- 기준 devel: `51ad998e33ef7f5191b0e1b0b656dc44cef33a1c`

## 1. 결과

`ApplyCharFormatCommand`가 문단당 모양 ID 하나 대신 적용 전후의 **구간 경계와 모양 ID 목록**을
기록한다. 새 Node WASM에 실제 Studio command/history/bridge를 연결해 혼합 색·굵기·크기의
형광펜 적용, Undo, Redo를 반복해도 전체 문자 모양/ID가 각각 전후 상태로 복원됨을 확인했다.
HWP·HWPX 저장·재적재에서도 글자색/형광펜이 유지됐다.

이는 Node 기반 실제 엔진/Studio 행위 검증이다. **Firefox 확장 UI 실환경 검증 완료나 배포 완료는
아니다.** 최적화된 web WASM·브라우저 시각 증적·전체 lint/회귀 게이트는 3단계에 남아 있다.

## 2. 구현

| 계층 | 변경 |
| --- | --- |
| `src/model/paragraph.rs` | 문자 offset 기준 `CharShapeRun`, 연속 모양 구간 조회, 선택 밖 및 문단 끝 ref를 보존하는 일괄 복원. |
| `src/document_core/commands/formatting_runs.rs` | 본문·중첩 셀 조회/복원 native API, 전체 payload 사전 검증, 문단별 reflow·dirty·rebuild·event 처리. |
| `src/wasm_api.rs` | `getCharShapeRuns`/`setCharShapeRuns`와 `...InCellByPath` export. 기존 단일 ID API는 유지. |
| Studio `core/types.ts`, `char-shape-runs.ts`, `wasm-bridge.ts` | typed 구간 계약, 응답/입력 구간 검증, 필수 binding 존재 확인. |
| Studio `engine/command.ts` | 모든 before 구간 사전 확보, 실행 후 after 확보, Undo/Redo 구간 복원. 셀 batch 유지, 빈 선택 history 제외. |
| Studio `core/mutation-method-registry.ts` | 두 복원 mutation을 명시적으로 분류. |
| Rust/Studio 테스트 | 구간 복원·잘못된 payload·문단 끝 경계·실제 WASM 왕복 회귀 추가 및 기존 어댑터/라우팅 가드 갱신. |

구간 payload는 `{startOffset, endOffset, charShapeId}` 목록이다. 문자 offset과 IR UTF-16 위치를
혼용하지 않는다. 요청 범위는 문단 안의 `[start, end)`이고, 빈 범위에는 빈 목록만 허용한다.
구간의 연속성·순서·길이·끝·ID 및 대상 경로를 **mutation 전에** 확인한다. 빈틈·겹침·범위 초과·
유효하지 않은 ID·잘못된 JSON은 오류이며 일부 구간만 먼저 적용하지 않는다.

복원 시 구간마다 reflow/WASM 호출을 반복하지 않는다. 본문 호출 횟수 회귀에서 Undo 또는 Redo
한 번당 **대상 문단 수만큼만** 복원 API가 호출됨을 확인했다. 새 API의 reflow는 기존 ID 복원과
동일한 본문 열 폭 또는 최내곽 셀 폭을 사용한다. renderer/layout 정책은 변경하지 않았다.

최초 명령 실행 중 오류가 나면 이미 시도한 문단의 before 구간을 역순 복원하고 오류를 전파한다.
이는 해당 범위의 서식 복구이며 DocInfo에 생성된 미사용 리소스까지 되감는 전체 문서 transaction은
아니다. 기존 CommandHistory의 실행 성공 후 기록·실패 시 처리 정책은 바꾸지 않았다.

### 구버전 WASM 대응

구간 조회 시 getter뿐 아니라 body/cell setter까지 네 export의 존재를 확인한다. 지원하지 않는
binding이면 **서식 적용 전에** 최신 WASM/새로고침 필요 오류를 낸다. 단일 ID 복원으로 조용히
fallback하지 않는다. 일반 배포는 JS/WASM을 함께 갱신해야 하며 최종 산출물 정합은 3단계에서 확인한다.

### 계획 대비

native 조회/복원은 대형 `formatting.rs`에 더하지 않고 같은 commands 계층의
`formatting_runs.rs`로 분리했다. 공개 API·payload·기능 범위는 계획대로다.
본문을 snapshot history로 전환하지 않았고, F5 및 머리말/꼬리말의 기존 snapshot 경로는 유지했다.

## 3. 검증 결과

| 검증 | 최종 결과 |
| --- | --- |
| `issue_6788_mixed_char_format` Rust focused | **15 passed, 0 failed, 0 ignored**. 1단계 11개 + 구간 복원/오류/끝 경계 4개. |
| Studio 관련 7개 test 파일 | **62 passed, 0 failed, 0 skipped**. 행위 테스트와 기존 라우팅·history 소스 계약 포함. |
| 실제 WASM 전용 runner 직접 실행 | `MIXED_CHAR_FORMAT_OK scenarios=13`. |
| TypeScript `tsc --noEmit` | 통과. |
| Cargo fmt 및 새 integration source 개별 rustfmt check | 통과. |
| review manifest `--check` | 통과: 1170 sources, 4940 static test attrs, 48/48 targets. |
| source root/review 비교 및 `git diff --check` | 통과. 파생 suite·manifest는 커밋하지 않음. |

13개 행위 시나리오:

1. 혼합 색·굵기·크기 + 형광펜 전체 선택, 단일 모양 선택, 모양 중간 경계 선택, 복수 문단 선택 (4개).
2. 일반 셀·중첩 셀의 command Undo/Redo, 각 셀의 실제 F5 operation + SnapshotCommand (4개).
3. 실제 머리말·꼬리말 선택 operation + SnapshotCommand (2개).
4. 복수 문단 실행 중 오류 주입 후 이전 서식 복구, 빈 선택 history 제외, 구버전 binding/응답 오류 (3개).

command와 bridge는 실제 클래스이며, DOM을 요구하는 InputHandler는 프로토타입에 선택 상태를
주입했다. F5/HF operation은 실제 메서드에서 얻어 실제 SnapshotCommand/History로 실행한다.
화면 입력·refresh·caret geometry·Firefox 이벤트는 이 방식의 검증 범위가 아니다.

### 재현 명령과 산출물

Rust는 별도 review worktree에서 파생 suite를 준비하고 공유 review target으로 순차 실행했다.
Node WASM도 그 worktree의 동일 source로 새로 빌드했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs --cargo-test issue_6788_mixed_char_format -- \
  --target-dir /Users/melee/Documents/projects/forks/rhwp/target/pr-review
node scripts/rust-test-suite-manifest.mjs --check
CARGO_TARGET_DIR=/Users/melee/Documents/projects/forks/rhwp/target/pr-review \
  scripts/wasm-pack-locked.sh --target nodejs \
  --out-dir /Users/melee/Documents/projects/forks/rhwp/pkg-node --no-opt
```

Studio 디렉터리:

```bash
./node_modules/.bin/tsc --noEmit
node --experimental-transform-types --no-warnings tests/support/mixed-char-format.runner.mjs
node --test tests/mixed-char-format.test.ts tests/pending-char-shape.test.ts \
  tests/apply-charformat-nested-cell.test.ts tests/mutation-routing-guard.test.ts \
  tests/cell-block-format.test.ts tests/command-history-hffn.test.ts \
  tests/command-history-snapshot.test.ts
```

- Node WASM: `pkg-node/rhwp_bg.wasm` (로컬 생성물, 미커밋).
- SHA-256: `ed080899b874b47c50727f2beffcad4b584e52c7a6d44f709e9f6313e7e8e336`.
- 빌드: locked native wrapper, `--target nodejs --no-opt`, 2분 24초. 최적화 배포 산출물로 간주하지 않는다.
- 빌드 source는 기준 devel + 1단계/2단계 Rust 변경이며 root/review 원본 일치를 확인했다.

검증 중 수정한 문제: 첫 Rust 테스트는 `Document`의 미지원 Serialize 사용으로 컴파일 실패했다.
문서의 Debug 표현을 비교하는 무변경 검사로 고친 뒤 통과했다. 끝 경계 테스트 추가 후 suite 배정이
바뀌어 한 번 0 tests가 실행됐으며, 이를 성공 증적으로 쓰지 않고 review `--prepare` 후 **15개가
실제 실행됨**을 재확인했다.

## 4. 다음 승인

2단계 결과와 변경을 로컬 커밋으로 마감한다. **3단계 제품 검증·PR 준비** 승인 후 최적화 산출물/
Firefox 가능 환경, 시각 증적, 전체 Rust lint·integration·Native Skia 및 Studio 검증을 수행한다.
이번 단계에서 전체 PR CI 성격 검증, remote push, PR 생성, merge, 이슈 종료는 수행하지 않았다.
