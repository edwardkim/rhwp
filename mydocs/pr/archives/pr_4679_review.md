---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-12
---

# PR #4679 검토 - load/save 한글 오라클 전수검사 하네스

## 접수

| 항목 | 기록 |
| --- | --- |
| PR | [#4679](https://github.com/edwardkim/rhwp/pull/4679) |
| 작성자 / source | @planet6897 / `feat/loadsave-sweep-harness` |
| base / source head | `devel` / `ed94b6da8fd53d6f4f3fe9bab01186a199fc5fdc` |
| 규모 | 8 files, +1,111 / -0, 1 commit |
| reviewer | @jangster77 지정 완료 |
| mergeable 참고값 | 작성 시점 `MERGEABLE` / `CLEAN` |
| 관련 이슈 | [#4678](https://github.com/edwardkim/rhwp/issues/4678) 참조 |
| 통합 검토 branch | `review/planet6897-20260812-r2` |

HWP/HWPX 입력의 h2h, h2x, x2h, x2x 저장 경로를 생성하고 한글 COM 오라클로
텍스트·컨트롤·페이지·열기 결과를 비교하는 독립 도구를 추가한다. 제품 Rust 코드나 renderer는
바꾸지 않아 전체 fidelity PDF 대조는 적용 대상이 아니다.

## 메인터너 보정

README가 특정 PC의 드라이브, 코퍼스, 저장소, Python 설치 경로를 전제해 다른 사용자가 그대로
실행할 수 없었다. `be02afb64`에서 다음을 일반화했다.

- repository root, tool, input corpus, sweep output, rhwp executable을 명시 변수로 분리했다.
- Python Launcher `py -3.12`와 사용자 지정 placeholder 경로를 사용하도록 바꿨다.
- COM 기본 버전 복원도 repository 변수에서 찾도록 통일했다.
- `make_lists.py` 도움말의 특정 코퍼스 경로도 일반 placeholder로 교체했다.

## 완료한 검증

- `python3 -m py_compile tools/loadsave_sweep/*.py`와 다섯 Python CLI의 `--help`를
  실행해 통과했다.
- Windows `win10-ted`에서 source를 checkout하지 않고 `upstream/pr4679-head` ref의
  `oracle_run.ps1` 내용을 PowerShell `Parser::ParseInput`으로 검사해 통과했다.
- README와 Python 도움말에서 개인 홈, 특정 `D:\\rhwp`, 특정 코퍼스, 고정 Python 설치
  경로가 남지 않았음을 검색으로 확인했다.
- 통합 candidate의 전체 Rust nextest, fmt, clippy, WASM build도 통과했다. 이 도구 PR의
  Rust 변경은 없지만 누적 candidate의 #4681 serializer 변경을 함께 검증한 결과다.

10,000건 COM 오라클 전수 실행은 한글 설치·코퍼스·장시간 단일 워커가 필요한 환경 의존
검증이므로 이번 Linux review host에서 다시 실행하지 않았다. contributor가 기록한 전수 측정은
merge 전 최신 source와 GitHub Actions 상태를 재확인할 때 참고값으로만 사용한다.

## 판단

**통합 수용 권고.** 검토 branch의 일반화 보정을 포함한 통합 PR에 넣는다. 최신 code head의
GitHub Actions와 작업지시자 승인을 확인한 뒤 병합하며, 그 뒤 원 PR은 통합 반영 사실을
코멘트로 남기고 close한다. #4678의 2단계 확대와 실제 코퍼스 운영은 별도 추적 과제로 유지한다.
