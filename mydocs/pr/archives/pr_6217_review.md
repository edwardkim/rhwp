---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6217 review - #6186 꼬리말 세로 정렬 보존

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6217
- 작성자: `planet6897`
- 원 PR head: `d93e12d2c50d`
- 통합 검토 브랜치: `review/planet6897-6199-6217-20260827`
- 최신 기준: `upstream/devel@9d6f69b4d1a0`
- 검증 실행 기준: `upstream/devel@584320e0ee02`
- 원 PR 상태: non-draft, source CI green, comments/reviews 0건
- 관련 이슈: #6186

## 검토 판단

**수용 권고**. HWPX header/footer subList의 `vertAlign`을 layout에서 읽고 serializer에서 보존한다.
정렬 적용은 text-only header/footer와 신뢰 가능한 `textHeight`가 있는 경우로 좁혀, text box가 섞인
기존 `exam_kor` 계열 fixture의 과보정을 피한다.

## 증적과 검증

- 원 PR 수치 보고서: `mydocs/report/issue-6186-footer-vertalign/README.md`
- 파일 버전 증적: `mydocs/pr/assets/pr_6199_6217_156755659_footer_vertalign_bottom_hwpx_info.json`
- Hancom 기준 PDF: `pdf/pr_6217_issue6186_footer_vertalign_bottom-2020.pdf`
- MCP 증적:
  `mydocs/pr/assets/pr_6217_issue6186_mcp2020_{start,status,download}.json`,
  `mydocs/pr/assets/pr_6217_issue6186_sha256.txt`
- 직접 visual_sweep: `mydocs/pr/assets/pr_6217_issue6186_footer_p2_review.png`,
  `mydocs/pr/assets/pr_6217_issue6186_visual_sweep_summary.json`
  - page 2, pixel match `93.02684%`, ink match `20.36145%`, `flagged_page_count=0`
  - 검토자가 review PNG를 직접 확인했고, 꼬리말이 페이지 하단에 유지되며 본문 및 담당부서 표와 겹치지
    않음을 확인했다.
- focused test: `issue_6186_footer_vertalign_bottom` 2 pass
- 메인터너 보정: serializer가 `vertAlign`을 보존하면서 더 이상 결손으로 남지 않는
  `hwpx ... controls[].list_attr` baseline 8건을 `tests/fixtures/ir_field_sweep_baseline.tsv`에서 제거했다.
  수정 후 dump와 baseline의 sorted diff는 0건이었다.
- 공통 검증: fmt, suite manifest, unit tier, clippy, 전체 nextest, Native Skia 3종, WASM build 통과.
  상세 명령과 숫자는 통합 구현 문서에 기록했다.
- 2026-08-28 최신 `upstream/devel@9d6f69b4d1a0`로 충돌 없이 rebase했다. 사용자 지시에 따라 별도
  중복 테스트는 수행하지 않았다.

## 후속

원 PR의 "export-svg header/footer text 누락" 보고는 이번 layout/serializer 보정과 별도 축이다. 통합 PR
본문에는 수용 가능 판단과 별도 follow-up 가능성을 분리해 적는다.
