# PR #918 검토 보고서 — WMF renderer 근본 개선 전략 분석

## 메타

| 항목 | 값 |
|------|---|
| PR | [#918](https://github.com/edwardkim/rhwp/pull/918) |
| 컨트리뷰터 | @jangster77 (Taesup Jang) |
| 규모 | +4,893/-67, 59 파일, 41 커밋 |
| CI | 전부 통과, MERGEABLE(BEHIND) |
| closes | #902 |

## PR 인도물 분류

### A. WMF SVG converter 개선 (3파일) — 가치 높음, 독립 적용 가능

`wmf/converter/svg/` 영역:
- DX byte-aware indexing fix (MBCS byte index vs grapheme index 근본 결함)
- POLYPOLYGON fill-rule 정합
- device_context 개선

이 영역은 기존 SVG 경로를 개선하는 것으로, WASM/native 모두에 영향. 독립 cherry-pick 가능.

### B. WMF RasterPlayer 신규 (4파일) — native CLI 전용으로 가치 있음

`wmf/converter/raster/` 영역 신규:
- `player.rs` (+910) — tiny-skia 기반 래스터 렌더러
- `state.rs` (+167) — GDI device context 상태
- `text.rs` (+263) — fontdue 기반 텍스트 렌더링
- LibreOffice emfio 알고리즘 포팅 (MPL 2.0)

headless Linux CLI에서 WMF를 고품질 PNG로 변환하는 용도. `export-png`, `export-svg` CLI에서 사용.

### C. WASM 영역 변경 (문제 영역)

| 변경 | 문제 |
|------|------|
| `include_bytes!("NanumGothic.ttf")` | TTF 4.2MB가 WASM에 직접 컴파일 |
| `include_bytes!("NanumGothic-Regular.woff2")` | woff2 1.5MB가 SVG 경로에 임베딩 |
| `wasm-opt = false` | 전체 프로젝트 WASM 최적화 비활성 (+900KB) |
| `tiny-skia` + `fontdue` WASM 포함 | 래스터 렌더러가 WASM에 불필요하게 포함 |
| `resvg` non-optional 화 | WASM 빌드에도 resvg 포함 |
| **WASM 크기** | **4.5MB → 11.0MB (2.5배 증가)** |

### D. fixture/PDF (12파일) + examples (4파일) — 가치 있음

sample17/18/19 HWP3 샘플 + PDF 권위 자료 + 진단용 예제 스크립트.

### E. 임베디드 폰트/라이센스 (2파일) — 제거 대상

`ttfs/embedded/NanumGothic.ttf` + `LICENSE.md` — 폰트 바이너리를 레포에 포함하는 것은 부적합.

## 아키텍처 정책 분석

### rhwp 렌더링 경로 정리

| 환경 | 렌더링 경로 | WMF 처리 |
|------|------------|---------|
| **rhwp-studio (WASM/브라우저)** | Canvas2D + PageLayerTree | SVG converter → `<image>` 임베딩 |
| **rhwp CLI native (export-svg)** | SVG 직접 출력 | SVG converter |
| **rhwp CLI native (export-png)** | Skia raster (`--features native-skia`) | SVG converter → resvg → PNG |

### 작업지시자 정책

> "skia 엔진은 웹용이 아니다. headless Linux 환경에서 HWP 포맷을 처리하기 위한 CLI 지원. WASM에 넣을 필요 없다."

이 정책에 따른 판단:

| 영역 | 판단 |
|------|------|
| RasterPlayer (tiny-skia + fontdue) | **native CLI 전용** — `cfg(not(target_arch = "wasm32"))` 가드 필요 |
| NanumGothic `include_bytes!` | **제거** — native CLI는 `--font-path`로 외부 지정, WASM은 브라우저 폰트 사용 |
| `wasm-opt = false` | **제거** — RasterPlayer가 WASM에서 빠지면 불필요 |
| `resvg` non-optional 화 | **검토** — native-skia feature에서만 필요하면 conditional 유지 |
| SVG converter 개선 (영역 A) | **수용** — WASM/native 공통, 핵심 가치 |

## 권고 전략: 분리 cherry-pick

### Phase 1: SVG converter 개선 (영역 A) — 즉시 머지

WMF SVG converter 3파일만 cherry-pick:
- `wmf/converter/svg/device_context.rs`
- `wmf/converter/svg/mod.rs`
- `wmf/converter/svg/util.rs`

WASM/native 모두에서 WMF 렌더링 품질 향상. WASM 크기 무영향.

### Phase 2: fixture + examples (영역 D) — 즉시 머지

sample17/18/19 + PDF + 예제 스크립트. 코드 무관.

### Phase 3: RasterPlayer (영역 B) — native 전용 가드 후 머지

컨트리뷰터에게 요청:
1. `src/wmf/converter/raster/` 전체를 `cfg(not(target_arch = "wasm32"))` 가드
2. `tiny-skia` + `fontdue` 의존성을 `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`에 배치
3. `include_bytes!("NanumGothic.ttf")` 제거 → native CLI `--font-path` 경로 사용
4. `wasm-opt = false` 제거
5. `resvg` optional 유지 (또는 non-wasm32 conditional)

### Phase 4: WASM 영역 (영역 C) — Phase 3 완료 후 검토

Phase 3 정리 후 WASM 빌드 크기가 ~4.5MB 유지 확인.

## WASM 크기 목표

| 상태 | 크기 |
|------|------|
| 현재 devel | 4.5 MB |
| PR #918 그대로 | 11.0 MB (2.5배) |
| Phase 1+2만 | ~4.5 MB (무변동) |
| Phase 3 (native 전용 가드) | ~4.5 MB (무변동) |

## 라이센스 영역

| 소스 | 라이센스 | 호환성 |
|------|---------|--------|
| LO emfio 알고리즘 참조 | MPL 2.0 (file-level reciprocity) | `raster/*.rs` 파일에 MPL 2.0 헤더 필요 |
| NanumGothic TTF | SIL OFL 1.1 | 재배포 가능하지만 레포 바이너리 포함은 부적합 |
| rhwp 본체 | MIT | MPL 2.0 file-level 호환 |

## 작업지시자 결정 요청

1. **Phase 1 (SVG converter 개선)** — 즉시 cherry-pick 승인?
2. **Phase 2 (fixture/examples)** — 즉시 cherry-pick 승인?
3. **Phase 3 (RasterPlayer native 전용)** — 컨트리뷰터에게 수정 요청?
4. **임베디드 폰트 정책** — `include_bytes!` 전면 금지, `--font-path` 외부 경로 전용?
5. **MPL 2.0 라이센스** — LO 포팅 코드의 file-level MPL 2.0 헤더 수용?
