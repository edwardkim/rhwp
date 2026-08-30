---
kind: pr-review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6471
issue: 6310
author: planet6897
---

# PR #6471 검토 기록

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

## 판정

원 head 그대로는 수용 불가였으나 중복 구현 제거 후 고유 CMYK 보정은 수용 가능하다. 통합 PR latest-head CI와 Render Diff를 기다리는 메인터너 보정 수용 후보이다.
