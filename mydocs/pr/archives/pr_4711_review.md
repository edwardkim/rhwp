---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4711 검토 - 3-sum 한글 행높이 오라클 Windows 보정

| 항목 | 기록 |
| --- | --- |
| PR | [#4711](https://github.com/edwardkim/rhwp/pull/4711) |
| 작성자 / 원 head | @planet6897 / `1ed3db584e` |
| 메인터너 보정 | `613e427f5` `fix(tools): 행높이 복원 합계 gate 보정 (#4711)` |
| base / 상태 | `devel`, 작성 시점 `MERGEABLE`·`CLEAN` |
| code candidate | `613e427f5` — Build & Test 및 CodeQL 녹색 |

## 범위와 보정

원 PR은 Windows+한컴 COM으로 `TABLE_DRIFT` 행높이를 대조하는 도구와 실행 기록을 추가한다.
Windows 재검증에서 C(21761835)는 78/78행·296/296셀을 훑은 뒤에도 합계가 `−706.89px`로
벌어졌지만, 도구가 성공 코드 0을 반환했다.

메인터너 보정은 기본 비교 모드의 수치 관측을 유지하면서, `--max-total-diff-px`를 지정한 호출에서
허용치를 넘으면 종료 코드 3을 반환하게 했다. C 러너북은 100px gate를 사용해 측정 불가를
자동화한다. B 전수 비교처럼 행높이 차 자체를 관측해야 하는 호출은 gate를 지정하지 않는다.

Office 2022 COM 패치 `[12,0,0,4605]`에서는 A와 D는 원 기록과 일치했지만 B와 C 절대값이
`[12,0,0,535]` 원 실측과 달랐다. 실행 문서와 결과 문서에 실제 `hwp.Version`을 기록하도록
보완했다.

## 완료한 검증

- `python -m py_compile tools\hangul_row_heights.py tools\test_hangul_row_heights.py`
- `python tools\test_hangul_row_heights.py` — 1 passed
- Hancom Office 2022 COM + `target\pr-review\release-test\rhwp.exe`:
  - A (`--pi 0 --max-total-diff-px 1`): 11/11셀, 6/6행, `−0.02px`, 종료 0
  - C (`--pi 4 --max-total-diff-px 100`): 296/296셀, 78/78행, `−706.89px`, 종료 3
- 이전 Windows 전수 확인: B 85개 표를 실행했고, D는 399/399셀·75/75행·`+8.68px`를 재현했다.
- `git diff --check` 및 최신 `upstream/devel`과의 merge tree가 통과했다.
- code candidate `613e427f5`의 GitHub Build & Test, Native Skia, test shard, lint 및 CodeQL이 모두 녹색이다.

renderer·WASM·fixture는 변경하지 않았으므로 별도 visual sweep은 적용 대상이 아니다. 한컴 COM
측정은 이 PR의 사용자-visible 렌더 결과가 아니라 진단 도구의 Windows 실행 계약을 검증한 것이다.

## 판정

**self-review 수용 권고.** 사용자 지시에 따라 별도 reviewer 요청은 하지 않았고 GitHub review/comment도
아직 게시하지 않았다. 실제 merge 전에는 최신 head·aggregate·mergeability와 작업지시자 승인을 다시 확인한다.
