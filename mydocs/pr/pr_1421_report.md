# PR #1421 처리 보고서 — exact font replay proof corpus 확장

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1421 |
| 제목 | `render: widen exact font replay proof corpus` |
| 작성자 | `seo-rii` |
| 관련 이슈 | #536 |
| PR base | `devel` (`ab1879c9`) |
| PR head | `2913f13d` |
| merge commit | `0bd39929` |
| 처리 방식 | GitHub PR merge |
| 처리 판정 | 수용 가능 |

## 2. 처리 내용

작업지시자 승인에 따라 PR #1421을 merge했다.

PR head는 검토 시점과 merge 시점 모두 `2913f13d`였고, GitHub Actions는 Render Diff, CI,
CodeQL 모두 통과 상태였다. merge 후 `devel`과 `local/devel`을 `origin/devel`의 merge commit
`0bd39929`로 fast-forward 동기화했다.

## 3. 변경 내용

`src/renderer/layer_renderer.rs`:

- `VariantRejectReason`에 `faceIndexUnsupported`, `variationUnsupported` 추가
- CanvasKit/native Skia glyph-run selection에서 variable font instance와 non-default TTC/OTC face
  index를 명시 reject
- face index `0`은 positive control로 유지하는 테스트 추가

`src/renderer/skia/renderer.rs`:

- native Skia proof reason에 `fontBlobDataRefMismatch`, `fontBlobDigestMismatch` 추가
- portable font blob의 `dataRef`, portability digest, blob digest를 resource bytes와 대조
- glyph id range guard 유지 테스트 추가

`docs/text-ir-v2.md`:

- P25 exact font replay proof corpus 정책 문서화

## 4. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Render Diff | pass |
| CI | pass |
| CodeQL | pass |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib renderer::layer_renderer -- --nocapture` | 통과, 24 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib --features native-skia native_skia_glyph_run_proof -- --nocapture` | 통과, 10 passed |
| `CARGO_INCREMENTAL=0 cargo check --lib` | 통과 |

## 5. 판정

**수용 가능**.

이 PR은 exact font replay를 기본 경로로 여는 변경이 아니라, 아직 정확한 backend construction이
검증되지 않은 variable font instance, non-default TTC/OTC face index, font blob metadata/digest
불일치 케이스를 더 명확히 fallback시키는 proof corpus 확장이다.

기본 `TextRun` fallback은 유지되며, schema/JSON 출력 형식 자체를 바꾸지 않는다.

## 6. 후속 절차

처리 완료:

- [x] 리뷰 문서 작성 — `mydocs/pr/archives/pr_1421_review.md`
- [x] 작업지시자 승인
- [x] PR #1421 merge — `0bd39929`
- [x] `devel`/`local/devel` 동기화 — `0bd39929`

남는 범위:

- CanvasKit/native Skia exact typeface construction은 구현하지 않았다.
- variable font axis replay와 TTC/OTC non-zero face index replay는 후속 P26 이후 작업에서 별도 검토한다.
