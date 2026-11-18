# 최종 로컬 검증 기록

측정일 2026-11-09 KST. 기준 `024d610633ede166ebcbb915f9c90e7a9d0649d8`, 제품·의존성·workflow source `2320351d2325a41d900e91f4e07f559d4a924a7d`, 추가 smoke·고지 보정 `42f82843ec5c957e4e06c414e7f9d0ade7cf052b`. [source 연결](source-review.md)을 따른다.

## Rust 실행

Rust 1.93.1, Node 24.15.0, WSL Ubuntu 24.04. 공유 target에 Cargo 작업을 순서대로 실행했다. CARGO_BUILD_JOBS=12.

| 검사 | 실제 명령 | exit | 초 | 원 실행 로그 SHA-256 |
| --- | --- | --- | --- | --- |
| fmt | `cargo fmt --all -- --check` | 0 | 26.595 | e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 |
| root-clippy | `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 0 | 130.43 | 9be0b4ef28973778663895cc3f6fd0bfe17b396351c8514a6709cffd35f0e9b3 |
| wasm32-clippy | `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` | 0 | 126.278 | 1f2b94eeadc9edbbb22974f5bbdb5170581d4c87de2546aa5eff70fc13d327c6 |
| workspace-build | `cargo build --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --workspace` | 0 | 209.405 | fb401f8b7df6aedb818a04e8e6cd8aec8a100e6392be75d5057dd389d1547c33 |
| workspace-clippy | `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --workspace --all-targets -- -D warnings` | 0 | 163.789 | 874e728059dc9e1f28b683ced9e29f5d8fd3240313a26998245746c422655171 |
| suite-policy | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/rust-test-suite-manifest.mjs --check --base-ref 024d610633ede166ebcbb915f9c90e7a9d0649d8` | 0 | 0.877 | a4697db9941ece09c6a6664938e96d0035a92394d100509dffefb0c659ebe6ce |
| storage-crypto-focus | `cargo nextest run --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --cargo-profile release-test --tests -E 'test(cfb) &#124; test(johab) &#124; package(rhwp-password-crypto)' --test-threads 12 --no-fail-fast` | 0 | 715.05 | 5b4d747856f328f76f4fc905f43ec6fb190f3fa494cbed7bdfd265783a584acc |
| full-release-test | `cargo nextest run --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --cargo-profile release-test --tests --test-threads 12 --no-fail-fast` | 0 | 649.231 | 5bd3f56f909fd73b6dbf747de85700f948f5fb9df868423f8c664b26cedefff8 |
| native-prepare | `cargo test --profile release-test --features native-skia --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib --test regression_suite_008 --test regression_suite_013 --no-run` | 0 | 436.053 | 735df978bbfaf74a55be67d2da63ea86972270bf12bc4ca9874173624adb1283 |
| native-lib | `cargo test --profile release-test --features native-skia --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib -- --test-threads 8` | 0 | 19.559 | d0de14e55b8684ef3a2335f1db400df6e74a860757c536293682592ae58a9af3 |
| native-missing-picture | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia` | 0 | 2.878 | 75ecac5af1206b442c29d5819bb378fced8cb6fae578900c82bd6c855904a235 |
| native-direct-pdf | `/home/edward/.nvm/versions/node/v24.15.0/bin/node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia` | 0 | 3.052 | eefbb21ea9e7995c5cc5c09163e53ec5062e4cf892533fe086842d50dcb20054 |
| candidate-native-build | `cargo build --profile release-test --features native-skia --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib --bin rhwp` | 0 | 371.127 | bfe514c798d50b2879dbd12651d91664300c08a8683a265ba83e3ebac6698ac4 |

전체 회귀 13,005 PASS / 기존 45 skip, 집중 저장·암호 83 PASS. Native lib 4,040 PASS / 기존 12 ignored, missing-picture 2 PASS와 direct-PDF 35 PASS를 합쳐 공식 Native 4,077 PASS다. 암호 소비 34건은 전체 회귀의 부분집합이며 합산하지 않는다. 기존 nextest 0.9.137은 권고 0.9.140보다 낮다는 경고가 있었으나 요구 0.9.91을 충족했고 실제 실행은 성공했다.

## 최소 글꼴 환경

회귀의 bwrap prefix 원문이다. 공개 fallback 5개만 노출하며 시스템·개인 글꼴과 생성한 visual cache는 차단했다. alias를 통해 생성 cache를 찾는 확인도 exit 1 / No such file or directory로 차단됨을 확인했다. 글꼴 프로그램은 커밋하지 않는다.

```sh
bwrap --ro-bind /usr /usr --symlink usr/bin /bin --symlink usr/lib /lib --symlink usr/lib64 /lib64 --ro-bind /etc /etc --proc /proc --dev /dev --ro-bind /sys /sys --bind /tmp /tmp --dir /home/edward --bind /home/edward/mygithub/rhwp /home/edward/mygithub/rhwp --bind /home/edward/mygithub/rhwp-worktrees/review-integration-20261109-dependabot-updates /home/edward/mygithub/rhwp-worktrees/review-integration-20261109-dependabot-updates --bind /home/edward/.cargo /home/edward/.cargo --ro-bind /home/edward/.rustup /home/edward/.rustup --tmpfs /usr/share/fonts --tmpfs /usr/local/share/fonts --setenv HOME /home/edward --setenv FONTCONFIG_FILE /home/edward/mygithub/rhwp/output/pr-review/dependabot-20261109/minimal-font/fonts.conf --unsetenv RHWP_FONT_PATH --ro-bind /home/edward/.nvm/versions/node/v24.15.0/bin/node /home/edward/.nvm/versions/node/v24.15.0/bin/node --ro-bind /mnt/wsl/resolv.conf /mnt/wsl/resolv.conf --tmpfs /home/edward/mygithub/rhwp/output/pr-review/native-webfont-cache --chdir /home/edward/mygithub/rhwp-worktrees/review-integration-20261109-dependabot-updates
```

## Web 실행

| 검사 | cwd | 명령 | exit | 초 | 원 실행 로그 SHA-256 |
| --- | --- | --- | --- | --- | --- |
| studio-build | rhwp-studio | `npm run build` | 0 | 6.942 | 2a0574b755d302bd43d0453ef709908b9ad447aa0c7d826a5f460cb6644ef932 |
| chrome-build | rhwp-chrome | `npm run build` | 0 | 2.447 | 76efeb1640e6a2abe2893ef79fd36d13e9ea7dce787befe1b0a5fc9a86178255 |
| firefox-build | rhwp-firefox | `npm run build` | 0 | 2.284 | 850e37db2d233b6eaf4c2e06917f4581e0a5efd822fba027d7faae1a0e95940b |
| vscode-final-compile | rhwp-vscode | `npm run compile` | 0 | 11.797 | 0535a09574d2f5c1cf6d0503b40f95a282a7b291513d089263256830a9842bdc |
| ai-settings-browser | rhwp-studio | `node e2e/run-with-vite.mjs -- node e2e/ai-settings.test.mjs --mode=headless` | 0 | 12.521 | 801fd207c926f155e580208d1a4b8cd2f2f6836593d5652fe01f982d880501a9 |
| ai-panel-browser | rhwp-studio | `node e2e/run-with-vite.mjs -- node e2e/ai-panel.test.mjs --mode=headless` | 0 | 21.272 | e75c56da5ba06afd4cd328f461363ca4ed0f1b3c456f8476a96b8163efca29eb |
| chrome-packaged-smoke | rhwp-chrome | `node e2e/extension-smoke.test.mjs` | 0 | 1.983 | a9ff9bb463f609b0b0ad2067d2b0fedefab324755f81259fe1fdb8d37f3a363e |

Studio `npm test`는 2,778 PASS / 기존 2 skip, TypeScript·네 package build/compile이 통과했다. AI 설정은 20개, AI panel은 94개 assertion을 통과했다. 실제 키·과금 호출 없이 합성 SSE·도구 호출·오류·편집·Undo 응답을 사용했다. 네 디렉터리의 npm ci는 성공했고 Studio만 ignore-scripts 설치 뒤 명시적 rebuild도 성공했다.

처음 브라우저 검사는 외부 pkg symlink가 Vite fs.allow 밖으로 해석되어 403이었다. 동일 산출물을 자신의 worktree pkg에 byte 동일하게 복사해 실제 허용 경로에서 다시 실행했다. Vite 보안 설정을 변경하지 않았다. 기존 성공 build와 실패한 브라우저 검사를 구분했다. 생성 JS glue는 base와 byte 동일하고 두 선언 파일의 nonblank 선언 multiset도 같으며 export 선언 순서만 달랐다. 실제 후보 TypeScript build도 통과했다.

최적화된 후보 WASM을 브라우저 초기화에서 받은 5개 독립 응답의 SHA-256은 모두 `33bbf5fcc887f2d1450ac6ca815793d6c468e717d3aa6ed0775df5010e024a89`와 일치했다.

## 기존 Chrome smoke 순서 보정

원 해결 [#8757](https://github.com/edwardkim/rhwp/pull/8757) → `b2b0b9f8ad3564fc2412a24ce9d7db3eaa63960c` → merge `0e6cfe91df7aabb886319346f78af568def25a64`를 추적했다. 기준과 원 결합 source의 smoke 파일은 동일했다. `main.ts:1501,1510,1594`와 `local-fonts-modal.ts:211`에서 font 선택을 기다린 뒤 document canvas를 만드는 실제 경로를 대조했다. 기존 smoke는 선택 전에 visible canvas를 기다려 Chrome 156·Chromium 166 모두 timeout이었다. 같은 canvas 대기를 기존 선택 버튼 클릭 뒤로 이동했다.

| 구분 | 테스트 파일 SHA-256 |
| --- | --- |
| 보정 전 | a2ae0f49075c6f19da750236c0a22c008dcbe10f19c12077d238e331f781a893 |
| 보정 후 | 0dabc54e5164a288c88603d9fac1876caae525cd2d3f3460e2944fb41154acaf |

주석 외 실행 문장의 multiset은 동일하다. assertion·timeout·page budget을 약화하지 않았다.

| 브라우저 | 명령 | 결과 |
| --- | --- | --- |
| Chromium 1660786 | `node e2e/extension-smoke.test.mjs` | exit 0, 전체 packaged smoke PASS |
| Chrome 156.0.8078.47 | `node e2e/extension-smoke.test.mjs` | exit 0, 전체 packaged smoke PASS |

보정 후 viewer/options/print/service worker/content script를 모두 확인했고 page-budget 4개·Node 구문 검사도 통과했다. 제품 runtime·lockfile·workflow는 검증 source와 같으므로 영향받은 smoke만 다시 검증했다.

## Actions·정책 검사

Python workflow 계약 72개(이 중 CodeQL 22개 포함), Node CI 정책 330개가 exit 0으로 통과했다. CodeQL 22개를 다시 더해 중복 집계하지 않는다. 실제 YAML 4개를 파싱했고 actionlint 설치·실행을 주장하지 않는다.

## 보존 범위

원시 build/test/lint/브라우저 로그와 생성 분석 JSON은 커밋하지 않는다. 실행 명령·종료 코드·집계·원본 해시·오류 원인은 이 Markdown과 [시각 검토](visual-review.md), [공급망 검토](supply-chain-review.md)에 정리했다. 최종 전쪽 TSV·댓글에 사용할 대표 PNG만 보존한다.
