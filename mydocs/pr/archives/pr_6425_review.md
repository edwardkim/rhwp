---
kind: pr-review
status: accepted-with-evidence-limit
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-30
pr: 6425
issue: 6310
author: kevin9327
---

# PR #6425 review - imgBrush ZOOM 종횡비 유지

## Metadata

| 항목 | 확인값 |
| --- | --- |
| 원 PR head / 통합 적용 | `fd7680dfe6eeadb741face3efca4f35443bda78a` / `480614f` |
| 규모 | 11 files, `+179/-2`, 1 commit |
| 작성 시점 원 PR 상태 | Open, non-draft, check `SUCCESS`, `mergeStateStatus=UNKNOWN` |

## 검토와 증적 한계

- parser, model, SVG, CanvasKit, web canvas, serializer가 `hc:imgBrush mode=\"ZOOM\"`을 `ImageFillMode::Zoom`으로 보존하고 SVG의 `preserveAspectRatio=\"xMidYMid meet\"`로 내보내도록 정렬한다.
- `issue_6310_imgbrush_zoom_cell`은 기존 seed HWPX를 메모리에서 변형해 1×1 PNG와 ZOOM borderFill을 삽입한다. parser enum과 native SVG `meet` contract는 full nextest에서 통과했다.
- `samples/tac-host-spacing.hwpx`와 canonical PDF는 존재하지만, 둘은 test가 합성한 ZOOM 입력이 아니다. 그 기존 쌍을 sweep해도 수정 경로를 검증하지 못하므로 가짜 시각 증적을 만들지 않았다. PR에 장기 보존된 정확한 ZOOM HWPX/PDF fixture도 추가되지 않았다.
- 원 PR comment는 자동 quota 안내뿐이다.

## 판단

**증적 한계가 있는 수용.** 합성 input의 parser/SVG 계약은 직접 검증됐고, 현 PR은 실제 사용자 문서의 before/after fidelity를 주장하지 않는다. 실제 ZOOM 문서가 후속으로 추가되면 해당 원본과 canonical PDF로 visual sweep을 별도 수행해야 한다.
