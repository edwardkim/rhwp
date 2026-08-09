---
kind: pr_review
status: maintainer-review
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4390 검토 — 실행 가능한 에이전트 하네스 스코어카드

## 결론

**메인터너 보정 뒤 조건부 수용 권고.** contributor 변경은 문서에 적은 하네스 성질 여섯 가지를
`tools/harness_proofs.py` 한 명령으로 실행해 PASS/FAIL로 판정하는 공개 스코어카드를 추가한다.
검토에서 P2의 문서 계약은 정확히 68개 명령인데 실행 판정은 50개 이상만 요구해, 최대 18개 명령이
사라져도 거짓 PASS가 되는 차단점을 확인했다. 원 contributor history 위의 별도 보정 commit으로
정확한 68개 계약과 focused 회귀 테스트를 추가했다.

local candidate는 검증을 통과했지만 아직 원격에 push되지 않았다. Python 실행 코드와 test가
바뀌었으므로 review-only fast-pass를 적용하지 않는다. 최신 candidate의 GitHub checks,
mergeability 재확인과 작업지시자의 push·review·merge 승인이 최종 수용 조건이다.

## 메타데이터

| 항목 | 2026-08-10 검토 시점 참고값 |
| --- | --- |
| PR | [#4390](https://github.com/edwardkim/rhwp/pull/4390) |
| 관련 이슈 | [#4389](https://github.com/edwardkim/rhwp/issues/4389) |
| 작성자 | `kevin9327` |
| base / draft | `devel` / 아님 |
| contributor source head | `02ce6d0b3840acb5c1b2569ee9154dc32832c789` |
| source 규모 | 2 files, +221 / -0, 2 commits |
| source merge 상태 | `MERGEABLE`, `CLEAN`; merge 전 최신 상태 재확인 필요 |
| source checks | source head 기준 required checks 성공; local correction에는 원격 CI 없음 |
| 가시성 branch | `review/kevin9327-20260810-pr4390` |
| local code candidate | `48a80dc09a9a876165292777546ec124f195a82a` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |

기준 devel은 contributor source head의 조상이고, 가시성 branch는 source 뒤에 메인터너 commit만
선형으로 추가했다. contributor commit을 amend·rebase하지 않았다.

## contributor 변경 범위

- `mydocs/tech/agent_roadmap/harness_scorecard.md`에 실행 가능한 성질 6종과 계약·PR로 검증되는
  성질 4종을 구분해 기록했다.
- `tools/harness_proofs.py`가 capabilities 결정론, 명령 표면, 오류 exit/stdout, 신뢰 경계,
  explain 결정론을 실제 CLI로 실행하고 표 또는 JSON 결과를 출력한다.
- 후속 문서 commit은 미병합 문서 상대 링크를 GitHub PR 참조로 바꿨다.

변경은 Python 검증 도구와 문서뿐이다. Rust renderer, layout, paint, WASM 출력, sample·golden은
바뀌지 않으므로 시각 sweep 대상이 아니다.

## 발견한 차단점과 메인터너 보정

문서는 P2를 "68개 명령의 계약 전수 자기서술"로 고정했지만 원 runner는
`n_cmd >= 50`만 확인했다. 명령이 68개에서 50개까지 줄어도 PASS가 되므로 스코어카드가 주장하는
전수 검증을 수행하지 못했다. 기존 detail도 두 계약 필드의 개별 상태가 아니라 전체 `ok`만
출력해 실패 원인을 구분하기 어려웠다.

메인터너 보정 `48a80dc09a9a876165292777546ec124f195a82a`은 다음을 추가했다.

- `EXPECTED_COMMAND_COUNT = 68`과 `command_surface_contract()`로 정확한 명령 수를 판정한다.
- detail에 실제·기대 명령 수와 `exitCodes`, `jsonContract` 존재 여부를 각각 출력한다.
- `tools/test_harness_proofs.py`에서 68개 성공, 67·69개 실패, 계약 필드 누락 실패를 고정한다.

실행 단계와 rollback 경계는 [구현·검토 계획](pr_4390_review_impl.md)에 기록했다.

## 완료한 로컬 검증

| 명령 | 결과 |
| --- | --- |
| `python -m unittest tools.test_harness_proofs -v` | 3 / 3 통과 |
| `python -m py_compile tools/harness_proofs.py tools/test_harness_proofs.py` | 통과 |
| `python tools/harness_proofs.py --json` | 6 / 6 통과 |
| P2 실물 detail | `commands=68 (expected=68), exitCodes=True, jsonContract=True` |
| `git diff --check origin/pr/4390..48a80dc0` | 통과 |

실물 proof에는 같은 검토 사이클에서 새로 빌드한 #4330 debug `rhwp`를 지정했다. #4390에는 Rust
변경이 없고 해당 binary가 실제 68-command capabilities를 제공해 수정된 runner의 green 경로와
나머지 다섯 proof를 검증했다. exact #4390 checkout에서 Rust binary를 별도로 재빌드하지 않았다.

## 잔여 위험과 최종 조건

- local Python·test·review commit에는 원격 CI가 없다. 최신 head의 required checks가 새로 성공해야 한다.
- 추가한 `tools/test_harness_proofs.py`는 focused 명령으로 검증했지만 기존 전체 CI가 모든 `tools/test_*.py`
  를 자동 discover한다고 가정하지 않는다. merge 전 CI log와 focused 명령 결과를 함께 확인한다.
- exact #4390 checkout binary를 별도로 빌드하지 않은 범위는 source에 Rust diff가 없다는 사실과 실제
  68-command binary proof로 제한했다. 이후 source에 Rust 변경이 추가되면 proof를 새 head binary로
  다시 실행해야 한다.
- push 직전 원 PR source head와 local branch의 시작 SHA가 여전히 같은지 재확인해야 한다.
- GitHub push, review/comment, merge 권한은 부여되지 않았다.

위 조건과 작업지시자 승인을 충족하면 merge를 권고한다. 하나라도 충족하지 않으면 보류한다.
