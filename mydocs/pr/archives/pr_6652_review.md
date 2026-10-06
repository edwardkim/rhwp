# PR #6652 검토 기록

## 판정: 승인

- 원 PR: [#6652](https://github.com/edwardkim/rhwp/pull/6652)
- 이슈: [#6651](https://github.com/edwardkim/rhwp/issues/6651)
- 기여자: `jeong-sik`
- 사전 검토자 지정: `jangster77`
- 검토 기준 head: `9058c8158323295dc0365461cac30fe88124d099`
- 최신 `upstream/devel` 위 통합 commit: `e0e07783d`

## 변경 검토

글상자 문단의 inline 글자처럼 개체 폭을 첫 줄 글자 오프셋으로 다시 전달하지 않도록 한다. 이전 경로는 도형 폭만큼 문단 시작 위치를 한 번 더 밀어 텍스트가 중복 이동할 수 있었다. 회귀 테스트는 `samples/table-in-tbox.hwp` 2쪽에서 inline 그림 뒤 텍스트의 시작 좌표를 고정한다.

## 검증

- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --all --check` 통과.
- `CARGO_TARGET_DIR=target/pr-review-jeong-sik-open-batch-20260903 cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 통과: `8968 passed`, `46 skipped`.
- Rust test manifest 및 unit-tier base 비교 통과.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` 통과.
- `native-skia` 릴리스 테스트 프로필 CLI 빌드와 WASM 웹 패키지 빌드 통과.

## 시각 증적

- 원본: `samples/table-in-tbox.hwp` 2쪽.
- 기준 PDF: `pdf/table-in-tbox-2022.pdf` 2쪽.
- 비교 첨부: [RHWP와 기준 PDF 2쪽 비교](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6652_rhwp_oracle.png).
- 비교 출력에서 글상자, inline 그림, 표의 물리 배치를 확인했다. 현 Mac 네이티브 Skia 환경의 한글 글꼴 대체 때문에 문자 모양의 픽셀 동일성은 주장하지 않는다. 회귀 테스트는 그림 뒤 텍스트 시작점의 좌표 계약을 직접 단언한다.

## Merge 후 contributor PR comment 계획

병합 후 [원 PR #6652](https://github.com/edwardkim/rhwp/pull/6652)에 아래 사실을 한 번 게시한다.

- [통합 PR #6657](https://github.com/edwardkim/rhwp/pull/6657)이 [merge commit `56d054852d03b9737f9a159073939d6bccebac77`](https://github.com/edwardkim/rhwp/commit/56d054852d03b9737f9a159073939d6bccebac77)으로 병합됐고, 이 PR의 source 반영 commit은 `e0e07783d`다.
- PR head의 CI, Rust CodeQL worker, Canvas visual diff, Adapter inter-diff, Proptest가 성공했다. devel push의 CI 및 CodeQL aggregate도 success이고 trusted post-merge reuse 정책에 따른 heavy worker skip은 expected skip이다.
- [Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 따르며, 실제 HWP 2쪽과 정본 PDF 2쪽의 글상자, inline 그림, 표의 물리 배치를 확인했다. 현 Mac 글꼴 대체 때문에 문자 모양의 픽셀 동일성은 주장하지 않는다.
- `![PR 6652 p2 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/56d054852d03b9737f9a159073939d6bccebac77/mydocs/pr/assets/pr_6642_6652_jeong_sik_integration_20260903/review_6652_rhwp_oracle.png)`를 실제 comment에 포함한다.

## Merge 후 issue comment 계획

[Issue #6651](https://github.com/edwardkim/rhwp/issues/6651)에는 위 merge commit, 실제 PR/devel 검증, `table-in-tbox.hwp` 2쪽과 정본 PDF 2쪽의 증적 범위, 수동 close 사유를 body-file comment로 남긴 뒤 CLOSED 처리한다. 통합 PR 본문에 closing keyword가 없으므로 auto-close가 아닌 수동 close임을 명시한다.

## 수용 범위

이 기록은 최신 head를 `upstream/devel` 위에 provenance-preserving cherry-pick한 통합 검토 결과다. 원 PR을 직접 병합하지 않고, 이후 통합 PR의 CI 및 병합 후 검증이 모두 성공할 때에만 원 PR 후속 처리 대상으로 삼는다.
