# PR #1776 리뷰 — Task #1769 per-line 측정 누적 가설 기각 조사

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1776 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1769` |
| 관련 이슈 | #1769, refs #1759, #1765 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE, mergeStateStatus=CLEAN |
| head 참고값 | `37cc623bf00573162069f234f01b77e474e07ab5` |
| reviewer assign | @jangster77 요청 완료 |
| 메인터너 보정 | #1815 merge 후 update branch 로 문서-only fast-pass 검증 |

## 변경 범위

소스 변경 없이 #1769 조사 결론을 문서화하는 PR이다.

- `mydocs/plans/task_m100_1769.md`
- `mydocs/report/task_m100_1769_report.md`

PR head 에는 `devel` merge commit 이 포함되어 있지만, #1815 이후 preflight 는 PR 의 최종 파일 목록 기준으로
review-only 여부를 먼저 판정한다. #1776 의 최종 PR diff 는 위 `mydocs/` 문서 2건뿐이다.

## PR 내용 검토

이 PR 의 핵심 결론은 #1765 에서 재분류된 "per-line 콘텐츠 측정 누적(+0.1px/줄)" 가설을 실제 줄 단위
대조로 기각하는 것이다.

문서의 결론은 다음 흐름으로 일관된다.

- row9 c2 셀은 실제 136줄 콘텐츠이고, 저장 LINE_SEG 의 `vpos` 리셋 2회는 원 문서의 행 내부 분할을 의미한다.
- rhwp p2 fragment 50줄 advance 와 저장 LINE_SEG advance 를 1:1로 대조한 결과 누적차가 `+0.0px` 로 일치한다.
- 따라서 per-line 줄 메트릭은 건강하며, 잔여 약 5px 차이는 행 내부 분할 컷(fragment) 경계 미세 오프셋으로 재분류한다.
- 해당 차이는 페이지/PI 판정에는 영향을 주지 않는 저우선 시각 차로 기록한다.

계획서와 최종 보고서 모두 "per-line 가설 기각"과 "소스 무변경 조사 종결"이라는 같은 결론을 가리킨다.
초기 가설이 확정 사실처럼 남아 있는 문구는 확인되지 않았다.

## 로컬 검증

- `git diff --name-status upstream/devel...HEAD`
  - `mydocs/plans/task_m100_1769.md`
  - `mydocs/report/task_m100_1769_report.md`
- `git diff --check upstream/devel...HEAD`
  - 통과

소스 변경이 없으므로 cargo build/test/clippy 는 수행하지 않는다.

## CI / fast-pass 확인

#1815 를 먼저 merge 한 뒤 #1776 에 `Update branch` 를 적용했다. 새 head
`37cc623bf00573162069f234f01b77e474e07ab5` 기준으로 PR 최종 diff 가 문서-only 로 판정되어 heavy job 이
skip 되는 것을 확인했다.

- CI preflight: `fast_pass=true`, `reason=all-review-only-no-code-impact`
- CodeQL preflight: `fast_pass=true`, `reason=all-review-only-no-code-impact`
- Render Diff preflight: `fast_pass=true`, `reason=all-review-only-no-render-impact`
- `Build & Test`: skipped
- `Canvas visual diff`: skipped
- CodeQL analyze: skipped

## 시각 검증

이 PR 은 렌더링 코드, 샘플, 기준 PDF, visual regression 자료를 새로 추가하지 않는 조사 문서 PR이다.
사용자-visible 렌더링 동작을 바꾸지 않으므로 visual sweep 은 수행하지 않는다.

## 결론

PR 내용은 #1769 의 조사 결론을 일관되게 문서화하며, 소스 변경이 없어 문서-only fast-pass 대상이다.
review 문서 remote push 후 최신 head 기준 preflight 가 다시 통과하면 merge 후보로 판단한다.
