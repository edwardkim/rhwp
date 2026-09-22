# Task #7280 Stage 58 — devel 통합본 최종 검증

- Issue: #7280. 이전: [Stage57](task_m100_7280_stage57.md).
- 승인: 작업지시자의 “진행을 승인합니다.”에 따라 전체 회귀·남은 lint/build·Native/fresh WASM 대조 진행.
- 검증 head: `29130d5397da3dc3cd3d1d9416b153584e1a4f41`.
- 비교 base: fetch로 재확인한 `1966af77fa8046c844d654b157b5168baad8a30e`.
- 제품 내용은 merge `6003fe35689ee9be02d2b05737430e648b07361e`와 동일하다.
- 상태: **2026-09-23 통합본 로컬 검증 완료. lint·빌드·전체 회귀·Native Skia·Native/fresh WASM 전후 대조 통과**.
  제품 변경·baseline 완화·push·PR·댓글은 수행하지 않았다.

## 환경과 증적

clean review worktree `rhwp-review-7280-r3k`를 위 head로 고정했다. 최초 baseline worktree와
기존 증적은 보존한다. 고정 Cargo target은 주 저장소의 `target/pr-review`이며 Cargo 작업은
순차 실행한다. 시작 시 16 CPU/31GiB RAM, 디스크 여유 약 6.6GiB를 확인했다.
빌드 동시성 4, 테스트 동시성 8을 사용한다. 산출물이나 다른 worktree를 삭제하지 않는다.

증적 접두사: `output/7280/stage58-integration/`. `lint.sh`는 prepare → fmt → Native Clippy →
WASM Clippy → workspace build → workspace/all-target Clippy → 고정 base 정책 검사 순서다.
전체 회귀·Native Skia·fresh Docker WASM·시각 대조의 실제 실행 결과와 한계는 아래에 보완한다.

## 실행 결과와 중단 지점

| 검사 | 결과 |
| --- | --- |
| prepare / fmt check | PASS |
| Native Clippy `--locked … -- -D warnings` | PASS |
| WASM Clippy `--locked -p rhwp --lib --target wasm32-unknown-unknown … -- -D warnings` | PASS (52.89초) |
| workspace build `--locked --workspace` | 최초 **환경 실패**: 디스크 부족, exit 101. 공간 확보 후 재시도 PASS (16.54초) |
| manifest / unit-tier `--check --base-ref 1966af77…` | PASS. 중단 후 경량 검사만 별도 실행 |
| workspace all-target Clippy | 공간 확보 후 PASS (1분 25초) |
| 전체 release-test | **10,137 passed / 0 failed / 50 skipped**, 실행 453.493초 |
| Native Skia lib | 4,112 passed / 0 failed / 기존 ignored 13 |
| Native Skia placeholder / 직접 PDF | 각각 2 passed / 4 passed, 실패 0 |
| 새 Native/fresh Docker WASM | base/head 모두 빌드 완료; Native/WASM 전후 데이터·선택 PNG 대조 PASS |

`build-workspace.log`에서 `rhwp-q-kit` 컴파일의 `No space left on device (os error 28)`와
`rhwp-q-font-layout-evidence` 링크의 Bus error를 확인했다. 조판 계약 실패나 Rust 소스 오류로
분류하지 않는다. 공간 부족 상태의 연속 빌드나 출력 생성을 중단했으며 Cargo/Rust 잔여 프로세스는 없다.

중단 후 `/dev/sdd` 가용 공간은 약 **220MiB**다. 읽기 전용 계측 결과
`target/pr-review/debug/incremental` 약 **63GiB**, `debug/deps` 약 **122GiB**,
`debug/build` 약 **483MiB**다. 이 경로는 공유 검증 캐시이므로 승인 없이 삭제하거나
고정 target 경로를 이동하지 않았다. 다른 worktree·원본·이전 증적도 보존했다.

`manifest.log`는 1,399 sources / 28 suites + 20 exceptions,
`unit-tier.log`는 4,205 tests / 298 modules / cfg support 28을 기록한다.
Stage57의 집중 403건 통과는 같은 제품 내용의 선행 증거이지만, 이번 전체 회귀를 수행했다는
의미가 아니다. 새 시각 산출물이나 시각 통과 판정도 없다.

## 승인된 공간 확보 및 재개

작업지시자의 “승인합니다.”에 따라 지정한 증분 캐시 경로만 삭제했다. 삭제 전 realpath와
일반 디렉터리 여부, Cargo/Rust/WASM 빌드가 실행 중이지 않음을 재확인했다.
`rm -r /home/edward/mygithub/rhwp/target/pr-review/debug/incremental` 후 가용 공간은
220MiB → 약 63GiB가 됐다. 삭제한 캐시는 재생성 가능하며 소스·샘플·증적·다른 target은 보존했다.
강제 삭제 명령은 실행 도구에서 거부되어 비강제 재귀 삭제를 사용했다.

review head `29130d539…`와 clean tracked 상태를 재확인했다. `resume-lint.sh`로
workspace build → all-target Clippy → 고정 base 정책 검사를 순차 통과했다.
최초 실패 로그는 보존하고 재시도는 `build-workspace-retry.log`, `clippy-workspace.log`,
`manifest-retry.log`, `unit-tier-retry.log`에 분리했다.

## 새 산출물 및 동작 보존 대조

`build-artifacts.sh head` → review를 base로 전환 → `build-artifacts.sh base` 순서로
동일 target에서 Native CLI와 Docker WASM을 빌드했다. Native는 `--locked --profile release-test`,
WASM은 `docker compose -p rhwp --env-file .env.docker run --rm --no-deps
-e CARGO_BUILD_JOBS=4 -e BINARYEN_CORES=4 wasm`이다. 각 바이너리와 pkg를
`bin/{base,head}`, `pkg/{base,head}`로 분리 보존하고 `artifact-hashes-{base,head}.txt`와
`source-{base,head}.txt`로 고정했다. 원래 baseline worktree와 주 작업트리 Studio pkg는 변경하지 않았다.
이후 review를 검증 head로 되돌려 `regression.sh`를 완료했다.

Native `native.mjs` / `compare-native.mjs`의 결과는 **14개 문서 772쪽**에서
페이지 구성·소유/컷 데이터, 전체 render tree와 SVG가 동일하다 (`native-comparison.json`).
PDF 대응 11개 문서의 선택 **23쪽 PNG**도 base/head 동일하다
(`native-visual-comparison.json`). 통합본의 Native/fresh WASM 선택 23쪽 PNG도 모두 같다.
WASM base/head **11개 문서 748쪽**의 전체 SVG·render tree 및 선택 23쪽 PNG도
동일하다 (`wasm-visual-comparison.json`). `compare-visual.mjs wasm`은 통합본의
Native/WASM 선택 PNG 동일성도 다시 검사했다.

실행 버전은 nextest 0.9.137(권고 0.9.140)이며, `ci-duration-observation.junit.report-skipped`
키를 무시한다는 경고를 기록한다. 전체 회귀의 실행 범위는 `--tests --no-fail-fast`로
유지하며, 경고를 숨기거나 테스트 필터/기준값을 변경하지 않았다.

## 통합 head 전체 회귀

`regression.sh`의 첫 단계는 컴파일 8분 19초 후 78개 바이너리에서 10,137건을
실행했다. **10,137 통과 / 실패 0 / 기존 제외 50**, slow 5건은 모두 통과다.
실행 시간 453.493초이며 `nextest-full.log`에 전체 검사 이름과 결과를 보존했다.
Native Skia 3종도 뒤이어 순차 통과했다:

- `skia-lib.log`: root 3,930 pass / 13 ignored, contracts 15, ooxml-chart 165,
  password-crypto 2 pass. 합계 4,112 pass, 실패 0. 컴파일 3분 17초, root 실행 137.41초.
- `skia-picture.log`: `issue_2225_missing_picture_placeholder_split`,
  `issue_2225_export_png_defaults_to_screen_skia_profile` 2 pass. 198 skipped는 suite 내 이름 필터다.
- `skia-pdf.log`: `render_p37_direct_pdf_export` 모듈 4 pass. 168 skipped는 suite 내 이름 필터다.

이 두 focused 명령의 skipped를 전체 회귀 제외 50건에 더하지 않는다. 전체 회귀와 Skia는
별도 feature 계약이며 중복 검사 수를 합쳐 고유 테스트 수로 보고하지 않는다.

대상은 `targets.json`에 입력/PDF/쪽을 고정했다. 기존 R5 대조군에 통합 변경 영향 문서
시장구조조사 4·5쪽과 화학제품 보고서 13·14쪽을 추가했다. 후자는 현재 base/head 모두 104쪽이다.
출력 동일성은 리팩토링 무회귀 근거이며 한컴 조판 정확성 전체 통과를 뜻하지 않는다.

직접 연 대표 `review`와 같은 경로의 `compare`·`overlay` 증적:

- Native `square-host/square-host/review/review_001.png`: 표/본문 흐름은 유사하나
  글꼴 굵기·자형 차이. PDF 대비 proxy 9.77316%.
- Native `market-rewind/market-rewind/review/review_004.png`: 목차 배치·줄 순서는
  유사하나 자형 차이. proxy 9.64910%.
- fresh WASM `market-rewind/market-rewind/review/review_005.png`: PDF 하단의
  `제5절 소결`이 rhwp 같은 쪽에 없는 차이. proxy 7.67035%. base/head 출력은 동일하며
  한컴 대비 완전 일치로 판정하지 않는다.
- Native `chemical-rewind/chemical-rewind/review/review_013.png`: 본문/그림의 같은 쪽
  배치, 글머리표·본문 세로 위치 차이. proxy 64.18848%.
- fresh WASM `chemical-rewind/chemical-rewind/review/review_014.png`: 표가 PDF보다
  위에 위치하는 차이 유지. proxy 21.71057%. Native base/head PNG 동일성으로
  리팩토링 유입 차이와 구별하지만, 이 차이를 올바른 조판으로 승인하지 않는다.
- fresh WASM `endnote-between20/endnote-between20/review/review_022.png`: 문단 시작·수식·그림
  위치의 PDF 대비 큰 차이가 유지된다. proxy 10.16997%. devel과 통합본의 동일 출력이며
  이 조판 차이를 고치거나 정확하다고 승인하는 것은 이번 구조 리팩토링 범위가 아니다.

각 접두사는 `output/7280/stage58-integration/{native,wasm}/head/`다. proxy는
내용 픽셀 중심 자동 일치율 보조값이지 사람 판정 정확도가 아니다. 직접 열지 않은 쪽까지
시각 통과했다고 주장하지 않는다. Chrome 시작 실패는 실패 로그를 보존하고 같은 head에서
재시도했으며, 성공한 `summary.json`만 완료 증적으로 사용한다.

### 최초 중단 당시 재개 조건 (이력)

작업지시자가 공간을 확보하거나, 구체적인 재생성 가능 캐시 정리 범위를 승인해야 한다.
정리 후보는 `/home/edward/mygithub/rhwp/target/pr-review/debug/incremental`의 증분 캐시다.
해당 캐시를 정리하면 다음 컴파일 시간이 늘지만 소스·샘플·검증 보고서는 제거 대상이 아니다.
실제 삭제는 별도 승인 후 대상 경로와 다른 빌드의 사용 여부를 재확인해 수행한다.

당시 미실행이었던 workspace build부터 위와 같이 재개하여 남은 검증을 완료했다.

## 최종 판정과 다음 절차

고정 devel `1966af77…` 대비 통합 제품의 구조 리팩토링 보존 검증은 이번에 실행한 범위에서
충족한다. 전체 회귀 실패 0, Native 772쪽/WASM 748쪽 데이터 및 선택 23쪽 PNG 전후 동일,
필수 lint·workspace build·Skia 3종 통과다. 원래 baseline 대신 최신 통합 base를 실제 빌드해
대조했으며, 한컴 PDF와 잔여 차이는 위 직접 판독 기록대로 보존한다. 모든 문서의 조판 정확성이나
모든 페이지의 사람 시각 통과를 선언하지 않는다.

검증 후 제품·테스트·baseline·ignore 수정은 없다. 주 작업트리의 변경은 이 보고서뿐이고
review worktree는 `29130d539…`의 clean tracked 상태다. 산출물은 ignored output에 보존한다.
`git diff --check`를 통과했으며 증적 Markdown 링크의 대상 파일을 확인했다.
정리 후 최종 디스크 여유는 약 54GiB다. 원격 push·PR·댓글·이슈 종료는 수행하지 않았다.

다음은 최종 결과보고/PR 본문과 보존용 시각 asset 준비, 별도 승인 후 원격 제출이다.
이번 완료는 고정 base/head에 대한 **로컬 검증 완료**이며 원격 PR CI나 merge 완료가 아니다.
