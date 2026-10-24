# #8183 3단계 — 실제 소비 검증과 사용자 가이드

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183) ·
[수행 계획](../plans/task_m100_8183.md) · [구현 계획](../plans/task_m100_8183_impl.md) ·
[2단계 결과](task_m100_8183_stage2.md) · [사용자 가이드](../manual/pdf_image_codecs.md)

2026-10-23 작업지시자가 「3단계 진행 승인」으로 실제 CLI·fresh WASM/Studio, 시각·성능,
빌드·사용자 가이드 검증을 승인했다. 원격 게시·push·PR 생성·merge·close는 하지 않았다.

**상태: 3단계의 검증 실행·성능 관측·가이드 보강을 마쳤다. 한컴 Print alpha 두 쪽은
미달이므로 구현 완료·PR 제출 판정은 보류한다.**
원본 PDF 기준의 정상 결과로 Print 미달을 상쇄하지 않는다. 아래 수치는 검증한 경로에만 적용한다.

**메인테이너 시각 확인:** [반투명 이미지 비교판·확대 뷰어](task_m100_8183_alpha_visual_review.md).
기존 캡처의 표시 자료이며 Print 미달·보류 판정을 바꾸지 않는다.

## source와 산출물

- base: `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` (`upstream/devel` 확인 후 고정).
- 단계 시작: `b799b1f47`, 지원 정보·초기 가이드: `a4039329a`.
- 최종 제품 Rust source: `cfc5be8fe`. 이후 Rust 변경은 테스트에만 있다.
- 정식 자료·검사 편입: `8f679c3bc`, 테스트의 비활성 feature 상수 경고 보정: `2a58c5f37`.
- 증적: ignored `output/pr-review/issue8183/stage3/`.
- Cargo: 기본 작업공간의 `target/pr-review` 공유, `CARGO_BUILD_JOBS=8`, 빌드는 순차 실행.
  별도 review checkout에서 파생 suite를 준비했다. 기존 cache·다른 worktree는 지우지 않았다.

Native 네 구성은 같은 `release-test`+`native-skia`다. WASM 네 구성은 표준 release,
codegen 16, wasm-opt 완료 산출물이다. Native와 WASM의 빌드 프로필 차이를 성능 비교에서 구분한다.
실행 파일은 `final/native-{default,jbig2,jpx,both}`, 패키지는 `wasm-final/`에 고정했다.

| WASM 구성 | 바이트 | default 대비 |
| --- | ---: | ---: |
| default | 21,749,986 | — |
| JBIG2 | 22,044,034 | +294,048 / 1.352% |
| JPEG 2000 | 22,027,633 | +277,647 / 1.277% |
| 양쪽 | 22,303,386 | +553,400 / 2.544% |

양쪽 WASM SHA-256은 `76cb5f0789ddb1c9891fb1d74252eebbaec642d93f50066b153913ac36e615d7`,
JS는 `2d83bb50a3af21f94749d3c72c8e8e07603a07414b65eec559e56e498eefae67`이다.
현재 로컬 pkg/public은 이 양쪽 코덱 빌드다. Cargo 기본 feature의 코덱 비활성 정책은 유지한다.
빌드 경과 시간은 순차 warm cache 관측이며 cold build의 공정한 전후 비교로 사용하지 않는다.

## 실제 소비와 공개 지원 정보

Native CLI는 19건×4=76건, 실제 브라우저의 fresh WASM도 19건×4=76건을 확인했다.
정상 JPEG, 개별 코덱 수용·미선택 거부, 공유 사전·palette·PDF Indexed·alpha·mask,
손상 입력과 HWP/HWPX 저장·재열기를 포함한다. 보정한 전체 스캔 입력으로도 모두 다시 실행했다.
별도의 정상 JPX 세 번 paint 입력은 실제 누적 작업 한도에 걸리고 다음 요청은 복구했다.

공통 `pdf_support::pdf_import_support()`의 세 boolean을 CLI `capabilities.pdfImportSupport`,
가져오기 `diagnostics.pdfImportSupport`, WASM `HwpDocument.getPdfImportSupport()`가 소비한다.
기존 필드·오류 코드와 `parserAllocationBoundsProven=false`는 유지한다.

라이브러리 앱이 다른 의존 경로로 backend feature를 켜면 root alias만으로는 지원을 잘못 표시했다.
별도 Rust 앱의 같은 의존 graph에서 `a4039329a`는 `accepted=true, declared=false`로 실패했다.
backend의 실제 합산된 feature를 조회하는 `cfc5be8fe`는 `true/true`로 통과했다.
`union-before.log`, `union-after.log`에 실행을 보존했다. decoder·한도·픽셀 처리는 이 보정에서 바꾸지 않았다.

실제 공통 Worker는 공개 WASM 가져오기 API에 진입한 것을 MessagePort로 관측한 뒤 취소·deadline을
검사했다. 입력 길이별로 진입을 구분해 앞선 실패 요청의 알림을 현재 요청으로 세지 않는다.
오류·미선택 코덱·누적 작업 초과·취소·시간 초과에서 활성 문서가 보존되고 다음 정상 요청이 성공했다.
시험 wrapper만 관측을 추가했고 공통 Worker 본체는 그대로 사용했다.
정식 `scripts/tests/pdf-codec-wasm.test.mjs`도 네 패키지에서 모두 통과했다.

Studio는 실제 파일 열기·CanvasKit 표시, 오류 뒤 digest·SVG·파일명 보존, 다음 정상 열기를 확인했다.
app 초기화의 HTTP 응답 WASM과 pkg/public의 JS·WASM hash가 고정 산출물과 일치했다.
최초 절대 out-dir은 wrapper의 자동 public 동기화 대상이 아니어서 남은 public 쌍을 확인하고
명시적으로 동기화했다. 현재 가이드는 지원되는 `--out-dir pkg` 표기를 사용한다.
현재 Studio 열기는 `wasm-bridge.ts`의 동기 `new HwpDocument(data)`다.
공통 Worker의 중단 보장을 동기 Studio UI의 보장으로 확대하지 않는다.

## 독립 자료와 원본 PDF 대조

[정식 입력과 출처](../../pdf/issue8183/README.md),
[전쪽 TSV·대표 PNG](../pr/assets/issue_8183/README.md),
[base 34행 대조](../pr/assets/issue_8183/base-comparison.json)를 보존했다.
검증 조건은 96 DPI·print·compat 2022다. source image의 완전성은 별도로 전체 픽셀을 대조했다.

| 경로 | 쪽 수 | 최저 raw % | 판독 |
| --- | ---: | ---: | --- |
| 원본 PDF → Native | 17 | 100 | 완료 |
| 원본 PDF → fresh WASM | 17 | 100 | 완료 |
| 저장 HWPX → Native / fresh WASM | 각각 17 | 각각 100 | 완료 |
| 같은 HWPX → base Native / fresh WASM | 각각 17 | 각각 100 | 완료 |

base는 원본의 선택 코덱을 지원하지 않으므로 같은 저장 HWPX 바이트의 표시를 비교했다.
모든 페이지에서 후보와 base의 raw 차이는 0pp다. 원본 PDF를 base가 가져왔다고 보고하지 않는다.
대표 9종의 전체 본문, 표의 행·열·외곽, 그림 경계·mask·색상·alpha를 직접 확인했다.
100%는 2px 내용 실루엣 일치율이며 모든 픽셀의 완전 일치율은 아니다.
스캔 문자는 원래 페이지의 래스터 영상이며 OCR이나 글꼴 조판 개선을 주장하지 않는다.

### 잘못된 스캔 생성 자료의 보정

최초 직접 판독에서 본문 누락을 발견했다. TIFF의 첫 strip만 옮긴 생성 오류였다.
최초 자료·ledger는 `inputs-initial*`에 보존했다. 단일 strip으로 보정한 뒤 전체 bitmap을
압축 전 원본과 MuPDF 추출로 대조했다. 96·200 DPI 모두 전체 바이트가 일치했다.
Poppler 추출의 경계 차이도 기록했다. 제품 decoder는 바꾸지 않았으며, 보정한 자료로 Native·
WASM 소비·TSV·대표 PNG를 다시 만들었다. 이전 자료의 완전성을 통과로 바꾸지 않는다.

### PDFium의 JPX 해석 차이

원본 PDFium 153 대조에서 alpha 두 쪽은 51.40187%, 75.70093%였다. Indexed는 실루엣이
100%여도 색상이 검정이었다. 사양의 source sample과 Ghostscript 10·MuPDF를 대조했다.
PDFium은 alpha=1에 alpha를 두 번 적용했고 alpha=2의 preblend를 복원하지 않았다.
PDF Indexed가 JP2 palette를 덮어쓰는 사양상 lookup도 원래 blue/cyan과 달랐다.

원본 PDF만 Ghostscript pdfwrite에서 PDF 1.4 Flate·외부 SMask로 독립 변환한 세 기준을
별도로 보존했다. RHWP 출력으로 기준을 만들지 않았고 sweep의 임계값·색상 보정도 바꾸지 않았다.
명령·source/hash는 [입력 provenance](../../pdf/issue8183/provenance.json)와
`independent/reference-policy.json`, 원래 미달 TSV는 `visual/native-direct`에 남겼다.
근거: [PDF 1.7 Table 89](https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf),
[PDFium의 해당 미구현 기록](https://pdfium.googlesource.com/pdfium.git/+/b18d018b2f9aa2913cc435b5ccdd59d0a5856ad8).

## 저장본의 한컴 Print — 미달을 유지

[한컴 Print 원본과 재저장 대조군](../../pdf/issue8183/hancom-print/README.md),
[양쪽 TSV·픽셀 증거](../pr/assets/issue_8183/hancom-print/README.md)를 확인한다.
한컴 2024 13.0.0.3901·Print method 0으로 9종을 출력했다.
7종은 양쪽 모두 99% 이상, 두 mask 자료는 100%다.
**alpha=1은 Native/fresh WASM 모두 27.39104%, alpha=2는 모두 52.99065%로 미달이다.**

이 두 자료를 한컴이 HWP로 다시 저장하게 했고 내장 PNG의 전체 RGBA가 원래와 같음을 확인했다.
그 HWP의 Print도 처음 HWPX Print와 같은 4×2·1비트 mask(`c0e0`, `50d0`)를 만들었으며,
양쪽 인쇄의 페이지 픽셀이 일치했다. 연속 alpha를 작은 원래 해상도에서 이진화하는 Print 경로의
제한이다. Native/fresh WASM과 base의 HWPX 렌더링은 원본 PDF의 연속 alpha를 보존한다.
이를 맞추기 위해 source alpha를 버리거나 decoder·저장 PNG를 확대하지 않았다.

이 원인 확인은 미달 비교를 통과로 바꾸거나 예외를 승인한 것이 아니다.
**Print 대조의 `re_review_required`를 남기며 PR 제출·완료 판정을 보류한다.**
일반 PDF 가져오기 정확성과 한컴의 모든 인쇄 경로 호환성을 구분해 사용자 가이드에도 기록했다.

## 회귀와 검증 명령

원본 PDF 기준의 전쪽 Native/fresh WASM 90% 이상과 직접 판독 뒤 정식 자료·검사를 편입했다.
`tests/cases/`의 두 파일은 기존 자동 suite 수집을 사용한다. CI에 중복 호출은 추가하지 않았다.
기본 CI는 비선택 구성의 검사를 수집한다. opt-in 전용 검사의 7건 실행과 네 WASM 브라우저
검증은 이번 단계의 명시적 로컬 feature 실행이며, 기존 CI가 이를 모두 실행한다고 주장하지 않는다.
source sample·전체 bitmap, 리소스·쪽 소유와 저장 후 보존을 검사하며 절대 배치 좌표·SVG hash를
고정하지 않는다. 스캔 생성기는 격리 경로에서 입력 PDF 20개를 바이트 단위로 재현했다.
Print 미달이 남아 있으므로 편입한 회귀도 현재 작업 브랜치의 WIP이며 제출 완료로 간주하지 않는다.

실행 명령·시간·종료 코드는 `rust-gates.json`, `focused-gates.json`과 각 `gate-*.log`에 보존한다.
Native·WASM 원본 소비는 `native-matrix-corrected-inputs.json`, `wasm-*-probe.json`,
Studio는 `studio-result.json`·`studio-artifact-sync.json`이다.

| 항목 | 실제 결과 |
| --- | --- |
| Rust fmt·Native root Clippy·WASM32 lib Clippy·workspace build·all-target Clippy | 통과 |
| suite policy | 1689 sources / 7408 attrs / 48 targets, 통과 |
| `release-test` 전체 nextest | **11,366 PASS / 0 FAIL / 45 skip**, 실행 710.309초 |
| Native Skia library | root 3859 PASS·12 ignore, workspace 182 PASS |
| Native Skia placeholder / direct PDF | 2 PASS / 14 PASS |
| doc tests | 8 PASS / 3 ignore |
| 양쪽 opt-in 정식 회귀 | **7 PASS / 0 FAIL**, 다른 suite 항목은 filter로 제외 |
| 양쪽 opt-in test Clippy / standalone backend Clippy | 통과 |
| 요청 자원 helper `--features scoped` | **4 PASS**, 기본 feature의 0건 실행과 구분 |
| CLI `cargo install --path . --locked --profile release-test --features native-skia,pdf-jbig2,pdf-jpeg2000 --root <격리 경로>` | 통과 |
| 설치한 CLI | 200 DPI 세 입력 수용·재열기, 작업 예산 초과 exit 1, 기존 파일 SHA 보존, 다음 정상 요청 |
| 정식 브라우저 계약 검사 | default/JBIG2/JPX/양쪽 각각 1 PASS, 내부 검사 50/86/119/158건 |
| Studio TypeScript / npm test | 통과 / 2361 PASS·0 FAIL·기존 2 skip |
| source preflight / policy unittest | source PASS, 12 packages·15 runtime includes / 18 PASS |
| suite policy 단위 검사 / unit-tier policy | 23 PASS / 통과 |
| Markdown 링크·공백 | 통과 |

초기 all-target Clippy에서 비활성 feature의 상수 assertion 경고를 발견해 같은 기대를
`#[cfg]`로 한정했다. `2a58c5f37`에서 fmt·all-target Clippy·전체 회귀를 다시 실행했다.
Native root/WASM lib lint의 제품 입력은 이전 성공 source `8f679c3bc`와 동일하며,
제품 source `cfc5be8fe` 뒤의 source diff가 테스트뿐임을 확인했다.
scoped helper의 첫 standalone 호출은 기본 feature라 0건이었다. 이를 예산 검증으로 세지 않고
`--features scoped`로 4건을 실제 실행했다. 실패·미수집 기록도 원래 로그에 보존한다.

source preflight는 Cargo crate 정규화·registry 소비·게시의 검증이 아니며
`publish_ready=false`, 기존 manifest blocker 11건을 유지한다. 빌드 도중 CI를 취소하거나
재실행하지 않았고 GitHub에 테스트를 대신 실행하도록 넘기지 않았다.


## 성능 관측

WSL2 Linux 6.6.114.1·Intel i9-9900KF(8 core/16 thread)·Rust 1.93.1·Chrome 154에서
작업 소유 Cargo/회귀를 종료한 뒤 측정했다. 다른 사용 프로세스는 변경하지 않았다.
각 조합 11회, Native는 2회·WASM은 3회 준비 실행 후 순서를 교대했다. Native와 WASM 각각
154개 측정의 [원문 TSV](../pr/assets/issue_8183/performance/README.md)를 보존했다.

Native는 새 CLI process의 읽기·가져오기·HWPX 저장·JSON 전체 wall time과 `/usr/bin/time`의
process peak RSS다. WASM은 모듈을 재사용한 동기 `HwpDocument.importPdf()`만 측정했고,
export·화면 렌더링을 포함하지 않는다. WASM 메모리는 선형 heap의 유지 용량이며 process RSS나
현재 live allocation이 아니다. 두 경로의 시간을 직접 비교해 backend 속도 우열을 판정하지 않는다.

| 입력 | 구성 | Native ms 중앙값 (min–max) | Native peak RSS 중앙값 MiB | WASM ms 중앙값 (min–max) | WASM linear capacity MiB |
| --- | --- | ---: | ---: | ---: | ---: |
| jpeg-control | base | 31.10 (30.37–33.17) | 24.40 | 31.80 (31.30–40.40) | 8.88 |
| jpeg-control | default | 31.84 (30.53–32.75) | 24.41 | 31.80 (31.50–45.00) | 8.88 |
| jpeg-control | jbig2 | 31.37 (30.72–33.25) | 24.40 | 31.80 (31.50–43.80) | 8.88 |
| jpeg-control | jpx | 31.86 (30.75–32.85) | 24.53 | 32.00 (31.40–37.30) | 8.94 |
| jpeg-control | both | 31.69 (30.32–33.49) | 24.66 | 32.10 (31.50–41.40) | 8.94 |
| jpeg-control-200 | base | 77.29 (76.47–82.76) | 32.41 | 115.20 (113.00–151.50) | 23.75 |
| jpeg-control-200 | default | 76.69 (75.19–93.56) | 32.17 | 115.50 (114.00–121.10) | 23.75 |
| jpeg-control-200 | jbig2 | 76.87 (75.86–93.17) | 32.41 | 115.30 (113.90–132.80) | 23.75 |
| jpeg-control-200 | jpx | 76.08 (74.50–78.68) | 32.41 | 115.50 (114.50–132.70) | 23.81 |
| jpeg-control-200 | both | 76.51 (74.80–77.83) | 32.29 | 117.40 (113.30–121.00) | 23.81 |
| jbig2-200 | jbig2 | 29.42 (28.40–32.44) | 23.93 | 32.00 (31.50–32.80) | 23.75 |
| jbig2-200 | both | 29.04 (28.01–30.38) | 24.05 | 32.30 (31.70–33.20) | 23.81 |
| jpx-200 | jpx | 131.20 (128.87–141.56) | 67.07 | 176.10 (174.90–181.60) | 60.94 |
| jpx-200 | both | 130.42 (128.41–139.31) | 66.95 | 173.70 (170.80–178.80) | 60.94 |

같은 JPEG 입력에서 default의 base 대비 중앙값 차이는 Native 96/200 DPI 각각 +2.38%/−0.78%,
WASM은 0.00%/+0.26%였다. 양쪽 선택은 Native +1.88%/−1.01%, WASM +0.94%/+1.91%다.
이 표본에서 기존 JPEG의 일관된 큰 지연 증가는 관측하지 못했지만 무영향을 보장하는 결과는 아니다.
관측 범위와 min–max를 함께 보존했다. Native default의 JPEG peak RSS 중앙값은 base와 거의 같고,
양쪽 WASM은 JPEG에서 선형 용량이 64 KiB 더 컸다.

200 DPI JPX의 양쪽 빌드는 Native 130.42 ms·peak RSS 중앙값 66.95 MiB,
WASM 173.70 ms·선형 용량 60.94 MiB였다. 기존 base는 이 코덱을 지원하지 않아 전후 속도 비율을
만들지 않는다. JBIG2·JPEG·JPX는 압축·bit depth가 달라 형식끼리 같은 작업의 속도 순위로 보지 않는다.
논리 `codecAllocatedBytes`·`codecWorkUnits`를 RSS·실제 CPU 명령 수·시간 상한으로 해석하지 않는다.

재현할 때 각 source를 표에 기록한 프로필로 먼저 빌드하고 `rhwp import-pdf`에 같은 입력을 넣는다.
예: `/usr/bin/time -f '%e %U %S %M' <고정-binary> import-pdf pdf/issue8183/scan-jpeg-control-200.pdf output/bench/result.hwpx --json`.
WASM은 같은 패키지의 `HwpDocument.importPdf(bytes, {})` 전후 `performance.now()`와
`wasm.memory.buffer.byteLength`를 수집한다. 준비 실행과 11회 교대 순서를 유지한다.


## 설치 중 관측한 기존 lockfile 경고

`cargo install --locked`에서 `chacha20 0.10.1`의 yank 경고를 확인했다. base와 현재 lockfile의
버전·checksum은 같다. codec 의존 추가로 유입한 변경이 아니며 lock을 임의로 재해석하지 않았다.
[공식 변경 기록](https://github.com/RustCrypto/stream-ciphers/blob/master/chacha20/CHANGELOG.md)은
0.10.1의 yank와 0.10.2의 RNG·legacy SSE2 backend 수정 사항을 기록한다.
이 경고를 무시한 배포 안전 판정은 하지 않는다. source preflight 통과나 위 설치 검증은
registry 게시·모든 CPU의 실행 검증을 대신하지 않으며, 릴리스 전에 기존 lock의 검토가 필요하다.

## 남은 조건

한컴 Print alpha 두 쪽의 미달 판정은 유지한다. 별도 이슈로 분류하거나 원본 PDF 대조 점수로
제출 조건을 대체하지 않았다. 이 결과를 포함한 검토가 필요하며 원격 PR은 열지 않았다.
코덱 선택 빌드는 #8174의 배포·MSRV 결정, crates.io 게시, 0.9.0 릴리스를 자동으로 완료하지 않는다.
