# Task #514 Stage 5 완료 보고서 — 시각 판정 + WASM 빌드 + Canvas 정정

## 핵심 발견 (계획서 외 추가 정정)

Stage 3 정정은 **CLI SVG renderer (`svg.rs`) 한정**으로 진행했는데, **Web Canvas (`web_canvas.rs`) 는 별도의 `detect_image_mime_type` 복사본**을 사용 중이었음. 따라서 작업지시자의 Canvas 시각 검증에서 학교 로고가 여전히 안 보임 → Stage 5 진행 중 web_canvas.rs 도 동일 정정 적용.

**렌더 경로별 정정 상태:**

| 경로 | Stage 3 | Stage 5 (현재) |
|------|---------|---------------|
| CLI SVG (`svg.rs::detect_image_mime_type`) | ✅ PCX → PNG | ✅ |
| Web Canvas (`web_canvas.rs::detect_image_mime_type`) | ❌ 누락 | ✅ **추가 정정** |

이 패턴은 메모리 `feedback_visual_regression_grows` (cargo test 통과만으로는 시각 결함 검출 불가) + 이슈 #516 (web 그림 회색조 누락) 와 동일한 구조적 원인 — **renderer 마다 별도 함수 / 적용 코드** 가 존재.

## 시각 판정 결과

### CLI SVG (Stage 3 검증 완료)

- 학교 로고 image href: `data:application/octet-stream;...` → **`data:image/png;base64,iVBORw0KGgo...`**
- PNG 디코드: 878×1001 px, color type 6 (**RGBA — 알파 채널 포함, 투명 처리 ★**)
- 워터마크 (JPEG) 회귀 0

### Web Canvas (Stage 5 검증 완료)

- 작업지시자 검증: ✅ "이제 PCX 이미지가 웹 캔바스에도 출력됩니다."
- studio dev server 재시작 + hard reload 후 양식 좌상단 고려대학교 학교 로고 정상 출력

## 정정 변경 요약 (Stage 3 + Stage 5)

| 파일 | 변경 |
|------|------|
| `Cargo.toml` | `pcx = "0.2"` dependency 추가 |
| `src/renderer/svg.rs` | +75 / -2 (detect MIME + pcx_bytes_to_png_bytes + 두 emit 사이트) |
| `src/renderer/web_canvas.rs` | **+10 / -0** (Stage 5 — detect MIME + draw_image emit 사이트, svg::pcx_bytes_to_png_bytes 재사용) |
| `tests/issue_514.rs` (신규) | +73 (3 tests) |

총 변경: 4 파일, 약 +160 / -2.

## WASM 빌드 + 동기화 결과

| 산출물 | 이전 (devel) | Stage 5 (현재) | 변동 |
|--------|--------------|---------------|------|
| `pkg/rhwp_bg.wasm` | 4,376,286 bytes | **4,452,204 bytes** | +75,918 bytes (pcx crate + PNG encoder + web_canvas 분기 + image RGBA) |
| `pkg/rhwp.js` | 230,417 bytes | 230,417 bytes | 변동 없음 (JS bindings 영역 미변경) |
| `rhwp-studio/public/rhwp_bg.wasm` | stale | ✅ 동기화 (4,452,204 bytes) | — |
| `rhwp-studio/public/rhwp.js` | byte-identical | ✅ 동기화 | — |

## 검증 게이트 (최종)

| 게이트 | 결과 |
|--------|------|
| `cargo build --lib` | ✅ Finished |
| `cargo build --release` | ✅ Finished |
| `cargo test --lib` | ✅ **1110 passed** (회귀 0) |
| `cargo test --test issue_514` | ✅ **3 passed** (신규) |
| `cargo test --test issue_418` (셀 padding) | ✅ 1 passed |
| `cargo test --test issue_501` (mel-001) | ✅ 1 passed |
| `cargo test --test svg_snapshot` | ✅ **6/6 passed** |
| `paint::json::tests` (PR #510) | ✅ 4/4 passed |
| `cargo clippy --lib -- -D warnings` | ✅ 0 건 |
| `cargo clippy --test issue_514 -- -D warnings` | ✅ 0 건 |
| WASM 빌드 (Docker) | ✅ exit 0, 1m 27s |
| **CLI SVG 시각 검증** | ✅ Stage 3 완료 |
| **Web Canvas 시각 검증 (작업지시자)** | ✅ **Stage 5 완료** ★ |

## 메모리에 추가할 가치 있는 패턴

본 task 에서 발견한 구조적 사실:

> **rhwp 의 image MIME 감지 / 변환 코드는 renderer 마다 별도 사본을 사용.**
> CLI SVG (`svg.rs`), Web Canvas (`web_canvas.rs`), Web SVG (별도 분기 가능성),
> PageLayerTree JSON (`paint/json.rs`) 등 각 경로마다 `detect_image_mime_type`
> 함수와 변환 분기가 독립적으로 정의되어 있음.
>
> **시각 회귀 정정 시 한 곳만 수정하면 다른 경로에서 누락**.
> Stage 3 의 svg.rs 정정 후 Web Canvas 가 그대로였던 본 task 가 명확한 사례.

→ 메모리 `feedback_image_renderer_paths` 후보로 등록 권장 (Stage 6 또는 별도 작업).

## 다음 단계

- web_canvas.rs 정정 + 본 보고서를 task 브랜치에 commit
- 최종 보고서 (`mydocs/report/task_m100_514_report.md`) + orders 갱신
- local/task514 → local/devel merge → devel push
- 이슈 #514 close (devel 머지 검증 후, 메모리 `feedback_close_issue_verify_merged`)

## 산출물

- 본 보고서 (`mydocs/working/task_m100_514_stage5.md`)
- `src/renderer/web_canvas.rs` (PCX 분기 추가, +10)
- `pkg/rhwp_bg.wasm` (4,452,204 bytes) → `rhwp-studio/public/` 동기화 완료
