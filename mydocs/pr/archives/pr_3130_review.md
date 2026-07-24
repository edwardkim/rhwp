# PR #3130 검토 기록 — revision 기반 render_normalized derived state

## 메타와 통합 판단

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3130](https://github.com/edwardkim/rhwp/pull/3130) |
| 작성자 / 관련 이슈 | `postmelee` / [#2308](https://github.com/edwardkim/rhwp/issues/2308) (open 유지) |
| 원 PR 기준 | `devel` / head `f4ac9cd9278ec9c6a79a5976b2622d8bab1a2fef` |
| 원 PR 최신 상태 | OPEN, MERGEABLE/BEHIND, required check 성공, maintainer 보류 코멘트 없음 (2026-07-24 확인) |
| 검토 브랜치 | `integrate/postmelee-20260724` |
| 통합 기준 / 순서 | `upstream/devel@1b5950a95` / 2/3 (#3125 뒤, #3136 앞) |
| 처리 결론 | #3125와 코드 접점이 있으므로 원 PR direct merge 대신 메인터너 조정 통합 PR로 수용 권고 |

검토자는 `devel` 위 가시성 브랜치에서 원 PR head를 fetch하고 누적 적용했다. 절차는
[PR 리뷰·통합 워크플로](../../manual/pr_review_workflow.md) 4.1.1절에 반영했다.

## 범위와 적용

#2308은 매 편집마다 전체 `render_normalized`를 복제하지 않도록 revision keyed overlay와
derived state를 도입한다. image stack projection cache, nested width sparse overlay, section invalidation,
fragment geometry 검증을 포함한다. 원 PR의 devel merge `f4ac9cd`는 제외했다.

```text
38df127 0db4b6e a09788a 850cb69 852277e fc91d90
b211442 599f643 b86723b 4e0032b
```

## 메인터너 충돌 조정

이 PR은 #3125가 이미 deferred pagination을 도입한 rendering 경로와 겹쳤다. 다음을 모두 보존했다.

- `document_core/mod.rs`: #3125의 deferred pagination 구조와 #3130의
  `RenderNormalizationState`/overlay를 함께 둔다.
- text editing: 기존 mutable refresh를 revision dirty marking으로 전환하되, deferred descriptor와
  section revision dirty 처리를 보존한다.
- rendering query: deferred pagination 시작 시 현재 section revision에 맞는 normalized
  paragraphs/composed 결과를 사용하고, 없으면 기존 source/composed로 fallback한다.
- `fc91d90` 충돌: #3125 profiler timing과 #3130 normalization 회귀 테스트를 각각 보존했다.

이는 체리픽으로 생긴 충돌을 메인터너가 기능 불변식 기준으로 보정한 것이며, 원 PR branch에 직접 push한
변경이 아니다.

## 누적 검증과 시각 확인

통합 tree에서 `git diff --check`, fmt, library check/release build, `cargo test --release --lib`
(2,888 passed), 최초 1회의 release-test integration suite, Native Skia 공식 gate, clippy/doctest를
실행했다. 문서/asset 추가 뒤에는 코드가 바뀌지 않아 전체 cargo suite를 재실행하지 않는다.

`samples/76076_regulatory_analysis.hwp`와
`samples/issue1891/76076_regulatory_analysis-2024.pdf`의 33–34쪽(복합 표)을 비교했다.

- export/render-tree/PDF 모두 82쪽, 선택 2쪽의 자동 후보는 0건이다.
- p033/p034 pixel match는 각각 85.587%/90.755%, 평균 88.171%다. 잉크 보조값 평균은
  10.599%로 폰트 rasterization 차이가 포함되므로 단독 통과 기준으로 삼지 않았다.
- p034에서 표 프레임, 셀 순서, 문단 흐름과 페이지 하단을 사람 판독해 붕괴가 없음을 확인했다.
- 임시 산출물: `output/pr-review-3130-20260724/pr3130-issue2308/{compare,overlay,review}/…_034.png`
- 안정 검토 자산: `mydocs/pr/assets/pr_3130_postmelee_issue2308_p034_review.png`

## 리스크와 권고

revision invalidation 누락은 stale layout/cache로 나타날 수 있으므로 #3125의 resumable path와 결합한
통합 CI가 최종 게이트다. 원 PR이 BEHIND이고 두 PR의 코드가 얽혀 있으므로 원 PR 자체를 update branch해
별도 merge하는 대신 통합 PR의 최신 head를 수용 대상으로 삼는다.

#2308은 후속 리팩터링 범위를 계속 추적하므로 open으로 둔다. 통합 PR merge 후 원 PR close/comment는
상태 확인 및 별도 승인 뒤 수행한다.
