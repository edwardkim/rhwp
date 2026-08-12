---
kind: pr-review
status: pending-ci-release-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4569 리뷰 - overlay 표 필러 흐름과 상향 클램프

## 접수

| 항목 | 문서 작성 시점 참고값 |
| --- | --- |
| PR | [#4569](https://github.com/edwardkim/rhwp/pull/4569) |
| 작성자 | `planet6897` |
| base / 원 head | `devel` / `217db887856529896fc7eae73e325d34c7182046` |
| 원 변경 규모 | 12 files, +712/-2 |
| GitHub merge 상태 | 원 PR 자체는 stacked base 때문에 `CONFLICTING`으로 표시됨 |
| 통합 적용 | `a5d7f1edc`(기능), `8b92c20ec`(overflow baseline) |
| 관련 이슈 | [#4514](https://github.com/edwardkim/rhwp/issues/4514), 후속 [#4568](https://github.com/edwardkim/rhwp/issues/4568) |

## 판정

빈 문단 필러를 비-TAC overlay 표의 유일한 흐름 공간으로 보존하고, 문단 기준 RowBreak overlay 표의
상향 clamp를 해제하는 수정은 실제 겹침을 제거한다. `issue_4514_overlay_table_flow`와
`issue_4515_table_overlap_diag`가 모두 통과했고, 기존의 6개 겹침 페이지는 render-tree 진단에서 재현되지
않았다.

다만 이 PR은 **#4514 전체 해결로 닫을 수 없다.** HWP 2020 기준 PDF와 직접 대조한 결과 기준은 46쪽,
현재 renderer는 48쪽이다. p8/p9/p10의 pixel match는 각각 79.22%, 81.24%, 78.82%
(diff 20.78%, 18.76%, 21.18%)였고, p10에는 다음 쪽으로 넘어가야 할 ECR-004 잔여 행이 그려지지 않은
공백이 남는다. 이는 overlay Shape 경로의 연속 행 paint 부재이며 [#4568](https://github.com/edwardkim/rhwp/issues/4568)에
이미 분리된 범위다.

따라서 이 통합은 “표 겹침 제거”라는 제한된 범위로 수용하되, #4514는 열어 두고 #4568의 fragment
paint 구현 전에는 한컴과 같은 자연 분할 또는 46쪽 정합을 주장하지 않는다.

## 시각 증적

- 원본: `samples/issue4514/sample1-repro.hwp`, SHA-256
  `72d3be39c8af8779387e7657cb9cd5823fda62dff1c8700ab1bfe73592baf793`.
- 기준: `pdf/issue4514/sample1-repro-2020.pdf`, HWP 2020 MCP `PrintToPDFEx`/1-up 변환,
  `run_status=0`, `validation=ok`, 46쪽 A4, SHA-256
  `5d7f56acd6938bbab74c450f67cab2ca549ea0d8fc72d55e241256c884e0f291`.
- 비교: `output/review-planet6897-20260812-issue4514/`, p8-p10 3쪽 완료, 자동 후보 0건.
  첫 `/tmp` 실행은 Snap Chromium namespace 때문에 PNG 쓰기에 실패했고, 저장소 `output/` 경로에서
  재실행해 정상 완료했다.
- 대표 시트: `mydocs/pr/assets/pr_planet6897_20260812_issue4514_p010_review.png`.
  왼쪽 한컴 p10의 ECR-004 잔여 조각과 오른쪽 rhwp 공백을 사람이 확인했다.

## 완료한 검증

- `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  리베이스 전 통합 후보에서 5,776 passed, 36 skipped, 7 slow, 447.357초.
- `cargo nextest run ... --test issue_4514_overlay_table_flow`: 1 passed.
- `cargo nextest run ... --test issue_4515_table_overlap_diag`: 1 passed.
- `cargo test --profile release-test --features native-skia skia --lib`: 58 passed.
- Native Skia integration 2종: 2 passed, 4 passed.
- `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `git diff --check`: 통과.
- `wasm-pack build --target web --out-dir pkg`: 산출물 갱신 확인.

## 최종 권고

통합 PR의 최신 code head CI가 통과하고 릴리스 준비 종료 뒤 작업지시자 승인이 있을 때만 수용한다.
현재는 릴리스 hold이므로 merge, 원 PR close, #4514 close를 수행하지 않는다.
