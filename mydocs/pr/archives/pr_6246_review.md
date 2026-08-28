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
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6186

## 검토 판단

**수용 권고.** 저장된 꼬리말 band 안에서 쪽번호 문단의 세로 위치가 위로 붙는 문제를, 저장
`vAlign`/band height를 반영하는 방향으로 고친다. HWPX roundtrip 후에도 꼬리말 수직 정렬이 유지되는
계약 test가 함께 들어와 회귀 방어가 가능하다.

검토 중 원 PR head가 `89eeca512dcf`에서 `37abb2599dca`로 force-push 되었다. 통합 브랜치에는 이전
head의 stale helper 형태가 남아 있었으므로, 최신 head의 inline match 구조와 최종 diff가 일치하도록
메인터너 정렬 커밋을 추가했다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/footer-band-valign-6186/{before_p2,after_p2,oracle_p2}.png`
- 검토자가 직접 확인한 대표 after/oracle: 꼬리말 쪽번호 `2 - 2`가 band 안에서 한컴 oracle과 같은
  아래쪽 정렬 위치로 내려온다.
- focused test: `issue_6186_footer_page_number_sits_on_the_textbox_line`,
  `issue_6186_footer_vert_align_survives_hwpx_roundtrip` 2 pass
- 최신 head 정렬 후 focused 재검증: 2 pass / 8,527 skipped
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 force-push 이후 최신 head와 통합 branch의 최종 diff를 맞췄다는 점,
그리고 `after_p2.png`/`oracle_p2.png` 직접 확인 결과를 적는다. 추가 visual sweep이 필요하면
`mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment`에 따라 대표 `review_*.png`를
`mydocs/pr/assets`에 보존하고, merge commit SHA 고정 raw URL로 댓글에 표시한다.

## 후속

추가 보정 필요 없음. 단, 이후 #6246 원 PR에 새 push가 생기면 이번 통합 범위에 자동 포함하지 않고
별도 승인 후 재검토한다.
