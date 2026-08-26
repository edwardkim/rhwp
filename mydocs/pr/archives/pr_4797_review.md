# PR #4797 검토 - load/save sweep 감독 안정화

## 메타데이터

| 항목 | 값 |
| --- | --- |
| 원 PR | [#4797](https://github.com/edwardkim/rhwp/pull/4797) |
| 통합 PR | [#4801](https://github.com/edwardkim/rhwp/pull/4801) |
| 관련 이슈 | `Closes #4749, #4751` |
| 작성자·검토 방식 | `planet6897` · collaborator 체리픽 통합 self-review |
| 원 base / head | `devel` / `a1b3a9bb60e9fbce630822eec7db62e060ed735c` |
| 적용 commit | `2ce9bfd57` |
| 통합 code candidate | `c7e2f1fe5586eca576daeb58495da5718f7eebfc` |
| 규모 | 3 files, +111/-14 (원 PR 기준) |
| 라우팅 | collaborator external PR · intake/review · local validation |

원 기능 commit은 최신 `upstream/devel@44bcba400072128bdc4e4d6c05bf822e3ff60996` 위에 충돌 없이
체리픽했다. 이 기록은 code candidate Full CI 성공 뒤 추가하는 review-only trailing 문서이므로,
merge 직전에는 이 문서 head의 fast-pass와 최신 mergeability를 다시 확인한다.

## 변경 범위와 판단

- sweep supervisor가 heartbeat 파일을 읽을 때 writer와 공유할 수 있게 열어, 일시적인 Windows
  `Share violation`을 supervisor 사망으로 오인하지 않게 한다.
- 실행 중 파일 크기에 비례한 stall 허용 시간을 사용하고, supervisor kill은 별도 TSV에 남긴다.
- judge는 알려진 supervisor kill을 입력 파일 결함(`OPEN_FAIL`)이 아니라 `ORACLE_TIMEOUT` 계열로
  분류해 defect 집계와 재검증 대상을 구분한다.

파일 공유 예외와 실제 변환 결함의 분류 경계를 확인했으며, Windows 특이 실패를 숨기거나 정상 결과로
바꾸는 보정은 발견하지 못했다. 메인터너 code 보정은 필요하지 않다.

## 완료된 검증

- `tools\loadsave_sweep\oracle_run.ps1`을 PowerShell AST로 파싱해 오류가 없음을 확인했다.
- `python -m py_compile tools\loadsave_sweep\judge.py`를 통과했다.
- `git diff --check upstream/devel...HEAD`를 통과했다.
- code candidate `c7e2f1fe5`의 GitHub Actions에서 Lint, Build & Test 전체 shard, Native Skia,
  Canvas visual diff, CodeQL이 모두 성공했다.

## 위험과 후속 범위

- 시간 한계는 실제 heartbeat 크기를 고려하는 보수적 감독 정책이다. 장시간 원격 oracle의 새 실패
  서명은 별도 sweep 결과로 관찰해 threshold를 조정한다.
- 이 변경은 Windows 공유 모드에서 발생하는 판독 경쟁을 좁힌다. 다른 외부 프로세스의 lock 정책을
  일반화하지 않는다.

## 최종 권고

수용을 권고한다. #4801의 review-only trailing head가 fast-pass 조건을 만족하고 최신
`MERGEABLE/CLEAN` 상태를 재확인한 뒤, 작업지시자 승인에 따라 통합 PR로 병합한다.
