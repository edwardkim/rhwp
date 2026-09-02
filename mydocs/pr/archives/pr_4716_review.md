---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4716 검토 - AWS 작업 표준 척추 표면 검증 보강

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4716](https://github.com/edwardkim/rhwp/pull/4716) |
| 작성자 / 원 head | @kevin9327 / `a81bb8ff0b` |
| 검토 적용 commit | `81ab1910e4` |
| 메인터너 보정 | `2fd830c606` `fix(tools): AWS 척추 표면 검증 보강` |
| 통합 PR | [#4722](https://github.com/edwardkim/rhwp/pull/4722) |
| base / code candidate | `devel` / `2fd830c606` |

## 범위와 보정

원 PR은 AWS/1.0 작업 표준과 관련 로드맵·공원·에이전트 진입점 사이의 연결을 선언하고,
`tools/adoption_spine.py`가 선언한 표면을 점검하도록 추가한다.

검토에서 두 결함을 확인해 통합 브랜치에서 보정했다.

- 표준 문서의 frontmatter `kind: standard`는 현재 메타데이터 허용값이 아니었다. 표준의 참조 문서
  성격에 맞춰 `kind: reference`로 고쳤다.
- 기존 척추 가드는 표면 파일의 존재만 확인해 `gym/PARK.md`의 표준 링크나 `AGENTS.md`의
  anchor가 빠져도 통과할 수 있었다. Markdown 링크 대상과 heading anchor를 실제로 검증하고,
  두 회귀 단위 검증을 더했다.

## 완료한 검증

- `python -m py_compile tools\adoption_spine.py scripts\tests\test_adoption_spine.py`
- `python tools\adoption_spine.py --json`
- `python -m unittest scripts\tests\test_adoption_spine.py` — 6 passed
- `python -m unittest scripts\tests\test_ci_impact_workflow.py` — 27 passed
- `python -m unittest scripts\tests\test_workflow_contract_wiring.py` — 3 passed
- `git diff --check`, 문서 메타데이터 검사(549건), 변경 Markdown 링크 검사
- code candidate `2fd830c606`의 GitHub Build & Test, Native Skia, test shard, lint 및
  CodeQL 전체 녹색

도구·문서·CI wiring만 바꾸며 renderer·WASM·fixture를 바꾸지 않았으므로 visual sweep은 적용 대상이
아니다. 표준 허브 PNG는 별도로 열어 읽을 수 있고 잘림이 없음을 확인했다. Cargo 검증은 변경 범위에
필요하지 않아 실행하지 않았다.

## 판정

**self-review 수용.** 사용자 지시에 따라 외부 reviewer를 요청하지 않는다. 이 문서와 구현 기록을
code candidate 뒤 trailing commit으로 올린 뒤에는 최신 head의 fast-pass와 현재 mergeability를 다시
확인하고 merge한다.
