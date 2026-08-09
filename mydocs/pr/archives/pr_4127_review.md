---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4127 검토 - 빈 표 호스트 문단의 캐럿 조회 비용 제거

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4127](https://github.com/edwardkim/rhwp/pull/4127) / @humdrum00001010 |
| contributor 원 head | `5eca7e8507ebc1d1db8e897c4cd65c00c5145870` |
| base / 규모 | `devel`, 6개 파일, +220/-39 |
| 관련 이슈 | [#4126](https://github.com/edwardkim/rhwp/issues/4126) |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |
| maintainer 수정 권한 | `maintainerCanModify=true` |

표나 도형만 호스팅하는 빈 문단은 텍스트 offset으로 page tree에서 적중할 수 없다. 이 PR은 그런 경우에
모든 후보 페이지의 tree 구축을 생략하고 기존 앵커 폴백을 사용한다. 후속 commit은 초기 가드가
글자처럼 취급하는 그림·수식 및 각주/미주 마커까지 생략하던 회귀를 보정해, 페이지 scan이 실제로
적중할 수 없는 경우로 범위를 좁혔다.

## 검증과 시각 판단

이 변경은 cursor query와 진단 카운터 경로이며 페이지네이션·paint 결과를 직접 수정하지 않는다. 별도
기준 PDF fixture가 PR에 포함되지 않아 독립 visual sweep을 merge 근거로 사용하지 않았다. 대신 원 head의
Render Diff와 Native Skia CI가 성공했고, 아래 query 회귀를 현재 `devel` 위 누적 브랜치에서 재실행했다.

| 검증 | 결과 |
| --- | --- |
| #4126 작업량 상한 | `issue_4126_cursor_rect_empty_para_pages` 1 passed |
| 인라인 그림 캐럿 회귀 | `issue_1452_saved_caret` 8 passed |
| 미주 수식 캐럿 회귀 | `issue_1139_endnote_equation_cursor_rects_do_not_rewind_to_line_start` 1 passed |
| Rust 전체 | 작업지시자가 `cargo test --profile release-test --tests`를 현재 누적 브랜치에서 정상 종료까지 실행했다. |
| Native Skia | `cargo test --profile release-test --features native-skia skia --lib` 58 passed |
| Clippy | `cargo clippy --all-targets -- -D warnings` 통과 |
| WASM | `wasm-pack build --target web --out-dir pkg` 통과 |

원 head의 CI, Render Diff, Native Skia, archive build 3개, slow shard와 regular shard 3개, CodeQL 및
Build & Test aggregate도 모두 성공했다.

## 최종 권고

**수용.** #4131이 이 head 위에 쌓인 stacked PR이므로, #4127의 commit 순서를 보존한 뒤 #4131의
고유 commit을 누적하는 경로가 필요하다. 원격 merge 전 최신 head·required checks·mergeability를 재확인한다.
