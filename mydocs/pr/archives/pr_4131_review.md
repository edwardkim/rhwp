---
kind: pr_review
status: accepted-for-integrated-merge
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4131 검토 - 거대 분할 표의 셀 캐럿·paginate 병목 제거

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4131](https://github.com/edwardkim/rhwp/pull/4131) / @humdrum00001010 |
| contributor 원 head | `bd2a65e9c7ba8fd5c507ea3442f7415536476158` |
| base / 규모 | `devel`, 15개 파일, +1427/-111 |
| 관련 이슈 | [#4128](https://github.com/edwardkim/rhwp/issues/4128), [#4129](https://github.com/edwardkim/rhwp/issues/4129) |
| stack 관계 | #4127의 `5eca7e…`가 이 PR의 ancestor임을 확인했다. |
| 작성 시점 원격 상태 | `mergeable=MERGEABLE`, `mergeStateStatus=CLEAN` |
| maintainer 수정 권한 | `maintainerCanModify=true` |

고유 변경은 두 축이다.

1. `PartialTable`의 행·cut 메타데이터와 `cell_units`를 대조해 셀 내부 위치가 실제로 렌더되는 1~2개
   페이지로 후보를 좁힌다. 해석 실패나 빈 결과는 기존 `find_pages_for_paragraph`로 넓게 되돌아간다.
2. `mixed_nested_flow_extra_from_cut`의 문단별 전체 unit 재스캔을 한 번의 단조 run walk로 바꿔
   거대 분할 표 paginate의 O(PxU) 반복을 제거한다.

## 검증과 시각 판단

renderer/layout 코드가 포함되므로 시각 검증 대상이다. 원 PR은 자체 A/C SVG·corpus 비교를 보고했지만,
이번 검토에서는 그 산출물을 독립 merge 근거로 재사용하지 않았다. PR에 기준 PDF 또는 보존할 visual asset이
없으므로 새 PDF sweep을 만들지 않았고, 현재 누적 브랜치의 페이지·cursor 작업량 회귀, Rust 전체,
Native Skia와 WASM build를 독립 실행했다. 원 head의 Render Diff CI도 성공했다. 따라서 이 판단은
출력 fidelity 전면 보증이 아니라, 변경 범위의 기능·회귀·renderer gate 수용 판단이다.

| 검증 | 결과 |
| --- | --- |
| #4128 셀 위치 페이지 좁히기 | `issue_4128_cell_cursor_page_narrowing` 1 passed |
| #4129 mixed nested scan 상한 | `issue_4129_mixed_nested_scan_budget` 1 passed |
| Rust 전체 | 작업지시자가 `cargo test --profile release-test --tests`를 현재 누적 브랜치에서 정상 종료까지 실행했다. |
| Native Skia placeholder·PDF | `issue_2225_missing_picture_placeholder` 2 passed, `render_p37_direct_pdf_export` 4 passed |
| Native Skia 라이브러리 | 58 passed |
| Clippy / WASM | `-D warnings` 통과, `wasm-pack build --target web --out-dir pkg` 통과 |
| 공백 검사 | `git diff --check` 통과 |

원 head의 CI, Render Diff, Native Skia, archive build 3개, slow shard와 regular shard 3개, CodeQL 및
Build & Test aggregate도 모두 성공했다.

## 최종 권고

**수용.** #4127을 먼저 포함하는 누적 경로에서 #4131 고유 commit만 적용해야 한다. merge 전 최신 head의
required checks와 mergeability를 다시 확인하고, 기준 PDF를 이용한 전면 fidelity 비교는 별도 과제로 유지한다.
