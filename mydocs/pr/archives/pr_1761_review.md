# PR #1761 리뷰 — Task #1759 razor-thin 메트릭 드리프트 측정 하니스

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1761 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1759` |
| 관련 이슈 | #1759, 후속 #1760 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE, mergeStateStatus=BEHIND |
| 누적 검토 순서 | #1761 → #1762 → #1764 |
| reviewer assign | @jangster77 요청 완료 |

## 변경 범위

쪽 경계 razor-thin mismatch 계열을 조사하기 위한 Python 하니스와 조사 문서를 추가한다.

- `tools/metric_drift_survey.py`
  - 한글 PDF/pyhwpx/PyMuPDF 결과와 rhwp SVG/render-tree를 대조하는 조사 도구다.
- `mydocs/plans/task_m100_1759*.md`, `mydocs/report/task_m100_1759_report.md`,
  `mydocs/working/task_m100_1759_stage*.md`
  - 조사 배경, 산출물, 한계, 후속 분류를 기록한다.

Rust 소스 변경은 없다.

## 로컬 검증

최신 `upstream/devel` 기준 누적 브랜치 `local/pr1761-1764-review`에서 #1761 → #1762 → #1764 순서로
cherry-pick 했다.

- `python3 -m py_compile tools/metric_drift_survey.py`
  - 통과
- `cargo fmt --check`
  - 통과
- `git diff --check upstream/devel..HEAD`
  - 통과

GitHub Actions도 PR #1761 최신 head 기준 Build & Test, Render Diff, CodeQL 모두 통과 상태를 확인했다.

## 리스크 / 후속 보완 후보

- `hangul_pdf()`의 캐시 PDF 파일명은 `src.stem[:60] + "_hangul.pdf"`이다. hwpdocs 조사처럼 숫자 ID가
  파일명 앞에 붙은 입력에서는 충돌 가능성이 낮지만, 임의 `--files` / `--list` 입력에서 서로 다른 경로의
  같은 stem 또는 앞 60자가 같은 파일이 섞이면 PDF 캐시를 잘못 재사용할 수 있다.
- 이 PR의 목적은 hwpdocs 1차 서베이 하니스와 결과 기록이고, 현재 조사 표본은 파일명이 고유한 전제라
  merge blocker로 보지는 않는다. 일반화 시에는 파일 경로 hash를 캐시 파일명에 포함하는 후속 보완이 좋다.

## 결론

PR 내용 기준으로 merge 후보로 판단한다. #1762가 #1761의 후속 조사 문서이므로 오래된 순서대로
#1761을 먼저 처리하는 것이 맞다. 캐시 파일명 일반화는 후속 보완 후보로 남긴다.
