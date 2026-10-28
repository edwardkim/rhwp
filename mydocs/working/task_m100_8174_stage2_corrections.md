# #8174 2단계 보완 — CLI·Rust 소비 수정

Issue: [#8174](https://github.com/edwardkim/rhwp/issues/8174)

- 착수 승인: 2026-10-23 사용자 「수정지점과 우선순위를 착수하는 승인을 합니다.」
- 착수 head: `f4342c4f1`.
- 제품 base: `def947c2149df5bc812d801e6e79be19a403b4d1`. 원격 `devel`과 일치함을 확인했다.
- 근거: [구조 재조사 A01–A07](../tech/investigations/issue-8174/README.md),
  [결정 대장·범위 v1](../plans/task_m100_8174_maintainer_decisions.md).
- 상태: **로컬 기술 보완 범위 v1과 PR 제출 전 로컬 게이트 완료. 실제 게시 준비 전체의 완료는 아니다.**
- 검증한 code head: `9df631769bd668d3e4b86f86d1359689a108759b` (`5cb42af13f`의 보완 구현과 Cargo include anchor 보정).

## 실행 순서와 완료 조건

1. **A01:** 기존 8개 crate 경계를 유지한다. 내부 path에 버전을 병기하고 `rhwp-svg2pdf`를 직접
   연결해 root patch 의존을 없앤다. 라이선스·고지·내장 파일을 포함하는 Cargo 파일 선별과 실제
   `.crate`를 확인한다. 로컬 검증 registry에서 포장한 source를 소비하며 repository path나 patch
   fallback으로 통과시키지 않는다. 기존 `publish = false`와 제품 0.8.7은 이번 보완에서 유지한다.
2. **A02:** `HwpError`에 표준 `Error`를 추가한다. variant·표시 문구는 유지하고 이미 재현한
   외부 소비 앱의 E0277을 수정 후 컴파일·실행으로 확인한다.
3. **A03·A06:** 기존 parser/IR/service/core와 형식별 저장·손실 보고·설치 bin의 사용 안내를 작성한다.
   `font-metric-gen`은 기본 설치에서 제외할 tools feature로 구분하고 기존 생성 명령을 함께 보정한다.
4. **A04:** `export-text`의 로더만 core 경로로 연결한다. 같은 core의 텍스트 추출을 사용하며
   실제 기본 출력·JSON·페이지 선택·절단·잘못된 입력·인자의 exit/stdout/stderr를 기존 CLI와 대조한다.
5. **A05·A07:** 현재 최소 feature가 parser-only가 아님을 안내하고 추가 crate type의 산출물을
   확인한다. 새 경량 feature·바인딩 crate 분리는 필요성과 비용을 확인하기 전 구현하지 않는다.

기존 PDF 보안 fork의 코드·기능 제한·feature 기본값·WASM start/export를 유지한다.
이번 실행은 원격 push·PR 생성·실제 registry 게시·인증 설정을 포함하지 않는다.

## 결과와 남은 조건

### A01 — 실제 package와 registry 소비

내부 path 3개에 버전을 병기하고 SVG 출력 fork를 `rhwp-svg2pdf 0.13.0-rhwp.1`의 직접 의존성으로
연결했다. root patch는 제거했다. root·내부·vendor의 Cargo include를 지정하고 내부 crate의
LICENSE·README와 패키지 사용자용 README를 추가했다. 제품은 0.8.7, 내부/fork 7개는
`publish = false`를 유지했다. 0.9.0은 다음 제품 릴리즈 값이며 이번 검증의 실제 버전과 구분한다.

실제 `cargo package`에서 `README.md`·`LICENSE` 같은 basename 패턴이 하위의 동명 파일까지
선택하는 문제가 검출됐다. 로컬 `node_modules`·venv의 파일이 dirty guard에 잡혔으며,
이를 `--allow-dirty`로 우회하지 않았다. Cargo include 전부를 `/`로 anchor한 뒤 깨끗한 code head에서 재생성했다.

8개 모두 Cargo가 정규화한 실제 `.crate`다. `cargo package --no-verify --registry crates-io`로
생성한 뒤 로컬 sparse source replacement에 등록해 다운로드·컴파일·실행했다. 업로드 API나
`cargo publish`는 실행하지 않았다. `--no-verify` 생성을 Cargo 자체 verify 성공으로 표시하지 않는다.

| package | 실제 버전 | 파일 수 | `.crate` bytes |
| --- | --- | ---: | ---: |
| rhwp-contracts | 0.1.0 | 11 | 31,968 |
| rhwp-ooxml-chart | 0.1.0 | 14 | 103,883 |
| rhwp-password-crypto | 0.1.0 | 7 | 15,862 |
| rhwp-jpeg-decoder | 0.3.2-rhwp.1 | 27 | 53,248 |
| rhwp-zune-jpeg | 0.5.15-rhwp.1 | 39 | 92,032 |
| rhwp-svg2pdf | 0.13.0-rhwp.1 | 27 | 59,076 |
| rhwp-pdf-import-backend | 0.3.78-rhwp.1 | 412 | 6,878,877 |
| rhwp | 0.8.7 | 1,796 | 8,318,522 |

archive의 각 source 파일을 Git의 파일 목록·blob과 대조했다. 생성 manifest/lock/VCS 정보 이외에는
committed blob과 바이트가 일치했고, 예정에 없는 파일과 필수 파일 누락도 검사했다.
독립 consumer의 manifest에는 path/patch가 없으며, 선택한 8개 package는 모두 registry source다.
본체·vendor·내부 crate의 checkout에서 링크하지 않았다. 각 `.crate`의 SHA-256과 실제 다운로드한
source 경로는 `registry-result.json`에 있다.

### A02·A03 — Rust API와 사용 예제

`HwpError`에 `std::error::Error`를 구현했다. variant·Display·기존 From 변환은 유지했다.
새 공개 API 회귀 `tests/cases/issue_8174_rust_error_consumer.rs`는 실제 열기 실패를
`Box<dyn Error + Send + Sync>`에 `?`로 전달하고 `HwpError`로 downcast할 수 있는지 확인한다.
기존 독립 probe는 E0277로 실패했고, 수정 후에는 path와 registry 소비의 같은 회귀가 각각 1/1 통과했다.

[패키지 README](../../packaging/crates-io/README_PACKAGE.md)에 parser/IR과 service/core의 차이,
HWP/HWPX 출력 선택, 손실 보고의 범위, feature·MSRV 검증 경계, CLI 설치 방법을 기록했다.
게재한 Rust 예제 2개는 그대로 추출해 registry 버전으로 컴파일·실행했다. 새로운 facade는 추가하지 않았다.

### A04 — CLI의 core 직접 소비

바뀐 호출 경로는 `export-text → load_document_core → DocumentCore::from_bytes* → extract_page_text_native`다.
기존 facade도 같은 core를 만들고 동일한 추출 메서드를 소비했다. 로더의 password 분기,
`LoadError` 분류와 exit mapping, JSON 구성·절단·파일 출력은 변경하지 않았다.

동일한 cwd·입력·인자로 base CLI와 최종 registry 설치본을 대조했다. 일반 HWP/HWPX,
페이지 선택, JSON 인자 위치, 절단, 잘못된 입력·인자와 암호 HWPX의 올바른 password·오입력·
미지정을 포함한 **17개 사례에서 exit/stdout/stderr가 일치**했다. password 미지정은 기존 계약의
exit 2이며 오입력은 exit 1이다. 다른 CLI 명령은 점진적 이관 대상으로 남겼으며,
CLI 전체가 `wasm-api`에서 독립했다고 기록하지 않는다.

### A05–A07 — feature·개발 bin·추가 산출물

- 기존 모든 feature와 기본값을 유지했다. `default-features = false`를 parser-only로 안내하지 않으며,
  renderer/PDF export를 새로운 feature로 분리하지 않았다.
- `font-metric-gen`에 additive한 `dev-tools` required feature를 붙여 기본 CLI 설치와 구분했다.
  기존 generator 검사의 build 인자·공용 target 실행 파일 참조와 매뉴얼도 함께 보정했다.
- 기존 generator 검사 5개 중 1개는 기대 버전을 `0.8.4`로 고정했다.
  #4964 → 원 보정 `f5f71f8c062719e73a70c2115f4265851e928758` → 통합 PR #5938
  `bf30bd792b9198cec23ce5bb0f897f7d94bf6463`을 추적했다. 당시 manifest는 0.8.4였고,
  생산자는 `env!("CARGO_PKG_VERSION")`을 사용했다. 검사를 현재 manifest와 대조하도록 보정하고
  metrics·provenance 검사를 유지했다. 최종 결과는 5/5 통과다.
- library-only consumer에서도 `rlib`와 `cdylib`가 함께 생성됐다. dev/debug=1의 관측에서
  rlib는 377,288,368 bytes, 추가 `.so`는 321,285,112 bytes였고 hard link는 각각 1개였다.
  이는 추가 파일의 보관량이며 컴파일 시간·실행 속도·release 크기의 비교가 아니다.
  이 관측만으로 바인딩 crate를 재분리하지 않고, 비교 측정이 필요한 후속 후보로 남겼다.

### 기능 보존과 검증

| 검사 | 결과·범위 |
| --- | --- |
| source 사전 검사 | 8개 후보·14개 내장 참조·5개 font의 source/고지/실체 검사 통과. `publish_ready=false` 유지 |
| source 검사 도구의 계약 | 18/18 통과 |
| 신규 Rust 오류 회귀 | path와 registry에서 각각 1/1 통과. review checkout의 자동 suite 수집·base 비교도 통과 |
| Rust 소비 | registry의 최소·기본 구성에서 parse → HWP/HWPX 저장·재독 → service/core 편집·재독 통과. 최상위 본문과 marker의 smoke 범위 |
| README의 Rust 예제 | registry 버전으로 2개 모두 compile/run 통과 |
| CLI 설치 | repository path 없이 로컬 registry에서 `cargo install rhwp --version =0.8.7 --bin rhwp --debug --locked` 통과. CLI text JSON이 base와 일치 |
| CLI 보존 | 최종 설치본에서 일반 14개＋암호 3개의 총 17개 사례, exit/stdout/stderr 일치 |
| 기존 PDF 입력 | 공개 입력에서 생성한 PDF를 base와 registry CLI의 `info --json`에 전달해 PDF/1페이지와 JSON 일치 확인 |
| PDF 출력 | 대표 1페이지의 base와 최종 registry CLI 출력이 같은 cwd에서 바이트 일치. 12,599 bytes, SHA-256 `4d6fc1c51a36ad4e90a02d4f71834ef9c764a8098e6e3dba9395a0d6b8b01548` |
| generator 계약 | 5/5 통과 |
| suite policy Node 검사 | 23/23 통과 |
| Format | 최종 code head의 review 전용 checkout에서 `cargo fmt --all -- --check` 통과 |
| Native Clippy | `-p rhwp --lib --bin rhwp -- -D warnings` 통과 |
| WASM Clippy | `-p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` 통과 |
| registry package의 rustdoc | `cargo doc -p rhwp --lib --no-deps` 통과. 실제 docs.rs 서비스 실행과 구분 |

parser/model/renderer/DocumentCore·PDF backend/decoder·font bytes는 base에서 변경하지 않았다.
SVG fork는 직접 path에 연결한 결과 `cargo fmt --all`의 대상이 되어 stable formatting만 적용했다.
base의 16개 source에 같은 formatter를 적용한 결과와 전 파일이 바이트 일치했다.
출력 본문·수치·fallback 조건을 보정하는 변경은 추가하지 않았다.

최초 PDF 대조에서는 cwd가 달라 repository bundle에 따른 Regular/ExtraLight font 선택 차이가 났다.
같은 cwd로 맞춰 최종 package와 base의 바이트 일치를 확인했다. 이를 package 결함이나
Visual Sweep 통과로 세지 않는다. 독립 Print PDF에 대한 새로운 시각 충실도를 주장하지 않는다.

기본 작업공간의 오래된 ignored suite가 제거된 원본을 참조해 최초 전체 fmt 검사가 중단됐다.
review 전용 sparse checkout에서 canonical 절차로 harness를 준비하고 최종 code head의 fmt와
suite 검사를 완료했다. 생성물을 커밋하거나 기존 검사를 완화하지 않았다.

1차 증거는 ignored `output/pr-review/issue8174/architecture-review/corrections/`에,
최종 source 사전 검사는 `output/pr-review/issue8174/source-preflight-corrections-final/`에 보관했다.
실행한 [로컬 registry 진단 source 원문](../tech/investigations/issue-8174/probes/registry_verify.py.txt)도
보존했다. 이 파일은 이력 자료이며 release workflow가 아니다. 재실행할 때는 source SHA,
baseline, 출력의 초기 상태와 기존 설치본을 확인하고 이전 결과를 재사용해 완료로 판정하지 않는다.

임시 review worktree는 검증 뒤 tracked 변경이 없음을 확인하고 제거했다. 약 769 MiB를 회수했고,
기본 작업공간·공용 target·실행 증거는 보존했다. 현재 등록된 worktree는 기본 작업공간 하나다.

## 남은 게이트

본체 0.9.0의 제품 버전 전환, MD01·MD05–MD09의 정책, 공개 API의 장기 호환성·하위 MSRV·
추가 target, publish 제한 해제, 실제 crates.io 등록·owner·인증은 이번 완료에 포함하지 않는다.
기존 lock의 `chacha20 0.10.1` yank 경고도 제거하지 않았다. source/파일 선별 개선을 미실행 감사나
보안 보증의 대용으로 삼지 않는다.

## PR 제출 전 로컬 검증 — 2026-10-23

사용자의 「PR 준비 하세요」 지시에 따라 local_validation 4.3의 필수 검증을 실행했다.
기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`과
입력 보존을 위한 `visual_fixture_evidence`다. 현재 계정 `edwardkim`의 열린 PR은 없었고,
fetch한 `upstream/devel`은 기존 base `def947c2149df5bc812d801e6e79be19a403b4d1`과 같았다.
이번 준비에서 rebase로 추가 반영한 PR은 없다.

별도 review worktree의 검증 head는 `b1a5695479d61322c1b61cc771cfbc2c2b375cb3`다.
앞서 포장·소비한 code head `9df631769bd668d3e4b86f86d1359689a108759b`와 제품·패키지
입력의 Git blob 목록이 같음을 확인했다. Cargo는 모두 공용 `target/pr-review`를 사용했고,
jobs 8과 전체 회귀 test threads 8을 선택했다. 명령·시간·종료 코드는 ignored
`output/pr-review/issue8174/pr-preparation/*.command.json`과 로그에 남겼다.

| 필수 검사 | 실제 결과 |
| --- | --- |
| review 전용 suite 준비·base 비교 | `--prepare`, `--check --base-ref def947c2149df5bc812d801e6e79be19a403b4d1` 통과 |
| Format | `cargo fmt --all`, `cargo fmt --all -- --check` 통과 |
| Native Clippy | `cargo clippy --locked -- -D warnings` 통과 |
| WASM Clippy | `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` 통과 |
| workspace build | `cargo build --locked --workspace` 통과 |
| workspace all-target Clippy | `cargo clippy --locked --workspace --all-targets -- -D warnings` 통과 |
| 신규 오류 회귀 | `node scripts/run-rust-test.mjs issue_8174_rust_error_consumer -- --cargo-profile release-test`의 실제 suite 실행에서 1/1 통과 |
| release-test 전체 | `cargo nextest run --locked --cargo-profile release-test --tests --test-threads 8 --no-fail-fast`: 55개 binary, 11,363/11,363 통과·45개 기존 제외·실패 0. 컴파일 5분 43초, 테스트 669.611초 |

조판 규칙·측정·배치·paint·WASM 출력 동작을 수정하지 않아 Native Skia 3종·fresh WASM
Canvas·Visual Sweep은 이번 변경의 필수 범위에 해당하지 않는다. 위의 source 동일성,
SVG formatting 대조와 포장 CLI의 실제 소비 증거를 적용 근거로 삼으며 한컴 충실도 개선을
주장하지 않는다. 기존 렌더링 회귀·golden·허용치를 변경하지 않았다.

### 재현 입력의 보존과 실제 대조

CLI 비교에서 사용한 합성 자료를 [정식 fixture](../../tests/fixtures/issue8174/README.md)와
[PDF 입력](../../pdf/issue8174/package-input.pdf)에 보존했다. 암호 HWPX는 같은 공개 빈 문서에
공개 테스트용 암호를 설정해 다시 생성했다. PDF는 기존 CLI가 만든 입력이며 Producer는 `rhwp`다.
각 파일의 출처·생성 방식·SHA-256은 fixture README에 기록했다.
같은 cwd에서 base와 registry 설치본의 SVG PDF를 다시 생성했고, 두 출력과 보존한 PDF의
바이트가 일치했다. 정확한 생성 인자·cwd·해시는 `pdf-output-parity.json`에 남겼다.

이 정식 경로의 같은 입력으로 기존 CLI와 포장한 registry 설치본을 다시 실행했다.
텍스트 17개와 PDF 입력 1개, 총 **18개 사례의 exit/stdout/stderr가 일치**했다.
`cli-parity-final.json`에 실제 인자·입력 해시·두 실행 파일 해시를 남겼다. 이전 output 경로의
합성 자료를 최종 재현 경로로 쓰지 않는다. 새 입력은 CLI의 구조·오류 계약 자료이며 렌더링
회귀나 독립 한컴 Print PDF의 역할을 부여하지 않는다.

기존 source 사전 검사 18/18·generator 5/5·suite policy 23/23은 같은 source에서 통과한
결과를 재사용했다. 변경 Markdown의 상대 링크와 장기 문서 6개의 메타데이터도 확인했다.
PR 본문·명령 초안은 ignored `output/pr-review/issue8174/pr-preparation/`에 작성했다.
원격 push·PR 생성은 별도 승인 단계이며 이번 준비에서는 실행하지 않았다.

검증 전용 worktree의 tracked 변경이 없음을 확인한 뒤 제거해 약 6.4 GiB를 회수했다.
현재 등록된 worktree는 기본 작업공간 하나이며 공용 target·검증 로그는 보존했다.

## Self-review 보정 — 독립 Skia host 도구

2026-10-23 「self review 로 진행합니다」 지시에 따라 [PR #8181](https://github.com/edwardkim/rhwp/pull/8181)을
생성하고 검토했다. 첫 후보 `06886117e5d272134a2108a50d5001075b0ae71d`의 독립 소비 경로에서
`tools/skia-surface-probe`의 오래된 root patch가 새 fork 이름과 충돌하는 결함을 발견했다.

- 선행 근거: #8097의 host Surface 보완을 통합한 [PR #8135](https://github.com/edwardkim/rhwp/pull/8135),
  실제 merge `a6e9d18fc1061318055b4eb348daec201427ea03`과
  [기존 검토의 software Vulkan 증거](../pr/archives/pr_8097_review.md)를 확인했다.
  당시 독립 workspace의 patch는 root patch가 path 소비자에게 전파되지 않는 구조를 보완했다.
- 기준 devel `def947c2149df5bc812d801e6e79be19a403b4d1`의 같은 도구는
  `cargo metadata --locked --offline`에 exit 0이었다. 첫 후보에서는 patch가 `svg2pdf`라는
  package를 찾지만 해당 경로의 실제 이름이 `rhwp-svg2pdf`여서 exit 101로 실패했다.
- 이제 rhwp가 fork를 직접 의존하므로 도구의 중복 patch를 제거했다. Cargo가 생성한 도구 lock의
  변경은 `svg2pdf 0.13.0` → `rhwp-svg2pdf 0.13.0-rhwp.1`뿐이며 다른 package 버전은 같았다.
- 수정 뒤 같은 `cargo metadata --locked --offline`과 도구의 fmt 검사가 통과했다.
  `CARGO_TARGET_DIR=/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=8`,
  `VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`으로
  `cargo run --locked --manifest-path tools/skia-surface-probe/Cargo.toml`을 실제 실행했다.
  llvmpipe의 software Vulkan에서 내부색·host state 복원·잘못된 인자의 pixel 보존 3개 계약을
  모두 통과했고 `status=passed`, `hardwareAccelerationVerified=false`였다.

처음 검토에서 독립 도구의 patch 확인이 누락됐다. 이번 보정은 도구 manifest·lock에 한정하며
Rust source·기존 회귀·root lock·실제 조판과 렌더링 로직은 변경하지 않았다. 본체 패키지의
전 파일 Git blob이 검증본과 같으므로 기존 전체 회귀·Rust lint·registry 소비 결과를 재사용한다.
추가 검증은 실제 결함이 발생한 독립 도구 경로를 대상으로 했고, 새 head의 GitHub CI는 별도로
확인한다. 이전 후보의 CI 성공으로 새 head를 통과 처리하지 않는다.

수정 전후 명령·종료 코드·tool lock 대조·실제 Surface 결과는 ignored
`output/pr-review/issue8174/pr-preparation/self-review/`에 보관했다.
