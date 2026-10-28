# #5575 Adapter inter-diff 문서 전용 fast-pass

## 배경

PR #5573은 `mydocs/**` 두 파일만 바꾼 검토 기록 PR이었다. 중앙 CI, CodeQL, Proptest는
fast-pass로 무거운 lane을 건너뛰었지만, 독립 `Adapter inter-diff` workflow는 항상 전체
adapter harness를 실행해 2분 9초가 추가됐다.

## 원인

`adapter-diff.yml`은 중앙 CI preflight 결과를 소비하지 않고 모든 `pull_request`에서
`adapter-diff` job을 실행한다. workflow-level `paths-ignore`는 required check 자체를
사라지게 해 pending 위험이 있으므로 해결책이 될 수 없다.

## 변경 계획

1. 항상 실행되는 `adapter inter-diff preflight` job에서 PR base/head diff를 확인한다.
2. 변경 파일이 비어 있지 않고 전부 `mydocs/**`일 때만 adapter job을 skip한다.
3. 비문서 변경, PR이 아닌 trigger, base/head 또는 diff 오류는 fail-closed로 전체 adapter
   검증을 유지한다.
4. workflow test로 skip gate, full-history diff, fail-closed 분기와 `paths-ignore` 부재를
   고정한다.

## 완료 기준

- 문서 전용 PR에서 `adapter inter-diff` check가 `skipped` 상태로 남는다.
- 코드와 fixture, CI, 테스트 또는 알 수 없는 변경은 기존 adapter 검증을 실행한다.
- push와 수동 실행은 전체 검증을 유지한다.
