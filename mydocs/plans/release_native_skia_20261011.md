# 공식 CLI의 Native Skia 누락 보정

## 근거와 범위

메인터너가 공식 릴리즈 CLI의 `export-png` 오류를 보고했다. 기준 devel은
`8761ac90270a5dddfa0860e3de700ee83d2bf7c8`, 작업 브랜치는
`fix/release-native-skia-20261011`이다. 공개 v0.8.7 Linux x86_64 archive를 다운로드하고
공식 SHA256SUMS와 대조한 뒤 실제 바이너리에서 `export-png` exit 2와 Native Skia 누락 안내를
확인했다. `capabilities --search export-png --json`도 `available:false`다.

`Cargo.toml`의 native-skia는 선택 feature다. v0.8.7과 현재 Release Binary workflow 모두
feature 없이 빌드하고 `--version`만 검사한다. renderer 결함과 배포 빌드 계약 누락을 구분한다.

O3 실행 계약 변경으로 다섯 target의 native-skia 빌드, Linux 의존성 설치와 실제 archive의
PNG capability·첫 쪽 출력 smoke gate를 추가한다. renderer·Cargo 기본 feature·WASM·버전·태그·
Release asset·npm/VS Code/브라우저 배포는 변경하지 않는다.

## 검증과 적용 순서

1. 기존 공식 archive에서 새 smoke gate 실패를 확인한다. 기존 workflow에서 새 계약 검사도
   누락 feature 때문에 실패해야 한다.
2. 공유 `target/pr-review`에서 locked release Native Skia CLI를 빌드하고 같은 archive 구조로
   패키징해 smoke gate 통과를 확인한다. PNG 형식/양수 크기 검사는 배치 피델리티를 주장하지 않는다.
3. 관련 release 계약 검사·workflow 문법/링크 검사를 실행한다. Rust source는 변경하지 않으므로
   조판 전수 회귀·새 WASM·새 시각 golden을 이 운영 수정의 필수 검사로 추가하지 않는다.
4. 원격 반영 승인 뒤 devel PR의 CI와 exact head Release Binary `tag=test` 다섯 플랫폼 결과를
   확인한다. Linux ARM, macOS ARM/Intel와 Windows는 이 로컬 Linux 검증으로 대체하지 않는다.
5. 정규 devel → main 통합 뒤 승인된 tag의 실제 배포 archive를 확인한다. 기존 v0.8.7을
   그대로 재실행해도 이 변경이 적용되지 않는다. 기존 asset 교체는 별도 배포 결정이다.

## 복구

실제 matrix 빌드나 archive 실행이 실패하면 해당 플랫폼 원인을 보정한다. PNG gate를 삭제하거나
실패를 허용해 배포하지 않는다. 변경 철회가 필요하면 이 workflow·script·관련 문서 diff를 revert하며,
이미 게시된 Release/태그를 로컬 변경의 부수 효과로 수정하지 않는다.

## 현재 결과

기존 공식 archive: PNG 명령 exit 2, 새 archive smoke exit 1. 새 workflow 계약을 v0.8.7에
연결한 검사도 feature 누락으로 FAIL, 환경 오류는 없다. 관련 release 계약 검사 26건 PASS.
원문 로그는 ignored `output/pr-review/release-native-skia-20261011/`에 보존한다.
`cargo build --locked --release --bin rhwp --features native-skia --target-dir
/home/edward/mygithub/rhwp/target/pr-review`는 최적화 빌드 exit 0(9분 16초)이다. 공식과 같은
archive 구조의 `rhwp-test-linux-x86_64.tar.gz`에서 CLI를 꺼내 새 smoke script를 실행한 결과
exit 0, PNG capability true, 794×1123 PNG 생성이 확인됐다. 직접 CLI 실행도 exit 0이다.
archive·binary·PNG SHA-256은 `native-smoke-final.json`에 보존했다.

관련 release 계약 26건 및 promotion 계약 35건(총 61건) PASS. Python AST, YAML 5-target
구조 및 명시적 Bash step 문법, 문서 상대 링크·git diff 검사도 통과했다. 자동 검사는
기존 CI Lint의 release channel policy 단계에 연결되어 있다. Rust renderer source는 동일하므로
이번 빌드 계약 보정을 새 조판 개선·Native/fresh WASM 시각 일치 주장으로 확대하지 않는다.

미검증: Linux ARM, macOS ARM/Intel, Windows의 실제 빌드/실행 및 원격 exact-head CI.
최초 로컬 검증 종료 당시 원격 push·PR 생성·workflow dispatch·게시 Release asset 교체·tag 변경은 수행하지 않았다.
한국어 PR 초안 `output/pr-review/release-native-skia-20261011/pr-body-ko.md`은 ignored 로컬
증적에 준비했다. 문서의 외부 링크 대신 실제 GitHub 게시 승인 후 본문을 전송한다.


## 이슈 등록과 PR 생성 승인

메인터너가 이슈 등록 후 PR까지 진행하도록 승인했다. 중복 열린 이슈를 증상 문자열로
검색한 뒤 [#7817](https://github.com/edwardkim/rhwp/issues/7817)을 등록했다. assignee=edwardkim,
milestone=v1.0.0, labels=bug/cli/ci다. 한국어 본문을 API 응답과 byte-equivalent하게 대조했다.
최신 devel은 여전히 `8761ac90270a5dddfa0860e3de700ee83d2bf7c8`이며 source 보정은 없다.

기본 경로는 collaborator_self_merge, 보조는 intake_and_review/local_validation이다.
운영 O3 검증은 github_operations.md에 따른다. PR 번호 기반 review/오늘할일은 정확한
code head CI 성공 후 작성한다. 이번 승인 범위는 이슈 등록·작업 branch push·devel PR 생성이며
merge와 기존 Release asset 교체는 포함하지 않는다.

## CI 오류 보정

메인터너가 CI 종료 뒤 오류 수정을 지시했다. PR #7818의 head
`efb169e57a9c123bdc23735f16d6c2f2cf4f4bfe`에서
[CI run 38120641361](https://github.com/edwardkim/rhwp/actions/runs/38120641361)의
Lint `Validate workflow contracts`가 이전 release 빌드 명령을 요구하는
`test_nextest_archive_workflow.py`의 검사 1건으로 실패했다. 마지막 Build & Test 실패는
Lint 실패를 전달한 aggregate이며, 네 archive의 build/worker, Native Skia와 Frontend package는
성공했다. [CodeQL run 38120641374](https://github.com/edwardkim/rhwp/actions/runs/38120641374)도
네 언어 분석 모두 성공했다. Clippy/WASM 검사는 선행 실패로 실행되지 않았다.

이 검사를 승인된 locked Native Skia release 계약으로 갱신했다. 기존 빌드/target 요구를
유지하면서 `--locked --features native-skia`를 함께 요구하며 실패 허용이나 게이트 제거는 없다.
수정 전 같은 로컬 nextest workflow 검사 15건 중 1건 FAIL, 수정 후 15건 PASS다.
CI의 `Validate workflow contracts` 명령을 그대로 실행해 Node 335건과 Python 205건 모두
PASS, 그 뒤의 adoption/planner/org/frame/reality 계약 26건도 PASS다. 테스트 변경은 Python
workflow 계약에 한정하므로 Rust 재빌드·새 WASM/시각 출력은 반복하지 않는다.

로그는 기존 ignored 증적 경로의 `ci-lint-failed.log`, `ci-aggregate-failed.log`,
`nextest-contract-before.log`, `nextest-contract-after.log`,
`workflow-contracts-ci-step-after.log`, `lint-later-contracts-after.log`에 보존한다.
새 head를 push한 뒤 CI/CodeQL을 다시 확인해야 하며, 이전 head의 성공을 새 head의 CI 성공으로
대체하지 않는다. 다섯 플랫폼 Release Binary 검증과 merge는 아직 수행하지 않았다.

## Self-review 검증

메인터너가 CI 완료 뒤 self-review 진행을 지시했다. code SHA
`95ca4097e486892499f0d9a281f5aa2a1c4c2ea8`의 CI 38122117591, CodeQL 38122117572,
Adapter 38122117587, Proptest 38122117564를 API로 재조회했으며 모두 success다.
CI Lint·Native Skia·Frontend package·네 archive build/worker와 GHAS CodeQL check도 success다.
정확한 code head가 녹색이 된 뒤 `mydocs/pr/archives/pr_7818_review.md`와 오늘할일 기록을 준비했다.

Release Binary는 일반 PR CI에서 실행되지 않으므로 동일한 branch/head에서 `tag=test` 모드로
[38123523720](https://github.com/edwardkim/rhwp/actions/runs/38123523720)을 dispatch했다.
다섯 플랫폼 모두 build·실제 archive PNG gate·artifact 업로드 success를 API와 로그로 확인했다.
공식 CLI 검증 범위의 다섯 플랫폼 결과가 충족됐으므로 문서-only trailing commit을 반영한다.
기존 reusable 후속 package verify(WASM/VSIX)는 별도로 진행 중이며 전체 run success로 보고하지 않는다.
이 테스트 모드에서는 Release attach와 외부 package publish가 skip되며 실제 배포는 하지 않는다.
