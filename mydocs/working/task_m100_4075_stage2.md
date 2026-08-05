# Task M100 #4075 Stage 2 - 신뢰된 PR의 즉시 비용 수집

- 이슈: [#4075](https://github.com/edwardkim/rhwp/issues/4075)
- 브랜치: pr/issue-4075-nextest-cost-shards
- 시작 기준: bc313cd6a (ci: nextest 비용 기반 shard 배정)
- 기록일: 2026-08-05 KST
- 상태: 로컬 구현·검증 완료, 원격 CI 대기

## 문제

Stage 1의 publisher는 devel push만 허용한다. 이 정책은 외부 fork의 cache 오염을 막지만, 동일 저장소의
신뢰된 collaborator PR에서 Build and Test가 끝난 직후 비용 model을 생성해 확인하려는 운영 흐름에는
맞지 않는다.

## 구현 계획

1. PR head repository가 현재 repository와 같은 경우만 trusted PR로 판정한다. 현재 repository에
   branch를 생성하려면 write 권한이 필요하므로 별도 author association 문자열에 의존하지 않는다.
2. trusted full PR과 devel push에서만 libtest-json-plus event를 수집하고 runtime slice artifact를
   upload한다.
3. publisher는 full trusted PR과 devel push에서 실행한다. PR에서 생성한 cache는 pull request ref
   scope이므로 해당 PR 재실행용이며, devel push가 공용 기준 cache를 갱신한다.
4. preflight fast-pass가 true인 docs-only trailing commit은 collector와 publisher를 모두 skip한다.
5. workflow 정적 테스트에 trusted PR, external fork, docs-only fast-pass의 조건을 고정한다.

## 성공 기준

- trusted PR의 Build and Test 성공 뒤 Publish nextest cost model job이 실제 실행된다.
- docs-only fast-pass와 external fork PR에서는 publisher가 skip된다.
- devel push cache의 정리 책임과 주기 sweep 예외는 Stage 1과 동일하게 유지된다.

## 검증 결과

### 1. trusted PR 판정 확인

- 명령: GitHub pull request API로 PR #4076의 head repository와 author association 조회.
- 결과: head repository는 edwardkim/rhwp, author association은 COLLABORATOR다. 따라서 이 PR은
  raw event 수집 및 PR ref scope cache 생성 조건을 충족한다.

### 2. workflow 정적 검증

- 명령: python3 -m unittest scripts/tests/test_ci_impact_workflow.py
- 결과: 14개 통과. trusted PR/devel push만 publisher를 실행하고, external fork 및 docs-only
  fast-pass는 collector와 publisher를 skip하는 조건을 확인했다.

### 3. 형식·단위 검증

- 명령: node --test scripts/tests/nextest_cost_model.test.mjs, Node syntax check, actionlint 4개
  workflow, git diff --check.
- 결과: 비용 모델 Node test 5개 및 나머지 검증이 모두 exit code 0으로 통과했다.

### 4. 원격 structured output 실패와 보정

- 관측: PR #4076의 첫 trusted PR 수집 실행에서 shard 2와 3이 실패했다. event 파일 첫 줄에
  info: sync 같은 stderr 진단이 섞여 JSON parser가 실패했다.
- 원인: structured nextest 실행에서 stdout과 stderr을 같은 events 파일로 redirect했다.
- 보정: stdout만 events JSONL로 보내고 stderr은 기존 log 파일로 분리한다. 실패 시에는 두 파일을
  함께 출력한다. 이 보정 뒤 새 PR CI로 publisher와 실제 cache를 다시 확인한다.

### 5. trusted PR publisher skip과 보정

- 관측: structured output 보정 뒤 Build and Test는 성공했지만 publisher가 skipped였다.
  preflight는 fast_pass=false였고 PR head repository도 현재 repository와 같았다.
- 원인: job-level expression의 author_association 추가 조건이 실제 payload에서 신뢰 판정에 안정적으로
  평가되지 않았다.
- 보정: 동일 저장소 branch 생성 자체가 write 권한을 요구하는 GitHub 경계를 사용해 head repository
  동일성만 확인한다. external fork는 여전히 publisher와 cost collector를 실행할 수 없다.

## 원격 검증 조건

이 commit push 뒤 최신 PR head에서 archive와 shard의 structured event artifact 4개, Build and Test,
Publish nextest cost model을 확인한다. docs-only trailing commit은 fast-pass여야 하므로 publisher가
skip되는 것이 정상이다.
