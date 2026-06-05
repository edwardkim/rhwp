# PR #1228 검토 — 표 셀(중첩) picture 복사(Ctrl+C) + 떠있는 개체 paste cascade

- **작성일**: 2026-06-02
- **PR**: #1228 (OPEN)
- **컨트리뷰터**: @johndoekim (핵심 컨트리뷰터 — #1177/#1150 머지, 표+picture 정합 전문)
- **연결 이슈**: #1161 (OPEN, assignee=johndoekim) — #1151 후속
- **base/head**: `devel` ← `feat/issue-1161-cell-picture-copy` (cross-repo)
- **규모**: 28 파일, +1026/−86 (Rust 코어 + rhwp-studio TS 6 + 테스트 3 + 작업문서 8) — 세션 최대
- **mergeable**: MERGEABLE / BEHIND
- **CI**: **전부 PASS** (Build&Test/Canvas visual diff/Analyze×3/CodeQL)
- **라벨**: bug / enhancement / 마일스톤 v1.0.0

## 1. 문제 (코드 확인)

표 셀(중첩 포함) inline picture 를 객체 선택 후 Ctrl+C 해도 클립보드에 저장 안 됨. 근본 원인:
복사 native 4종(`copy_control_native`/`export_control_html_native`/`get_control_image_data_native`/
`get_control_image_mime_native`)이 본문 컨트롤만 접근(cell_path 미수신). 더 근본은 렌더 트리
**ImageNode 가 TextRun 과 달리 다단계 cellPath 미보유**(단일 레벨 스칼라만) → 프런트가 중첩 셀
picture 경로 복원 불가.

## 2. 수정 내용 검토

| 영역 | 변경 |
|------|------|
| `render_tree.rs` | ImageNode 에 `cell_context: Option<CellContext>`(TextRunNode 정합). `ImageNode::new` 기본 None |
| layout 3 chokepoint | `layout_picture_full`(picture_footnote.rs:64, 실제 활성 셀 picture 경로) + `make_picture_image_node`(inline TAC) + 1 → `cell_context: cell_ctx.cloned()` |
| `rendering.rs` | ImageNode → JSON `cellPath:[...]` 방출(TextRun 동일 포맷) |
| `clipboard.rs` | 복사 native 4종 `cell_path` 인자 + 공통 헬퍼 `resolve_control_para` |
| `cursor_nav.rs` | `resolve_control_para`: 빈 경로=본문, 아니면 `resolve_paragraph_by_path` 위임(표 셀·글상자 공통) |
| `wasm_api.rs` | 클립보드 4 래퍼 `cell_path_json` 인자(parse_cell_path) → **WASM API 4인자화** |
| rhwp-studio TS 6 | 선택 ref cellPath 배선(복사/오려두기/컨텍스트 메뉴) + 타입 |
| `clipboard.rs` cascade | floating(tac=false) 붙여넣기마다 위치 오프셋 누적, inline(tac=true) 제외 |

설계 평가:
- **cellPath 단일 진실원**: ImageNode `cell_context`(TextRun 정합) 권위값, 기존 단일 레벨 스칼라는
  innermost 투영 유지 → **additive, 하위호환**(스칼라 불변). #1171(이중 nested)의 공유 기반.
- **resolve_control_para**: 빈 경로=기존 본문 동작 → 복사 4종 시그니처는 변하나 동작 보존.
- **inline vs floating cascade**: 한글 5.0 스펙 표 70 bit 0 "글자처럼 취급" 근거(추정 아님). inline 미적용.

## 3. 위험 평가

- **중간(ImageNode 모델 + 클립보드 + TS 광범위)이나 가드 견고.** ImageNode cell_context 는 그리기가
  아닌 경로 식별 메타데이터 → renderer별(svg/canvas/web_canvas) 그리기 함수 무관(좌표는 layout 확정).
  `feedback_image_renderer_paths_separate` 우려 낮음. 스칼라 불변 + golden SVG 가드로 검증.
- WASM API 4인자화 → TS 가 WASM 재빌드 전이면 tsc 실패(아래 검증에서 재현·해소).

## 4. 검증 결과 (로컬 머지 시뮬 `pr1228-verify`)

| 단계 | 결과 |
|------|------|
| merge | ✅ CLEAN (clipboard/paragraph_layout/wasm_api/input-handler 최근 머지와 무간섭, 충돌 0) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1933 passed, 0 failed** (golden SVG 포함 — ImageNode 변경 회귀 0) |
| issue_1161 회귀 | ✅ 셀 복사 4건 + cellpath 방출 1건(pic-in-table-01 p16 2단계 중첩 → 2-엔트리 cellPath) |
| cascade 단위 | ✅ (전체 스위트 포함) floating 누적 / inline 불변 |
| **WASM 재빌드** | ✅ 신규 4인자 API 노출(`getControlImageData(sec,para,cell_path_json,ctrl)` 등) |
| **rhwp-studio `npm run build`** | ✅ WASM 재빌드 후 tsc 0 errors + vite build (4인자 정합 — PR 본문대로) |
| CI(PR) | ✅ 전부 PASS |

## 5. 판단 — 머지 권고 (편집기 동작 판정 게이트)

- 진단 정확, additive 하위호환(스칼라 불변), 1933 passed·golden 회귀 0, issue_1161 5건, WASM 재빌드 후
  tsc 0 errors, CI green. #1171 기반 마련.
- rhwp-studio 편집기 인터랙션(셀 picture 복사→붙여넣기, cascade)이 본질 → **작업지시자 편집기 동작 판정**
  게이트(셀/중첩 셀 picture Ctrl+C→붙여넣기 + floating cascade 계단식).
- 승인 + 동작 확인 시 메인테이너 로컬 `--no-ff` 머지 + push + WASM 빌드(신규 클립보드 API 노출).
  이슈 #1161 수동 클로즈.

## 6. 비고

- WASM API 4인자화는 머지 후 WASM 재빌드 필수(편집기가 쓰는 export). 미빌드 시 tsc 실패.
- #1171(사각형→글상자→picture 이중 nested)은 본 PR ImageNode cell_context + resolve_paragraph_by_path
  기반 위 후속 — PR 본문 명시.
- @johndoekim #1177(표+picture 정합) 연속 작업.
