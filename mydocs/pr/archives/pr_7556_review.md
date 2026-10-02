# PR #7556 self-review — macOS 릴리스 러너 전환

## 최종 판정

**머지 보류 — 실제 macOS 15 빌드와 최신 PR head CI 검증 진행 전.** 두 macOS target의 build·`--version`·Mach-O archive, 다섯 CLI archive, 수동 실행의 Release/외부 publish skip과 최신 required check 통과가 해제 조건이다. merge는 별도 사용자 승인 후 수행한다.

## 접수 정보

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR·작성자·base | [#7556](https://github.com/edwardkim/rhwp/pull/7556) / edwardkim / devel |
| Issue | [#7555](https://github.com/edwardkim/rhwp/issues/7555), Refs만 사용, 자동 종료 없음 |
| 구현 commit | `1d3ac74f6a7949c20af8f10b7290331f52f5025b` |
| PR 생성 head | `0c461e2ff1f944aefb5ba3c5f89d1cb354a5de28` |
| base SHA | `e1ecaa248ecf7f667d8fccab4d9938e70a253392` |
| reviewer | 작성자 self-review, 별도 reviewer 지정 없음 |
| 최초 PR 규모 | 5파일, +96/-7, commit 3개. 이 self-review 후행 문서는 별도 추가 |
| 최초 상태 | Open·MERGEABLE, merge state blocked, CI 진행 중. merge 전 재조회 필요 |

라우팅: `collaborator_self_merge`; modifier `intake_and_review`, `local_validation`, `review_only_fast_pass`.
권위 문서 `pr_review_workflow.md`, `pr_review/README.md`와 해당 자식 및 `review_template.md`, `github_operations.md`, `publish_guide.md`를 읽었다.
2026-10-03 사용자는 remote push·Open PR 생성·Release Binary `tag=test` 실행을 승인했다. 승인 범위에 merge·실제 publish·issue close는 없다.

## 변경과 검증

macOS 두 matrix entry를 `macos-15`로 전환하고, Cargo cache key/restore prefix에 runner label을 추가했다. 기존 cache의 `target/`를 새 SDK로 복원하지 않는다. 첫 실행은 다섯 target의 cache miss 비용이 있다.

[로컬 결과](../../report/task_m100_7555_report.md)의 40/40 계약 검사·PyYAML base 구조 비교·actionlint 1.7.12·diff/link 검사는 완료됐다. 구조 비교는 runner 두 값·cache key/prefix 외 event, permission, job/check, build/verify/package 명령, artifact와 Release 조건이 동일함을 확인했다. shell 명령을 바꾸지 않았으며 shellcheck 미설치에 따른 외부 shellcheck만 생략했다.

이 변경은 runner 환경 전환이다. static 계약 검사와 macOS 실제 build·바이너리 실행 검증을 구분한다. Linux의 로컬 Cargo 빌드로 macOS 검증을 대신하지 않는다.

## 공통 조판 원칙·시각·입력 확인

- 조판 원칙: **비해당**. source·test·layout·paint·baseline/golden·sample·기준 PDF 변경이 없고 시각 출력 개선을 주장하지 않는다. 측정·배치·분할·줄 소속·기준값 변경 모두 비해당이다.
- Visual Sweep·Native/fresh WASM: 비해당. 바뀌는 것은 CLI release runner와 cache 경계뿐이며 renderer와 실행 build 명령은 동일하다.
- 검증 입력 commit 확인: **비해당**. HWP/HWPX/PDF 입력을 검증 근거로 사용하지 않았다.
- 동작 검증: 실제 Release Binary dry-run의 두 macOS build/실행/artifact로 확인한다. 현재 미실행이며 문자열 검사 통과를 실행 성공으로 승격하지 않는다.
- Merge 후 contributor 시각 comment: 비해당. 본인 운영 PR이며 시각 증거를 사용하지 않는다.

## 실행 순서와 잔여 조건

1. self-review와 오늘할일을 같은 branch의 문서-only 후행 commit으로 push한다.
2. 그 exact head에서 Release Binary `workflow_dispatch(tag=test)`를 실행하고 run/head/architecture·검증 로그·publish skip을 확인한다.
3. 최신 head CI/mergeability와 실제 run 결과로 이 판정을 갱신한다. 녹색 candidate 이후 허용된 문서만 바뀌면 canonical trusted reuse와 최신 aggregate를 확인한다.
4. merge 승인 후 devel 통합, 정상 devel → main 승격·다음 release tag에 workflow가 포함됐는지 확인한다. 이슈는 그때까지 OPEN이다.

실제 macOS 15 실행 검증·Actions cache 동작은 현재 **미검증**이며 blocker는 필수 실행 증거 부족이다. 실행으로 재현된 제품 회귀를 발견했다는 뜻이 아니다.
