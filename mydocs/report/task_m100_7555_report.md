# #7555 macOS 릴리스 러너 전환 로컬 결과

- Issue: [#7555](https://github.com/edwardkim/rhwp/issues/7555)
- branch/worktree: `fix/7555-macos15-release-runner` / `/tmp/rhwp-7555-macos15`
- PR base: `e1ecaa248ecf7f667d8fccab4d9938e70a253392` (`upstream/devel`)
- 검증한 실행 파일 변경 commit: `1d3ac74f6a7949c20af8f10b7290331f52f5025b`
- [계획](../plans/task_m100_7555.md), 운영 권위: [GitHub 운영](../manual/github_operations.md), [배포 가이드](../manual/publish_guide.md)

## 변경과 영향

`.github/workflows/release-binary.yml`의 두 macOS runner를 `macos-15`로 바꿨다. ARM64 runner에서 Intel target을 교차 빌드하며, ARM64 target은 native 빌드한다.

Cargo cache의 key와 restore prefix에 `matrix.runner`를 포함한다. cache는 `target/`를 포함했으므로 이전 SDK 산출물이 전환한 runner로 복원되지 않아야 한다. 새 prefix는 기존 cache와 분리된다. 다섯 target 모두 첫 실행의 cache miss 비용이 발생한다.

YAML의 구조를 기준 base와 비교해 runner 두 값·cache key/prefix 외의 event, permissions, job/check 이름, build/verify/package 명령, archive/artifact·Release 조건이 같음을 확인했다. 제품 소스·테스트 변경이나 required check 변경은 없다.

## 로컬 검증

| 명령 | 결과 |
| --- | --- |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts/tests/test_release_publish_orchestration.py scripts/tests/test_release_channel_policy_workflow.py scripts/tests/test_nextest_archive_workflow.py` | 40/40 PASS |
| `python3 /tmp/rhwp-7555-check-matrix.py` | PyYAML 6.0.1 BaseLoader 파싱·base 구조 비교 PASS, target 5개·suffix 중복 없음, macOS 두 target만 macos-15 |
| `/tmp/rhwp-7555-actionlint/actionlint -shellcheck='' .github/workflows/release-binary.yml` | actionlint 1.7.12 PASS. shellcheck 미설치이며 shell 명령은 바꾸지 않아 외부 shellcheck만 생략 |
| `python3 scripts/check_markdown_links.py mydocs/plans/task_m100_7555.md mydocs/orders/20261003.md mydocs/manual/publish_guide.md` | 3문서 내부 링크 PASS |
| `git diff --check` | PASS |

검사는 수정된 workflow·guide 상태에서 실행했으며 실행 계약은 위 commit에 포함된다. 구조 비교 진단은 임시 파일이고 정식 회귀 테스트를 추가하지 않았다. Rust/Cargo·WASM·Studio 전체 검증은 제품 코드·명령 변경이 없어 비해당이다.

## 미검증과 다음 단계

실제 macOS 15에서 두 바이너리의 build, Intel 바이너리의 `--version` 실행(Rosetta 경로 포함), 패키징·Mach-O architecture, Actions cache 동작은 **미검증**이다. Linux의 로컬 검사 통과를 해당 항목의 통과로 보고하지 않는다.

승인 후 다음 순서로 진행한다.

1. latest devel과 head 충돌 재확인 후 branch push와 devel 대상 PR 생성.
2. 같은 branch exact head의 `Release Binary`를 `tag=test`로 실행. 다섯 build 성공, macOS 두 실행·archive, Release/외부 publish skip과 verify-only 증거를 확인.
3. 최신 PR head required CI와 dry-run 결과를 검토한 뒤 merge 승인을 받는다.
4. 정상 devel → main 승격과 다음 tag의 workflow 포함을 확인한다. 이 경계까지는 이슈를 닫지 않는다.

등록된 이슈는 OPEN, 담당 edwardkim, milestone v1.0.0, labels ci/github_actions/packaging이다. 사용자 승인 후 원격 branch push와 [PR #7556](https://github.com/edwardkim/rhwp/pull/7556) 생성을 완료했다. [self-review](../pr/archives/pr_7556_review.md)와 오늘할일을 같은 branch에 포함한 뒤 승인된 exact-head dry-run을 실행한다. merge·실제 publish·close는 수행하지 않았다.

## 복구

지원 중인 macOS ARM64 image에서 동일 target의 build·실행을 다시 검증한다. macOS 14로 revert하는 것은 brownout/종료 이후 유효한 복구가 아니다. trigger·권한·Release 조건을 우회해 검증 실패를 숨기지 않는다.
