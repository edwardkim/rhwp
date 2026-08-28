---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6259 review - #6167 TAC 표 자기 줄 leading 제거

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6259
- 작성자: `planet6897`
- 원 PR head: `87447c260737`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6167

## 검토 판단

**수용 권고.** 저장 `linesegarray`가 자리차지 표에 자기 줄을 이미 부여한 경우, 앞 줄 공백 폭을 표
좌표 leading에 다시 싣지 않도록 좁힌다. `text_start == ctrl_pos`와 `column_start == 0` 조건으로
종전 leading 보정이 필요한 통제군과 분리한 점이 적절하다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/leading-space-tac-table-6167/{p38_table_before,p38_table_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6167_info.json`
  - 저장 제품: `hancom-office-2024 13.0.0.1053` -> MCP `engine 2024`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6167_leading_space_tac_table-2024.pdf`
    (1 page)
  - visual sweep: `pr6275-issue6167-p1`, slice p1(원 PR 설명의 p38 축약 샘플), flagged 0,
    pixel match `96.11433%`, visual accuracy proxy `38.92434%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6167_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6167_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6167_visual_overlay_metrics.json`
- 초기 확인 때 stale `target/pr-review/release/rhwp`를 사용한 결과는 폐기했다. 최종 증적은
  2026-08-28 14:25 빌드된 `target/pr-review/release-test/rhwp`와 `hancom-office-2024` 기준
  PDF로 다시 산출했다.
- 검토자가 직접 확인한 대표 after: before에서 오른쪽으로 밀려 용지 밖에 걸리던 표가 after에서 본문
  좌단 기준으로 배치되고 오른쪽 열이 잘리지 않는다. 통합 head visual sweep review PNG에서도 TAC 표가
  오른쪽으로 밀려 잘리는 원 결함은 재현되지 않았다.
- focused test: `issue_6167_leading_space_tac_table_own_line` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p38_table_after.png`에서 표 좌단과 우측 열 잘림이 보정된 직접
확인 결과를 적는다. `pr_6275_issue6167_visual_review_p1.png`를 merge commit SHA 고정 raw URL로
표시하고, "내용 픽셀 중심 자동 일치율 보조값 = 약 38.92%" 및 자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
