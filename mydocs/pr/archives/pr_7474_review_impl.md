# PR #7474 적용 기록 — #7488 통합과 재측정

## 완료한 순서

1. 원 #7474 `9d05998e09e4698391fe44f543abdd66ddc86c99`에 devel `6ed1682c3267ea786734eb2e75252916a861d1fd`를 충돌 없이 merge했다: `0c669defcd5260c2c8d8b90c9f7e2f0691a5ded3`.
2. #7488 `e8e57b36d4703556128dae31e37eaa9acdceee0d`를 원 이력 보존 merge했다: `883edf279c44c67172e56aa4ea596fb83e056d9e`.
3. 빌드 구성별 cache·수동 비교·runner 관측을 보정했다: `0eb4cc2ffde257a15080d0add104247c941fcbf9`.
4. 코드 후보의 전체 CI 완료 후 dev/standard/cdylib 각 3회 비교와 [검토 기록](pr_7474_review.md)을 완료했다.
5. #7488 본문을 메인테이너 직접 승인 대기로 정리하고 Ready로 전환했다. GitHub 승인을 대신하거나 merge하지 않았다.
6. 같은 코드 후보 위에 문서 후행 커밋을 추가하고 해당 head CI를 확인한다. #7474는 Draft를 유지한다.

## 판단과 되돌리기

[보고서](../../report/task_m100_7473_report.md)의 비용과 LTO 정책 수용이 남은 판단이다.
이번 요청의 적용·측정·리뷰는 완료했으며 merge·이슈 종료는 하지 않는다.
cdylib 경로만 되돌릴 때는 PR의 build-mode 선택을 standard로 바꾸고 해당 구성을 재검증한다.
수동 비교는 `wasm-profile=release`, `wasm-build-mode=standard`로 재현한다.
wrapper 기본값은 opt-in 해제 시 기존 경로이며, 서로 다른 cache variant를 재사용하지 않는다.
원 PR 이력을 강제로 다시 쓰지 않는다.

## 임시 자원

이번 측정에서 만든 `codex/measure-7474-*-20261018` 8개 ref는 모두 같은 검증 코드 SHA를 가리킨다.
모든 run 완료·최종 SHA·열린 PR 없음 확인 후 이 ref만 정리한다.
실제 #7474 source branch와 #7488 fork branch는 유지한다.
원시 artifact·집계는 ignored `output/pr-review/7474/`, 최종 표·대표 PNG는 보고서에 보존한다.
