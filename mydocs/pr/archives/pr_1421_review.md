# PR #1421 리뷰 - exact font replay proof corpus 확장

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1421 |
| 제목 | render: widen exact font replay proof corpus |
| 작성자 | seo-rii |
| 관련 이슈 | #536 |
| base | `devel` |
| head | `seo-rii:render-p25` |
| draft | false |
| mergeable | true |
| 현재 head | `2913f13d` |
| 변경량 | 3 files, +368 / -12 |
| 변경 파일 | `docs/text-ir-v2.md`, `src/renderer/layer_renderer.rs`, `src/renderer/skia/renderer.rs` |

PR 목적은 exact font replay를 기본 경로로 여는 것이 아니라, CanvasKit/native Skia glyph-run
direct replay 이전 단계에서 아직 안전하지 않은 font 조건을 더 명시적으로 fallback시키는 것이다.

## 2. 변경 범위

핵심 변경:

- `src/renderer/layer_renderer.rs`
  - `VariantRejectReason`에 `faceIndexUnsupported`, `variationUnsupported` 추가
  - CanvasKit/native Skia backend 선택에서 variable font instance와 non-default TTC/OTC face index를
    glyph-run direct replay reject reason으로 기록
  - face index `0`은 positive control로 유지하는 테스트 추가
- `src/renderer/skia/renderer.rs`
  - native Skia proof reason에 `fontBlobDataRefMismatch`, `fontBlobDigestMismatch` 추가
  - portable font blob의 `dataRef`, portability digest, blob digest를 resource bytes와 대조
  - glyph id range guard 유지 테스트 추가
- `docs/text-ir-v2.md`
  - P25 exact font replay proof corpus 정책 정리

## 3. GitHub 상태

| 항목 | 상태 |
|---|---|
| PR state | open |
| mergeable | true |
| requested reviewer | `edwardkim` |
| comments | 없음 |
| reviews | 없음 |
| commits | 3 |

GitHub Actions:

| 체크 | 상태 |
|---|---|
| Render Diff | pass |
| CI | pass |
| CodeQL | pass |

## 4. 로컬 검증

PR head를 `local/pr1421-upstream`으로 fetch하여 검증했다.

```bash
git fetch origin pull/1421/head:local/pr1421-upstream
git switch local/pr1421-upstream
git diff --check origin/devel...HEAD
cargo fmt --check
CARGO_INCREMENTAL=0 cargo test --lib renderer::layer_renderer -- --nocapture
CARGO_INCREMENTAL=0 cargo test --lib --features native-skia native_skia_glyph_run_proof -- --nocapture
CARGO_INCREMENTAL=0 cargo check --lib
git switch devel
```

결과:

- `git diff --check origin/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `renderer::layer_renderer`: 24 passed
- `native_skia_glyph_run_proof`: 10 passed
- `cargo check --lib`: 통과

## 5. 코드 검토 메모

검토한 주요 지점:

- `collect_glyph_run_reject_reasons`가 backend별로 variation/non-zero face index를 reject reason에
  추가하되, Canvas2D fallback 경로에는 영향을 주지 않는다.
- `face_index != 0`만 거부하므로 face index `0` positive control은 기존 direct glyph-run selection을
  유지한다.
- native Skia proof는 `PortableBlob { data_ref, digest }` 내부 `data_ref`와 `FontBlobResource.data_ref`
  metadata를 비교하고, 실제 bytes digest도 함께 검증한다.
- digest mismatch는 best-effort construction이 아니라 portable contract 실패로 남는다.
- glyph id는 Text IR에서 `u32`로 유지하지만 backend proof 전 `u16::MAX` range guard를 테스트로 고정한다.

현재 검토 범위에서 명시적인 결함은 발견하지 못했다.

## 6. 리스크

| 항목 | 평가 |
|---|---|
| 기본 TextRun fallback 회귀 | 낮음. PR은 strict glyph-run direct replay gate를 더 보수적으로 만드는 방향 |
| CanvasKit/native Skia direct replay 확대 | 낮음. exact construction을 구현하지 않고 unsupported reason을 세분화 |
| font resource metadata 계약 | 중간. `dataRef`/digest 불일치를 실패로 처리하므로 기존 best-effort 기대가 있었다면 fallback이 늘 수 있음. PR 의도와 부합 |
| schema/JSON 호환성 | 낮음. 출력 형식 자체 변경은 없고 reason 문자열 추가 중심 |
| 테스트 범위 | 양호. PR 본문 검증과 로컬 검증 모두 통과 |

## 7. 권고

현재 상태에서는 merge 가능으로 판단한다.

권고 순서:

1. 작업지시자 처리 승인
2. PR #1421 merge
3. `devel`/`local/devel` 동기화
4. PR 리뷰 문서를 `mydocs/pr/archives/`로 이동
5. 필요 시 #536 후속 P26 작업에서 exact typeface construction 범위 분리
