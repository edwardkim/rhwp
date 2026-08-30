---
kind: pr-review
status: approved-via-integration
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-31
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
| 현재 상태 | 승인: #6481 통합 후보 기준으로 코드·계약·claim-scoped 시각 증적에 차단 finding 없음 |
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

원 PR의 시각 자료는 변경 의도의 보조 근거로만 사용했다. 통합 code head에서 HWP MCP 2020 기준 PDF와 1쪽을 직접 비교했다. `pixel_match=91.23962`, `visual_accuracy_proxy_percent=33.36546`, 자동 후보 0건이며 line-band drift는 최대 4px이었다. review PNG에서 표 행과 wrap fragment의 흐름이 겹치거나 같은 행을 이중 소비하는 모습은 보이지 않았다. 대표 증적은 [p1 review PNG](../assets/pr_6481_issue6299_p001_review.png)이고, 재현 명령과 원본/PDF SHA는 [PR #6481 visual sweep 기록](pr_6481_planet6897_visual_sweep.md)에 있다.

## 현재 결론

**최종 판정: 승인.** line-wrap과 행 배치라는 이번 주장에 대한 직접 확인과 계약 검증에는 차단 finding이 없다. 글꼴 raster 차이로 낮은 proxy 수치를 전체 fidelity 합격으로 쓰지 않으며, 원 PR은 직접 merge하지 않고 #6481 통합 결과로만 수용한다. remote push, merge, #6422 close는 별도 지시가 있을 때만 수행한다.
