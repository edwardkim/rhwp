# #7555 macOS 릴리스 러너 전환 결과

- Issue: [#7555](https://github.com/edwardkim/rhwp/issues/7555), 담당 edwardkim
- PR: [#7556](https://github.com/edwardkim/rhwp/pull/7556), base devel
- branch/worktree: `fix/7555-macos15-release-runner` / `/tmp/rhwp-7555-macos15`
- PR base: `e1ecaa248ecf7f667d8fccab4d9938e70a253392`
- 구현 commit: `1d3ac74f6a7949c20af8f10b7290331f52f5025b`
- Actions 실행 head: `9a3552f60adf3fb20c1a5117403eb85411de526d`
- [계획](../plans/task_m100_7555.md), [self-review](../pr/archives/pr_7556_review.md), [검증 요약 JSON](assets/issue7555/verification-summary.json)

## 변경과 영향

Release Binary의 macOS 두 runner를 `macos-15`로 바꾸고, Cargo cache key와 restore prefix에 `matrix.runner`를 포함했다. 기존 cache는 `target/`를 포함했으므로 이전 SDK 산출물이 전환한 runner로 복원되지 않게 분리했다. 다섯 target의 첫 실행은 cache miss 비용이 있었다.

YAML base 구조 비교로 runner 두 값·cache key/prefix 외 event, permissions, job/check 이름, build/verify/package 명령, archive/artifact·Release 조건이 동일함을 확인했다. 제품 source/test, required check, renderer·baseline 변경은 없다. 배포 가이드의 runner 표도 갱신했다.

## 로컬 검증

| 명령 | 결과 |
| --- | --- |
| `PYTHONDONTWRITEBYTECODE=1 python3 -m unittest scripts/tests/test_release_publish_orchestration.py scripts/tests/test_release_channel_policy_workflow.py scripts/tests/test_nextest_archive_workflow.py` | 40/40 PASS |
| `python3 /tmp/rhwp-7555-check-matrix.py` | PyYAML 6.0.1 BaseLoader 파싱·base 구조 비교 PASS, target 5개·suffix 중복 없음 |
| `/tmp/rhwp-7555-actionlint/actionlint -shellcheck='' .github/workflows/release-binary.yml` | actionlint 1.7.12 PASS. shellcheck 미설치이며 shell 명령 변경이 없어 외부 shellcheck만 생략 |
| 수정 문서의 `scripts/check_markdown_links.py`, `git diff --check` | PASS |

로컬 검사는 구현 commit의 workflow·guide 상태에서 수행했다. 구조 비교·artifact 검사는 임시 진단으로 실행했으며 정식 회귀 테스트를 추가하지 않았다. source를 바꾸지 않아 로컬 Cargo·WASM·Studio 전체를 반복하지 않았다. 원격 PR CI에서는 적용되는 Full 검증이 실제 성공했다.

## 실제 Actions 검증

[Release Binary run 37079742187](https://github.com/edwardkim/rhwp/actions/runs/37079742187)은 위 exact head의 `workflow_dispatch(tag=test)`로 실행해 **success**였다.

| target | build·--version·archive | 직접 확인 |
| --- | --- | --- |
| macOS Intel | PASS | macos-15-arm64 / macOS 15.7.9에서 교차 빌드, `rhwp v0.8.6` 실행, Mach-O 64-bit x86_64 |
| macOS ARM64 | PASS | 같은 image에서 native 빌드, `rhwp v0.8.6` 실행, Mach-O 64-bit arm64 |
| Linux x86_64·ARM64, Windows x86_64 | 모두 PASS | 각 build job과 해당 Verify binary step success, archive 생성 |

macOS 두 압축 파일은 다운로드해 실제 Mach-O CPU type(`0x01000007`·`0x0100000c`), 실행 권한, `rhwp/rhwp`·LICENSE·README 두 파일을 검사했다. 다운로드 ZIP의 SHA-256은 API digest와 일치했다.

- Intel archive artifact #11258861049: `a16ed93eec0fd4492ea524aaf37d3ae049435b3eb997b18eac5d9201c4d1d67c`
- ARM64 archive artifact #11258801424: `44ba06103f7013595518109ee01e70500cfedb8fa379d0df28654501d3ce9f68`
- 명령: `gh api repos/edwardkim/rhwp/actions/artifacts/<id>/zip`, `python3 /tmp/rhwp-7556-verify-artifacts.py`, `file`, `sha256sum`.
- 완료된 macOS job 로그는 실제 OS 15.7.9, 두 target의 runner별 새 cache key miss, `rhwp v0.8.6`을 확인했다.

다섯 CLI archive와 wasm-pkg·vscode-vsix·release-publish-evidence, 총 8개 필수 artifact를 확인했다. WASM·VSIX 및 채널 집계는 PASS였고, Release 첨부·네 외부 publish job은 모두 skipped였다. artifact #11259307950의 evidence JSON은 exact SHA·mode=verify·accepted=true·verdict=completed·errors=[]이며 네 채널 모두 verify-only였다. ZIP SHA-256 `ecd695baf6ba38d64143532e930256d2a3544c2b9614dbca9f57c3b506fa1e25`도 API와 일치했다. `python3 /tmp/rhwp-7556-verify-run.py`로 run/job/artifact 원본 API 응답과 evidence를 대조해 PASS였다.

동일 head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/37079714910), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/37079715008), [Adapter](https://github.com/edwardkim/rhwp/actions/runs/37079715009), [Proptest](https://github.com/edwardkim/rhwp/actions/runs/37079714962)가 모두 성공했다. Build & Test·Native Skia·integration A/B/C/D·lint·Frontend package gate의 실제 check success를 확인했다.

## 완료 범위와 다음 단계

사용자 승인 범위의 push·PR 생성·dry-run·실행 검증은 완료했다. 이 결과 기록은 검증한 head 위에 제품·workflow를 바꾸지 않는 문서-only commit으로 추가한다. 최종 후행 head의 CI와 mergeability는 별도로 재확인한다.

**merge·실제 publish·이슈 close는 수행하지 않았다.** merge는 작업지시자 승인 후 진행한다. 정상 devel → main 승격과 다음 release tag에 수정 workflow가 포함됐는지 확인할 때까지 #7555는 OPEN으로 유지한다. 오래된 macOS 버전에서의 바이너리 실행 호환성은 이번 runner 전환 검증의 실행 범위가 아니다.

## 복구

지원 중인 macOS ARM64 image에서 동일 target의 build·실행을 다시 확인한다. macOS 14로 revert하는 것은 brownout/종료 이후 유효한 복구가 아니다. 검증 실패 시 trigger·권한·Release 조건을 우회하지 않는다.
