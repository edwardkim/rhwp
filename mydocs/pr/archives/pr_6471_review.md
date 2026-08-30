---
kind: pr-review
status: maintainer-fix-visual-hold
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6471
issue: 6310
author: planet6897
---

# PR #6471 검토 기록

## 결정

| 구분 | 현재 결정 |
|---|---|
| 원 PR head 직접 병합 | 불가: 최신 devel과 중복된 Zoom 구현으로 원 CI 실패 |
| 수용 대상 | 중복 Zoom을 제거하고 CMYK JPEG 정규화만 보존한 메인터너 보정 #6481 |
| 현재 상태 | 메인터너 보정 후 수용 보류: CMYK/Zoom 출력의 visual sweep과 최신 CI가 아직 완료되지 않음 |
| 승인 뒤 처리 | #6481 수용·병합 뒤 #6471을 원 head가 아닌 보정된 통합 결과 기준으로 close |

## 식별과 provenance

| 항목 | 값 |
|---|---|
| 원 PR | https://github.com/edwardkim/rhwp/pull/6471 |
| 원 head | `5af8e20ca0e8b2e807b9e33dbb8d55ba4fc0402f` |
| 원 commits | `f1791b6109df1adebf27954cf252520ed63b9905`, `5af8e20ca0e8b2e807b9e33dbb8d55ba4fc0402f` |
| 통합 기준 | `upstream/devel@8a150f9a8bb19a9918e195da3a646690f68f4328` |
| 통합 commits | `60197deaa24793978e5f9b3b534a175961d2affe`, `44f1176dc9afe446ff76e522ece0b7f1b47892e5` |
| 메인터너 보정 | `de5209d52d20749ec413a996f0c89da0e7af1362` |
| 통합 순서 | 7/8 |

## 원 CI 실패와 메인터너 보정

원 PR CI는 최신 devel에 이미 존재하는 `ImageFillMode::Zoom` enum과 parser·serializer·renderer match arm을 다시 추가해 lint, build, render, proptest, adapter lane에서 실패했다. 이 실패는 CMYK JPEG 정규화 자체의 결함이 아니라 current-base 중복 구현이다.

통합에서는 중복 Zoom 선언과 분기를 제거하고 기존 devel의 `preserveAspectRatio="xMidYMid meet"` 계약을 유지했다. 원 PR의 고유 변경인 four-component JPEG 감지, CMYK JPEG의 PNG 정규화, 관련 fixture와 계약 테스트는 보존했다. 원 PR의 별도 explicit rectangle Zoom 분기를 시험 적용했을 때 기존 `zoom_cell_fill_svg_meets_the_cell_box`가 실패했으므로 그 과도한 변경은 수용하지 않았다.

최종 후보에서 기존 `header_imgbrush_zoom_is_not_collapsed_to_tile`, `zoom_cell_fill_svg_meets_the_cell_box`와 신규 `hwpx_zoom_mode_is_not_parsed_as_tile`, `four_component_jpeg_is_detected`가 모두 통과했다. 공통 필수 native·WASM·workspace clippy, workspace build, manifest와 format 검증도 통과했다.

## 현재 결론

원 head는 CI를 통과하지 못하므로 merge 불가다. 중복 Zoom 제거 뒤 CMYK JPEG 정규화와 해당 계약은 수용 가능한 코드 후보이지만, 이미지 fill과 CMYK 출력은 visual sweep 없이 수용을 확정할 수 없다. 현재 판정은 `메인터너 보정 후 수용 보류`다.
