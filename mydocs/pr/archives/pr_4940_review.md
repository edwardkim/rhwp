# PR #4940 검토 - HWPX lineseg 범위와 표 셀 lineWrap 보존

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#4940](https://github.com/edwardkim/rhwp/pull/4940) |
| 작성자 | `planet6897` |
| 검토자 | `jangster77` |
| 검토 방식 | 외부 기여 PR 누적 체리픽 검토 및 메인터너 보정 |
| base / 원본 head | `devel` / `00e4dc7a859a4143753864d05b5b9aa14642fc84` |
| 검토 브랜치 반영 | `994ea507e`, `06536d461` |
| 메인터너 보정 | `1afe41afb` |
| 검토일 | 2026-08-16 |
| 작성 시점 병합 상태 | `CLEAN` |

## 검토 범위와 판단

- HWP5 `LIST_HEADER` bit 19~20과 HWPX `lineWrap`의 BREAK/SQUEEZE/KEEP 매핑을 IR에 보존해,
  셀 줄 수와 표 높이가 저장 과정에서 바뀌지 않도록 한 점은 적절하다.
- 원본 구현의 section lineseg 권위 판정은 최상위 문단만 확인하고 셀 문단은 이전의 문단 단위
  reflow를 유지했다. 따라서 저장 lineseg가 있는 표 내부의 0 높이 블록을 다시 조판해 페이지 수를
  바꿀 수 있었다.
- 메인터너 보정 `1afe41afb`은 중첩 표 셀까지 양수 lineseg를 재귀 판정하고, 셀 재조판에도 같은
  section 권위를 적용한다.

## 검증

- `cargo fmt --all -- --check` 통과
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --lib issue4898` 통과, 2건
- `CARGO_TARGET_DIR=target/pr-review-planet6897 cargo test --lib cell_line_wrap` 통과, 2건
- 누적 검토 브랜치에서 `git diff --check` 통과

## 최종 권고

원본 PR head에는 중첩 표 lineseg 보정이 아직 없으므로, 메인터너 보정을 해당 PR 또는 통합 PR에 반영한 뒤
병합을 권고한다.

## 통합 병합 판단 및 local validation

- 통합 PR: [#4941](https://github.com/edwardkim/rhwp/pull/4941)
- 통합 코드 후보: `1afe41afb`를 포함한 `integrate/planet6897-open-prs-20260816`
- `cargo fmt --all -- --check`: 성공
- `cargo test --profile release-test --tests`: 성공 (exit 0)
- `cargo clippy --all-targets --all-features -- -D warnings`: 성공 (exit 0)
- `cargo build --release`: 성공 (exit 0)

중첩 표 셀까지 section lineseg 권위를 적용하는 메인터너 보정은 `1afe41afb`에 포함되어
#4941로 반영됐다. 원본 #4940 head에는 이 보정이 없으므로 개별 병합하지 않고, 전체 local
validation을 통과한 #4941의 최신 head CI가 통과하면 #4941만 병합한다.
