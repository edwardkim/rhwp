# Task #1151 최종 결과 보고서 — 표 셀 안 이미지 삽입 (floating picture)

이슈: [#1151](https://github.com/edwardkim/rhwp/issues/1151)
브랜치: `local/task1151` → PR 대상 upstream `edwardkim:devel`

수행계획서: [task_m100_1151.md](../plans/task_m100_1151.md)
구현계획서: [task_m100_1151_impl.md](../plans/task_m100_1151_impl.md)
단계별 보고서: [Stage 1](../working/task_m100_1151_stage1.md) · [Stage 2](../working/task_m100_1151_stage2.md) · [Stage 3](../working/task_m100_1151_stage3.md)

## 1. 해결한 문제

**현상**: rhwp-studio 에서 표 셀 안에 커서를 두고 이미지를 삽입하면 이미지가 항상 표 밖 본문 문단에 배치됨. 한컴 2022 (incellpicture.hwp 검증) 와 동작 불일치.

**근본 원인**: `insert_picture_native` 가 본문 paragraph 만 받도록 설계되어 표 셀 영역에 picture 를 배치할 경로 부재.

## 2. 한컴 패턴 분석 (incellpicture.hwp dump)

한컴 2022 가 표 셀 안에 picture 를 배치할 때 사용하는 구조:

```
[2] 표: 셀들은 모두 비어있음 (paras=1, ctrls=0, text="")
[3] 그림: tac=false (treat_as_char=false)
        wrap=Square (어울림)
        horz_rel_to=Paper offset=11845
        vert_rel_to=Paper offset=15595
```

→ **floating picture (`tac=false`, wrap=Square, Paper/Page-relative offset)** 를 표와 같은 paragraph 의 sibling control 로 삽입. 셀 자체는 비어있다.

비교: `samples/pic-in-table-01.hwp` page 22 의 inline 셀 picture (tac=true, 셀 paragraph 내부) 는 rhwp/한컴 모두 클릭 시 cursor 진입 불가 — inline 의 본래 동작.

## 3. 변경 내용

### Rust

**`src/document_core/commands/object_ops.rs`** — `insert_picture_native`:
- 시그니처에 `cell_path: &[(usize, usize, usize)]` 인자 추가.
- 본문 (`cell_path.is_empty()`): 기존 inline 동작 그대로 (tac=true, pic_para + empty_para 본문에 삽입).
- 셀 (`cell_path` 있음): **floating picture 분기** — tac=false, wrap=Square, horz/vert_rel_to=Page, offset 은 셀의 render bbox 좌상단 HWPUNIT. Picture 는 표가 들어있는 paragraph 의 sibling control 로 `controls.push()`. 셀 paragraph 는 손대지 않음.
- 신규 helper `compute_cell_page_offset` (object_ops.rs): render tree 순회 → TableCell 매칭 → bbox.x/y px → ×75 HWPUNIT. 매칭 실패 시 (0, 0) fallback.
- 셀 분기 사후처리: outer Table.dirty=true + mark_section_dirty + paginate_if_needed.

**`src/wasm_api.rs`** — `insertPicture` WASM export:
- 시그니처에 `cell_path_json: &str` 추가.
- 빈 문자열 또는 `"[]"` → 본문 inline. 그 외 → `DocumentCore::parse_cell_path` 파싱 후 native 호출.

### TypeScript

**`rhwp-studio/src/core/wasm-bridge.ts`** — `insertPicture` 래퍼:
- `cellPathJson: string` 인자 추가. JSDoc 으로 본문 inline vs 셀 floating 분기 명시.

**`rhwp-studio/src/engine/input-handler-table.ts`** — `finishImagePlacement`:
- `hit.cellPath` 가 있고 `hit.parentParaIndex !== undefined` 이면 outer parentParaIndex + JSON.stringify(cellPath) 전달. 본문 클릭은 기존 paragraphIndex.

**`rhwp-studio/src/engine/input-handler-keyboard.ts`** — Ctrl+V paste 이미지:
- `cursor.getPosition()` 의 cellPath 검사 후 동일 패턴으로 parentParaIndex + cellPath JSON 전달.

### 테스트

**`src/document_core/commands/object_ops.rs`** 내 `#[cfg(test)] mod issue_1151_cell_picture_insert_tests`:

1. `issue1151_insert_picture_into_table_cell_is_floating_sibling`: 1×1 표 → 셀 안 picture 삽입 → 셀 paragraph controls 비어있음 + table 같은 paragraph 에 sibling Picture 존재 + `picture.common.treat_as_char == false` + `text_wrap == Square` 단언.
2. `issue1151_insert_picture_body_keeps_existing_inline_behavior`: cell_path=&[] 본문 삽입 → inline (`treat_as_char == true`) 회귀 확인.
3. `issue1151_invalid_cell_path_returns_error`: 범위 초과 셀 → Err.

TDD RED → GREEN → REFACTOR.

## 4. 검증 결과

| 검증 | 결과 |
|---|---|
| `cargo test --lib` | **1425 passed, 0 failed**, 6 ignored |
| `cargo test --tests` | 통합 그룹 모두 통과 (FAILED 0) |
| `cargo test --lib issue_1151` | **3 passed, 0 failed** |
| `cargo clippy --lib -- -D warnings` | **무경고** |
| `cargo fmt --all -- --check` | **clean** |
| WASM 빌드 (docker compose) | success |
| `npx tsc --noEmit` (rhwp-studio) | 본 작업 관련 에러 0 (사전 canvaskit 모듈 부재 에러만 잔존, 무관) |
| **사용자 시각 검증** | "이게 정답이였음. 잘 된다" ✓ |

## 5. 수동 검증 절차

1. WASM 빌드 후 dev server 기동:
   ```bash
   docker compose --env-file .env.docker run --rm wasm
   cd rhwp-studio && npx vite --port 7700
   ```
2. http://localhost:7700 → 신규 문서 → 표 (예: 3×3) 생성
3. 셀 안 커서 → 메뉴 → 입력 → 그림 → 이미지 파일 선택 → 클릭으로 위치 확정
4. **기대 결과**: 이미지가 셀 영역에 floating 으로 배치 + **다른 셀/본문 클릭 시 cursor 이동 정상** (#1151 핵심)
5. 회귀: 본문 클릭 → 이미지 삽입 → 본문에 inline 정상 삽입
6. Ctrl+V 클립보드 이미지 paste → 셀 안 / 본문 모두 정상

## 6. v1 → v2 설계 전환 기록

v1 (cell_path 로 셀 내부 inline 삽입) 은 한컴 패턴과 불일치 + 사용자 보고된 "셀 클릭 무반응" 결함 재현. 분석 결과 inline 셀 picture 는 한컴 원본 파일 (`pic-in-table-01.hwp` page 22) 에서도 동일한 "클릭 시 cursor 진입 불가" 동작. 한컴이 실제 셀 picture 삽입에 사용하는 방식은 floating (incellpicture.hwp 검증).

v1 진행 중 시도한 fix (segment_width 셀 폭 조정, horz_rel_to=Para, text_wrap=InFrontOfText, Table.dirty, reflow_cell_paragraph 제거 등) 모두 inline 의 구조적 한계 해소 못함 → v2 floating 으로 완전 재설계 후 사용자 검증 통과.

자세한 분석 흐름 + 사용자 incellpicture.hwp 비교는 수행계획서 / 구현계획서 참조.

## 7. 후속 이슈 권고 (본 PR scope 외)

본 task 진행 중 발견된 별개 사안 — 후속 이슈로 분리:

### A. 셀 안 picture 복사 (Ctrl+C) 미지원

- `src/document_core/commands/clipboard.rs:copy_control_native(section_idx, para_idx, control_idx)` 가 outer paragraph controls 만 인덱싱 — 셀 paragraph 내부 picture 의 control_idx 를 받으면 outer 에서 lookup 실패.
- 영향: HWP 원본 파일의 inline 셀 picture (samples/pic-in-table-01.hwp 등) 복사 불가.
- 해결 방향: `copy_control_native` 에 cell_path 인자 추가 또는 별도 `copy_control_in_cell_native` 신규.
- **본 PR 의 floating sibling picture 는 outer 에 직접 위치하므로 영향 없음.**

### B. 한컴 Automation `InsertPicture` 옵션 호환

`sizeoption` / `reverse` / `watermark` / `effect` — #194 에서 제안된 옵션. 본 PR 은 기본 동작만 다룸.

### C. 중첩 표 (N-depth) 셀 picture 삽입

본 PR 은 1-level cell_path 우선. N-depth 의 cell page bbox 계산 + 정합은 후속.

## 8. 커밋 이력 (local/task1151)

| Commit | 내용 |
|---|---|
| `a1b60239` | 수행계획서 + 구현계획서 (floating picture 방식) |
| `e47052c0` | Stage 1 (A): Rust insert_picture_native floating 분기 + WASM export + 테스트 |
| `5b179e7b` | Stage 2 (B): TS bridge + handler floating 전달 |
| (Stage 3) | 본 최종 보고서 |

## 9. 외부 컨트리뷰터 패턴 정합

johndoekim Fork → `local/task1151` → upstream `edwardkim:devel` PR. CONTRIBUTING.md "Fork & PR 워크플로우" 준수. 머지 전 `cargo fmt --all -- --check`, `cargo test`, `cargo clippy -- -D warnings` 통과 확인.
