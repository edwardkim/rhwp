# #6213 Stage 1: green PR 검증의 안전한 post-merge 재사용

## 배경

PR #6207은 B/C target 실행시간을 수집하기 위해 `devel` CI가 성공한 뒤
측정 정책을 갱신하도록 구현했다. 그러나 이 방식은 PR에서 이미 통과한 CI,
CodeQL, adapter, proptest worker를 merge 뒤 다시 실행한다.

`devel` run `33064128254`는 refresh job까지 성공했지만 B/C 측정 artifact와
data branch의 target map이 모두 비어 있었다. 따라서 green job 결론만으로
측정 정책의 기능적 성공을 판단할 수 없다.

## 확인한 운영 제약

- `devel` push에서 CI, CodeQL, Adapter inter-diff, Proptest roundtrip이 실행된다.
- `devel` 직접 push, 후보 누락, stale head, 실패 또는 pending 후보는 전체 검증을
  유지해야 한다.
- 저장소의 기본 브랜치는 `main`이다. 새 `workflow_run` 또는
  `workflow_dispatch` workflow를 `devel`에만 추가해 post-merge worker를 분리하면
  GitHub가 실행하지 않는다.
- write 권한 경로는 trusted `devel` 코드만 실행하며, PR head의 코드나 script를
  checkout 또는 실행하면 안 된다.

## 설계 결정

1. 기존 `devel` push trigger와 required check 이름은 유지한다.
2. CI, CodeQL, Adapter inter-diff, Proptest preflight가 공통 검증기로 merge commit을
   해석한다.
3. 다음 조건이 모두 충족될 때만 해당 workflow worker를 재사용한다.
   - 현재 `devel` tip이 merge commit이다.
   - merge commit의 source parent, PR head, 대상 `devel` base 계보가 일치한다.
   - 같은 PR head의 해당 workflow 최종 candidate가 완료·성공했다.
   - 후보가 stale, fork, direct push, 재실행 중, 누락 또는 실패가 아니다.
4. 어느 하나라도 불명확하면 worker를 skip하지 않고 기존 전체 CI로 fail-closed 한다.
5. CI의 duration 갱신은 재사용한 PR run ID의 B/C artifact만 읽는다. 두 artifact는
   같은 run/head 출처여야 하며, B/C target map이 비어 있거나 현재 integration
   target 집합과 일치하지 않으면 publish를 거부한다.

## 완료 기준

- 공통 검증기 unit test가 exact candidate, stale candidate, base/head 불일치, direct
  push, fork, pending 또는 failed 후보를 구분한다.
- 네 workflow 계약 test가 post-merge 재사용 및 full fallback 조건을 고정한다.
- PR CI는 workflow 변경으로 full 검증한다.
- merge 뒤 `devel`에서는 exact green PR만 worker가 재사용되고, duration data에는
  비어 있지 않은 target 시간이 기록된다.

## 제외

- `devel` 직접 push의 전체 검증 제거
- required check 이름 변경
- B/C shard 수 변경
