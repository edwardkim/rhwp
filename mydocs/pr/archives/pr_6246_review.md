---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6246 review - #6186 꼬리말 세로 정렬과 HWPX 왕복 보존

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6246
- 작성자: `planet6897`
- 원 PR head: `37abb2599dca`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6186

## 검토 판단

**수용 권고.** 저장된 꼬리말 band 안에서 쪽번호 문단의 세로 위치가 위로 붙는 문제를, 저장
`vAlign`/band height를 반영하는 방향으로 고친다. HWPX roundtrip 후에도 꼬리말 수직 정렬이 유지되는
계약 test가 함께 들어와 회귀 방어가 가능하다.

검토 중 원 PR head가 `89eeca512dcf`에서 `37abb2599dca`로 force-push 되었다. 통합 브랜치에는 이전
head의 stale helper 형태가 남아 있었으므로 메인터너 정렬 커밋으로 제거했다. force-push 이력 때문에
`git log --cherry-pick --right-only`에는 최신 직렬화 commit 1개가 남지만, 해당 commit의 test/golden
파일은 통합 head와 차이가 없고 focused #6186 2건을 다시 통과했다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/footer-band-valign-6186/{before_p2,after_p2,oracle_p2}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6186_info.json`
  - 저장 제품: `hancom-office-2018 10.0.0.12409` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6186_defense_press_release-2020.pdf`
    (2 pages)
  - visual sweep: `pr6275-issue6186-p2`, p2, flagged 0, pixel match `93.02684%`,
    visual accuracy proxy `20.36145%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6186_visual_review_p2.png`,
    `mydocs/pr/assets/pr_6275_issue6186_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6186_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after/oracle: 꼬리말 쪽번호 `2 - 2`가 band 안에서 한컴 oracle과 같은
  아래쪽 정렬 위치로 내려온다. 통합 head visual sweep review PNG에서도 꼬리말 쪽번호가 별도
  윗줄로 갈라지는 원 결함은 재현되지 않았다.
- focused test: `issue_6186_footer_page_number_sits_on_the_textbox_line`,
  `issue_6186_footer_vert_align_survives_hwpx_roundtrip` 2 pass
- 최신 head 정렬 후 focused 재검증: 2 pass / 8,527 skipped
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 force-push 이후 stale helper 흔적을 제거했고 최신 직렬화 commit의
test/golden 산출물과 focused 검증을 확인했다는 점을 적는다. 또한
`pr_6275_issue6186_visual_review_p2.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 20.36%" 및 자동값의 한계를 함께 남긴다.

## 후속

추가 보정 필요 없음. 단, 이후 #6246 원 PR에 새 push가 생기면 이번 통합 범위에 자동 포함하지 않고
별도 승인 후 재검토한다.
