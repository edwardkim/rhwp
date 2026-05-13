---
PR: #809
제목: feat — 쪽 새 번호로 시작: insertNewNumber WASM API + dialog UI (closes #791)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 9번째 PR
base / head: devel / contrib/insert-new-number
mergeStateStatus: DIRTY
mergeable: CONFLICTING
CI: 결과 부재 (DIRTY 영역)
변경 규모: +174 / -1, 5 files
커밋: 3
검토일: 2026-05-11
---

# PR #809 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #809 |
| 제목 | feat: 쪽 새 번호로 시작 — insertNewNumber WASM API + dialog UI (#791) |
| 컨트리뷰터 | @oksure (Hyunwoo Park) — 20+ 사이클 핵심 (5/11 사이클 **9번째 PR** — #786 → #787 → #788 → #794 → #795 → #796 → #807 → #808 → **#809**) |
| base / head | devel / contrib/insert-new-number |
| mergeable | CONFLICTING (DIRTY — 4 파일 충돌) |
| CI | 결과 부재 |
| 변경 규모 | +174 / -1, 5 files |
| 커밋 수 | 3 (1 본질 + 1 리뷰 반영 + 1 Copilot 리뷰 반영) |
| closes | #791 |
| 관련 | PR #755 close 후속 (잘못된 API 호출), PR #745 (Task #634 NewNumber Page 표시) |

## 2. 본질 (Issue #791)

`page:new-page-num` (쪽 > 새 번호로 시작) stub 커맨드 영역 실동작 구현.

### 두 본질 분리 (Issue #791 명시)
| 영역 | 의미 | 본 환경 API |
|------|------|------------|
| paragraph numbering | 1.1.1 / 가/나 자동 번호 | `setNumberingRestart` (기존, 본 PR 무관) |
| **page number (쪽번호)** | footer 의 "1쪽, 2쪽" | **`NewNumber` 컨트롤 — 본 PR 영역 신규 `insertNewNumber` API** |

### PR #755 영역 close 이력
PR #755 영역 영역 `setNumberingRestart` (paragraph numbering) 영역 잘못 매핑 영역 close. 본 PR 영역 `NewNumber` 컨트롤 영역 신규 WASM API 영역 정확.

## 3. 정정 본질 — 5 files, +174/-1

### 3.1 Rust WASM API (2 files)

**`src/document_core/commands/object_ops.rs` (+63)** — `insert_new_number_native`:
```rust
pub fn insert_new_number_native(
    &mut self, section_idx: usize, para_idx: usize,
    char_offset: usize, start_num: u16,
) -> Result<String, HwpError>
```
- `find_control_text_positions` 영역 삽입 인덱스 결정 (char_offset 기준)
- `paragraph.controls.insert(insert_idx, Control::NewNumber(...))`
- `ctrl_data_records.insert(insert_idx, None)` 정합
- `char_offsets` +8 조정 + `control_mask |= 1u32 << 0x0012` 갱신
- `reflow_paragraph` → `recompose_section` → `paginate_if_needed` → `invalidate_page_tree_cache` 후처리
- 범위 가드 (section_idx / para_idx)

**`src/wasm_api.rs` (+21)** — `insertNewNumber` 바인딩:
```rust
#[wasm_bindgen(js_name = insertNewNumber)]
pub fn insert_new_number(
    &mut self, section_idx: u32, para_idx: u32, char_offset: u32, start_num: u32,
) -> Result<String, JsValue>
```
- **start_num 범위 검증** (리뷰 반영 commit `d764dfb2`): `1 <= start_num <= 65535` (u16 범위)

### 3.2 TypeScript (3 files)

**`rhwp-studio/src/ui/new-number-dialog.ts` (+65, 신규)** — `NewNumberDialog`:
- `ModalDialog` 패턴 정합
- 시작 번호 input (type=number, min=1, max=65535, default=1)
- `onConfirm` 영역 `wasm.insertNewNumber` 호출 + `eventBus.emit('document-changed')`
- 범위 검증 (NaN/<1/>65535 시 false 반환 영역 dialog 유지)

**`rhwp-studio/src/core/wasm-bridge.ts` (+5)** — `insertNewNumber` 래퍼:
```typescript
insertNewNumber(sec: number, para: number, charOffset: number, startNum: number): string {
  if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
  return (this.doc as any).insertNewNumber(sec, para, charOffset, startNum);
}
```

**`rhwp-studio/src/command/commands/page.ts` (+19/-1)** — stub 영역 실커맨드 교체:
- `canExecute: (ctx) => ctx.hasDocument && !ctx.inTable` (Copilot 리뷰 반영 commit `05f487df`)
- `execute` 영역 `NewNumberDialog` 호출 + 커서 위치 전달

### 3.3 Copilot 리뷰 반영 (commit `05f487df`)
`!inTable` 가드 — 표 셀 내부 영역 NewNumber 삽입 금지 (한컴 호환 — paragraph 컨트롤 영역 본문만).

## 4. PR #745 (Task #634) 정합

PR #745 영역 `typeset.rs` 영역 `Control::NewNumber` 영역 스캔 → `new_page_numbers` 벡터 적재 → 페이지 렌더링 영역 해당 번호부터 표시. 본 PR 영역 `insertNewNumber` API 영역 페이지 번호 정합 — PR #745 인프라 활용.

## 5. 본 환경 충돌 분석

### 5.1 4 파일 충돌
| 파일 | base | our (devel) | their (PR) |
|------|------|-------------|------------|
| `rhwp-studio/src/command/commands/page.ts` | e4914985 | 8bbeca61 | ad31fe40 |
| `rhwp-studio/src/core/wasm-bridge.ts` | 874d01a0 | ae5d6e83 | cfd3748e |
| `src/document_core/commands/object_ops.rs` | 5548b345 | 43d424d7 | 68b130e4 |
| `src/wasm_api.rs` | f442f5e5 | ab55a34b | 4fdfe264 |

devel 5/11 누적 변경 (PR #786~#808 영역) 영역 비대칭. 그러나 본 PR 영역 영역 신규 추가 (delete 부재) → 정합 영역 영역 우리 측 (devel) 보존 + PR 측 신규 영역 추가 정합 영역 영역 어려움 부재 예상.

### 5.2 정합 전략
- `wasm_api.rs` / `object_ops.rs` — 함수 끝 영역 영역 PR 측 신규 함수 추가 정합 (impl 블록 영역 영역 추가)
- `wasm-bridge.ts` / `page.ts` — 영역 영역 PR 측 신규 메서드 / 커맨드 추가 정합 (기존 영역 영역 무관)
- `new-number-dialog.ts` — 신규 파일 영역 영역 충돌 부재

## 6. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `find_control_text_positions` (기존 helpers) | 삽입 인덱스 결정 |
| `Control::NewNumber` + `AutoNumberType::Page` (기존 model) | 컨트롤 변형 |
| `reflow_paragraph` / `recompose_section` / `paginate_if_needed` / `invalidate_page_tree_cache` (기존 후처리) | 페이지 갱신 |
| `ModalDialog` (기존 UI 패턴) | dialog 구조 |
| PR #745 (Task #634) `typeset.rs` NewNumber 표시 인프라 | 페이지 번호 갱신 |

→ 신규 인프라 도입 부재 — 기존 model + helpers + UI 패턴 + PR #745 표시 인프라 모두 활용.

## 7. 영역 좁힘 (회귀 부재 가드)

- `start_num` 범위 검증 (`1 <= start_num <= 65535`, u16 범위)
- `!inTable` 가드 (표 셀 내부 삽입 금지, Copilot 리뷰)
- `section_idx` / `para_idx` 범위 가드
- `safe_offset = char_offset.min(text_len)` — char_offset 초과 시 안전
- 신규 API 영역 영역 기존 동작 무영향 (opt-in)

## 8. CI 결과 부재

mergeStateStatus = `DIRTY` 영역 CI 미실행. 충돌 해결 + 본 환경 검증 필수.

## 9. 처리 옵션

### 옵션 A (권장) — 3 commits cherry-pick + 충돌 수동 해결 + no-ff merge

```bash
git checkout local/devel
git cherry-pick 7b8016a0 d764dfb2 05f487df
# 4 파일 충돌 수동 해결 (devel 측 + PR 측 신규 추가 정합)
git checkout devel
git merge local/devel --no-ff
```

### 옵션 B — squash cherry-pick + 충돌 수동 해결

3 commits 영역 단일 commit. 본 환경 영역 영역 commit 이력 보존 권장 옵션 A.

## 10. 검증 게이트

### 10.1 자기 검증
- [ ] cherry-pick 3 commits + 4 파일 충돌 수동 해결
- [ ] tsc --noEmit
- [ ] cargo test + cargo clippy
- [ ] WASM 재빌드 (Rust 신규 API 영역 영역 재빌드 필수)
- [ ] 광범위 sweep (Rust 변경 영역 영역 sweep 필수)

### 10.2 시각/인터랙션 판정 게이트 — **작업지시자 인터랙션 검증 권장**
- 메뉴 → "쪽 → 새 번호로 시작" → dialog 표시
- 시작 번호 입력 (예: 5) → 확인 → 페이지 번호 갱신 (현재 페이지부터 5쪽)
- PR #745 정합 — 첫 NewNumber 발화 후 페이지 번호 표시 시작
- 표 셀 내부 영역 영역 메뉴 비활성 (`!inTable` 가드)
- 범위 외 입력 (0 / 65536) 영역 영역 dialog 유지

## 11. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 9번째 PR) |
| `feedback_image_renderer_paths_separate` | Rust + TypeScript 변경 영역 영역 sweep 영향 점검 필수 (paragraph controls 영역 영역 typeset 영역 영역 NewNumber 표시 정합) |
| `feedback_process_must_follow` | 인프라 재사용 (model + helpers + UI 패턴 + PR #745) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `!inTable` 가드 + `start_num` 범위 검증 + `safe_offset` 가드 영역 영역 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | paragraph numbering vs page number 두 본질 분리 (Issue #791 명시) — PR #755 잘못된 매핑 본질 진단 |
| `feedback_visual_judgment_authority` | NewNumber 페이지 번호 영역 영역 한컴 viewer + 한글 2022 PDF 정합 검증 권장 |
| `feedback_pr_supersede_chain` | PR #755 (close, 잘못된 API) → **PR #809** (본질 정정, 신규 NewNumber API) — (a) 패턴 + PR #745 (NewNumber 표시) 영역 영역 본 PR (NewNumber 삽입) 정합 — 한컴 호환 단계적 진전 |

## 12. 처리 순서 (승인 후)

1. `local/devel` 영역 cherry-pick 3 commits + 4 파일 충돌 수동 해결
2. 자기 검증 (tsc + cargo test + cargo clippy + sweep)
3. WASM 재빌드
4. 작업지시자 웹 에디터 인터랙션 검증 (메뉴 + dialog + 페이지 번호 갱신)
5. 검증 통과 → no-ff merge + push + archives + 5/11 orders + Issue #791 close
6. PR #809 close

---

작성: 2026-05-11
