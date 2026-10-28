# #8174 v2 S05 — 실제 포장과 독립 소비

- 승인: 2026-10-25 사용자 「다음 절편 진행을 승인합니다.」
- 수행 기준: [v2 구현 계획](../plans/task_m100_8174_impl.md#범위-v2--기본-기능과-환경별-자원-정책),
  [결정 대장](../plans/task_m100_8174_maintainer_decisions.md),
  [S04 결과](task_m100_8174_v2_stage4.md).
- 상태: **S05의 로컬 필수 검증 완료.** 실제 registry 게시·소유권 설정·제품 버전 전환은 수행하지 않았다.
- 시작 source: `bf44a8e56a51f3954c75da1d32942ab5eef34349`.
- 원격 기준: 시작 시 `git fetch upstream devel`로 `9418ed0c7cf289c5c4ac2b3bb8cdc18a1b637a9a`의
  유지와 #8174의 OPEN·assignee `edwardkim`을 확인했다.
- 중간 로그·실제 archive: ignored `output/pr-review/issue8174/v2-s05-20261025/`.

## 실제 포장에서 발견한 누락과 보정

v1의 원 PR #8181 → merge `d9fd85ed5`, 코덱 #8183의 PR #8190 → merge `e7a0dec55`와
로컬 S02 `ec06a6d83f2a03a660ee48ace2ce2df8439114ad`를 대조했다. 기존 8개에서 14개로
확대한 source 검사만으로 Cargo 포장과 독립 소비를 완료한 것으로 간주하지 않았다.

첫 실제 포장에서 PDF backend의 자체 잠금 파일이 S02의 `rhwp-qcms`를 반영하지 않아
Cargo가 `vendor/rhwp-pdf-import-backend/Cargo.lock`을 수정한 뒤 dirty guard로 중단했다.
`--allow-dirty`로 우회하지 않고 upstream `qcms 0.3.0`의 잠금 항목을 이미 사용 중인
`rhwp-qcms 0.3.0-rhwp.1`과 `rhwp-codec-resource` 연결로 보정했다. root 잠금 파일과
Rust 구현·feature·조판 기대값은 바꾸지 않았다. 보정은 `202ef54168b811ef84bbf5e80e21aa7e49a69bc4`에 기록했다.

그 커밋에서 14개를 Cargo로 포장하고 파일 목록·원본 바이트·고지·내장 파일·깨끗한 VCS 정보를
검사했다. 합계 16,045,620 bytes였다. 이어 새 CLI 사용 예제의 출력 인자를 실제 positional
계약으로 정정한 `9c9c272fe`에서 본체를 다시 포장했다. 이미 내려받은 동일 버전 archive를
덮어쓰지 않도록 최종 본체는 별도의 로컬 sparse registry에서 소비하며, 13개 자식 archive는
그대로 보존하고 모든 source 바이트를 최종 Git과 다시 대조한다.

`cargo package --no-verify --registry crates-io`의 성공과 별도 소비자의 빌드·실행을 구분한다.
Cargo 자체의 포장 후 verify나 실제 crates.io 등록을 수행했다고 표시하지 않는다.
[Cargo package 공식 계약](https://doc.rust-lang.org/cargo/commands/cargo-package.html)에 따라
정규화 manifest에는 path·workspace·patch가 남지 않는지 별도로 검사한다.

## 소비 환경과 검증 계약

Rust 및 WASM 앱은 저장소 밖 `/tmp/`의 독립 workspace다. manifest에 repository path·patch를
넣지 않았고, 내려받은 모든 후보의 `.crate` 체크섬과 실제 registry source 경로를 확인한다.
외부 의존성의 버전 집합은 root `Cargo.lock`과 같으며 소비 앱 자체 외의 새 버전은 없다.

모든 빌드는 공용 `target/pr-review`, `CARGO_BUILD_JOBS=4`를 사용한다. Native는
`release-test`, WASM은 최적화를 포함하는 locked wasm-pack 경로를 사용한다. 제품 `pkg/`와
Studio public bundle은 덮어쓰지 않고 별도 소비 앱의 산출물로 검사한다.

S04에서 검증한 `e6d7b774c1437ad502a1e702bf8f01f2b6541c28`과 비교하면 Rust source·manifest·
root lock·build script·글꼴은 같고, 위 backend의 별도 lock만 다르다. S04의 전체 회귀·Studio
검증은 이 동일 구현의 근거로 연결하고, 이번 포장 후 실제 소비 결과와 구분한다.

## 사용자 가이드

[패키지 README](../../packaging/crates-io/README_PACKAGE.md)에 다음 내용을 보완했다.

- `import-pdf`의 `constrained` 프로필과 실제 출력 인자 사용 예제.
- Rust의 `PdfImportPolicy` → `DocumentCore` → 명시적인 HWPX 저장 예제.
- 직접 Rust 호출의 동기 실행과 `import-pdf` 작업자 감독의 구분.
- F07의 데이터 보존·조회 API와 Studio의 제한된 실행기, F08의 브라우저 복구 저장소 구분.
- 주 CLI의 `--bin rhwp` 명시와 패키지 밖 가이드의 절대 링크.

## 검증 결과

최종 source는 `9c9c272fe1670baf305800dfc906349fa7933d4a`다. [정식 증적](../pr/assets/issue8174/v2-s05/README.md)에
실제 명령·exit·검사 범위·체크섬·consumer 원문과 전쪽 TSV·대표 PNG를 보존했다.

### 실제 14개 archive

| package | 검증한 실제 버전 | 파일 수 | `.crate` bytes |
| --- | --- | ---: | ---: |
| rhwp-codec-resource | 0.1.0 | 7 | 5,301 |
| rhwp-contracts | 0.1.0 | 11 | 31,968 |
| rhwp-jpeg-decoder | 0.3.2-rhwp.1 | 27 | 53,251 |
| rhwp-jpeg-encoder | 0.7.1-rhwp.1 | 22 | 42,457 |
| rhwp-ooxml-chart | 0.1.0 | 14 | 103,884 |
| rhwp-password-crypto | 0.1.0 | 7 | 15,862 |
| rhwp-svg2pdf | 0.13.0-rhwp.1 | 27 | 59,077 |
| rhwp-zune-jpeg | 0.5.15-rhwp.1 | 39 | 92,035 |
| rhwp-hayro-ccitt | 0.3.0-rhwp.1 | 15 | 16,748 |
| rhwp-hayro-jpeg2000 | 0.4.0-rhwp.1 | 42 | 96,168 |
| rhwp-qcms | 0.3.0-rhwp.1 | 22 | 56,517 |
| rhwp-hayro-jbig2 | 0.3.0-rhwp.1 | 34 | 70,526 |
| rhwp-pdf-import-backend | 0.3.78-rhwp.1 | 414 | 6,894,014 |
| rhwp | 0.8.7 | 1,819 | 8,507,821 |

합계 2,500개 항목·16,045,629 bytes다. [원장](../pr/assets/issue8174/v2-s05/metadata/final-packages.json)의
archive별 VCS SHA를 구분했고, 모든 source를 최종 Git과 대조했다. 생성 manifest·lock·VCS는
Cargo 생성물로 구분하며, 원래 manifest도 `Cargo.toml.orig`와 대조했다. 2,459개 source 경로의
실행 권한과 비정규 항목 부재도 검사했다. 13개의 `publish = false`는 유지했다.

### 독립 소비와 검증 집계

| 항목 | 실제 결과 |
| --- | --- |
| Rust 기본·최소 | HWP/HWPX parse → 양 형식 저장·재독 → service/core 편집·재독 통과. 최상위 문단 텍스트·section·marker의 smoke 범위이며 모든 story의 무손실 주장과 구분 |
| 표준 오류 연동·기존 CLI·정책·OS 감독 | 1 + 5 + 6 + 7 = 19 PASS. 기존 검사 원문을 독립 앱에 복사하고 필요한 roxmltree/libc dev dependency를 명시. 기대값 변경 없음 |
| README Rust 예제 | 3개 모두 추출·컴파일·실제 실행 통과 |
| PDF 기본 기능 | Rust와 실제 WASM 각각 41개 입력의 가져오기·저장·재열기·쪽 소속 통과. JBIG2·JPEG 2000·ICC·xref 기본 지원 확인 |
| 개발용 코덱 제외 구성 | 일반 복구 PDF 처리, JBIG2/JPX 거부와 정상 후속 요청 통과 |
| CLI 설치·보존 | registry의 주 bin 기본 Native·PDF 제외 구성 설치 통과. S04와 16개 exit/stdout/stderr 일치. 새 가이드 명령과 PDF 제외 오류/출력 계약 통과 |
| 암호·미지원·손상·실패 후 복구 | Rust·CLI·WASM 각각 5개 거부·정상 후속 요청. CLI는 기존 출력 보존도 확인 |
| Rust 문서 | registry package의 `cargo doc --lib --no-deps` 통과. 실제 docs.rs 서비스 실행과 구분 |
| 독립 WASM API on/off | 두 구성 모두 release·wasm-opt 완료. API off 앱의 자체 start와 HWP/HWPX 재열기, on 앱의 기본 facade·전체 페이지 보존 확인 |
| 기존 실제 WASM/Worker | 240 assertions·Worker 12개(정상 9·메모리 실패 대조 3) PASS |
| 기존 코덱 실제 WASM/Worker | 19 cases·158 checks PASS |
| 별도 실제 브라우저 소비 | 53 assertions PASS. 정책·deadline·복구, 보호 저장·오입력, 실제 HWP 단추 source 보존, 6종 링크와 위험 URI 거부, SVG artwork 7종·할당 거부 포함 |
| source 사전 검사 도구 | 기존 경계 검사 18 PASS. 실제 소비 완료와 구분 |

Rust 1.93.1, Linux/WSL2 `x86_64-unknown-linux-gnu`와 `wasm32-unknown-unknown`에서 검증했다.
추가 OS·하위 MSRV를 보장하지 않는다. 전체 Rust·Studio source와 root manifest/lock은 S04의
최종 제품과 같으므로 그 전체 11,804/2,458 PASS를 연결하고, 코드 변경이 없는 전체 suite를
반복하지 않았다. [source 대조](../pr/assets/issue8174/v2-s05/metadata/product-source-cohorts.json)를 따른다.

### 전쪽 TSV와 직접 판독

| 범위 | Native raw 최저 | fresh WASM raw 최저 | 전체 쪽수 |
| --- | ---: | ---: | --- |
| PDF 41개 | 100% | 100% | 각각 41쪽 |
| 양식 동작 전·후 | 96.98675% | 96.93730% | 각각 2쪽 |
| 일반 표 HWP·TAC HWPX | 96.61580% | 96.11700% | 각각 6쪽 |

[TSV·쪽수 집계](../pr/assets/issue8174/v2-s05/metadata/visual-summary.json)의 90개 원본 TSV,
총 98페이지 모두 독립 PDF 쪽수와 일치했다. 90% 미만·누락·측정 불가 0쪽이며,
같은 backend의 **S04와 전체 RGBA·TSV가 동일**했다. 새 렌더링 fixture/golden·좌표 핀은 추가하지 않았다.

ICC 간접 filter·Indexed ICC premultiplied alpha·복구 xref·200 DPI 스캔, 양식 전·후의
compare·overlay·review를 새 산출물로 직접 판독했다. 색·alpha 합성·표 외곽·글자·값과 소속은
유지됐다. 양식의 기존 serif/sans fallback과 컨트롤 theme 차이는 남아 있으며 S04와 같다.
raw 100%를 독립 PDF와의 엄격 RGB 일치율로 설명하지 않는다. 상세는
[직접 판독](../pr/assets/issue8174/v2-s05/direct-review.json)을 따른다.

### 게시 인계 경계

실제 버전·archive SHA-256·의존 순서·다운로드 source·명령은 위 원장과
[검증 집계](../pr/assets/issue8174/v2-s05/validation.json)에 연결했다. 원래 v1의 8개 소비,
12/14개 source 사전 검사와 이번 14개 실제 Cargo/registry 소비를 혼합하지 않는다.
source checker의 `registry_consumption_verified=false`는 검사 종류의 계약으로 유지하고,
S05의 별도 집계에서 로컬 실제 소비 완료를 표시했다.

Native 검증은 `release-test`로 수행한 것이며 표준 release의 성능 측정이 아니다. 독립 WASM
앱은 소비자 export 3개를 추가하므로 기존 제품 bundle의 크기·속도 변화로 해석하지 않는다.
실제 crates.io의 parent dry-run·게시·이름 소유권 확인과 publish 제한 해제, 0.9.0 제품 버전
전환은 수행하지 않았다. MD05와 미결정 MD를 완료하기 전에는 PR 제출·게시 준비 전체 완료로
보고하지 않는다.

### 기존 잠금 의존성 경고

v1에서 이미 기록한 `chacha20 0.10.1`의 yank 경고는 이번 설치에서도 재현됐다. 설치는
`--locked`로 통과했으며 경고를 없애려고 잠금을 무제한 갱신하지 않았다.
[upstream 보정 #580](https://github.com/RustCrypto/stream-ciphers/pull/580)과
[변경 기록](https://raw.githubusercontent.com/RustCrypto/stream-ciphers/master/chacha20/CHANGELOG.md)은
RNG·legacy 64-bit counter의 SSE2 경로 보정과 0.10.2를 연결한다.

이번 graph에서는 `chacha20poly1305 0.11.0`의 하위 의존성으로 `cipher`·`xchacha`만 선택됐다.
선택한 source의 `XChaChaCore`는 IETF/u32 counter를 사용하고 `legacy`·`rng`는 꺼져 있음을
정적으로 대조했다. 이를 모든 소비 앱의 feature 합산이나 구형 CPU 실행에 대한 안전성 보증으로
확대하지 않는다. 기존 경고와 게시 전 의존성 갱신 검토는 MD05의 남은 감사에 그대로 남긴다.

## 남은 전체 이슈 조건

S05의 포장·독립 소비와 MD05-B의 비신뢰 PDF 보안 감사는 별개다. #7911 B01–B05·B07의
남은 할당·font·cache·환경별 격리 검증 및 MD01·MD08·MD09의 결정 전에는 v2 전체·비신뢰
업로드 공식 지원·실제 crates.io 게시 준비 전체의 완료를 선언하지 않는다.
