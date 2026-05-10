---
PR: #739
제목: Task #731 — 수식 신규 입력 (insertEquation WASM API + 입력 메뉴 항목)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 8번째 PR)
처리: 옵션 A — 2 commits cherry-pick + 자기 정정 + no-ff merge
처리일: 2026-05-10
머지 commit: d0a75656
PR_supersede: (c) 패턴 후속 — Issue #766/#767 별 PR 영역 영역 후속 처리
---

# PR #739 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + 자기 정정 + no-ff merge `d0a75656`)

| 항목 | 값 |
|------|-----|
| 머지 commit | `d0a75656` (--no-ff merge) |
| Cherry-pick commits | `0941672e` (Task #731) + `e93f60dc` (Copilot 리뷰) |
| 자기 정정 commit | `76f9ade6` (`inTableCellEditing` → `inTable`) |
| closes | #731 |
| 자기 검증 | cargo test ALL GREEN + tsc + sweep 170/170 same + WASM 4.65 MB |
| 작업지시자 검증 | 메뉴 클릭 경로 ✅ 정합 동작 + 두 별 결함 발견 → Issue #766/#767 |

## 2. 정정 본질 — 5 files, +152/-0 + 자기 정정

### 2.1 `src/document_core/commands/object_ops.rs` (+94)
- `insert_equation_native()` — `insertFootnote` 패턴 정합
- `treat_as_char: true` (`project_equation_always_tac` 정합)
- `CTRL_EQUATION` 상수 (Copilot 리뷰)
- `control_mask |= 1u32 << 11` (Copilot 리뷰)

### 2.2 `src/wasm_api.rs` (+22)
- WASM 바인딩 `insertEquation(sec, para, charOffset, script, fontSize, color)`

### 2.3 `rhwp-studio/src/core/wasm-bridge.ts` (+5)
- TypeScript 바인딩 `insertEquation()`

### 2.4 `rhwp-studio/src/command/commands/insert.ts` (+30)
- `'insert:equation'` 커맨드 + 빈 수식 삽입 후 EquationEditorDialog 자동 open
- 표 셀 내부 실행 차단 (이중 가드)

### 2.5 `rhwp-studio/index.html` (+1)
- 입력 메뉴 영역 "수식 (Ctrl+N,M)" 항목

### 2.6 자기 정정 (commit `76f9ade6`)
PR 영역 영역 `EditorContext.inTableCellEditing` 속성 영역 영역 본 환경 영역 영역 부재 (정의 영역 영역 `inTable`) → 본 환경 정합 영역 영역 `inTable` 영역 영역 대체. tsc 에러 (TS2339) 정정. execute 내부 영역 영역 cellIndex 가드 영역 영역 이중 보장 보존.

## 3. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `insertFootnote` 패턴 | `insert_equation_native()` 정합 |
| `EquationEditorDialog` (PR #738) | 자동 진입 — 듀얼 모드 + 자동완성 + 탭 130+ + 기호 검색 |
| `Control::Equation` IR + 기존 API | 보존 |

## 4. 본 환경 cherry-pick + 검증

### 4.1 결정적 검증

| 검증 | 결과 |
|------|------|
| `cargo build --release` | ✅ 통과 (29.44s) |
| `tsc --noEmit` (rhwp-studio, 자기 정정 후) | ✅ 통과 |
| `cargo test --release` (전체) | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (신규 API 영역 영역 호출 부재 영역 영역 영향 부재 보장) |
| WASM 빌드 (Docker, 강제 재빌드) | ✅ 4.65 MB |

### 4.2 WASM 빌드 영역 영역 의 caching 영역 영역
1차 빌드 영역 영역 `insertEquation` API 노출 부재 발견 → `cargo clean --target wasm32-unknown-unknown` + Docker 강제 재빌드 → 정합. WASM Docker 빌드 영역 영역 caching 영역 영역 의 한계 영역 영역 학습.

### 4.3 작업지시자 웹 검증
- **메뉴 클릭 경로**: 본문 캐럿 위치 영역 빈 수식 삽입 + EquationEditorDialog 자동 진입 ✅ 정합 동작
- 두 별 결함 발견 (PR #739 본질 영역 영역 무관)

## 5. 발견된 별 결함 — 후속 처리

### 5.1 Issue #766 — 수식 객체 backspace 삭제 시 "지정된 컨트롤이 Shape이 아닙니다" 오류

**본질**: `input-handler-keyboard.ts` 영역 영역 picture-object selection ref dispatch 영역 영역 — `ref.type === 'image'` 외 영역 영역 `deleteShapeControl` 호출 → `Control::Equation` 영역 영역 매칭 실패 → `delete_shape_control_native` (`object_ops.rs:1832`) 영역 영역 에러.

**처리**: `feedback_pr_supersede_chain` (c) 패턴 — 별 PR 영역 영역 후속 정정 처리.

### 5.2 Issue #767 — Ctrl+N+M 단축키 영역 chordMapN 'm' 매핑 부재 → 브라우저 새 창 발생

**본질**: `input-handler-keyboard.ts:chordMapN` 영역 영역 `m: 'insert:equation'` 매핑 부재. PR 영역 영역 `shortcutLabel: 'Ctrl+N,M'` 명시 영역 영역, 그러나 코드 영역 영역 미반영.

**처리**: 별 PR 영역 영역 후속 정정 — `chordMapN['m'] = 'insert:equation'` + `'ㅡ'` (한글 IME) 추가.

## 6. 영향 범위

### 6.1 변경 영역
- 본문 캐럿 위치 영역 영역 수식 신규 삽입 (`insertEquation` API)
- rhwp-studio 입력 메뉴 영역 영역 "수식" 항목

### 6.2 무변경 영역 (sweep 170/170 same 영역 영역 입증)
- 기존 수식 편집 (EquationEditorDialog, PR #738)
- HWP3/HWPX 변환본 영역 영역 시각 정합
- 기존 WASM API

### 6.3 후속 결함 (별 PR 영역 영역 처리)
- Issue #766 — backspace 삭제 경로
- Issue #767 — Ctrl+N+M 단축키 매핑

## 7. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738/#739 영역 8번째 PR) |
| `feedback_pr_supersede_chain` | **(c) 패턴 적용** — PR #739 머지 유지 + Issue #766/#767 영역 별 PR 영역 영역 후속 정정 통합. PR #723 → PR #732 동일 패턴 정합. |
| `feedback_image_renderer_paths_separate` | rhwp-studio TypeScript + Rust 영역 영역 격리 — 다른 layout/render 경로 영역 영역 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (insertFootnote 패턴 + EquationEditorDialog) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | 작업지시자 dev server 인터랙션 검증 — 본질 통과 + 별 결함 발견 영역 영역 후속 PR 영역 영역 처리 |
| `project_equation_always_tac` | 한컴 수식 영역 영역 항상 TAC — `treat_as_char: true` 정합 |
| `feedback_self_verification_not_hancom` 영역 — 별 사례 | WASM 빌드 caching 영역 영역 — 자기 검증 (`cargo test --release ✅`) 통과 영역 영역 영역, 그러나 WASM 빌드 영역 영역 의 cache 영역 영역 영역 영역 노출 부재 영역 영역 발견 영역 영역. 작업지시자 웹 검증 영역 영역 시 영역 영역 만 검출 가능 → `cargo clean --target wasm32-unknown-unknown` 영역 영역 강제 재빌드 영역 영역 정합 영역. **WASM 빌드 영역 영역 caching 한계 학습** |

## 8. 잔존 후속

- **Issue #766** OPEN — 수식 객체 backspace 삭제 오류 (별 PR 영역 영역 후속 정정 권장)
- **Issue #767** OPEN — Ctrl+N+M 단축키 매핑 부재 (별 PR 영역 영역 후속 정정 권장)

---

작성: 2026-05-10
