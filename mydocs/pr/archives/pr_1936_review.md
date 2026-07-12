# PR #1936 검토 — Task #1891: HWP5-origin HWPX 쪽수 보정

- 작성일: 2026-07-05 / 검토자: Codex (메인테이너 대행 검토)
- PR: `task/m100-1891-pr-candidate-clean` → `devel`
- PR URL: https://github.com/edwardkim/rhwp/pull/1936
- 연결 이슈: https://github.com/edwardkim/rhwp/issues/1891
- merge commit: `a6469f4f61388abbe5987f2a6088ca0ffc903fa1`
- 처리 방식: #1913 merge 후 남은 HWP5-origin HWPX pagination 잔여분을 별도 PR로 수용

## 1. PR 요약

#1913은 외부 참조 BinData Link 그림의 HWPX 왕복 소실과 #1891 클러스터 판별을 처리했다.
#1936은 그 뒤에도 남아 있던 HWP5-origin HWPX 계열의 쪽수 불일치를 공식 PDF 기준으로
정리한 후속 수정이다.

핵심 변경은 다음과 같다.

- HWP5 원본에서 export된 HWPX를 marker 기반으로 식별해 일반 HWPX와 pagination tolerance를 분리했다.
- 빈 텍스트 문단이지만 `CharShape`/`ParaShape`/`char_count`가 남아 있는 경우 문서 속성 기반 fallback line metric을 적용했다.
- 1행 RowBreak 표의 저장 object height와 실제 cell content height가 크게 다른 케이스를 표/control 속성 기준으로 처리했다.
- RowBreak rowspan/cut height 계산에서 저장 행 높이보다 내용 높이가 유의미하게 큰 경우 content cut height를 우선하도록 보정했다.
- 공식 PDF/HWP/HWPX 샘플과 전용 회귀 테스트를 추가했다.

## 2. 검토 포인트

- 보정 기준은 특정 파일명, 페이지 번호, PR/issue 번호가 아니라 HWP5-origin marker, `LineSeg`,
  `ParaShape`, `CharShape`, 표/셀 속성, control 저장 높이와 RowBreak 속성에 근거한다.
- #1913의 BinData Link 수정 영역과 중복되지 않도록 최신 `upstream/devel` 기준으로 rebase했으며,
  #1913 커밋은 이미 적용된 것으로 제거했다.
- `tests/issue_1891.rs`는 공식 PDF 기준 쪽수와 HWP export HWPX 재파스 쪽수 보존을 함께 검증한다.
- 렌더 영향 PR이므로 쪽수 정합을 핵심 사용자-visible 결과로 보았다. 별도 PNG visual sweep asset은
  남기지 않았고, 공식 PDF 페이지 수와 rhwp 렌더 페이지 수 매트릭스 및 GitHub Render Diff 통과를
  merge 판단 근거로 사용했다.

## 3. 페이지 매트릭스

순서: 공식 PDF / HWP / HWPX / HWP export HWPX

| 샘플 | PDF | HWP | HWPX | HWP export HWPX |
| --- | ---: | ---: | ---: | ---: |
| `76076_regulatory_analysis` | 82 | 82 | 82 | 82 |
| `80168_regulatory_analysis` | 157 | 157 | 157 | 157 |
| `80250_regulatory_analysis` | 17 | 17 | 17 | 17 |
| `86712_regulatory_analysis` | 65 | 65 | 65 | 65 |

merge 후 docs-only PR 작성 중 다음 명령으로 재확인했다.

```bash
cargo build --bin rhwp
target/debug/rhwp dump-pages <sample>
target/debug/rhwp export-hwpx <sample.hwp> <tmp-output.hwpx> --verify-pages
pdfinfo <baseline.pdf>
```

## 4. 검증 결과

로컬 검증:

- `env CARGO_INCREMENTAL=0 cargo build --bin rhwp`
- 공식 PDF/HWP/HWPX/HWP export HWPX 페이지 매트릭스 확인
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1891`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1133_nested_table_valign`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_rowbreak_chart_overlap`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --lib wasm_api::tests::test_reflow_linesegs_keeps_hwpx_sample2_page_count_for_textrun_warnings`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- rebase 후 재확인:
  - `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1891`
  - `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`
  - `git diff --check upstream/devel..HEAD`

GitHub Actions:

- CI preflight: success
- Build default-feature tests: success
- Native Skia tests: success
- Build & Test: success
- CodeQL: success
- Render Diff preflight / Canvas visual diff: success
- WASM Build: skipped

## 5. 판단

merge 가능으로 판단했고, PR #1936은 `a6469f4f61388abbe5987f2a6088ca0ffc903fa1`로 merge 완료됐다.

#1891은 PR description의 `Closes #1891`만으로 자동 close되지 않아 후속 docs-only PR merge 후
수동 후속 코멘트와 close 처리를 진행한다.
