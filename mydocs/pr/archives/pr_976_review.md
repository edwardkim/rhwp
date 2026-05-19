# PR #976 검토 — 복학원서 JPEG 워터마크 배경 사각형 제거 및 톤 보정

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| 번호 | #976 |
| 제목 | Task #938: 복학원서 JPEG 워터마크 배경 사각형 제거 및 톤 보정 |
| 작성자 | postmelee (Taegyu Lee) — 기존 컨트리뷰터 (25번째 PR, 첫 PR 아님) |
| base ← head | `devel` ← `codex/issue-938-watermark` |
| 연결 이슈 | Refs #938 (assignee 본인 지정 완료) |
| mergeable | MERGEABLE / BEHIND (충돌 아님) |
| CI | Build & Test ✅ / Analyze(rust/js/py) ✅ / Canvas visual diff ✅ / CodeQL ✅ |

## 2. 배경 (이슈 #938)

`samples/복학원서.hwp` 중앙 JPEG 워터마크가 rhwp SVG/WebCanvas/studio
overlay 경로에서 옅은 회색 사각형 배경처럼 렌더. 원본 JPEG는 alpha 없고
near-white 배경 픽셀 포함. 기존 렌더러가 JPEG 전체에 밝기/대비 + opacity
0.17 + blend 를 적용해 배경까지 보정되어 사각 흔적 잔존.

## 3. 변경 내용

코드 4 + Cargo.toml + 테스트 3 + 문서 8 파일.

| 파일 | 변경 |
|------|------|
| `src/renderer/svg.rs` | `watermark_jpeg_bytes_to_hancom_baked_png_bytes()` 신규 — 워터마크성 JPEG 를 opaque gray PNG 로 bake. baked 시 effect/bc 필터 + opacity 0.17 생략 |
| `src/document_core/queries/rendering.rs` | overlay JSON 에 `bakedWatermark: true` + baked PNG data URL |
| `src/renderer/web_canvas.rs` | baked PNG 사용, 중복 필터 생략 |
| `rhwp-studio/src/view/page-renderer.ts` | `bakedWatermark` overlay 는 CSS filter/blend/opacity 미적용 |
| `Cargo.toml` | `image` crate `jpeg` feature 추가 (WASM JPEG decode) |
| `tests/issue_938.rs` (신규), `issue_514.rs`, `golden_svg/issue-677/...svg` | 기대값 갱신 |

## 4. 검토 의견

### 4.1 강점

- **다중 렌더 경로 일관성 확보**: SVG / WebCanvas / studio overlay 가
  서로 다른 필터 경로를 타던 문제를, 단일 baked PNG 로 통일 — 구조적 개선.
- WASM overlay 경로에서 JPEG decode 불가로 톤이 어긋나던 문제 해소
  (`image` crate `jpeg` feature).
- baked 시 중복 필터(effect/bc/opacity) 명시적 생략 — 이중 보정 제거.
- 워터마크 미감지/decode 실패 시 기존 fallback 정책 유지 (격리).

### 4.2 핵심 쟁점 — 단일 문서 기반 매직 넘버 (⚠️ 중점 검토)

`map_watermark_gray()` 의 6구간 piecewise 계수
(198.0/0.46, 221.0/0.47, 235.1/0.14, 237.9/0.385, 245.6/0.1625,
252.1/0.032) 가 **복학원서 정답 PDF 한 문서에서 역산한 매직 넘버**.

- 다른 워터마크 JPEG 문서에 일반화 보장 없음 — 복학원서 전용 튜닝.
- 관련 메모리 룰:
  - `feedback_pdf_not_authoritative`: 한컴 PDF 는 정답지 아님,
    환경별 상이 — PDF 톤 역산을 정답으로 고정하는 접근의 위험.
  - `feedback_v076_regression_origin`: 컨트리뷰터가 PDF 정답지로
    맞춰 회귀 발생한 전례. 동일 위험 패턴.
  - `feedback_hancom_compat_specific_over_general`: 일반화보다
    구조 가드가 안전 — 본 PR 은 구조(near-white opaque)는 일반,
    톤(gray ramp)은 단일 문서 특화. 톤 부분이 우려.

### 4.3 휴리스틱 게이트

워터마크 감지: near-white 임계 RGB≥245, border near-white ≥0.85,
전체 near-white ≥0.20. 휴리스틱이라 다른 문서에서 false
positive/negative 가능. `PicEffect != RealPic` + brightness/contrast
+ JPEG 시그니처 동반 확인으로 좁히긴 함.

### 4.4 회귀 가드 변경

`tests/issue_514.rs` JPEG 워터마크 기대값 + `golden_svg/issue-677`
갱신. 기존 회귀 가드의 기대를 "투명 alpha" → "opaque gray PNG" 로
전환 — 이전 기대가 무효화됨. issue #514/#677 의 원래 시각 의도와
상충하지 않는지 확인 필요.

## 5. 검증 결과 (cherry-pick `7a1f22f6`, 최신 devel 위)

작업지시자 지시로 PR 단일 커밋 `2a48ed37` 을 최신 `local/devel`
(PR #971 + Task #987 + fmt fix 반영) 위에 cherry-pick. 충돌 없음.

| 항목 | 결과 |
|------|------|
| `cargo build --release` | ✅ Finished |
| 타깃 테스트 (issue_938/514/516) | ✅ 2 + 3 + 8 passed |
| 전체 `cargo test` | ✅ 1484 passed, 0 failed (issue_938 신규 +2) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| `cargo fmt --all -- --check` | ✅ 위반 0건 (PR #971 fmt 사고 위험 없음) |
| WASM 빌드 (Docker) | ✅ 성공, jpeg feature 반영 (4.62→4.82MB, +201KB) |
| 워터마크 emit | ✅ image/png 2건, image/jpeg 0, opacity="0.17" 0 |

산출물: `output/poc/pr976/복학원서.svg`

- [ ] **작업지시자 시각 판정** — 복학원서 1페이지 워터마크
      (PDF `pdf/복학원서-2022.pdf` 는 참고, 정답지 아님).
      issue #514/#677 시각 회귀 동시 확인.

## 6. 판단 (잠정)

구조적 개선(다중 경로 통일, baked 일원화)은 타당. 단 **톤 매핑이
단일 문서 PDF 역산 매직 넘버**라 일반화·회귀 위험이 핵심 쟁점.

→ 옵션 분류 (작업지시자 판단 필요):
- **A. 수용**: 복학원서 전용으로도 시각 개선 명확하면 merge.
  단 톤 매핑이 단일 문서 특화임을 코드 주석/이슈에 명시.
- **B. 구조만 수용**: near-white opaque 배경 처리(일반적)만 채택,
  gray ramp 톤 매핑은 단순화 요청 (예: 기존 brightness/contrast
  유지하되 배경만 opaque).
- **C. 수정 요청**: 톤 매핑 일반화 또는 문서별 가드 구조 요청.

시각 판정 결과 + 위 옵션 선택에 따라 `pr_976_report.md` 작성.
