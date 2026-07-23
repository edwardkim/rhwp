# M100 #2308 Stage 3 — #2004 이미지 스택 정규화 이전

## 기준

- 브랜치: `issue-2308-hyper-waterfall-rebuild`
- 선행 Stage: `task_m100_2308_stage2.md`
- 완료일: 2026-07-23

## 구현

- `DocumentCore.render_normalized` mutable tuple을 `RenderNormalizationState.sections`의
  `RenderNormalizedSection`으로 교체했다.
- #2004가 발동한 section의 paragraph/composed projection은 immutable `Arc<Vec<_>>`로
  소유한다.
- projection에 source section revision을 기록하고 동일 revision의 반복 정규화에서는 기존
  `Arc`를 재사용한다.
- deferred cell edit은 projection 내부 paragraph를 직접 mirror하지 않는다. logical path
  revision을 올리고, 해당 section에 compatibility projection이 있으면 source IR에서 다시
  파생한다.
- section/path 불일치는 조용히 stale state를 사용하지 않고 `RenderError`로 표면화한다.

#2004 셀 이미지 스택은 셀 문단 cardinality, 합성 `LINE_SEG`, `ComposedParagraph`를 함께 바꾸는
구조 projection이므로 scalar overlay로 분해하지 않았다. 이슈가 허용한 revision 기반 derived
cache로 한정했으며 editable `Document` IR은 계속 단일 권위 상태다.

Stage 3 중간 상태에서는 #2195 기존 동작을 보존하기 위해 같은 immutable projection을 이용해
nested-table width clone 변환을 유지한다. 이 임시 경로는 Stage 4에서 sparse overlay로 제거한다.

## 검증

| 명령 | 결과 |
| --- | --- |
| `cargo test --lib issue_2308_stable_compat_projection_reuses_arc_identity` | 1 passed |
| `cargo test --test issue_2004_cell_image_stack_pagination` | 2 passed |
| `cargo fmt --all` | PASS |

검증된 계약:

- stable section revision에서 #2004 projection `Arc` 재사용
- HWP/HWPX #2004 fixture 모두 8쪽 유지
- 원본 IR 직접 mirror 없이 source 기반 재파생
