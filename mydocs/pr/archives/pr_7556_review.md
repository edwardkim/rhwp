# PR #7556 self-review — macOS 릴리스 러너 전환

## 최종 판정

**승인 — 검증한 Actions head `9a3552f60adf3fb20c1a5117403eb85411de526d`가 runner 전환 범위의 실제 실행 증거를 충족했다.** 두 macOS build·`--version`·Mach-O archive, 다섯 CLI archive와 verify-only package 검증, Full CI가 통과했다. merge 전 조건은 결과 기록을 포함한 최신 후행 head CI·mergeability 재확인과 별도 사용자 승인이다. 이 판정은 GitHub approve event나 merge 실행이 아니다.

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
| 검증 완료 시점 상태 | head 9a3552f60은 Open·MERGEABLE·CLEAN, CI·CodeQL·Adapter·Proptest와 Release Binary success. 후행 head는 merge 전 재조회 필요 |

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
- 동작 검증: 실제 [Release Binary dry-run](https://github.com/edwardkim/rhwp/actions/runs/37079742187)의 두 macOS build·실행·다운로드 artifact를 확인했다. 문자열 검사만으로 성공을 판정하지 않았다. [결과와 SHA-256](../../report/task_m100_7555_report.md#실제-actions-검증)을 연결한다.
- Merge 후 contributor 시각 comment: 비해당. 본인 운영 PR이며 시각 증거를 사용하지 않는다.

## 실행 순서와 잔여 조건

1. self-review와 오늘할일을 포함한 head 9a3552f60을 push하고 그 exact SHA의 `workflow_dispatch(tag=test)`를 실행했다.
2. macOS 15.7.9의 두 target 모두 `rhwp v0.8.6`을 실행했다. 다운로드 파일의 Mach-O x86_64·arm64 형식, 실행 권한·내부 파일·API digest 일치를 확인했다. runner별 새 cache key miss도 완료 로그로 확인했다.
3. 다섯 CLI build와 WASM·VSIX·채널 집계가 성공했다. 8개 필수 artifact와 Release/네 외부 publish skip을 확인했고 evidence의 네 채널 모두 verify-only였다. [검증 요약](../../report/assets/issue7555/verification-summary.json)에 run/head·산출물·관측값을 보존했다.
4. 동일 head의 CI·CodeQL·Adapter·Proptest가 성공했다. 이 결과 기록의 후행 commit은 mydocs만 변경한다. 최신 head의 trusted reuse 판정·required aggregate·mergeability를 재확인한 뒤 merge 승인을 받는다.
5. merge 승인 후 devel 통합, 정상 devel → main 승격·다음 release tag에 workflow가 포함됐는지 확인한다. 이슈는 그때까지 OPEN이다.

필수 실행 증거 부족 blocker는 해소됐다. macOS 15의 실행 검증과 cache 분리가 실제로 확인됐으며 제품 회귀를 검출한 항목은 없다. merge·실제 publish·이슈 close는 수행하지 않았다.
