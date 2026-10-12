# PR #7818 self-review — 공식 CLI Native Skia 빌드 계약

## 최종 판정

**승인 — 공식 CLI 빌드 계약과 다섯 플랫폼의 실제 release archive 검증을 충족했습니다.**
코드 후보의 PR CI와 CodeQL도 성공했습니다. 문서-only trailing head의 CI와 메인터너
merge 승인은 별도 남은 조건입니다. 기존 Release asset 교체·실제 배포 완료를 뜻하지 않습니다.

## 접수와 범위

- 작성자 `edwardkim`의 self-review입니다. reviewer는 지정하지 않습니다.
- 관련 [이슈 #7817](https://github.com/edwardkim/rhwp/issues/7817),
  [PR #7818](https://github.com/edwardkim/rhwp/pull/7818), base `devel`.
- 검증한 code SHA `95ca4097e486892499f0d9a281f5aa2a1c4c2ea8`, base SHA
  `8761ac90270a5dddfa0860e3de700ee83d2bf7c8`. 작성 시점 PR은 Open/non-draft,
  mergeable=true/clean입니다. merge 직전에 재조회합니다.
- 기본 경로 `collaborator_self_merge`, 보조 `intake_and_review`, `local_validation`,
  `review_only_fast_pass` 및 GitHub 운영 O3 절차입니다.
- 다섯 release target에 locked Native Skia feature를 포함하고 Linux 의존성을 설치합니다.
  archive에서 실제 CLI를 꺼내 PNG capability와 첫 쪽 출력 성공을 검사한 뒤 업로드합니다.
  Rust engine·Cargo 기본 feature·WASM·버전·tag·기존 Release asset은 변경하지 않습니다.

## 코드 검토

| 항목 | 판정과 근거 |
| --- | --- |
| 공식 빌드 feature | 충족: 공통 Build release binary가 `--locked --features native-skia --target`을 요구합니다. 다섯 target 모두 같은 step을 사용합니다. |
| archive 실행 gate | 충족: 공통 Verify packaged PNG export에 플랫폼별 skip이 없고 Upload artifact보다 먼저 실행됩니다. capability 미지원·명령 오류·PNG 없음은 nonzero exit입니다. |
| archive 입력 | 충족: tar의 정규 파일 또는 ZIP의 정확한 `rhwp/rhwp[.exe]` 하나만 임시 파일에 기록합니다. archive 전체를 extract하지 않습니다. 실행 timeout은 120초입니다. |
| renderer/조판 원칙 | 비해당: production renderer·layout·parser·Rust test/golden은 변경하지 않습니다. 빌드된 CLI의 기능 가용성 검증이며 한컴 배치 일치나 신규 시각 회귀를 주장하지 않습니다. |
| 배포 경계 | 충족: manual `tag=test`는 Release attach와 네 외부 package publish를 skip합니다. 정상 tag 배포와 devel → main workflow 승격 정책은 유지합니다. |
| CI 계약 보정 | 충족: 기존 nextest workflow 검사도 locked Native Skia 빌드 명령을 요구하게 갱신했습니다. assertion 제거·실패 허용은 없습니다. |

## 검증 입력과 로컬 결과

입력은 기존 공개 `samples/issue2527_empty_linesegs.hwpx`입니다. 실제 파일과 code SHA의
Git blob SHA-256 모두 `43b3090f33370cc678da8ed3e55e32901ccb379b50153114ae41fa3984006f10`입니다.
첫 쪽 PNG의 signature/IHDR/양수 크기를 확인하는 smoke이며, 관측된 794×1123를 golden으로
고정하지 않습니다. 독립 Print PDF·Visual Sweep은 조판 변경이 없어 비해당입니다.

- 공식 v0.8.7 Linux x86_64 archive의 공식 SHA256SUMS 대조 후 `export-png` exit 2,
  capability false 및 새 archive smoke FAIL을 확인했습니다.
- 공유 target에서 `cargo build --locked --release --bin rhwp --features native-skia
  --target-dir /home/edward/mygithub/rhwp/target/pr-review` 성공. 같은 구조의 로컬 archive에서
  실제 CLI capability true와 PNG 출력 exit 0을 확인했습니다.
- release/promotion 계약 61건 PASS. CI 보정 전 nextest workflow 15건 중 1건 FAIL,
  보정 후 15건 PASS. CI Validate workflow contracts 전체 Node 335·Python 205건 및
  후속 정책 검사 26건(총 566건) PASS. Python/YAML/Bash 문법·문서 링크·diff 검사 PASS.
- 원문 증적은 ignored `output/pr-review/release-native-skia-20261011/`에 보존합니다.
  기존 공식 archive SHA-256 `24de2bdaa0b69f86302a27e7fdfb570ad3c53b0ab3a4eac06446adc651dfb988`,
  로컬 보정 archive `4766f860442a5c97ca9ddbe238d8fb1c521ebda1bc3bb621c3870cd26ea8e21d`입니다.

## 정확한 code head의 원격 검증

| 실행 | 결과 |
| --- | --- |
| [CI 38122117591](https://github.com/edwardkim/rhwp/actions/runs/38122117591) | success: Lint(fmt/Clippy/WASM check), Native Skia, Frontend package, 네 archive build·worker, Build & Test가 모두 성공했습니다. |
| [CodeQL 38122117572](https://github.com/edwardkim/rhwp/actions/runs/38122117572) | success: Actions/Python/JavaScript·TypeScript/Rust Analyze 및 별도 GHAS CodeQL check 성공. |
| [Adapter 38122117587](https://github.com/edwardkim/rhwp/actions/runs/38122117587) | success. |
| [Proptest 38122117564](https://github.com/edwardkim/rhwp/actions/runs/38122117564) | success. |
| [Release Binary 38123523720](https://github.com/edwardkim/rhwp/actions/runs/38123523720) | code SHA에 고정한 `workflow_dispatch/tag=test`. 다섯 target 모두 build·Verify packaged PNG export·archive artifact 업로드 success. Release attach skipped, 후속 package verify 진행 중입니다. |

같은 code head에서 완료된 Full CI를 보존하고, 이미 성공한 광범위 Rust 회귀를 로컬에서
중복 실행하지 않았습니다. 다섯 플랫폼 release build는 일반 PR CI와 별도 검증입니다.

Release Binary run의 Linux x86_64 artifact `11696178351`을 내려받아 archive 내부 CLI의
PNG smoke를 로컬에서도 재실행했으며 exit 0, capability true와 PNG 생성을 확인했습니다.
해당 archive SHA-256은 `a375fa282ba0537d319cd971453902f5a5c5f6bbcd4fff7218b5b24ccf8779bb`이며,
`ci-linux-x86_64-artifact-sha256.json`과 `ci-linux-x86_64-local-smoke.json`에 run/source SHA를
연결해 보존했습니다.

## 남은 조건과 후속 처리

다섯 플랫폼 archive gate가 성공했으므로 허용된 review/오늘할일 문서만 single-parent trailing commit으로
반영합니다. 최신 base와의 merge-tree·문서 링크·기존 오늘할일 보존을 확인하고 push하며, 새 head의
trusted reuse/required check·mergeability를 재조회합니다. 코드 후보 성공을 trailing CI 성공으로
대체하지 않습니다. merge는 메인터너 승인 뒤 수행합니다.

정규 devel → main 승격 전 기존 v0.8.7 tag 재실행은 옛 workflow를 사용합니다. 기존 Release
asset 교체와 새 tag/실제 배포는 이번 self-review의 실행 결과에 포함하지 않습니다.

다섯 target의 post-cache는 설정된 예산 한도에 도달해 새 캐시 저장을 거부했다는 경고를
남겼습니다. build/PNG gate/artifact 업로드는 성공했습니다. 캐시 저장 성공과 기능 검증 성공을
구분하며, 기존 캐시 삭제나 예산 설정 변경은 이 self-review에서 수행하지 않습니다.
