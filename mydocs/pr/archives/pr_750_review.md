---
PR: #750
제목: feat: 다단 설정 대화상자 구현 (closes #733)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 18번째 PR)
base / head: devel / contrib/column-settings-dialog
mergeStateStatus: BEHIND
mergeable: MERGEABLE
CI: SUCCESS (Build & Test + CodeQL js/ts/py/rust + Canvas visual diff)
변경 규모: +156 / -1, 5 files
검토일: 2026-05-10
---

# PR #750 검토

## 1. 메타

| 항목 | 값 |
|------|-----|
| PR 번호 | #750 |
| 제목 | feat: 다단 설정 대화상자 구현 (closes #733) |
| 컨트리뷰터 | @oksure — 20+ 사이클 (5/10 사이클 18번째 PR) |
| base / head | devel / contrib/column-settings-dialog |
| mergeStateStatus | BEHIND, mergeable: MERGEABLE — 충돌 부재 |
| CI | ✅ Build & Test + CodeQL (js/ts/py/rust) + Canvas visual diff 통과 |
| 변경 규모 | +156 / -1, 5 files |
| 커밋 수 | 2 (feat + Copilot 리뷰) |
| closes | #733 (다단 생성후 단설정 활성화 부재) |

## 2. 결함 본질 (Issue #733)

다단 생성 후 단설정 (메뉴 또는 `Ctrl+Alt+Enter` 단축키) 호출 부재. 메뉴 항목 영역 영역 `stub('page:col-settings', '다단 설정', undefined, 'Ctrl+Alt+Enter')` 영역 영역 placeholder 만 존재 영역 영역 실제 동작 미구현.

### 2.1 채택 접근 — `SectionSettingsDialog` 패턴 정합

기존 `SectionSettingsDialog` (구역 설정 대화상자) 영역 동일 `ModalDialog` 베이스 클래스 + 동일 호출 패턴 (`new XxxDialog(services.wasm, services.eventBus, sectionIdx)`):

| 항목 | SectionSettingsDialog (기존) | ColumnSettingsDialog (본 PR) |
|------|----------------------------|------------------------------|
| 베이스 클래스 | `ModalDialog` | `ModalDialog` |
| 호출 | `new SectionSettingsDialog(wasm, eventBus, sectionIdx).show()` | `new ColumnSettingsDialog(wasm, eventBus, sectionIdx).show()` |
| onConfirm | `ModalDialog` 추상 메서드 | 동일 |
| 호출 위치 | `page:section-settings` 커맨드 | `page:col-settings` 커맨드 |

### 2.2 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `setColumnDef` WASM API (기존, `wasm_api.rs:1172`) | 다단 설정 적용 영역 영역 직접 호출 |
| `find_initial_column_def` 헬퍼 (기존, `wasm_api.rs:77`) | `getColumnDef` 신규 API 영역 내부 활용 |
| `ModalDialog` (기존) | `ColumnSettingsDialog` 베이스 클래스 |
| `SectionSettingsDialog` 패턴 | 동일 호출 패턴 |
| `ColumnDef` / `ColumnType` (기존 IR) | `getColumnDef` 영역 영역 JSON 직렬화 |

→ 신규 인프라 도입은 **`getColumnDef` WASM API 1개** 만 (조회 전용). 설정 적용은 기존 `setColumnDef` 재사용.

## 3. PR 의 정정 — 5 files, +156/-1

### 3.1 `src/wasm_api.rs` (+17, Rust)

```rust
#[wasm_bindgen(js_name = getColumnDef)]
pub fn get_column_def(&self, section_idx: u32) -> Result<String, JsValue> {
    let sec = self.core.document.sections.get(section_idx as usize)
        .ok_or_else(|| JsValue::from_str("구역 인덱스 범위 초과"))?;
    let col_def = HwpDocument::find_initial_column_def(&sec.paragraphs);
    let col_type = match col_def.column_type {
        crate::model::page::ColumnType::Normal => 0,
        crate::model::page::ColumnType::Distribute => 1,
        crate::model::page::ColumnType::Parallel => 2,
    };
    Ok(format!(
        "{{\"columnCount\":{},\"columnType\":{},\"sameWidth\":{},\"spacing\":{}}}",
        col_def.column_count, col_type, col_def.same_width, col_def.spacing,
    ))
}
```

기존 `find_initial_column_def` 재사용 + JSON 수동 포맷 (rhwp-studio JSON.parse 영역 영역 단순 객체 영역 영역 충분).

### 3.2 `rhwp-studio/src/core/wasm-bridge.ts` (+5)

```typescript
getColumnDef(sec: number): { columnCount: number; columnType: number; sameWidth: boolean; spacing: number } {
  if (!this.doc) throw new Error('문서가 로드되지 않았습니다');
  return JSON.parse((this.doc as any).getColumnDef(sec));
}
```

WASM 바인딩 추가.

### 3.3 `rhwp-studio/src/ui/column-settings-dialog.ts` (+119, 신규 파일)

`ColumnSettingsDialog extends ModalDialog`:
- 단 수 (1~8) — input number + clamp
- 종류 (일반/배분/평행) — select 3 options
- 너비 동일 — checkbox
- 간격 (mm 단위, 0~32767 HWPUNIT clamp) — input number
- `populateFields()`: `getColumnDef` 호출 → 실패 시 default (1단/일반/동일/8mm)
- `onConfirm()`: clamp 적용 후 `setColumnDef` 호출 → `eventBus.emit('document-changed')`

### 3.4 `rhwp-studio/src/command/commands/page.ts` (+13/-1)

`stub('page:col-settings', ...)` → 실제 구현 (이전 placeholder 제거 + ColumnSettingsDialog 호출).

### 3.5 `rhwp-studio/src/command/shortcut-map.ts` (+1)

```typescript
[{ key: 'enter', ctrl: true, alt: true }, 'page:col-settings'],
```

`Ctrl+Alt+Enter` 단축키 (한컴 영역 영역 표준 단축키 정합).

## 4. Copilot 리뷰 반영 (commit `a8f2f319`)
- `onOk()` → `ModalDialog` 추상 메서드 `onConfirm()` 정합
- `this.close()` 제거 (base class 영역 영역 onConfirm 반환값 영역 영역 자동 hide)
- count(1~8), type(0~2), spacing(0~32767) 범위 클램프 추가
- `HWPUNIT_PER_MM` 하드코딩 `283.46` → `7200/25.4` 계산식 (정밀도)

## 5. 충돌 / mergeable

- `src/wasm_api.rs` — devel HEAD 영역 영역 추가/변경 가능 영역 영역 (PR #739 등 영역 영역) → 점검 필요
- `rhwp-studio/src/core/wasm-bridge.ts` — devel HEAD 영역 영역 추가 가능 영역 (다른 PR 영역) → 점검 필요
- `rhwp-studio/src/command/commands/page.ts` — devel 영역 안정 영역
- `rhwp-studio/src/command/shortcut-map.ts` — PR #749 영역 영역 +2 (Ctrl+O 매핑) — Ctrl+Alt+Enter 영역 영역 다른 영역 영역 충돌 부재
- `rhwp-studio/src/ui/column-settings-dialog.ts` — 신규 파일, 충돌 부재

mergeable=MERGEABLE → cherry-pick 충돌 0건 예상 (auto-merge 정합).

## 6. 본 환경 점검

### 6.1 변경 격리
- Rust: `wasm_api.rs` 영역 영역 1 메서드 추가 (조회 전용)
- TypeScript: 신규 파일 1개 + 기존 파일 4개 영역 영역 영역 wired
- 렌더링 경로 무관 (`feedback_image_renderer_paths_separate` 정합)

### 6.2 CI 결과
- Build & Test ✅ (cargo test 1173 통과 PR 본문 명시)
- CodeQL (js/ts/py/rust) ✅
- Canvas visual diff ✅
- WASM Build SKIPPED (CI 영역 영역 변경 무관 자동 판정)

### 6.3 의도적 제한 (PR 본문 미명시 영역 영역 본 환경 추론)
- `populateFields()` `getColumnDef` 실패 시 default 폴백 (1단/일반/동일/8mm) — **데이터 손실 가능 영역 영역 점검 필요**: 이미 다단 설정된 구역 영역 영역 `getColumnDef` 실패 시 기존 설정 영역 영역 1단 default 영역 영역 덮어쓰기 발생 가능 영역? 그러나 `find_initial_column_def` 영역 영역 안전 default 반환 영역 영역 실제 throw 발생 부재 — 운영 영역 영역 영향 부재 추정.

## 7. 처리 옵션

### 옵션 A — 2 commits 개별 cherry-pick + no-ff merge

```bash
git checkout local/devel
git cherry-pick b7dd44b9 a8f2f319
git checkout devel
git merge local/devel --no-ff -m "Merge PR #750 (closes #733): feat 다단 설정 대화상자"
```

→ **권장**.

## 8. 검증 게이트 (구현 단계 DoD)

### 자기 검증
- [ ] cherry-pick 충돌 0건
- [ ] `cargo build --release` 통과
- [ ] `cargo test --release` ALL GREEN (PR 본문 cargo test 1173 통과 명시)
- [ ] `cd rhwp-studio && npx tsc --noEmit` 통과
- [ ] 광범위 sweep — 7 fixture / 170 페이지 회귀 0 (rhwp-studio editor 신규 UI 영역 영역 SVG 무영향 입증)

### 시각 판정 게이트 — **WASM 빌드 + 작업지시자 인터랙션 검증 권장**

본 PR 본질은 **rhwp-studio editor 다단 설정 대화상자 신규 UI**:
- WASM 빌드 후 dev server 영역 영역:
  - `Ctrl+Alt+Enter` 단축키 → ColumnSettingsDialog 호출
  - 메뉴 영역 영역 "쪽 → 다단 설정" → 동일 동작
  - 단 수 / 종류 / 너비 동일 / 간격 변경 → 다단 적용 정합
  - 기존 다단 영역 영역 dialog 영역 영역 현재 설정 표시 정합
- E2E 자동 테스트 신규 부재 → 작업지시자 직접 인터랙션 검증 권장

## 9. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure 20+ 사이클 (5/10 사이클 18번째 PR) |
| `feedback_image_renderer_paths_separate` | wasm_api 1 메서드 + TypeScript UI 영역 — Rust 렌더링 경로 무영향 |
| `feedback_process_must_follow` | 인프라 재사용 (`SectionSettingsDialog` 패턴 + `setColumnDef` API + `ModalDialog`) — `getColumnDef` 1 메서드 만 신규 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | rhwp-studio editor 인터랙션 영역 영역 작업지시자 인터랙션 검증 권장 |

## 10. 처리 순서 (승인 후)

1. `local/devel` 영역 영역 옵션 A cherry-pick (2 commits)
2. 자기 검증 (cargo test + tsc + 광범위 sweep)
3. WASM 빌드 + 작업지시자 인터랙션 검증 (Ctrl+Alt+Enter + 다단 설정 적용)
4. 인터랙션 검증 통과 → no-ff merge + push + archives 이동 + 5/10 orders 갱신
5. PR #750 close (closes #733 자동 정합)

---

작성: 2026-05-10
