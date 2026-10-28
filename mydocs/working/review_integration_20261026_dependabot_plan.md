# Dependabot 21개 PR 통합 체리픽 검토 계획

- 작업 브랜치: `review/integration-20261026-dependabot-updates-r2`
- 시작 기준: `upstream/devel` `01ced3d42724285c8f9765a453393ab6af326617`
- 제출 기준: `upstream/devel` `49bc935a149c20b942d3045c2f5c733aeccf57d0`
- 사용자 지시: Dependabot PR을 체리픽 통합한 뒤 검토한다. 봇이므로 개별 PR 문서는 만들지 않는다.
- 개별 review 문서를 요구하는 일반 규칙 대신 이 계획과 통합 PR의 검토 기록 한 묶음으로 각 PR의
  원 head·적용 SHA·충돌·검증·판정을 추적한다. 별도 대표 이슈가 없어 날짜 기반 integration 이름을 사용한다.
- 원 PR 21개 모두 `jangster77` reviewer를 지정하고 API의 users에서 확인했다. 이 gate 뒤 상세 metadata와 patch를 수집했다.
- 원 head는 변경하지 않는다. 아래 고정 SHA를 번호 순서대로 `git cherry-pick -x`한다.

## 고정한 원 PR과 범위

| 원 PR | 고정 head SHA | 변경 파일 |
| --- | --- | --- |
| [#8241](https://github.com/edwardkim/rhwp/pull/8241) | `6abde855288dad72b42be68ac9f2a25fc5c29867` | `rhwp-studio/package-lock.json`, `rhwp-studio/package.json` |
| [#8242](https://github.com/edwardkim/rhwp/pull/8242) | `ea4ee557ec1dddeb99219f90a7774c2201ef80c6` | `Cargo.lock`, `Cargo.toml` |
| [#8243](https://github.com/edwardkim/rhwp/pull/8243) | `d9f5713786817202a400dfac553be5f503065bb7` | `rhwp-vscode/package-lock.json`, `rhwp-vscode/package.json` |
| [#8244](https://github.com/edwardkim/rhwp/pull/8244) | `d579b8d47929346e89e2d812ce15838402cd9421` | `rhwp-studio/package-lock.json`, `rhwp-studio/package.json` |
| [#8245](https://github.com/edwardkim/rhwp/pull/8245) | `97894d388b61344334c012480a2bdf52bcd7576a` | `Cargo.lock` |
| [#8246](https://github.com/edwardkim/rhwp/pull/8246) | `6a205a14f5890b2cde35ee10b4d2d606c9c0f252` | `rhwp-chrome/package-lock.json`, `rhwp-chrome/package.json` |
| [#8247](https://github.com/edwardkim/rhwp/pull/8247) | `1c2f11fbefea498251f45b19eba2fb0e3ecd245c` | `rhwp-firefox/package-lock.json`, `rhwp-firefox/package.json` |
| [#8248](https://github.com/edwardkim/rhwp/pull/8248) | `8612331ffb3167a77dbf55e8a338f74a063e98b5` | `Cargo.lock`, `Cargo.toml` |
| [#8249](https://github.com/edwardkim/rhwp/pull/8249) | `f0a9c0c71eaa7bd00b91d9603e78debfeae3329a` | `rhwp-studio/package-lock.json`, `rhwp-studio/package.json` |
| [#8250](https://github.com/edwardkim/rhwp/pull/8250) | `be55d654a85340383771e4d7b5725f56f7cddd4c` | `rhwp-chrome/package-lock.json`, `rhwp-chrome/package.json` |
| [#8251](https://github.com/edwardkim/rhwp/pull/8251) | `4886fb003aa40a5a9075ce61cc06d749f3a3b8e6` | `Cargo.lock` |
| [#8252](https://github.com/edwardkim/rhwp/pull/8252) | `b284c60df1cf3fad0d69c478be103f4a3c41f2c4` | `rhwp-studio/package-lock.json`, `rhwp-studio/package.json` |
| [#8253](https://github.com/edwardkim/rhwp/pull/8253) | `b9f9a7b9361bc4de771d2c8e575acb838036de24` | `Cargo.lock` |
| [#8254](https://github.com/edwardkim/rhwp/pull/8254) | `81abf49cd2a1ade119e5ae22d24b3cc75d5966b5` | `.github/workflows/release-binary.yml`, `.github/workflows/release-operations-contracts.yml`, `.github/workflows/skill-router-gate.yml` |
| [#8255](https://github.com/edwardkim/rhwp/pull/8255) | `b6646a38774d54fcb3f8bf91ca5543020027052c` | `Cargo.lock` |
| [#8256](https://github.com/edwardkim/rhwp/pull/8256) | `97b73c91bb54853ff1930874241931a8c595f634` | `Cargo.lock` |
| [#8257](https://github.com/edwardkim/rhwp/pull/8257) | `45b87dcac8a8cb794c798a368c46cd1fafd884e4` | `Cargo.lock` |
| [#8258](https://github.com/edwardkim/rhwp/pull/8258) | `37df0d004768590dd531c718b075bd86be38fceb` | `.github/workflows/codeql.yml` |
| [#8259](https://github.com/edwardkim/rhwp/pull/8259) | `cc6a5a10e972502160cb2a6dd9e75250d0ae36c4` | `.github/workflows/codeql.yml` |
| [#8260](https://github.com/edwardkim/rhwp/pull/8260) | `2f2c5f3b9ae9a0c75863dc2007f83f19aa89142f` | `.github/workflows/build-nextest-archives.yml`, `.github/workflows/run-nextest-archives.yml` |
| [#8261](https://github.com/edwardkim/rhwp/pull/8261) | `101f2ae74f552f264ea6ac0c4904260858b5ca71` | `.github/workflows/adapter-diff.yml`, `.github/workflows/build-nextest-archives.yml`, `.github/workflows/ci.yml`, `.github/workflows/fuzz-smoke.yml`, `.github/workflows/layout-anomaly-advisory.yml`, `.github/workflows/oracle-public-advisory.yml`, `.github/workflows/proptest-roundtrip.yml` |

## 검증과 판정

1. 각 고정 ref를 fetch하고 실제 SHA를 확인한 뒤 오래된 번호부터 누적 적용한다. 적용 SHA·순서와
   conflict 및 보정은 이 기록에 남긴다. 이미 merged된 source 변경을 되돌리지 않는다.
2. Rust 8개, npm 8개, Actions 5개의 업데이트와 공식 변경 근거를 확인한다. read-fonts/skera의
   버전·타입 연결과 Anthropic SDK major 변경은 실제 호출부·빌드·행동 검사로 확인한다.
3. lockfile 정합성, action pin, 세 Clippy·workspace build·전체 nextest와 npm 빌드·단위·실제 browser를 확인한다.
4. 실제 소비 경로에 따라 폰트 검증을 정한다. `webfont_subset`은 wasm32에서 제외되는 Native SVG
   부분 글꼴 경로다. 이 경로의 두 쪽을 브라우저에서 전체 face 출력과 대조하고 Unicode·폭·윤곽을
   독립 파서로 확인한다. 기존 sample16 입력·Print·전쪽 TSV·기대값은 보존한다.
   이번에 해당 384쪽을 다시 렌더했다고 주장하지 않는다. 최종 검증 수용 범위는 아래 사용자 지시를 따른다.
5. 코드 보정과 필수 검증을 마친 후보를 통합 PR로 제출하고 정확한 최신 head CI를 확인한다.
   봇 원 PR별 문서 없이 통합 검토에서 수용/보류와 적용·보정 SHA를 구분한다.

## 이전 작업과 자원 경계

#8188·#8273·옵션 2 문서 #8274는 이미 merged이고 devel에 포함됐다. 원본·PDF·768개 PNG·
개인 글꼴과 복구 자료를 보존하고, 승인된 이전 작업 branch/worktree/output 정리로 검증 공간을 확보한다.
공유 `/home/tsjang/rhwp/target/pr-review`와 사용자·다른 작업 자료는 보존한다.
문서 전용 merge에서도 metadata job의 checkout이 실행되는 점은 확인해 사용자에게 보고했으며,
이번 의존성 업데이트와 섞어 수정하지 않는다.

## 재시작 기록

사용자가 임시 output을 삭제한 뒤 처음부터 다시 체리픽하도록 지시했다. 원 PR 21개의 metadata와
patch를 재수집했고 reviewer 지정·각 head SHA가 모두 동일함을 확인했다.
기존 통합 branch는 사용자 지시로 삭제하며, 기존 head `9f6c373ad246e3454325a71eaa0ae10c7d48533f`의
patch는 로컬 영구 복구 경로에 보관했다. 새 branch에서 21개 원 commit을 다시 번호 순서로 적용한다.

## 재적용 결과

21개 원 commit의 저자와 `cherry picked from` 원 SHA를 보존하여 새 branch에 모두 적용했다.

| 원 PR | 재적용 commit | 충돌 처리 |
| --- | --- | --- |
| #8241 | `6d1668e9c5049b331bf1c1ae2a882868090ccbb1` | 충돌 없음 |
| #8242 | `ed9c59d227d56fc7c8e3af9cbc3762f599cdc9e8` | 충돌 없음 |
| #8243 | `4d8af2f166a49ad296caf744980ead51adf3fff1` | 충돌 없음 |
| #8244 | `440d5ce3e5c7a384e5ce9d84f69ff0f7929af322` | 충돌 없음 |
| #8245 | `9b4249717db96896e65db21e255fd33e746a6207` | 충돌 없음 |
| #8246 | `08484248e204013748bea0d93d67ef3dbd5e7192` | 충돌 없음 |
| #8247 | `7a19a3d66452802556d01c09d6585a42ec9c16b9` | 충돌 없음 |
| #8248 | `d477a8e868a20906bcab1ff1810a5d8fa152dfb6` | rerere 해소 결과를 이전 파일 전체와 비교: read-fonts 0.46 보존, skera 0.9만 적용 |
| #8249 | `4065b00163d3482a57aae54e173bec152e54e769` | 충돌 없음 |
| #8250 | `016c2ca69b2546a376240af810f5701a821127e4` | 충돌 없음 |
| #8251 | `5e07601025f9519855f0c15a1030f02da650c837` | 충돌 없음 |
| #8252 | `e78706d748b4b81b6d469b454f86caaac5513eb7` | puppeteer-core 25.16.0과 vite 8.3.5를 유지하고 @types/chrome 0.3.6 적용 |
| #8253 | `d1fecb3a30c904325c5c67ffd3fa0d9e20757905` | 충돌 없음 |
| #8254 | `06548b1fbf47521651e44ab7da2ed43432e5e3c7` | 충돌 없음 |
| #8255 | `8c0ab62f15b4bfe94057f9b044d451ccc605b697` | 충돌 없음 |
| #8256 | `72c26c3fc038db559d32f7a0f59368942c768998` | proptest의 rand_core 단일 버전 정규화를 적용하고 read-fonts 0.46.0을 보존 |
| #8257 | `1e90527989e7c85af28be93f15fca1f9091b438d` | getrandom 0.4.3 단일 버전 상태의 정규화된 참조를 보존하고 sha2 0.11.1 적용 |
| #8258 | `4fdd0f478051c5e0db87e7a372ff4c9cf54be539` | 충돌 없음 |
| #8259 | `11e8f1b58640ba299943d0eb94ebf3c0be674e1c` | 충돌 없음 |
| #8260 | `b1f960d8706da1d8f9e03317b8ee8787f4dd848d` | 충돌 없음 |
| #8261 | `5d309c517f42866b560b4b82327e93a8ac91a110` | 충돌 없음 |

Cargo와 npm lockfile의 누적 버전 정합성을 확인하고 실제 빌드·검증을 시작한다. 이 단계는 검증 완료 판정이 아니다.

## 누적 lockfile 정합성 보정

`cargo clippy --locked`가 lockfile 갱신 필요로 중단됐다. `cargo metadata --offline`으로 resolver의
변경을 대조했으며, 앞선 read-fonts/skera 동시 갱신 뒤 참조가 없어진 `read-fonts 0.45.1`의
13줄 package 항목 삭제만 발생했다. 다른 버전·checksum·의존성은 바꾸지 않았다.
오프라인 metadata 자체는 미캐시 `bit-set 0.11.1` 다운로드 단계에서 중단됐으므로,
온라인 fetch 뒤 같은 `--locked` lint 묶음부터 실제 재검증한다.

## SDK major 갱신의 브라우저 검사 보정

AI 패널 E2E는 대화·도구 적용/undo·오류 복구를 실행했으나 기존 `x-api-key` 기대 1건이 실패했다.
원 검사는 #8228 merge `3e02fe2749866a8453af3eb3901b9c1a674cccaf`에서 들어왔다.
공식 SDK의 [0.137.0 client](https://github.com/anthropics/anthropic-sdk-typescript/blob/sdk-v0.137.0/src/client.ts)는
`X-Api-Key`를, [1.1.0 client](https://github.com/anthropics/anthropic-sdk-typescript/blob/sdk-v1.1.0/src/client.ts)는
`Authorization: Bearer`를 생성한다. [공식 changelog](https://github.com/anthropics/anthropic-sdk-typescript/blob/sdk-v1.1.0/CHANGELOG.md)는
1.0.0의 API 버전 2026-10-21 변경을 명시한다. 제품의 인증 동작은 SDK에 맡긴 기존 구현을 유지하고,
E2E가 정확한 Bearer 키·기존 헤더 미전송·API 버전을 보호하도록 정정했다.
실제 API 키·외부 Anthropic 요청은 사용하지 않으며 응답 가로채기 범위다. 재실행 결과를 별도로 기록한다.

같은 이유로 AI 설정 E2E의 모델 조회 응답 대역도 올바른 키를 401로 거절하여 대기 시간이 초과됐다.
대역이 Bearer 키로 성공/실패를 분기하게 고치고, 잘못된 키와 올바른 키 양쪽의 전송값·기존 헤더
미전송·API 버전을 검사하도록 보강했다. 초기 Firefox 실행은 Snap의 /tmp 격리로 프로필을 읽지 못했으며,
공유 가능한 검토 output 아래 TMPDIR를 지정한 재실행은 다운로드 3회와 편집 보존을 통과했다.

## 공식 변경 근거와 action pin 보정

- read-fonts 0.46.0과 skera 0.9.0의 배포 `.cargo_vcs_info.json`은 모두
  [fontations 47a3016](https://github.com/googlefonts/fontations/commit/47a301615fd95acf0da9fb7dbcd0ea7c7df36943)을 가리킨다.
  실제 직접 소비 지점은 `src/renderer/svg/webfont_subset.rs`다. 같은 release의 타입 연결을 유지한다.
- proptest 1.12.0은 rand 0.10과 BitSet 변경·MSRV 1.88을 포함한다. sha2 0.11.1도 MSRV 1.88이다.
  저장소 1.93.1 toolchain에서 잠긴 의존성으로 integration target까지 컴파일하고 실행한다.
- ml-kem 0.3.3은 sha3/shake 갱신과 secret-dependent CBD 조회 제거를 포함한다.
  zip 9.0.5는 raw copy AES extra field·플랫폼 mode 처리 보정 등을 포함한다.
  배포 crate changelog와 checksum으로 식별했으며 full Rust 검사로 실제 소비 경로를 검증한다.
- [rust-cache c8037a8](https://github.com/Swatinem/rust-cache/commit/c8037a86dde8214a8310e9f54a8fc61ab362aa31)는
  virtual workspace root Cargo.toml/lockfile을 cache key에 포함한다. 그 commit의 package version은 2.9.3이다.
  bot가 남긴 v2.9.1 주석을 실제 commit 의미로 바로잡았고, 기존 restore/save 신뢰 경계는 유지했다.
- [CodeQL 4.39.1 changelog](https://github.com/github/codeql-action/blob/9b3254085494b55a0f7c4c2b3d2d52cc712ce370/CHANGELOG.md)는
  기본 bundle 2.27.3을 명시한다. init/analyze의 고정 SHA를 일치시키고 낡은 날짜 주석을 바로잡았다.
- setup-python의 고정 commit action.yml은 node24, install-action은 checksum 기본 true를 확인했다.
  input·permission·fork event·archive matrix·worker timeout은 이번 통합에서 변경하지 않는다.

## 최종 검증과 충돌 없는 리베이스의 공식 채택

- 현재 전체 회귀는 **11,573 PASS·45 skip·exit 0**이다. 실행 단계는 4,740.241초였고,
  이 시간은 그 전에 수행한 컴파일과 구분한다. fmt·세 Clippy·workspace build·suite 정책도 통과했다.
- Studio는 2,572 PASS·2 skip, 네 npm package 설치/빌드와 dependency 정합성이 통과했다.
  AI 패널 94개·설정 20개 단언, Chrome package 실행, Firefox 다운로드 3회·편집·저장이 통과했다.
- workflow 계약 Python 195개·CI impact 관련 Node 334개·actionlint가 통과했다.
- Native SVG 부분/전체 글꼴 대조 두 쪽은 raw100·정수 미대응 0/0이다. 독립 FontTools로
  사용 문자 cmap·전진/좌측 여백·윤곽 좌표/플래그와 공통 수직 메트릭을 확인했다.
  font payload/format 이외 SVG는 동일하다. RGB 차이는 p1 325px·p2 8,974px로 별도 기록하며
  RGB 동일성이나 독립 Print와의 일치율로 표현하지 않는다. 두 쪽의 변경 영역도 확대 판독했다.
- `read-fonts`/`skera`의 직접 사용은 `src/renderer/svg/webfont_subset.rs` 한 곳이며,
  `src/renderer/svg.rs:58`의 cfg로 wasm32에서 제외된다. Canvas2D/CanvasKit·Skia text layout의
  공통 구현을 수정한 작업으로 확대하지 않는다. 다른 Rust 의존성은 전체 회귀·WASM32 Clippy로 확인했다.

사용자 지시: “회귀 테스트가 끝나면 upstream/devel 위로 리베이스 충돌하는곳이 없으면 현재
회귀테스트를 공식인정. 다시 테스트 할 필요 없음.”

`d612e9f1710e6bf72f8bd21538820eeb41f9529c`를 최신 `49bc935a149c20b942d3045c2f5c733aeccf57d0` 위로 리베이스했고
결과는 `648ed728b8f0514cbf16cc7d5775b5a12787db59`다. 27개 commit 모두 자동 적용됐으며 충돌은 없었다.
반영한 새 devel은 문서 전용 duration trigger 보정과 그 검토 문서다. Rust·Cargo·기존 test·
Studio/extension·shared package 입력은 리베이스 전후 동일하다. 사용자 지시에 따라 현재 결과를
공식 채택하고 같은 검사를 재실행하지 않는다. 예약된 추가 Native 전체 배치와 fresh WASM/384쪽
재산출 배치는 시작하지 않았으며, 기존 전쪽 자료를 새 검증으로 합산하지 않는다.

[실제 검증·리베이스·원 SHA/적용 SHA](../pr/assets/dependabot_20261026/validation.json),
[SVG 소비 경로 증거](../pr/assets/dependabot_20261026/svg-control.json)를 보존한다.
임시 실행 스크립트·로그·내장 font payload는 커밋하지 않는다. 원 PR 21개 별도 review 문서는 만들지 않는다.

| 원 PR | 제출 기준 재적용 commit |
| --- | --- |
| #8241 | `6e2ac46c931aa8d41acd6ef20a28d324db730b41` |
| #8242 | `fa78c37fb4dbb2271a75fe7791717ed3088f86c5` |
| #8243 | `a1dcc0b0ba89770bbdcc501217685c456aa098e9` |
| #8244 | `fe91ac495bcbbc51012224accf6a07fbdb12c758` |
| #8245 | `851ade02e7e4ae08363ffe8a2a00f7f0dbd8905f` |
| #8246 | `4ecd612ac73bc7dc36be523dd0b20ff23fe33ec9` |
| #8247 | `2d41ae8026b329d531fa918266cd44144c630bbc` |
| #8248 | `7c5c68e210ae6f292997bd332e3c12bcb0630289` |
| #8249 | `24890c79118e53e3f9b31f538bbef43fd22421b3` |
| #8250 | `3999ce9b0de124a04415d274cc4993f52f4b2528` |
| #8251 | `f697ea7332acdacc6f58bc60ac2535e6ad73d6ad` |
| #8252 | `db54f241468d6dc119bf9ea7db7d723bd9f00af0` |
| #8253 | `aaa3592c9f85a97ad7bd325d831f5bbd467afed7` |
| #8254 | `36e972c8a17fb7cafd94b3cf80da83f081a1aad6` |
| #8255 | `1bbd1b65e003a0949fc029392c43e6c6fa4365c2` |
| #8256 | `776a07c714730982bd1998545696c856d6d18fba` |
| #8257 | `304e096053f0a4d52920c5269eba8aa8c2ae7cd2` |
| #8258 | `06eb3deb8dd198fe6e2643f23152b2f3e478950e` |
| #8259 | `59fb6167a7bcc79fbe5cef05e5ca438f8a967fb4` |
| #8260 | `9ae83f9ea45d1b30aadc4806440641c11d5d7db7` |
| #8261 | `5776f13ff812c877bae63b30aed4c3751e258449` |
