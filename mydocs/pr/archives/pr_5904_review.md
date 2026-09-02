---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5904 검토 - nested table partial tail row

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5904](https://github.com/edwardkim/rhwp/pull/5904) / `@kevin9327` |
| 관련 issue | closes #5846 |
| source head | `9d1919a27aa9da9f5d35542054f0ad6359bc44bb` |
| 상태 | non-draft, `CLEAN`, source CI 완료 |
| 적용 commit | `2ea709c8ac` |

## 검토와 시각 증적

- 비종료 조각에서 일부만 보이는 중첩 표의 꼬리 행을 다음 조각으로 넘긴다. 온전한 행이 하나
  남는 경우만 적용해 무한 이월을 막고, `terminal`ㆍ`recursive_cut` 경로와 page flow 회계는 유지한다.
- `issue_5846_mixed_nested_partial_row_duplicate`는 59쪽 body의 용지 밖 TextRun이 없고,
  `단기연체정보`가 60쪽에만 존재하는지 검사한다. 통합 focused test 1건, 전체 nextest와 Native
  Skia를 통과했다.
- 59쪽 비교에서 본문 하한 초과 `<text>`가 549개에서 정상 꼬리말 4개로 줄고, 66쪽 page count는
  유지됐다. 사람 검토에서 수정 후 상자가 정본처럼 `채무불이행정보` 표에서 닫히는 것을 확인했다.
- 보존 asset: `mydocs/pr/assets/pr_5904_issue5846_p59_review.png`
  (`sha256:a18a2ea09840845b024ed6f00adae258587c9b1915c13333071520bfb4c513e1`).
  원본 정본과 재현 절차는 `mydocs/report/task_m100_5846_report.md`에 기록돼 있다.

## 판정

**통합 후보 수용.** 중복 렌더만 제거하고 다음 쪽의 원본 내용은 보존한다.
