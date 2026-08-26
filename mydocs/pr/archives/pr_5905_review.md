---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5905 검토 - justify footer page number

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5905](https://github.com/edwardkim/rhwp/pull/5905) / `@kevin9327` |
| 관련 issue | closes #5899 |
| source head | `2a96dc9071de117339788624e5661fad76d4ece2` |
| 상태 | non-draft, `CLEAN`, source CI 완료 |
| 적용 commit | `873f1043be` |

## 검토와 시각 증적

- 양쪽정렬 슬랙 분배의 공백 수를 모델 placeholder가 아니라 그려지는 `display_text` 기준으로
  세어, 꼬리말 쪽번호가 수만 px 밖으로 밀리던 분모 불일치를 고친다. 일반 run은 같은 text를
  반환하므로 동작 범위를 필드 치환 문단으로 제한한다.
- `issue_5899_footer_page_number_justify` 2건은 1ㆍ115ㆍ150쪽에서 종이 밖 glyph가 없고
  쪽번호가 오른쪽 여백에 놓이는지를 검사한다. 통합 focused test, 전체 nextest, Native Skia를
  통과했다.
- 151쪽 sweep은 종이 밖 글자 150쪽에서 0쪽으로 감소했고, p116의 쪽번호 위치는 한글 2020 정본과
  0.58pt 차이다. 사람 검토에서도 수정 후 footer가 정본의 오른쪽 끝과 일치한다.
- 보존 asset: `mydocs/pr/assets/pr_5905_issue5899_p116_review.png`
  (`sha256:a2259eeb98a7a643ba2d05ebd8299084b954ebb5ab233d57440d1b9637385547`).
  원본 정본과 재현 절차는 `mydocs/report/task_m100_5899_report.md`에 기록돼 있다.

## 판정

**통합 후보 수용.** 필드가 없는 일반 문단에 대한 코퍼스 SVG self-diff도 0으로 기록돼 있다.
