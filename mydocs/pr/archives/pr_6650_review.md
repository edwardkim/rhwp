# PR #6650 검토 기록

## 판정: 승인

- 원 PR: [#6650](https://github.com/edwardkim/rhwp/pull/6650)
- 이슈: [#6648](https://github.com/edwardkim/rhwp/issues/6648)
- 기여자: `jeong-sik`
- 사전 검토자 지정: `jangster77`
- 검토 기준 head: `43d66964dff910dd3a70ee0a50f4d867ff94fca2`
- 최신 `upstream/devel` 위 통합 commit: `018b64322`

## 변경 검토

1x1 바깥 표 셀 안에서 안쪽 표를 풀 때 안쪽 표의 바깥 여백을 셀 안 여백 안쪽에 남긴다. 안쪽 표의 폭과 높이, 시작 좌표, 흐름 끝 좌표에 바깥 여백을 일관되게 반영해 바깥 표 padding과 안쪽 표 margin이 겹치지 않도록 한다. 회귀 테스트는 `samples/k-water-rfp.hwp` 17쪽의 실제 중첩 표를 대상으로 좌표 계약을 고정한다.

## 검증

- `git diff --check upstream/devel...HEAD` 통과.
- `cargo fmt --all --check` 통과.
- `CARGO_TARGET_DIR=target/pr-review-jeong-sik-open-batch-20260903 cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast` 통과: `8968 passed`, `46 skipped`.
- Rust test manifest 및 unit-tier base 비교 통과.
- `cargo clippy --locked --workspace --all-targets -- -D warnings` 통과.
- `native-skia` 릴리스 테스트 프로필 CLI 빌드와 WASM 웹 패키지 빌드 통과.

## 시각 증적

- 원본: `samples/k-water-rfp.hwp` 17쪽.
- 기준 PDF: `pdf/k-water-rfp-2022.pdf` 17쪽.
- 비교 첨부: [RHWP와 기준 PDF 17쪽 비교](../assets/pr_6642_6652_jeong_sik_integration_20260903/review_6650_rhwp_oracle.png).
- 비교 출력에서 중첩 표와 도식의 물리 배치는 확인했다. 현 Mac 네이티브 Skia 환경은 해당 한글 글꼴을 모두 제공하지 않아 RHWP 쪽 일부 글리프가 대체 문자로 보이며, 이 첨부는 문자 모양의 픽셀 동일성 증명이 아니다. 회귀 테스트의 좌표 단언이 여백 계약의 실행 증적이다.

## Merge 후 contributor PR comment 계획

병합 후 [원 PR #6650](https://github.com/edwardkim/rhwp/pull/6650)에 아래 사실을 한 번 게시한다.

- [통합 PR #6657](https://github.com/edwardkim/rhwp/pull/6657)이 [merge commit `56d054852d03b9737f9a159073939d6bccebac77`](https://github.com/edwardkim/rhwp/commit/56d054852d03b9737f9a159073939d6bccebac77)으로 병합됐고, 이 PR의 source 반영 commit은 `018b64322`다.
- PR head의 CI, Rust CodeQL worker, Canvas visual diff, Adapter inter-diff, Proptest가 성공했다. devel push의 CI 및 CodeQL aggregate도 success이고 trusted post-merge reuse 정책에 따른 heavy worker skip은 expected skip이다.
- [Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 따르며, 실제 HWP 17쪽과 정본 PDF 17쪽의 물리 배치와 회귀 테스트의 여백 좌표 계약을 확인했다. 현 Mac 글꼴 대체 때문에 문자 모양의 픽셀 동일성은 주장하지 않는다.
- `![PR 6650 p17 visual review](https://raw.githubusercontent.com/edwardkim/rhwp/56d054852d03b9737f9a159073939d6bccebac77/mydocs/pr/assets/pr_6642_6652_jeong_sik_integration_20260903/review_6650_rhwp_oracle.png)`를 실제 comment에 포함한다.

## Merge 후 issue comment 계획

[Issue #6648](https://github.com/edwardkim/rhwp/issues/6648)에는 위 merge commit, 실제 PR/devel 검증, `k-water-rfp.hwp` 17쪽과 정본 PDF 17쪽의 증적 범위, 수동 close 사유를 body-file comment로 남긴 뒤 CLOSED 처리한다. 통합 PR 본문에 closing keyword가 없으므로 auto-close가 아닌 수동 close임을 명시한다.

## 수용 범위

이 기록은 최신 head를 `upstream/devel` 위에 provenance-preserving cherry-pick한 통합 검토 결과다. 원 PR을 직접 병합하지 않고, 이후 통합 PR의 CI 및 병합 후 검증이 모두 성공할 때에만 원 PR 후속 처리 대상으로 삼는다.
