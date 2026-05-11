---
PR: #809
제목: feat — 쪽 새 번호로 시작: insertNewNumber WASM API + dialog UI (closes #791)
컨트리뷰터: @oksure (Hyunwoo Park) — 5/11 사이클 9번째 PR
처리: 옵션 A — 3 commits cherry-pick + 2 파일 충돌 수동 해결 + no-ff merge
처리일: 2026-05-11
머지 commit: 47ef8fca
---

# PR #809 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (3 commits cherry-pick + 2 파일 충돌 수동 해결 + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `47ef8fca` (--no-ff merge) |
| Cherry-pick commits | 3 (본질 + 리뷰 반영 2) |
| closes | #791 (PR #755 close 후속 정정) |
| 시각 판정 | ✅ 작업지시자 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test/clippy ALL GREEN + sweep 170/170 same + WASM 4.5 MB |

## 2. 본질 (Issue #791)

`page:new-page-num` (쪽 > 새 번호로 시작) stub 커맨드 → 실동작 구현. PR #755 영역 `setNumberingRestart` (paragraph numbering) 영역 잘못 매핑 close 후, `NewNumber` 컨트롤 영역 신규 `insertNewNumber` API 정확.

### 두 본질 분리 (Issue #791 명시)
| 영역 | 의미 | 본 환경 API |
|------|------|------------|
| paragraph numbering | 1.1.1 / 가/나 자동 번호 | `setNumberingRestart` (기존, 본 PR 무관) |
| **page number (쪽번호)** | footer 의 "1쪽, 2쪽" | **`NewNumber` 컨트롤 영역 신규 `insertNewNumber`** |

## 3. 정정 본질 — 5 files, +174/-1

### 3.1 Rust WASM API (2 files)

**`src/document_core/commands/object_ops.rs` (+63)** — `insert_new_number_native`:
- `find_control_text_positions` 영역 삽입 인덱스 결정 (char_offset 기준)
- `paragraph.controls.insert(insert_idx, Control::NewNumber(NewNumber { number_type: Page, number: start_num }))`
- `ctrl_data_records.insert(insert_idx, None)` 정합
- `char_offsets[safe_offset..]` += 8 조정 + `control_mask |= 1u32 << 0x0012`
- `reflow_paragraph` → `recompose_section` → `paginate_if_needed` → `invalidate_page_tree_cache` 후처리
- `section_idx` / `para_idx` 범위 가드

**`src/wasm_api.rs` (+21)** — `insertNewNumber` 바인딩:
- `start_num == 0 || start_num > 65535` → `Err` (u16 범위 검증, 리뷰 반영 commit `05a3f5e6`)

### 3.2 TypeScript (3 files)

**`rhwp-studio/src/ui/new-number-dialog.ts` (+65, 신규)** — `NewNumberDialog`:
- `ModalDialog` 패턴 정합
- 시작 번호 input (type=number, min=1, max=65535, default=1)
- `onConfirm` 영역 `wasm.insertNewNumber` 호출 + `eventBus.emit('document-changed')`
- 범위 검증 (NaN/<1/>65535 시 false 반환 영역 dialog 유지)

**`rhwp-studio/src/core/wasm-bridge.ts` (+5)** — `insertNewNumber` 래퍼

**`rhwp-studio/src/command/commands/page.ts` (+19/-1)** — stub 영역 실커맨드 교체:
- `canExecute: (ctx) => ctx.hasDocument && !ctx.inTable` (Copilot 리뷰 반영 commit `f285df13`)
- `execute` 영역 `NewNumberDialog` 호출 + 커서 위치 전달

### 3.3 리뷰 반영 commits
- `05a3f5e6` — start_num 범위 검증 (1~65535, u16) + 이벤트명 정정
- `f285df13` — Copilot 리뷰 `!inTable` 가드 (표 셀 삽입 금지, 한컴 호환)

## 4. PR #745 (Task #634) 정합

PR #745 영역 `typeset.rs` 가 `Control::NewNumber` 스캔 → `new_page_numbers` 적재 → 페이지 렌더링 영역 해당 번호부터 표시. 본 PR 영역 `insertNewNumber` API 영역 페이지 번호 정합 — PR #745 인프라 활용.

## 5. 본 환경 충돌 수동 해결 (2 파일)

| 파일 | 본질 |
|------|------|
| `rhwp-studio/src/command/commands/page.ts` | import 양쪽 보존 (devel `ColumnSettingsDialog` + incoming `NewNumberDialog`) |
| `rhwp-studio/src/core/wasm-bridge.ts` | 두 메서드 모두 보존 (devel `getColumnDef` + incoming `insertNewNumber`) |

`src/document_core/commands/object_ops.rs` / `src/wasm_api.rs` — auto-merge 성공 (impl 블록 끝 추가 영역).

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
- 신규 API opt-in 영역 영역 기존 동작 무영향 (sweep 170/170 same 입증)

## 8. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` 3 commits + 2 파일 충돌 수동 해결 | ✅ |
| `tsc --noEmit` | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| `cargo clippy --release --lib -- -D warnings` | ✅ 통과 |
| **광범위 sweep (7 fixture / 170 페이지)** | ✅ **170 same / 0 diff** (신규 API opt-in 입증) |
| WASM 빌드 (Docker) | ✅ 4.5 MB |

## 9. 작업지시자 인터랙션 검증 ✅ 통과

- 메뉴 → "쪽 → 새 번호로 시작" → dialog 표시
- 시작 번호 입력 → 확인 → 페이지 번호 갱신 정합
- PR #745 정합 — 첫 NewNumber 발화 후 페이지 번호 표시
- 표 셀 내부 영역 영역 메뉴 비활성 (`!inTable` 가드)
- 범위 외 입력 (0 / 65536) 영역 영역 dialog 유지
- 기존 페이지 번호 표시 회귀 부재

## 10. CI 결과 부재 (DIRTY 영역)

mergeStateStatus = `DIRTY` 영역 영역 CI 미실행. 본 환경 자기 검증 + sweep + 작업지시자 시각 판정 통과 영역 영역 보완.

## 11. 영향 범위

### 11.1 변경 영역
- Rust: `document_core/commands/object_ops.rs` (+63) + `wasm_api.rs` (+21)
- TypeScript: `rhwp-studio/src/ui/new-number-dialog.ts` (신규 +65) + `core/wasm-bridge.ts` (+5) + `command/commands/page.ts` (+19/-1)

### 11.2 무변경 영역
- HWP3/HWPX 변환본 시각 정합 (sweep 170/170 same 입증)
- 기존 페이지 번호 표시 (PR #745 인프라 활용만)
- 기존 paragraph numbering (`setNumberingRestart` 무관)

## 12. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/11 사이클 9번째 PR — #786 → #787 → #788 → #794 → #795 → #796 → #807 → #808 → **#809**) |
| `feedback_image_renderer_paths_separate` | Rust paragraph controls 변경 영역 영역 sweep 필수 진행 영역 영역 170/170 same 회귀 부재 입증 |
| `feedback_process_must_follow` | 인프라 재사용 (model + helpers + UI 패턴 + PR #745) — 신규 인프라 도입 부재 |
| `feedback_hancom_compat_specific_over_general` | `!inTable` 가드 + `start_num` 범위 검증 + `safe_offset` 가드 + `section_idx`/`para_idx` 범위 가드 영역 영역 영역 좁힘 |
| `feedback_diagnosis_layer_attribution` | paragraph numbering vs page number 두 본질 분리 (Issue #791 명시) — PR #755 잘못된 매핑 본질 진단 |
| `feedback_visual_judgment_authority` | 작업지시자 인터랙션 검증 ✅ 통과 — NewNumber dialog + 페이지 번호 갱신 정합 |
| `feedback_pr_supersede_chain` | **권위 사례 강화** — PR #755 (close, 잘못된 API) → **PR #809** (본질 정정) (a) 패턴 + PR #745 (NewNumber 표시) → 본 PR (NewNumber 삽입) 한컴 호환 단계적 진전 |
| `feedback_small_batch_release_strategy` | 신규 API (opt-in, 하위 호환 100%) 영역 영역 PATCH cycle 머지 정합 — 활발한 컨트리뷰션 사이클 영역 영역 빠른 회전 |

## 13. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #791 close 완료
- 한컴 viewer + 한글 2022 PDF 시각 정합 추가 검증 권장 (선택)

---

작성: 2026-05-11
