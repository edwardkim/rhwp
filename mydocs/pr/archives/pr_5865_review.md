---
kind: pr-review
status: review-complete-pending-trailing-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5865 검토 - 다음 페이지 표 조각 residue 억제

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5865](https://github.com/edwardkim/rhwp/pull/5865) / [@kevin9327](https://github.com/kevin9327) |
| 관련 issue | [#5863](https://github.com/edwardkim/rhwp/issues/5863) |
| base / source head | `devel` / `609aca5b401451a91dcd6ed3b6a628d885b9455c` |
| 변경 규모 | 6 files, +261 / -12 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `CLEAN`, maintainerCanModify=true |
| 통합 반영 | `d912b4229` (`review/open-prs-20260822`) |

## 범위와 메인터너 보정

- 현재 쪽 cell clip 아래에서 시작한 nested table은 다음 쪽 render가 소유하므로 현재 쪽 SVG에서 억제한다.
- source의 owning-page 회귀는 억제가 보이는 표 내용을 지우지 않음을 보장한다.
- 통합 검토에서 `hwpctl_ParameterSetID_Item_v1.2.hwp` overflow 관측이 29에서 8로 실제 감소했다. 메인터너 보정 `4b28259bb`은 실측 dump 재검증 뒤 baseline을 8로 강화했다.

## 검증과 시각 범위

- 통합 candidate `4b28259bb`에서 전체 nextest **8,160 passed, 39 skipped**, overflow baseline 780 samples/571 rows의 exact diff, clippy, native-Skia, WASM build를 통과했다.
- 같은 head의 GitHub Build & Test, archive build/shard, Lint, Native Skia, CodeQL Rust/JavaScript/Python, Canvas visual diff, Adapter inter-diff, Proptest roundtrip도 성공했다. WASM과 frontend unit은 변경 범위 정책에 따른 정상 skip이다.
- `samples/hwpx_sample2.hwp`와 `pdf/hwpx_sample2-2024.pdf`의 p8-9 sweep은 `output/visual_open_prs_20260822_5865_a`에 보존했다.
- 대표 asset은 `mydocs/pr/assets/pr_5865_issue5863_p008_review.png`(SHA-256 `bc4b4c4707bfa5c738597fb572ed0d13442daef08b6c52924d072017dbc2f61f`)다.
- source 주장처럼 raster는 전후 동일할 수 있고, 변경 대상은 보이지 않는 SVG DOM residue다. sweep의 전체 페이지 후보는 기존 cell height/clip fidelity 차이이며 이 PR의 해결 완료로 과장하지 않는다.

## 최종 판정

**수용 권고.** current-page residue만 제거하고 owning page의 table draw를 가드한다. merge 전 PR #5889 최신 head CI와 작업지시자 승인을 확인한다.
