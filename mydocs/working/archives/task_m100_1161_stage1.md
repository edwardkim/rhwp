# 단계별 완료 보고서 — Task M100-1161 Stage 1

## 목표

clipboard.rs 4개 native 함수에 `cell_path` 지원 추가 + 공통 헬퍼 일원화 + 셀 picture 복사 회귀 테스트.

## 변경 사항

### 1. 공통 헬퍼 신설 — `src/document_core/queries/cursor_nav.rs`
- `resolve_control_para(sec, para, cell_path) -> &Paragraph` (`pub(crate)`).
- 빈 `cell_path` = 본문 `sections[sec].paragraphs[para]`, 아니면 기존 `resolve_paragraph_by_path` 위임(셀/글상자, **다단계 중첩** 지원).

### 2. 4 native 함수 cell_path 확장 — `src/document_core/commands/clipboard.rs`
- `copy_control_native` / `export_control_html_native` / `get_control_image_data_native` / `get_control_image_mime_native` 에 `cell_path: &[(usize,usize,usize)]` 인자 추가.
- 동일 프롤로그(`sections→paragraphs→controls`)를 `let para = self.resolve_control_para(...)?; let control = para.controls.get(control_idx)?` 로 교체.
- `copy_control_native`(&mut self): 헬퍼로 불변 차용 → control clone + clip_para(owned) 구성 후 차용 종료 → `self.clipboard` 기록. **빌드/차용검사 통과**(NLL).

### 3. 호출처 갱신(컴파일 유지)
- `src/wasm_api.rs` 4 래퍼: `&[]` 전달(Stage 2에서 `cell_path_json` 연결 예정).
- `src/main.rs` CLI 2곳: `&[]`.
- `src/wasm_api/tests.rs` 기존 테스트 2곳: `&[]`.

### 4. 회귀 테스트 신설 — `tests/issue_1161_copy_picture_in_cell.rs`
- 샘플의 셀 picture 를 **재귀 탐색**(중첩 표 대응)하여 다단계 cell_path 조립.
- 4 케이스: ① 셀 경로 복사 → "[그림]" ② 빈 경로(본문)로는 셀 picture 안 잡힘(결함 박제) ③ 셀 경로 image data 비어있지 않음 ④ 본문 표 복사 회귀("[표]").

## 검증 결과

| 항목 | 결과 |
|------|------|
| `cargo build` | ✅ |
| `cargo test --test issue_1161_copy_picture_in_cell` | ✅ 4 passed |
| `cargo test --lib clipboard` (기존) | ✅ 5 passed |
| `cargo fmt --check`(변경 파일) | ✅ |
| `cargo clippy --lib` / 새 테스트 타깃 | ✅ 0 warning |

## ⚠️ 중요 발견 — Stage 3 범위에 영향

`samples/pic-in-table-01.hwp` 의 셀 picture(이슈가 가리키는 page 22)는 **2단계 중첩 표** 안에 있다:
- 본문 → 외부 표(3×3) → 셀[5].p[0] → **내부 표(3×9)** → 셀[1].p[0].ctrl[0] = 그림 (`tac=true`, 글자처럼 취급 = 표 70 bit 0).
- 정답 cell_path 예: `[(외부표ctrl, 5, 0), (내부표ctrl=0, 1, 0)]`, inner_ctrl=0 (2개 엔트리).

**문제**: 프런트 picture 선택 ref 는 **단일 레벨 스칼라**(`outerTableControlIdx`, `cellIdx`, `cellParaIdx`)만 저장(cursor.ts:1228). 단일 레벨로 조립한 cell_path 는 이 **다단계 중첩 picture 에 도달 불가**.

→ **Stage 3 착수 시 우선 조사 필요**: `getPageControlLayout`(WASM)/`findPictureAtClick`(TS) 가 중첩 picture 에 대해 (a) 전체 다단계 cellPath 를 제공하는지, 아니면 (b) 스칼라만 제공하는지. (b)라면 end-to-end 수정을 위해 picture 선택 ref 를 **다단계 cellPath 보유**로 확장해야 하며 Stage 3 범위가 커진다. native 계층(Stage 1)은 이미 다단계 지원 완료.

## 다음 단계

Stage 2 — `src/wasm_api.rs` 4 래퍼에 `cell_path_json: &str` 인자 추가 + `parse_cell_path` 연결(본문 호출 `""` 호환).
