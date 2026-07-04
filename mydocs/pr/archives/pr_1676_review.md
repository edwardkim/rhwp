# PR #1676 검토 - HWPX 이미지 페이로드 렌더링 정규화

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1676](https://github.com/edwardkim/rhwp/pull/1676) |
| 제목 | `[IR 지원] HWPX 이미지 페이로드 렌더링 정규화` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-image-payload` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 12 파일, +265/-19 |
| 원 PR 커밋 | `b5a58d818521`, `842f64d7ae9a` |
| 로컬 적용 커밋 | `092936c38`, `123033088` (`git cherry-pick -x`) |

## 2. 변경 범위

HWPX 이미지 페이로드와 serializer roundtrip 지원을 보강하는 PR이다. PR 본문도 단독 페이지 시각 수정이 아니라 Format/IR 지원으로 분류한다.

- `TOTAL` 이미지 fill mode를 `FIT`으로 접지 않고 별도 모드로 보존한다.
- TIFF/BMP/PCX 및 시각적으로 grayscale인 JPEG처럼 브라우저/SVG 호스트가 바로 다루기 어려운 payload를 임베드 전에 PNG로 정규화한다.
- HWP/HWPX serializer 경로에서 새 `TOTAL` 모드와 image payload 정보가 roundtrip되도록 `style`, `paint/json`, `serializer` 계층을 같이 보정한다.
- `Cargo.toml` / `Cargo.lock` 변경으로 이미지 디코딩 의존성 그래프가 바뀐다. 따라서 첫 검증은 target cache가 있어도 일부 재빌드가 발생할 수 있다.

변경 파일은 `src/renderer/image_resolver.rs`, `src/renderer/html.rs`, `src/renderer/skia/image_conv.rs`, `src/paint/json.rs`, `src/parser/hwpx/header.rs`, `src/serializer/*`, `Cargo.toml`, `Cargo.lock` 중심이다.

## 3. 검토 의견

이 PR은 렌더러가 "그릴 수 있는 image byte"를 얻는 배관을 넓히는 성격이라 수용 가치가 있다. 특히 HWPX에서 이미지 payload 자체는 존재하지만 브라우저/SVG 경로가 형식 때문에 실패하는 케이스를 줄인다.

주의할 점은 payload 정규화가 원본 byte 보존과 렌더용 byte 생성의 경계를 흐리지 않아야 한다는 점이다. serializer roundtrip은 원본 형식 의미를 보존하고, renderer resolver는 표시 가능한 대체 표현을 만든다는 구분이 유지되는지 최종 검증이 필요하다.

`TOTAL` fill mode는 포맷 의미 확장이라 렌더링뿐 아니라 저장/재직렬화에도 영향이 있다. 이번 검토에서는 기존 `FIT` 처리와의 호환성을 targeted test와 통합 테스트로 확인했다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 2개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태이므로 개별 PR head를 그대로 merge하기보다, 일괄 브랜치에서 최신 `devel` 위 검증을 완료한 뒤 원 PR은 superseded 처리하는 편이 안전하다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `svg_snapshot`/`visual_roundtrip_baseline` 포함)
- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 공통 검증 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 5.1 PR 내용별 targeted 검증

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1676 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| HWPX `imgBrush mode="TOTAL"`를 `FitToSize`로 접지 않고 보존 | `cargo test --profile release-test --lib test_img_brush_total_keeps_total_mode` 통과 |
| grayscale JPEG는 PNG로 정규화 | `cargo test --profile release-test --lib grayscale_jpeg_is_normalized_to_png` 통과 |
| color JPEG는 불필요하게 PNG로 변환하지 않음 | `cargo test --profile release-test --lib color_jpeg_keeps_jpeg_path` 통과 |
| BMP payload는 PNG 변환 가능 | `cargo test --profile release-test --lib test_bmp_to_png_success` 통과 |
| PCX payload는 PNG 변환 가능, 흰색 투명 처리 유지 | `cargo test --profile release-test --lib test_pcx_to_png_maps_white_to_transparent`, `cargo test --profile release-test --test issue_514` 통과 |
| TIFF payload는 resolver에서 PNG + `FormatConverted`로 정규화 | maintainer 보강 테스트 `tiff_image_payload_is_normalized_to_png` 추가 후 `cargo test --profile release-test --lib tiff_image_payload_is_normalized_to_png` 통과 |

보강 사항: 원 PR의 TIFF 정규화 주장은 코드에는 있었지만 직접 단위 테스트가 없어서, `src/renderer/image_resolver.rs`에 TIFF resolver contract 테스트를 추가했다. 이는 contributor 원 변경 위의 collaborator 보강이다.

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. `2022-09`, `2024-09-below20`, `2024-09-between20`은 마지막 페이지 equation/order 후보, `2024-09-below20-above20`, `2024-11-practice-above0-between20-below2`는 tail/question/large ink 후보가 남았다. 이는 이미지 payload 정규화 자체 실패로 보이지 않으며, 비교 이미지 기반 수동 시각 판정 대상으로 분리한다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보. PR 핵심 주장(`TOTAL`, BMP/PCX/TIFF/grayscale JPEG 정규화)은 targeted 검증으로 확인했고, WASM/browser build와 단독 렌더 snapshot도 통과했다. 이미지 fixture나 golden binary는 커밋되지 않았으므로 PR 첨부 시각 증거는 참고 자료로만 보고, 저장소 내 deterministic test와 필요한 수동 시각 판정을 merge gate로 삼는다.
