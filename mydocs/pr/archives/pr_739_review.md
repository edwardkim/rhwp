---
PR: #739
제목: Task #731 — 수식 신규 입력 (insertEquation WASM API + 입력 메뉴 항목)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 영역 영역 8번째 PR)
base / head: devel / contrib/insert-equation
mergeStateStatus: BEHIND
mergeable: MERGEABLE — `git merge-tree` 충돌 0건
CI: ALL SUCCESS (Build & Test / Canvas visual diff / CodeQL / Analyze rust+js+py / WASM SKIPPED)
변경 규모: +152 / -0, 5 files (Rust 2 + TypeScript 2 + HTML 1)
검토일: 2026-05-10
---

# PR #739 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #739 |
| 제목 | Task #731 — 수식 신규 입력 (insertEquation WASM API + 입력 메뉴 항목) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738 후속 영역 8번째) |
| base / head | devel / contrib/insert-equation |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 0건 |
| CI | ALL SUCCESS (Canvas visual diff 포함) |
| 변경 규모 | +152 / -0 (순수 추가), 5 files |
| 커밋 수 | 2 (Task 1 + Copilot 리뷰 반영 1) |
| closes | #731 |

## 2. 결함 본질 (Issue #731)

### 2.1 결함 영역
rhwp-studio 영역 영역 메뉴바 **입력** 메뉴 영역 영역 **수식 (Equation)** 신규 입력 항목 영역 영역 미구현 — 한컴 오피스 표준 (입력 → 수식, Ctrl+N+M) 영역 영역 누락.

### 2.2 채택 접근
PR #738 (closes #144 — 수식 편집 UI 개선) 영역 영역 의 후속 영역 영역 — 본문 캐럿 위치 영역 영역 빈 수식 객체 영역 영역 삽입 + 기존 `EquationEditorDialog` 영역 영역 자동 진입 (인프라 재사용).

## 3. PR 의 정정 — 5 영역

### 3.1 `src/document_core/commands/object_ops.rs` (+94)

**`insert_equation_native()` 함수 신규** — `insertFootnote` 패턴 정합:

```rust
pub fn insert_equation_native(
    &mut self,
    section_idx: usize, para_idx: usize, char_offset: usize,
    script: &str, font_size: u32, color: u32,
) -> Result<String, HwpError> {
    let equation = Equation {
        common: CommonObjAttr {
            ctrl_id: CTRL_EQUATION,  // Copilot 리뷰 영역 영역 상수 사용
            treat_as_char: true,
            width: 0, height: 0,
            ..Default::default()
        },
        script: script.to_string(),
        font_size, color,
        font_name: "HYhwpEQ".to_string(),
        ..Default::default()
    };
    
    // 1. controls 영역 insert_idx 영역 영역 삽입 (positions 정합)
    paragraph.controls.insert(insert_idx, Control::Equation(Box::new(equation)));
    paragraph.ctrl_data_records.insert(insert_idx, None);
    
    // 2. char_offsets 영역 영역 8 bytes gap (모든 후속 char_offsets += 8)
    paragraph.char_count += 8;
    paragraph.control_mask |= 1u32 << 11;  // Copilot 리뷰 영역 영역 mask 설정
    paragraph.has_para_text = true;
    
    // 3. 본문 문단 reflow + recompose + paginate + invalidate cache
    reflow_line_segs(body_para, final_width, &self.styles, self.dpi);
    self.recompose_section(section_idx);
    self.paginate_if_needed();
    self.invalidate_page_tree_cache();
    
    Ok(format!("{{\"ok\":true,\"paraIdx\":{},\"controlIdx\":{}}}", para_idx, insert_idx))
}
```

`treat_as_char: true` — 한컴 수식 영역 영역 항상 TAC (`project_equation_always_tac` 정합).

### 3.2 `src/wasm_api.rs` (+22)

```rust
#[wasm_bindgen(js_name = insertEquation)]
pub fn insert_equation(
    &mut self,
    section_idx: u32, para_idx: u32, char_offset: u32,
    script: &str, font_size: u32, color: u32,
) -> Result<String, JsValue> { ... }
```

### 3.3 `rhwp-studio/src/core/wasm-bridge.ts` (+5)

`insertEquation()` 메서드 영역 영역 추가 (TypeScript 바인딩).

### 3.4 `rhwp-studio/src/command/commands/insert.ts` (+30)

```typescript
{
    id: 'insert:equation',
    label: '수식',
    shortcutLabel: 'Ctrl+N,M',
    canExecute: (ctx) => ctx.hasDocument && !ctx.inTableCellEditing,  // Copilot 리뷰 영역 영역
    execute(services) {
        const ih = services.getInputHandler();
        const pos = ih.getPosition();
        // 본문 전용 — 표 셀 내부 실행 차단 (이중 가드)
        if ((pos as any).cellIndex !== undefined && (pos as any).cellIndex >= 0) return;
        try {
            const result = services.wasm.insertEquation(
                pos.sectionIndex, pos.paragraphIndex, pos.charOffset,
                '', 1000, 0x00000000
            );
            if (result.ok) {
                services.eventBus.emit('document-changed');
                if (!equationEditorDialog) {
                    equationEditorDialog = new EquationEditorDialog(services.wasm, services.eventBus);
                }
                equationEditorDialog.open(pos.sectionIndex, result.paraIdx, result.controlIdx);
            }
        } catch (err) {
            console.warn('[insert:equation] 수식 삽입 실패:', err);
        }
    },
}
```

빈 수식 삽입 후 **수식 편집 대화상자 자동 진입** — PR #738 (closes #144) 영역 영역 의 EquationEditorDialog 영역 영역 자동 활용.

### 3.5 `rhwp-studio/index.html` (+1)

입력 메뉴 영역 영역 "수식 (Ctrl+N,M)" 항목 영역 영역 추가.

## 4. Copilot 리뷰 반영 (commit `8a37adbd`)
- **셀 내부 실행 차단**: `canExecute: !ctx.inTableCellEditing` + execute 영역 영역 가드 (이중 방어)
- **CTRL_EQUATION 상수 사용**: magic number `0x65716564` ('eqed') → `CTRL_EQUATION` 상수
- **control_mask 비트 설정**: `paragraph.control_mask |= 1u32 << 11` — 한컴 호환 정합

## 5. 인프라 재사용 점검

| 인프라 | 활용 |
|--------|------|
| `insertFootnote` 패턴 | `insert_equation_native()` 영역 영역 정합 (controls.insert + char_offsets gap + reflow + paginate) |
| `EquationEditorDialog` (PR #738 영역) | 빈 수식 삽입 후 자동 진입 — 듀얼 모드 + 자동완성 + 탭 130+ 템플릿 + 기호 검색 |
| `renderEquationPreview` / `getEquationProperties` / `setEquationProperties` | 기존 API 보존 |
| `Control::Equation` IR | 기존 IR 활용 — 백엔드 영역 영역 신규 모델 부재 |

→ `feedback_process_must_follow` 정합 — 인프라 재사용 영역 영역 위험 좁힘.

## 6. 본 환경 점검

- merge-base: `c9dd6f9c` (5/9 영역 영역 가까움)
- merge-tree 충돌: **0건** ✓
- 변경 영역 영역 격리: 5 files 영역 영역 모두 신규 추가 — 기존 영역 영역 변경 부재
- 시각 출력 영역 영역 영영 부재 — 신규 API 영역 영역 호출 시 영역 영역 만 영향

## 7. 영향 범위

### 7.1 변경 영역
- 본문 캐럿 위치 영역 영역 수식 신규 삽입 (`insertEquation` API)
- rhwp-studio 입력 메뉴 영역 영역 "수식 (Ctrl+N,M)" 항목

### 7.2 무변경 영역
- 기존 수식 편집 (`EquationEditorDialog`, PR #738 영역)
- HWP3/HWPX 변환본 영역 영역 시각 정합 (광범위 sweep 영역 영역 영영 부재)
- 기존 WASM API (`renderEquationPreview` / `getEquationProperties` / `setEquationProperties`)

### 7.3 위험 영역
- **opt-in** — 메뉴 항목 영역 영역 명시 호출 시 영역 영역 만 영향
- 표 셀 내부 / 글상자 내부 실행 차단 (이중 가드)

## 8. 충돌 / mergeable

- `git merge-tree --write-tree`: **CONFLICT 0건** ✓
- BEHIND — devel 영역 영역 5/10 사이클 영역 영역 진전, 본 PR 영역 영역 신규 추가 영역 영역 충돌 부재

## 9. 처리 옵션

### 옵션 A — 2 commits cherry-pick + no-ff merge (추천)

```bash
git checkout -b local/task731 199ad8dd
git cherry-pick b64590ab 8a37adbd
git checkout local/devel
git merge --no-ff local/task731
```

→ **옵션 A 추천**.

## 10. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` — 전체 lib + 통합 ALL GREEN
- [ ] `cargo clippy --release --all-targets -- -D warnings` clean
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (신규 API 영역 영역 호출 부재 영역 영역 영향 부재 보장)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 영역 영역 의 본질 영역 영역 **rhwp-studio editor 영역 영역 사용자 인터랙션 (수식 신규 삽입 + 편집 진입)**:
- WASM 빌드 후 dev server 영역 영역 입력 → 수식 (Ctrl+N+M) 영역 영역 점검
  - 본문 캐럿 위치 영역 영역 빈 수식 삽입 정합
  - 수식 편집 대화상자 자동 진입 (PR #738 영역 영역 의 듀얼 모드 + 자동완성 + 탭 / 기호 검색)
  - 표 셀 내부 영역 영역 실행 차단 점검
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 영역 영역 PR #728/#729/#730/#734/#735/#737/#738/#739 영역 8번째 PR) |
| `feedback_image_renderer_paths_separate` | rhwp-studio TypeScript + Rust 영역 영역 격리 — 다른 layout/render 경로 영역 영역 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (insertFootnote 패턴 + EquationEditorDialog) — 신규 인프라 도입 부재 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 — Canvas visual diff CI + 작업지시자 인터랙션 검증 권장 |
| `project_equation_always_tac` | 한컴 수식 영역 영역 항상 TAC — `treat_as_char: true` 정합 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 2 commits cherry-pick (옵션 A)
2. 자기 검증 (cargo test/build/clippy + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (dev server 영역 영역 입력 → 수식 메뉴)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #739 close (closes #731 자동 정합)

---

작성: 2026-05-10
