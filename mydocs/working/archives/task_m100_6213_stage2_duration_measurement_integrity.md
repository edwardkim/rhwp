# #6213 Stage 2: B/C 실행시간 측정의 출처와 완전성 고정

## 목표

Stage 1의 post-merge 재사용은 B/C artifact가 실제 선택 target의 실행시간을 담는다는
전제가 있어야 한다. PR #6207의 `devel` run `33064128254`는 성공했지만 수집 결과가
0 target이었다. 이 단계를 먼저 고쳐 다음 단계가 빈 측정값을 신뢰하지 못하게 한다.

## 원인

nextest JUnit은 test binary를 `testsuite`로, 개별 test 실행시간을 `testcase`로 기록한다.
기존 수집기는 `testsuite`의 `time`만 읽었으므로, suite-level 시간이 없는 실제 JUnit에서
모든 target을 누락했다.

## 구현

1. `testcase.classname`에서 test binary 이름을 읽고 `time`을 binary별로 합산한다.
2. 수집 결과가 빈 target map이면 artifact를 쓰지 않고 worker를 실패시킨다.
3. refresh는 B/C 각각이 비어 있지 않고 같은 `run_id`, `ref`, `sha` 출처임을 요구한다.
4. 성공한 `devel` B/C artifact는 30일, 같은 저장소 PR B/C artifact는 trusted
   post-merge 후보 소비를 위해 3일 보관한다. fork PR artifact는 만들지 않는다.

## 보안 및 운영 경계

- PR artifact는 이 단계에서 duration policy를 갱신하지 않는다.
- 다음 단계의 trusted verifier가 same-repository PR, exact merge lineage, 성공한
  workflow run을 모두 확인할 때만 PR artifact를 읽을 수 있다.
- direct `devel` push와 후보 누락 또는 출처 불일치는 기존 전체 검증과 `devel` 측정
  경로로 fail-closed 한다.
- 같은 Stage의 shared verifier는 PR head가 merge base를 포함하고 merge tree와 같은지를
  검증한다. 따라서 stale PR의 source-head green 결과로 merge 결과를 대체하지 않는다.
- CI worker 재사용은 source PR run의 B/C artifact 이름·존재까지 검사한 뒤에만 허용한다.
  artifact가 없는 기존 PR, fork, 검증 오류는 full `devel` worker로 fallback한다.
- verifier 스크립트는 merge commit이 아니라 pre-PR source parent에서 checkout한다. 이
  bootstrap PR은 base에 verifier가 없어 full CI를 실행하고, 이후 PR부터 재사용한다.

## 완료 기준

- suite-level `time`이 없는 JUnit fixture에서 B/C target별 합계가 생성된다.
- 빈 map 및 B/C 출처 불일치가 policy refresh 전에 거부된다.
- workflow 계약 test가 `devel` 30일, same-repository PR 3일 artifact 경계를 고정한다.
