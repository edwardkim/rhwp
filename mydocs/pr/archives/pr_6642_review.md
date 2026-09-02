# PR #6642 검토 기록

## 판정: 승인

- 원 PR: [#6642](https://github.com/edwardkim/rhwp/pull/6642)
- 이슈: [#6632](https://github.com/edwardkim/rhwp/issues/6632)
- 기여자: `jeong-sik`
- 사전 검토자 지정: `jangster77`
- 검토 기준 head: `b5c55677a3e065ccc227550210d02dfe888f9591`
- 최신 `upstream/devel` 위 통합 commit: `76c27710a`

## 변경 검토

셀 안 문단에서는 저장된 줄 높이를 접지 않도록 범위를 `cell_ctx.is_none()`으로 제한한다. 따라서 셀 밖의 기존 축소 규칙은 유지하면서, 셀 안의 글자 및 글자처럼 취급되는 도형 줄이 저장 높이보다 작아지지 않도록 한다. 추가 회귀 테스트는 `samples/exam_kor.hwp`의 5쪽과 `samples/hwpspec/hwpspec_2018_2_page106.hwp`의 106쪽을 대상으로 이 계약을 고정한다.

## 검증

- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --all --check` 통과.
- `CARGO_TARGET_DIR=target/pr-review-jeong-sik-open-batch-20260903 cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 통과: `8968 passed`, `46 skipped`.
- Rust test manifest 및 unit-tier base 비교 통과.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` 통과.
- `native-skia` 릴리스 테스트 프로필 CLI 빌드와 WASM 웹 패키지 빌드 통과.

## 시각 증적

- 원본: `samples/exam_kor.hwp` 5쪽.
- 기준 PDF: `pdf/exam_kor-2022.pdf` 5쪽.
- 비교 첨부: [RHWP와 기준 PDF 5쪽 비교](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6642_rhwp_oracle.png).
- 네이티브 Skia 출력에서 셀, 표, 글자처럼 취급되는 도형의 페이지 내 배치가 기준 PDF와 같은 구조로 유지됨을 확인했다. 폰트 래스터화 차이가 있어 픽셀 동일성은 주장하지 않는다.

## 수용 범위

이 기록은 최신 head를 `upstream/devel` 위에 provenance-preserving cherry-pick한 통합 검토 결과다. 원 PR을 직접 병합하지 않고, 이후 통합 PR의 CI 및 병합 후 검증이 모두 성공할 때에만 원 PR 후속 처리 대상으로 삼는다.
