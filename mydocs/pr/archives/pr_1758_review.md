# PR #1758 리뷰 — verify_pi_page_vs_hangul 알려진 한계 문서화

## PR 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1758 |
| 작성자 | @planet6897 |
| base / head | `devel` / `planet6897:pr/devel-1757` |
| 관련 이슈 | #1757 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE, mergeStateStatus=BLOCKED |
| maintainer 수정 | `maintainerCanModify=true` |

## 변경 범위

PR은 `verify_pi_page_vs_hangul.py`의 알려진 한계를 문서화한다.

- `tools/verify_pi_page_vs_hangul.py`
  - 모듈 docstring에 캐럿-개체 분리 오탐 2유형을 추가한다.
  - 실행 로직 변경은 없다.
- `mydocs/manual/verify_pi_page_vs_hangul.md`
  - 도구 사용법, 판정 의미, 캐럿-개체 분리 오탐 판별 절차를 문서화한다.
- `mydocs/plans/task_m100_1757.md`
  - 작업 배경과 게이트를 기록한다.

## 로컬 검증

Rust 렌더링 코드나 workflow 변경이 아니므로 cargo full 검증은 수행하지 않았다.

- `python3 -m py_compile tools/verify_pi_page_vs_hangul.py`
  - 통과
- `python3 tools/verify_pi_page_vs_hangul.py --help`
  - argparse help 출력 확인
- `git diff --check upstream/devel...HEAD`
  - 통과

## CI 처리 판단

파일 범위에 `tools/verify_pi_page_vs_hangul.py`가 포함되므로 GitHub preflight는 코드 영향으로 보고 full CI를
시작할 수 있다. 다만 실제 diff는 docstring과 문서 추가이며 실행 로직은 바뀌지 않는다.

작업지시자 지시에 따라 full CI 완료를 기다리지 않고, PR review 문서와 오늘할일 기록을 PR head에
remote push한다.

## 결론

PR 내용 기준으로는 merge 후보로 판단한다. 다만 최종 merge 전에는 GitHub branch protection 상태와
최신 head 상태를 별도로 확인한다.
