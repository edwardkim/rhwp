# PR #1228 처리 보고서 — 표 셀(중첩) picture 복사 + 떠있는 개체 paste cascade

- **작성일**: 2026-06-02
- **PR**: #1228 → **MERGED** (devel, 로컬 `--no-ff` 머지)
- **컨트리뷰터**: @johndoekim (핵심 컨트리뷰터)
- **연결 이슈**: #1161 → **CLOSED** (`closes #1161`)
- **판단**: **머지** ✅ (작업지시자 편집기 동작 판정 통과 — 표 셀 picture 복사→붙여넣기 성공)

## 결정 사유

표 셀(중첩) picture 복사가 안 되던 결함을 ImageNode `cell_context`(다단계 cellPath) + 복사 native
4종 cell_path 인자로 해결. additive 하위호환(스칼라 불변), 1933 passed·golden 회귀 0, WASM 재빌드 후
tsc 0 errors. 작업지시자 동작 확인: 표 안 인라인 그림 복사→붙여넣기 성공.

## 변경 요약 (28 파일, +1026/−86)

| 영역 | 변경 |
|------|------|
| `render_tree.rs` | ImageNode `cell_context: Option<CellContext>`(TextRunNode 정합) |
| layout 3 chokepoint | layout_picture_full(활성 셀 picture) + make_picture_image_node(inline TAC) + 1 → cell_ctx.cloned() |
| `rendering.rs` | ImageNode → JSON cellPath 방출(TextRun 동일 포맷) |
| `clipboard.rs` | 복사 native 4종 cell_path 인자 + resolve_control_para 헬퍼 + floating cascade offset |
| `cursor_nav.rs` | resolve_control_para(빈 경로=본문, 아니면 resolve_paragraph_by_path 위임) |
| `wasm_api.rs` | 클립보드 4 래퍼 cell_path_json → WASM API 4인자화 |
| rhwp-studio TS 6 | 선택 ref cellPath 배선 + 타입 |

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ CLEAN (충돌 0) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1933 passed, 0 failed** (golden SVG — ImageNode 회귀 0) |
| issue_1161 회귀 | ✅ 셀 복사 4 + cellpath 방출 1 |
| WASM 재빌드 | ✅ 신규 4인자 API 노출 |
| rhwp-studio build | ✅ WASM 재빌드 후 tsc 0 errors |
| **편집기 동작 판정** | ✅ **통과** (작업지시자, 표 안 인라인 그림 복사→붙여넣기 성공) |
| CI(PR) | ✅ 전부 PASS |
| WASM | (머지 후 빌드 — 신규 클립보드 API 노출) |

## 후속 — 글상자 내 이미지 선택·복사 (별도 이슈)

작업지시자 동작 검증 중 **글상자(textbox) 안 그림이 선택(클릭)되지 않음**을 발견. 본 PR 범위 밖:
- 경로 복원(resolve_paragraph_by_path)은 이미 글상자 지원(Task #919). 막힌 것은 **선택(hit-test)** —
  사각형 글상자가 클릭 가로채는 이중 nested 구조(= 기존 이슈 #1171 "글상자 안 picture click hit-test 미지원").
- PR #1228 은 #1171 이 소비할 기반(ImageNode cell_context + cellPath 계약)을 제공하되 글상자 hit-test 는
  의도적 범위 밖(PR 본문 "관련 #1171 (공유 기반 제공)" 명시).
- 작업지시자 지시로 **글상자 내 이미지 선택·복사를 별도 이슈로 등록**.

## 비고

- WASM API 4인자화 → 머지 후 WASM 재빌드 필수.
- 후속 #1227(무관한 기존 직렬화 버그, mini_cfb DIFAT) — PR 본문 명시, 분리.
- @johndoekim #1177 연속 작업.
