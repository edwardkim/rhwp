---
kind: implementation-record
status: code-ci-success-review-only-fast-pass-pending
canonical: mydocs/pr/archives/pr_3740_review_impl.md
last_verified: 2026-08-03
---

# PR #3740 구현·보정 기록 — #3738 시각 fidelity 조사와 CI 회복

## 원 변경과 collaborator 보정의 경계

| 구분 | commit / 범위 | 판정 |
| --- | --- | --- |
| contributor 핵심 수정 | `5be9b6ec` | 같은 host 문단의 TAC sibling `LINE_SEG` 선택과 TAC cap 계상을 함께 바로잡아 뒤 문단의 page-out 축을 해결한다. |
| visual fidelity 보정 | `a268e24d`부터 `91afc42a` 등 source head 뒤의 별도 commits | 실제 개인정보 제거 HWP/HWPX와 한컴 PDF를 기준으로 그림·caption·RowBreak 표·각주 owner를 focused하게 복원하고 stage별 evidence를 고정한다. |
| upstream 동기화 | `e697ee673` | 최신 `devel` merge commit. |
| CI 회복 보정 | `fd069734` | broad TopAndBottom raw anchor를 실제 그림 구조로 좁히고 빈 각주 virtual fragment range를 clamp하며 회귀·IR baseline을 추가한다. |

contributor 원 commit은 rewrite하거나 squash하지 않는다. collaborator의 보정은 각각 독립 commit으로
source head 뒤에 추가했고, 원 PR의 변경 범위와 후속 visual fidelity 조사를 혼동하지 않게 이 기록에
분리한다.

## `fd069734`의 필요성

`e697ee673` head의 GitHub CI에서는
`issue_3637_split_cell_nested_table_vpos::nested_table_snap_stays_inside_the_split_cell`가 native
HWPX nested split row에서 page count가 2로 축소되어 실패했다. 원인은 HWP5에만 성립하는 선언 높이
예외가 HWPX의 일반 1×1 table까지 raw stored anchor로 넓어진 것이었다.

보정은 다음 fail-closed 경계를 둔다.

1. HWP5/HWPX 공통 stored anchor 예외는 실제 그림 구조인 non-TAC 1×1 picture 또는 2×1
   picture-caption table에만 적용한다.
2. 일반 simple 1×1 declared-height 특례는 native HWP5에서만, 측정 높이가 declared height의
   1.5배 이하일 때만 적용한다.
3. 빈 각주 virtual fragment는 `line_start`와 `line_end`를 실제 `composed.lines.len()`으로
   clamp하여, range slice panic 없이 fallback layout을 사용한다.

이는 #3637 HWPX regression과 issue1891 generic table overflow를 보호하면서, #3738의 HWP5 그림
anchor 복원 범위만 유지한다. `tests/issue_3738_rowbreak_table_footnote_fragment.rs`에 empty virtual
fragment regression을 추가했고, IR field sweep baseline의 여섯 행은 renderer 의미 변경이 아니라
문서화된 HWP5RB serialization normalization만 반영한다.

## 기준 PDF 조사와 상태

기준 자료는 `pdf/pr3740/hwp/`의 한컴 2020 PDF와 동일 개인정보 제거 HWP/HWPX다. visual sweep은
pixel score만으로 판정하지 않고 PDF/SVG text owner, table fragment ledger, overlay, review PNG를
함께 사용했다.

| 범위 | 현재 판정 | 근거 |
| --- | --- | --- |
| p23–24, p25–27, p30–32, p37, p43, p44–45 | 해결 또는 current exact revision 비재현 | Stage 16/18/19/23/25–27 evidence |
| p52–54, p66–67, p76–80, p83–85 | 해결 또는 current exact revision 비재현 | Stage 17, 28–30 owner ledger·review PNG |
| p90 표 27 | 해결 | actual HWP focused regression: relationship row는 p90, `기타`는 p91, footer non-overlap |
| p127 | 해결 | 수정 전 자동 detector 1 → 현재 0; direct review PNG |
| p94, p106, p107–108, p156, PDF 215 ↔ native 219 | 이 PR 범위 밖 후속 | [#3820](https://github.com/edwardkim/rhwp/issues/3820), [#3821](https://github.com/edwardkim/rhwp/issues/3821) |

특히 p156은 자동 `square_wrap_text_overlap` 후보와 PDF 직접 대조로 확정된 결함이므로, 현재 PR의
통과 지표로 덮거나 r29 10k oracle 결과로 대체하지 않는다.

## 검증 실행 기록

```text
CARGO_TARGET_DIR=target/review-pr3740-full-regression CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --tests
# exit 0; #3637, #1733, #1921, #1937, #2430, #2559,
# issue_3738_rowbreak_table_footnote_fragment 18 tests, IR sweep 2 tests 포함

CARGO_TARGET_DIR=target/review-pr3740-native-skia CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --features native-skia skia --lib
# 58 passed

CARGO_TARGET_DIR=target/review-pr3740-native-skia CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --features native-skia skia \
  --test issue_2225_missing_picture_placeholder
# 2 passed

CARGO_TARGET_DIR=target/review-pr3740-native-skia CARGO_INCREMENTAL=0 \
  cargo test --profile release-test --features native-skia skia \
  --test render_p37_direct_pdf_export
# 4 passed
```

`cargo fmt --all -- --check`, `git diff --check`,
`CARGO_TARGET_DIR=target/review-pr3740-clippy CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`도
통과했다. 사용자 수동 WASM package build도 성공으로 확인됐지만, 이 기록은 이를 자동 CI 결과로
바꾸어 주장하지 않는다.

현재 code candidate의 [CI 30791602165](https://github.com/edwardkim/rhwp/actions/runs/30791602165)는
Build & Test, 8개 default-feature shard, Lint, Native Skia, Canvas visual diff, CodeQL을 모두
통과했다. 원작자의 r29 10k 주석은 `8cd4901dd` 시점의 유효한 비교 결과이나 현재 대형 head의
검증 결과가 아니므로, review 결론에는 이 full CI와 로컬 검증을 사용했다.

## 문서-only 종료 경로

이 구현 기록, [검토 기록](pr_3740_review.md), 오늘할일은 code/test/fixture를 포함하지 않는
single-parent trailing commit으로 함께 push한다. source head와 remote ref가 `fd069734`로 같은지
확인하고 파일별 LFS attribute 및 `git lfs status`를 검사한다. Markdown-only임이 확인되면
`GIT_LFS_SKIP_PUSH=1`의 dry-run과 실제 push를 사용한다.

그 뒤 preflight가 A 경로 candidate `fd069734`의 green Build & Test를 재사용하는지, latest docs head의
`Build & Test` aggregate가 success인지 확인한다. 하나라도 누락·실패·허용범위 이탈이면 full CI로
전환하며 merge하지 않는다. 성공한 뒤 실제 merge SHA와 issue 상태를 재확인하고, 사용자 승인 전에는
원작자 코멘트나 merge announcement를 게시하지 않는다.
