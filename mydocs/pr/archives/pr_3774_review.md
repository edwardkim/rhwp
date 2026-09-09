---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-03
---

# PR #3774 검토 - zone 전환의 실제 쪽 소비 높이 사용

## 라우팅

base route: `maintainer_general.md`. 적용 보조 절차는 `intake_and_review.md`,
`local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`다.

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#3774](https://github.com/edwardkim/rhwp/pull/3774) / @planet6897 |
| 원 기능 commit | `0834a6a3bc0db910113616ce8d27394227fe3f05` |
| 원 head | `229d5b21c62811f04370cb519464877f771eea5c` |
| 기준 devel | `6ab503fe97b7abfd1839800c5c018da9f9abf4c5` |
| 가시성 검토 브랜치 | `review/planet6897-20260803` |
| 누적 적용 commit | `928b5f639` |
| 적용 제외 | 원 head의 `Merge branch 'devel'` commit은 누적 체리픽에서 제외 |
| 충돌 | 없음 |
| 작성 시점 원 PR 상태 | `MERGEABLE` / `BEHIND`, 원 head CI 성공. merge 전 재확인 필요 |

## 변경 검토

zone 전환 가드는 저장 사다리의 vpos를 현재 쪽의 사용 높이로 사용했다. 한글의 쪽
경계와 rhwp의 쪽 경계가 어긋난 경우 사다리는 다음 쪽 상단 값으로 되감기며, 이 값은
이미 배치한 zone의 실제 높이를 과소평가한다. 변경은 사용 가능 높이 범위의 사다리값과
`st.current_height` 중 큰 값을 택해, 꽉 찬 쪽에 다음 zone을 겹쳐 얹지 않도록 한다.

기존의 섹션 누적 좌표 예외(`max_vpos_px > available`)는 `st.current_height` 폴백으로
그대로 유지한다. fixture는 0이 아닌 400 HWPUNIT 되감기와 `ColumnDef`를 함께 고정해
다른 vpos-reset 가드가 먼저 동작하는 경우를 배제한다.

## 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| `issue_3765_zone_switch_ladder_understates_page` | 완료. 2 / 2 통과 |
| IR field sweep baseline | 완료. 589행 기준 TSV와 일치 |
| overflow-cell baseline | 완료. 22행 기준 TSV와 일치 |
| `cargo test --profile release-test --tests` | 완료. 실패 표식 없이 종료, 마지막 visual round-trip baseline 3 / 3 통과 |
| Native Skia 라이브러리 | 완료. 58 / 58 통과 |
| Native Skia `issue_2225_missing_picture_placeholder` | 완료. 2 / 2 통과 |
| Native Skia `render_p37_direct_pdf_export` | 완료. 4 / 4 통과 |
| WASM build | 완료. `wasm-pack build --target web --out-dir pkg` 성공, 생성물은 ignore 경로에만 존재 |
| `cargo fmt --check` / `git diff --check` | 완료. 모두 통과 |
| `cargo clippy --all-targets -- -D warnings` | 완료. 경고 없이 통과 |

## 시각 검증

페이지가 넘치던 다단 zone 배치를 고치는 renderer/typeset 변경이므로 한컴 PDF/SVG sweep을
수행했다.

- 원본 fixture SHA-256: `7f858d9c5c3ad30660d583b6628372963ce9f9d0f14a9d04a35363c77b624c29`
- 기준 PDF: `pdf/issue3765/zone_switch_ladder_understates_page-2020.pdf`
  (SHA-256 `913c987fc7170c1deec4cfbbf820e27be2fb513938cd46772f9b399c082c407d`, A4 2쪽).
  MCP 결과는 `run_status=0`, `validation=ok`, `PrintToPDFEx`, `PrintMethod=0`이었다.
- sweep 임시 경로: `output/pr-planet6897-20260803/visual/pr3774-zone-switch/`.
  2쪽 모두 비교했고 자동 구조 후보는 `0 / 2`였다. 평균 pixel match는 `99.17597%`,
  평균 visual accuracy proxy는 `15.39461%`였다.
- 대표 asset: `mydocs/pr/assets/pr_3774_zone_switch_ladder_review_001.png`,
  `mydocs/pr/assets/pr_3774_zone_switch_ladder_review_002.png`.

사람 확인에서 1쪽의 앞 zone 25행과 새 zone 머리말, 2쪽의 새 zone 본문 01–20 순서가
한컴 PDF와 rhwp에서 같은 페이지 경계를 유지했다. 낮은 ink 지표는 한컴 PDF와 SVG PNG의
글꼴 raster 차이이며, overlay에서도 두 zone이 한 쪽에 겹치거나 쪽 밖으로 사라지는 현상은
보이지 않았다.

## 현재 판정

**로컬 검증 수용 권고.** 새 fixture의 구조와 페이지 경계 계약, release-test 전체, Native
Skia 3종, WASM과 정적 검사를 통과했고 한컴 PDF/SVG sweep의 2쪽 구조도 확인했다. 통합 PR의
최신 원격 CI와 작업지시자 승인만 merge 전 외부 조건으로 남는다.
