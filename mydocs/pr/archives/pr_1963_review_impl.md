# PR #1963 리뷰 구현 기록

## 대상

- PR: https://github.com/edwardkim/rhwp/pull/1963
- 관련 이슈: https://github.com/edwardkim/rhwp/issues/1948
- 작성자: planet6897
- 문서 작성 시점 head SHA: `70fb3eaee1a8801d0af2f4cbaba98cad6e120251`
- merge commit: `2e1c6929acc2b0f02bb6a2473597c45900fe70f4`
- mergedAt: `2026-07-05T17:03:43Z`

## Stage 1. 메타 확인 및 로컬 반영

완료.

- reviewer `jangster77`를 assign했다.
- PR base가 `devel`이고 draft가 아님을 확인했다.
- changed files는 `src/serializer/hwpx/section.rs`와 신규 축소 샘플 1개다.
- `closingIssuesReferences`는 비어 있음을 확인했다.
- `review/pr1963` 브랜치에서 `pull/1963/head`를 가져와 `upstream/devel`과 충돌 없이 대조했다.

## Stage 2. 변경 내용 검토

완료.

- `render_runs` slot 방출 루프가 일반 slot 방출 전에 같은 `char_idx`의 미방출 `orphan_field_end`를 먼저
  방출하도록 보정한다.
- 보정 근거는 `para.orphan_field_ends`, `orphan_emitted`, `expected_utf16_pos`, `char_idx`다.
- 특정 샘플명, 페이지 번호, 임의 계수로 결과를 맞추는 하드코딩은 확인되지 않았다.
- 변경 범위는 serializer 순서 보정으로 좁고, 기존 fieldEnd/slot split 흐름과 충돌하지 않는다.

## Stage 3. 첨부 PDF 기반 시각 검증

완료.

- GitHub PR comment에 첨부된 `issue1948_cross_para_fieldend-2024.pdf`를 `pdf/`에 복사해 기준 PDF로 사용했다.
- visual sweep은 `samples/hwpx/issue1948_cross_para_fieldend.hwpx`와 기준 PDF 1쪽을 비교했다.
- 결과는 SVG/PDF 1/1쪽, `flagged=0/1`이다.
- 대표 review PNG를 `mydocs/pr/assets/pr_1963_issue1948_cross_para_fieldend_review_1948.png`로 복사했다.
- 자동 일치율 보조값은 폰트/raster 차이를 반영해 낮지만, PR 핵심인 roundtrip offset 보존에는 blocker가
  아니라고 판단했다.

## Stage 4. 로컬 검증

완료.

순차 실행 결과:

- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo build`: 통과
- `target/debug/rhwp hwpx-roundtrip samples/hwpx/issue1948_cross_para_fieldend.hwpx -o output/pr1963-roundtrip`: `PASS diff=0 r2=0`
- `target/debug/rhwp render-diff samples/hwpx/issue1948_cross_para_fieldend.hwpx --via hwpx -o output/pr1963-render-diff`: `PASS`, max displacement `0.00 px`
- `env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx::section`: 49 passed
- `env CARGO_INCREMENTAL=0 cargo test --test hwpx_roundtrip_baseline`: 4 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## Stage 5. 결론

merge 완료로 정리한다.

최신 PR head 기준 CI 상태를 확인한 뒤 admin merge를 수행했다. #1948은 GitHub Actions auto-close로
`2026-07-05T17:03:55Z`에 CLOSED 상태가 되었음을 확인했다. 옵션 2 docs-only PR merge 후 #1948에 수동
후속 코멘트를 남긴다.
