---
kind: pr-review
status: approved-integration-candidate
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5824 검토 - HWPX 자기닫힘 도형 자식

## 판정

- source head `46d2d521981438469ccaea210595b24b2d2d2e95`를 적용했다.
- 자기닫힘 요소가 뒤 글상자·도형 형제를 삼키지 않도록 파서를 보정한다. 차단 결함은 없다.

## 검증

- `issue_5797_hwpx_shape_self_closing_child` 5/5, 통합 전체 nextest 8,109/8,109 통과.
- 같은 통합 후보의 #5787/#5788 관련 HWP 2020 기준도 확인했다. PDF는 각각 `nested_square_table_horz_offset-2020.pdf`(job `ad7aa2d5-1741-46ad-96cf-faa517efdc76`, SHA-256 `4acecce15598ac9cc962d01a0c7e0c6b1415f49fb0f17d922df6e261694b0860`) 및 `tac_table_missing_anchor_line_spacing-2020.pdf`(job `dbb4d6ec-0f4c-4b55-8328-8479172028bb`, SHA-256 `911122f759d1962246e258d707c0c8820f3710155f4212da1fa3b053747e20d1`)다.
- 대표 검토 이미지는 `mydocs/pr/assets/pr_5824_issue5787_review_001.png`, `mydocs/pr/assets/pr_5824_issue5788_review_001.png`에 보존했다.
