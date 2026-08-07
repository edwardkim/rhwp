---
kind: pr_review_implementation
status: pending-review-only-fast-pass
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-08
---

# PR #4178 메인터너 보정 이행 기록

## 고정한 기준

- contributor source head: `72704d909265811bf3a643a4cb6bfbe05d9682b2`
- 보정 뒤 code candidate: `7f38e41efea7c29c8e7c22ccf0299a9928db9b1d`
- 최신 검토 기준: `upstream/devel` `80fd91263132d6cdca0220c164e7d26586d5a3ea`
- 가시성 검토 branch: `review/planet6897-4178-20260808`
- 원격 collaborator 보정: `3cf9e53824ea26b630ce571c2da259d287e327bb`,
  `7f38e41efea7c29c8e7c22ccf0299a9928db9b1d`

## 완료한 단계

1. 원 contributor 두 commit을 최신 `devel` 위에 clean하게 적용해 serializer와 P0 하니스의 통합 tree를
   검토했다.
2. 오라클 major 판정, stale 산출물 격리, 기본 무종료 프로세스 정책, ParameterSet Item 521 계약을
   별도 메인터너 code/test/doc commit으로 고정했다.
3. serializer focused Rust test, 하니스 계약 9건, Python/Node 구문, 원장 검사, package dry-run을
   실행해 통과했다.
4. Windows 10의 SSH Services session 0은 `pyhwpx.Hwp()` COM 초기화가 멈춰 실제 오라클 실행 환경으로
   부적합함을 확인했다. 활성 RDP session 2의 interactive scheduled task에서 `field-read`를 실행해
   한글 `12, 0, 0, 535`, 호출 8건, fatal/error 0건과 종료 뒤 잔류 프로세스 없음을 확인했다.
5. 전체 gate의 첫 실행에서 COM Quit 직후의 비동기 한글 종료를 `LEFTOVER`로 잘못 판정한 것을
   확인했다. `7f38e41ef`에서 최대 10초 settle 대기를 추가한 뒤 같은 interactive session에서 전체
   gate를 재실행해 exit 0, `field-read=OK`, 비교 대상 1건을 확인했다. L2는 MATCH 3, VALUE_DIFF 2,
   MISSING_API 3으로 P0 차등 측정 결과이며 L3 저장 검증은 해당 시나리오에 없다. 검증에 사용한 일회성
   scheduled task와 임시 디렉터리는 실행 후 제거했고 Hwp/HwpFrame/python 잔류 프로세스가 없음을
   재확인했다.
6. `7f38e41ef`를 contributor branch에 fast-forward push했다. 같은 head의 GitHub Full CI run
   `31204218519`와 CodeQL run `31204217839`이 모두 성공했고, 문서 작성 시점 PR 상태는
   `MERGEABLE`/`CLEAN`이다.

## 남은 순서

1. 이 archive review·이행 기록과 PR #4178 오늘할일만 `7f38e41ef` 뒤에 trailing commit으로 추가한다.
2. commit 범위가 review-only인지, 최신 `upstream/devel` merge simulation과 Markdown 링크 검사가
   통과하는지 확인한 뒤 contributor branch에 push한다.
3. 최신 head의 fast-pass preflight와 Build & Test aggregate, CodeQL, mergeable 상태를 확인한다.
4. 작업지시자의 merge 승인 뒤에만 병합하고, PR comment·devel 동기화·worktree 정리는 merge 후속
   처리 절차로 진행한다.
