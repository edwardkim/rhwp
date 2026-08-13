---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4734 검토 - 함초롬바탕 라틴 폭을 실제 hmtx로 복원

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4734](https://github.com/edwardkim/rhwp/pull/4734) |
| 관련 이슈 | [#4701](https://github.com/edwardkim/rhwp/issues/4701) |
| 작성자 / source | @planet6897 / `fix/4701-hcr-latin-hmtx` |
| 원 source head | `9fc612ec839d3d21ff59640b305c200cfe8fcf59` |
| 기준 devel | `e550a270f457691597da4e9cff76774297597bb5` |
| 가시성 통합 branch | `review/planet6897-20260813` |
| 적용 순서 | 첫 번째: `c47fa5c47` → `9fc612ec8`을 로컬 `0235817a6` → `b6f12fed2`로 cherry-pick |
| reviewer | @jangster77 지정 완료 |

검토 경로는 `maintainer_general`이며, `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `multi_pr_update_branch`를 함께 적용했다. 원 PR은
`함초롬바탕`과 `HCR Batang`의 ASCII 폭을 임의 전각 정규화 상수로 덮던 경로를 제거하고,
내장 글꼴 hmtx 측정값을 그대로 사용한다. 한글과 공백은 기존 측정 경로를 유지한다.

## 완료한 검증

| 범위 | 명령 또는 근거 | 결과 |
| --- | --- | --- |
| 글꼴 폭 계약 | `cargo test --profile release-test --target-dir target/pr-review --lib issue_4701_hcr_batang_latin_uses_font_hmtx -- --nocapture` | 1 passed. ASCII, HCR alias, 한글·공백 경계를 확인했다. |
| SVG golden | `cargo test --profile release-test --target-dir target/pr-review --test svg_snapshot -- --nocapture` | 8 passed. `form-002`와 `issue-157` golden 갱신을 포함한다. |
| 통합 Rust | `cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast` | 5,930 passed / 37 skipped / 7 slow, 486.981초. |
| 최신 source 대조 | `git fetch upstream pull/4734/head:refs/remotes/upstream/pr4734-head` 후 변경 파일 비교 | 원 source head와 체리픽 파일 내용이 일치했다. |
| 품질 | `git diff --check` | 통과. |

## 시각·fixture 검토

한컴 2022 기준 PDF와 HWPX를 사용해 실제 폭 변화가 줄바꿈·표 흐름 붕괴로 이어지지 않는지 확인했다.

| fixture / 기준 PDF | 페이지 | pixel match | visual accuracy proxy | 자동 후보 | 대표 asset |
| --- | ---: | ---: | ---: | ---: | --- |
| `samples/hwpx/form-002.hwpx` / `pdf/hwpx/form-002-2022.pdf` | 1 | 80.53668% | 30.18353% | 0 | `mydocs/pr/assets/pr_4734_form_002_p001_review.png` |
| `samples/hwpx/issue_157.hwpx` / `pdf/hwpx/issue_157-2022.pdf` | 2 | 94.00782% | 28.56761% | 0 | `mydocs/pr/assets/pr_4734_issue_157_p002_review.png` |

- `form-002` 원본 / PDF SHA-256: `5ab8f7c368e02538f75f1cd2bd82bbd8de2f925a54ba7b38ec9395b2cdb804d4` /
  `629f1d93be234e4c4c551d319e247c1158d225cfe8a86bb179754a1b6cf2e077`.
- `issue_157` 원본 / PDF SHA-256: `120be7f6b5d09d1a87c1598ebbb11d66015ad982b471c84efbd80be1dc0ffdc1` /
  `fe5bcf697e5343d36cef8e6c4ee3532a55deb95b54e1fd139e8a927d1a080e37`.
- 임시 산출물은 각각
  `output/pr-review/planet6897-20260813/4734/form-002/pr4734-form-002/` 및
  `output/pr-review/planet6897-20260813/4734/issue-157/pr4734-issue-157/`에 있다.
- 대표 PNG SHA-256은 각각
  `27837f64d5387ed4eeddbcb36b1f8acbc4e8b850ce8109a0537b32aed7c19d56`,
  `5b3e7b90642d3c4139d162f6d9f54dc86e9de5a1d41497fd4bbd557d370ec238`이다.

두 페이지 모두 프레임 밖 본문, 열 흐름 붕괴, red marker drift 후보가 없었다. 다만 절대 pixel/ink
지표의 차이는 이 PR만의 회귀로 해석하지 않는다. 한컴 PDF와 rhwp의 기존 글꼴·조판 fidelity 차이가
포함된 수치이며, 잔여 fidelity 개선은 [#3820](https://github.com/edwardkim/rhwp/issues/3820)에서
추적한다.

## 판정

**통합 수용 권고.** hmtx를 다시 사용하는 범위는 한정돼 있고, 실제 glyph 폭 계약·golden·두 HWPX
시각 검토와 전체 Rust 회귀가 모두 통과했다. 통합 PR 생성 뒤에는 해당 최신 head의 GitHub required
checks와 mergeability를 다시 확인하고 작업지시자 승인을 받은 뒤에만 merge한다.
