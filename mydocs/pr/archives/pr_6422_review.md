---
kind: pr-review
status: visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6422
issue: 6299
author: planet6897
---

# PR #6422 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 하지 않음 |
| 수용 대상 | 통합 PR #6481에 포함된 `a9e3f759c2dcd0e87ade3aaaadbefd6b1246036a` |
| 현재 상태 | 수용 보류: 통합 head visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6422를 포함 수용 근거와 함께 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6422 |
| 원 head | `f83110d28fffea4620701ffbc2ec0aca9f8df2e8` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commit | `a9e3f759c2dcd0e87ade3aaaadbefd6b1246036a` |
| 통합 순서 | 2/8 |

## 검토

동일 vertical position의 line segment가 한 행을 중복 소비하지 않게 하는 #6299 회귀 보정이다. 원 PR CI는 수집 시점에 비성공 check 없이 완료된 상태였고, 최신 devel 위 cherry-pick에는 충돌이 없었다.

통합 후보에서 `wrap_fragment_rows_do_not_double_count`, `header_cell_content_matches_the_hangul_oracle`가 통과했다. 공통 필수 검증인 native·WASM·workspace clippy, workspace build, rust test suite manifest check와 `cargo fmt --check`도 통과했다.

원 PR의 시각 자료는 변경 의도의 보조 근거이며 통합 head 직접 산출물이 아니다. 최종 시각 판정은 통합 PR Render Diff 성공을 조건으로 한다.

## 현재 결론

코드·계약 검증에는 차단 finding이 없지만 line-wrap과 표 행 배치가 사용자-visible 범위다. visual sweep 미실행 상태이므로 수용 확정이나 원 PR close를 하지 않는다. 작업지시자 승인 뒤 #6481의 CI와 visual sweep이 확인된 경우에만 포함 수용한다.
