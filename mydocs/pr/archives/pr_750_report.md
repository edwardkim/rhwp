---
PR: #750
제목: feat — 다단 설정 대화상자 구현 (closes #733)
컨트리뷰터: @oksure (Hyunwoo Park) — 20+ 사이클 핵심 컨트리뷰터 (5/10 사이클 18번째 PR)
처리: 옵션 A — 2 commits cherry-pick + no-ff merge
처리일: 2026-05-10
머지 commit: 577befdd
---

# PR #750 처리 보고서

## 1. 처리 결과

✅ **머지 완료** — 옵션 A (2 commits cherry-pick + no-ff merge)

| 항목 | 값 |
|------|-----|
| 머지 commit | `577befdd` (--no-ff merge) |
| Cherry-pick commits | `2e6b5ddf` (feat) + `1f88877f` (Copilot 리뷰) |
| closes | #733 (다단 생성후 단설정 활성화 부재) |
| 시각 판정 | ✅ 작업지시자 웹 에디터 인터랙션 검증 통과 |
| 자기 검증 | tsc + cargo test ALL GREEN + sweep 170/170 same + WASM 4.66 MB |

## 2. 정정 본질 — 5 files, +156/-1

### 2.1 `src/wasm_api.rs` (+17, Rust)
`getColumnDef` WASM API 신규 (조회 전용) — `find_initial_column_def` 헬퍼 재사용 + JSON 수동 포맷.

### 2.2 `rhwp-studio/src/core/wasm-bridge.ts` (+5)
WASM 바인딩 추가.

### 2.3 `rhwp-studio/src/ui/column-settings-dialog.ts` (+119, 신규 파일)
`ColumnSettingsDialog extends ModalDialog`:
- 단 수 (1~8) — input number + clamp
- 종류 (일반/배분/평행) — select
- 너비 동일 — checkbox
- 간격 (mm 단위, 0~32767 HWPUNIT clamp)

### 2.4 `rhwp-studio/src/command/commands/page.ts` (+13/-1)
`stub('page:col-settings', ...)` → 실제 구현.

### 2.5 `rhwp-studio/src/command/shortcut-map.ts` (+1)
`Ctrl+Alt+Enter` 단축키 매핑 (한컴 표준 정합).

## 3. Copilot 리뷰 반영 (commit `1f88877f`)
- `onOk()` → `ModalDialog` 추상 메서드 `onConfirm()` 정합
- `this.close()` 제거 (base class 자동 hide)
- count(1~8), type(0~2), spacing(0~32767) 범위 클램프 추가
- `HWPUNIT_PER_MM` 하드코딩 → `7200/25.4` 계산식 (정밀도)

## 4. 인프라 재사용

| 인프라 | 활용 |
|--------|------|
| `setColumnDef` WASM API (기존) | 설정 적용 영역 영역 직접 호출 |
| `find_initial_column_def` 헬퍼 (기존) | `getColumnDef` 신규 API 영역 내부 활용 |
| `ModalDialog` (기존) | `ColumnSettingsDialog` 베이스 클래스 |
| `SectionSettingsDialog` 패턴 | 동일 호출 패턴 |
| `ColumnDef` / `ColumnType` (기존 IR) | `getColumnDef` JSON 직렬화 |

→ 신규 인프라 1개만 (`getColumnDef` 조회) 영역 영역 위험 좁힘 (`feedback_process_must_follow` 정합).

## 5. 본 환경 검증

| 검증 | 결과 |
|------|------|
| `cherry-pick` (2 commits) | ✅ auto-merge 충돌 0건 |
| `tsc --noEmit` (rhwp-studio) | ✅ 통과 |
| `cargo test --release` | ✅ ALL GREEN |
| 광범위 sweep (7 fixture / 170 페이지) | ✅ **170 same / 0 diff** (Rust 조회 API 추가 영역 영역 SVG 무영향) |
| WASM 빌드 (Docker) | ✅ 4.66 MB |

## 6. 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과
- Ctrl+Alt+Enter 단축키 → ColumnSettingsDialog 호출
- 메뉴 → "쪽 → 다단 설정" → 동일 동작
- 단 수 / 종류 / 너비 동일 / 간격 변경 → 다단 적용
- 기존 다단 설정 영역 영역 dialog 영역 영역 현재 설정 표시

## 7. 영향 범위

### 7.1 변경 영역
- Rust: `wasm_api.rs` 영역 영역 1 메서드 추가 (조회 전용)
- TypeScript: 신규 파일 1개 + 기존 파일 4개 영역 영역 wired

### 7.2 무변경 영역
- 렌더링 경로 (sweep 170/170 same 입증)
- 기존 `setColumnDef` API (재사용 만)

## 8. 메모리 룰 적용

| 룰 | 적용 |
|----|------|
| `feedback_contributor_cycle_check` | @oksure **20+ 사이클** (5/10 사이클 18번째 PR) |
| `feedback_image_renderer_paths_separate` | wasm_api 1 메서드 + TypeScript UI 영역 — Rust 렌더링 경로 무영향 (sweep 170/170 same 입증) |
| `feedback_process_must_follow` | 인프라 재사용 (`SectionSettingsDialog` 패턴 + `setColumnDef` API + `ModalDialog`) — `getColumnDef` 1 메서드 만 신규 영역 영역 위험 좁힘 |
| `feedback_visual_judgment_authority` | 작업지시자 웹 에디터 인터랙션 검증 ✅ 통과 |

## 9. 잔존 후속

- 본 PR 본질 정정의 잔존 결함 부재
- Issue #733 close 완료

---

작성: 2026-05-10
