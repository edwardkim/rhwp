---
kind: guide
status: active
canonical: mydocs/manual/verification/README.md
last_verified: 2026-07-16
---

# 시각 검증 문서 지도

`mydocs/manual/verification/`은 PDF, SVG, 페이지네이션, 개체 geometry의 시각 검증을 반복 수행하기 위한
문서 묶음이다. 최종 판정 권위는 한컴 편집기와 기준 PDF를 확인하는 작업지시자에게 있으며, 자동 도구는
후보 검출과 무회귀 확인을 보조한다.

| 목적 | 우선 문서 | 보조 문서 |
| --- | --- | --- |
| 시각 검증 적용 판단과 증적 규약 | [시각 검증 거버넌스](visual_verification_governance.md) | [PR 리뷰·통합 워크플로우](../pr_review_workflow.md) |
| PDF/SVG overlay·drift 후보 탐색 | [PDF/SVG visual sweep 가이드](visual_sweep_guide.md) | [SVG 회귀 diff](svg_regression_diff.md) |
| 표·그림 geometry 무회귀 | [개체 시각 회귀 하니스](object_visual_regression.md) | [페이지·PI 매칭 오라클](verify_pi_page_vs_hangul.md) |
| HWPX→HWP 페이지네이션 정합 | [roundtrip fidelity 하니스](roundtrip_fidelity_harness.md) | [한글 페이지 충실도 오라클](hangul_page_oracle.md) |
| 한글 PDF 줄 baseline 대조 | [한글 PDF baseline](hangul_pdf_baseline.md) | [시각 클리핑 검출기](visual_clipping_detector.md) |

## 이전 경로

GitHub issue/PR 외부 이력에서 자주 참조된 기존 경로만 redirect stub으로 남긴다. 각 stub의 `canonical`
메타데이터가 allowlist의 정본이며, 이 지도에 경로 목록을 복제하지 않는다. 저장소 내부 링크는 위의 현재
경로를 사용한다. 문서 구조를 변경할 때는 [문서 링크와 메타데이터 로컬 검사](../markdown_link_check_guide.md)로
redirect 이전 경로 재참조를 확인한다.
