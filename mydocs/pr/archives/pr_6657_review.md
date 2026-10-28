# PR #6657 통합 self-review 기록

## 판정: 승인

- PR: [#6657](https://github.com/edwardkim/rhwp/pull/6657)
- 대상 브랜치: `devel`
- 통합 브랜치: `review/jeong-sik-open-3pr-20260903`
- 사전 담당자 지정: `jangster77`
- 기준 `upstream/devel`: `8d4c25d014dc42992aad6fa92c8eb761254c6bfc`

## provenance

- [#6642](https://github.com/edwardkim/rhwp/pull/6642) `b5c55677a3e065ccc227550210d02dfe888f9591` -> `76c27710a`
- [#6650](https://github.com/edwardkim/rhwp/pull/6650) `43d66964dff910dd3a70ee0a50f4d867ff94fca2` -> `018b64322`
- [#6652](https://github.com/edwardkim/rhwp/pull/6652) `9058c8158323295dc0365461cac30fe88124d099` -> `e0e07783d`

각 source head를 최신 `upstream/devel` 위에 `-x` 방식으로 적용했다. #6647은 기여자가 CLOSED 처리했고 동일 figure-space 변경이 이미 `devel`의 #6597에 있으므로 중복 적용하지 않았다.

## 검토 범위

- 셀 안 한 줄 문단에서 저장 줄 높이를 보존한다.
- 1x1 바깥 표 셀 안의 안쪽 표가 바깥 여백을 유지한다.
- 글상자 문단의 inline 글자처럼 개체 폭이 첫 줄 텍스트 오프셋에 중복 적용되지 않는다.

원 PR별 변경 검토와 실제 HWP/PDF의 대상 쪽 증적은 다음 기록에 분리해 남겼다.

- [#6642 검토 기록](pr_6642_review.md)
- [#6650 검토 기록](pr_6650_review.md)
- [#6652 검토 기록](pr_6652_review.md)
- [통합 시각 검토](pr_6642_6652_jeong_sik_visual_sweep.md)

## 로컬 검증

- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --all --check` 통과.
- `CARGO_TARGET_DIR=target/pr-review-jeong-sik-open-batch-20260903 cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 통과: `8968 passed`, `46 skipped`.
- Rust test manifest 및 unit-tier base 비교 통과.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` 통과.
- native Skia CLI 및 WASM 웹 패키지 빌드 통과.

## 병합 전 조건

이 문서 commit을 포함한 최신 PR head의 required CI가 success 또는 정책상 expected skip이고, `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN`일 때만 병합한다. 병합 뒤 실제 devel CI가 성공한 후에만 #6642, #6650, #6652의 comment/close를 `post_merge.md`에 따라 처리한다.
