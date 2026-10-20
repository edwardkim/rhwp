# PR #7474 적용 기록 — standard release 검증과 배포 재사용

## 2026-10-20 완료한 순서

1. #7488에 최신 devel `5e2b9bf`를 통합하고 개발 안내 충돌을 해결했다: `3bfe08e24`.
   opt-in 제품 코드를 유지했으며 31 success·5 skipped를 확인했다.
2. #7474에 위 head를 이력 보존 통합했다: `cb994afdb`. 기존 오늘할일 양쪽 기록을 보존했다.
3. standard 기본값·배포 패키지 동일성 검사·반례 테스트를 구현했다: `1ae9b510a`.
4. 로컬 계약 검사와 code head Full CI·CodeQL·Render Diff를 통과했다. Pages와 package는
   같은 head에서 verify-only로 실행해 WASM/VSIX/Pages artifact를 만들고 실제 게시는 건너뛰었다.
5. artifact digest, 실제 소비 22건, 세 경로의 JS/WASM 동등성, 최종 VSIX bytes를 확인했다.
   [보고서](../../report/task_m100_7473_report.md)·[검토](pr_7474_review.md)에 근거와 한계를 남겼다.
6. 문서 후행 head의 CI를 확인한다. #7488 Ready, #7474 Draft를 유지하며 승인·merge·close는 하지 않는다.

## 되돌리기와 남은 조건

실행 계약 변경을 되돌리려면 `1ae9b510a`의 구현 diff를 revert하고 관련 workflow 계약과 exact-head
검증을 다시 수행한다. #7488 opt-in은 별개이며 기본값을 다시 자동 활성화할 이유는 없다.
배포 변경은 정상 PR 통합·release 절차로 적용하며 이번 verify-only 성공을 production 배포로 표현하지 않는다.
기존 비교 ref·원자료의 보존 범위는 아래 역사적 기록을 따른다. 활성 PR branch·fork와 공유 target은 보존한다.

## 이전 기록 — 2026-10-18


### 완료한 순서

1. 원 #7474 `9d05998e09e4698391fe44f543abdd66ddc86c99`에 devel `6ed1682c3267ea786734eb2e75252916a861d1fd`를 충돌 없이 merge했다: `0c669defcd5260c2c8d8b90c9f7e2f0691a5ded3`.
2. #7488 `e8e57b36d4703556128dae31e37eaa9acdceee0d`를 원 이력 보존 merge했다: `883edf279c44c67172e56aa4ea596fb83e056d9e`.
3. 빌드 구성별 cache·수동 비교·runner 관측을 보정했다: `0eb4cc2ffde257a15080d0add104247c941fcbf9`.
4. 코드 후보의 전체 CI 완료 후 dev/standard/cdylib 각 3회 비교와 [검토 기록](pr_7474_review.md)을 완료했다.
5. #7488 본문을 메인테이너 직접 승인 대기로 정리하고 Ready로 전환했다. GitHub 승인을 대신하거나 merge하지 않았다.
6. 같은 코드 후보 위에 문서 후행 커밋을 추가하고 해당 head CI를 확인한다. #7474는 Draft를 유지한다.

### 판단과 되돌리기

[보고서](../../report/task_m100_7473_report.md)의 비용과 LTO 정책 수용이 남은 판단이다.
이번 요청의 적용·측정·리뷰는 완료했으며 merge·이슈 종료는 하지 않는다.
cdylib 경로만 되돌릴 때는 PR의 build-mode 선택을 standard로 바꾸고 해당 구성을 재검증한다.
수동 비교는 `wasm-profile=release`, `wasm-build-mode=standard`로 재현한다.
wrapper 기본값은 opt-in 해제 시 기존 경로이며, 서로 다른 cache variant를 재사용하지 않는다.
원 PR 이력을 강제로 다시 쓰지 않는다.

### 임시 자원

이번 측정에서 만든 `codex/measure-7474-*-20261018` 8개 ref는 모두 같은 검증 코드 SHA를 가리킨다.
모든 run 완료·최종 SHA·열린 PR 없음 확인 후 이 ref만 정리한다.
실제 #7474 source branch와 #7488 fork branch는 유지한다.
원시 artifact·집계는 ignored `output/pr-review/7474/`, 최종 표·대표 PNG는 보고서에 보존한다.
