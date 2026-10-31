---
kind: guide
status: active
canonical: mydocs/manual/pdf_image_codecs.md
last_verified: 2026-10-24
---

# PDF 이미지 코덱과 색 변환

PDF 가져오기·JBIG2·JPEG 2000·ICC 색 변환은 기본 빌드에 포함됩니다. 일반 사용자는
코덱별 feature를 추가하지 않아도 됩니다. 파일 확장자는 같아도 쪽마다 그림의 압축 방식과
색 공간이 다를 수 있습니다.

이 안내는 #8174 v2-S02·S03의 로컬 source에 적용합니다. `devel` 반영이나 crates.io 게시 완료를 뜻하지 않습니다.
공식 배포 바이너리·Studio의 지원은 그 산출물에 실제 포함한 feature로 결정됩니다.

## 필요한 코덱 확인

```bash
rhwp capabilities
pdfimages -list input.pdf
```

`capabilities`의 `pdfImportSupport`는 `enabled`, `jbig2`, `jpeg2000`, `icc`,
`boundedXrefRecovery` 다섯 boolean입니다. 각각 PDF 가져오기, 두 decoder, ICC CMM,
예산 있는 손상 xref 복구의 실제 포함 여부입니다.
`pdfimages`는 별도 Poppler 도구이며 `jbig2`는 `pdf-jbig2`, `jpx`는 `pdf-jpeg2000`에 대응합니다.
PDF의 `/Filter /JBIG2Decode`·`/JPXDecode`도 각각 같은 의미입니다.

가져오기 성공 봉투의 `diagnostics.pdfImportSupport`도 같은 정보를 담습니다.
이 정보는 입력에 들어 있는 코덱 목록이나 모든 PDF의 호환성 보장이 아닙니다.

## Native CLI 설치와 빌드

아래 명령은 저장소 루트에서 실행합니다. 기본 기능을 포함한 CLI를 설치합니다.

```bash
# 로컬 source에서 기본 기능을 포함한 CLI 설치
cargo install --path . --locked --bin rhwp

# 검토용 빌드: 기존 공용 target을 재사용합니다.
CARGO_TARGET_DIR="${rhwp_review_target_dir:?공용 target의 절대 경로를 먼저 지정하세요}" \
  cargo build --locked --release --bin rhwp
```

`native-skia`는 Native PNG·PDF·Sixel·Kitty 출력의 별도 선택입니다. 코덱 선택만으로
이 출력 기능을 활성화하지 않습니다. 기본 기능을 명시적으로 제외하는 개발 빌드는 아래와
같습니다. CLI는 `wasm-api`도 필요합니다.

```bash
# 개발용: PDF는 유지하고 JBIG2·JPX·ICC를 제외합니다.
CARGO_TARGET_DIR="${rhwp_review_target_dir:?공용 target의 절대 경로를 먼저 지정하세요}" \
  cargo build --locked --release --bin rhwp --no-default-features --features wasm-api,pdf-import
```

특정 기능만 다시 넣을 때는 `pdf-jbig2`, `pdf-jpeg2000`, `pdf-icc`를 추가합니다.
`boundedXrefRecovery`는 `pdf-import`에 포함되며 별도 활성화 옵션은 없습니다.

```bash
rhwp capabilities
rhwp import-pdf input.pdf imported.hwpx --json
rhwp info imported.hwpx --json
```

PDF 원본의 선택 쪽은 `--source-pages 0,2-3`처럼 0부터 지정합니다.
저장한 HWP/HWPX에는 복호화한 그림 리소스를 넣으므로 다시 열 때 원 PDF 코덱은 필요하지 않습니다.
변환은 원 저작 문단·복잡한 paint를 완전히 복원하는 기능이 아니므로 결과를 확인합니다. 스캔 문자는 그림으로 보존하며 OCR로
검색·편집 가능한 텍스트를 생성하지 않습니다.

## Rust 라이브러리

현재 checkout을 소비하는 앱의 예입니다. `path`는 실제 checkout 경로로 바꿉니다.

```toml
[dependencies]
rhwp = { path = "/path/to/rhwp" }
```

```rust
let support = rhwp::pdf_support::pdf_import_support();
assert!(support.enabled && support.jbig2 && support.jpeg2000 && support.icc);
let result = rhwp::parser::pdf::parse_pdf_with_options(bytes, &Default::default())?;
let saved = rhwp::serializer::serialize_hwpx(&result.document)?;
```

기본 feature에는 JS facade·자동 초기화인 `wasm-api`도 포함됩니다. 라이브러리에서 이를
제외하면서 PDF 기능을 유지하려면 `default-features = false`와
`features = ["pdf-import", "pdf-jbig2", "pdf-jpeg2000", "pdf-icc"]`를 지정합니다.
각 코덱과 `pdf-icc`는 `pdf-import`도 포함합니다.
Cargo feature는 합쳐지므로 앱의 다른 의존성이 선택한 기능도 최종 산출물에 포함될 수 있습니다.
지원 정보는 backend의 실제 합산된 feature를 읽으므로 이 경우도 반영합니다.
registry 의존·설치는 해당 feature가 들어간 버전이 실제 게시된 뒤 그 버전을 지정합니다.

## WASM과 Studio

코덱은 런타임 설정이나 내려받는 플러그인이 아닙니다. WASM을 다시 빌드합니다.
[개발 환경 가이드](dev_environment_guide.md#wasm-빌드)의 도구·공용 target을 준비한 뒤 실행합니다.

```bash
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
CARGO_TARGET_DIR="${rhwp_review_target_dir:?공용 target의 절대 경로를 먼저 지정하세요}" \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

위 예제는 POSIX 셸용입니다. Windows PowerShell에서는 개발 환경 가이드의 공용 target·
환경 변수 설정과 복원 절차를 적용한 뒤 다음 wrapper에 feature를 전달합니다.

```powershell
.\scripts\wasm-pack-locked.ps1 --target web --out-dir pkg
```

기본 feature를 유지하므로 `wasm-api`도 포함됩니다. `--out-dir pkg`가 성공하면 wrapper가
`rhwp-studio/public/`의 JS·WASM을 동기화합니다. 다른 out-dir은 자동 동기화하지 않습니다.
최적화 종료와 pkg/public 파일 해시를 확인하고 실행 중인 Studio에서 캐시를 새로 고칩니다.
실제 HTTP로 받은 JS·WASM 바이트도 같은 해시인지 확인합니다.

초기화한 실제 WASM의 `HwpDocument.getPdfImportSupport()`를 JSON으로 읽으면 문서를 열기 전에
지원 코덱·ICC·복구 기능을 알 수 있습니다. `supportsPdfImport()`는 PDF 입력 자체의 활성 여부만 반환합니다.
오래된 JS glue와 새로운 WASM을 섞지 않습니다. 직접 `HwpDocument.importPdf()`는 동기 호출입니다.
로컬 S04 Studio는 같은 변환기를 전용 Worker에서 실행하고 성공한 후보를 편집 문서로 재열어
교체합니다. 프로필·진행·취소·재시도는 [Studio 사용자 가이드](studio_document_features.md)를 따릅니다.
동기 재열기 자체를 중간에 선점하지는 못합니다. 별도 Worker를 쓰는 호스트도 공통 `rhwp-shared/pdf/import-session.js`의
입력 복사 전 정책 조회, 취소·시간 초과 후 Worker 종료·재생성 계약을 사용할 수 있습니다.
`resourceProfile`·`resourceLimits`와 `getPdfImportPolicyWithOptions()`는
[WASM 자원 정책](pdf_resource_policies.md#wasm과-공통-worker)을 따릅니다. 프로필을 바꿔도
코덱을 비활성화하지 않으며 같은 실제 decoder/CMM에 낮춘 요청 예산을 전달합니다.

## 오류와 자원 한도

| 관측 | 대응 |
| --- | --- |
| `PDF_IMPORT_DISABLED` | PDF 자체를 제외한 빌드입니다. `pdf-import` 또는 필요한 코덱 feature를 포함합니다. |
| `PDF_PARSE_FAILED`와 미활성 filter 진단 | 현재 산출물의 지원 정보를 확인하고 기본 기능을 포함한 빌드인지 확인합니다. 손상된 입력도 이 코드일 수 있으므로 코드만으로 미활성을 단정하지 않습니다. |
| `PDF_IMAGE_UNSUPPORTED` | 지원하지 않는 mask·색 공간·성분 조합일 수 있습니다. 진단과 원본을 확인합니다. |
| `PDF_RESOURCE_LIMIT` | 현재 요청 예산을 초과했습니다. 지원 코덱이어도 발생합니다. 원본 도구에서 해상도·그림 수를 줄이거나 선택 쪽을 나눕니다. |

기본 빌드도 기존 단일 예약·누적 decoded·그림 픽셀 한도를 유지하며 decoder·ICC의
입력 크기에 비례하는 저장소와 작업, xref 복구 scan을 같은 요청 예산으로 계상합니다. `resourcePolicy.backend.codecWorkUnits`와
`resourceUsage.codecWorkUnits`는 논리 작업량이며 밀리초·CPU 명령 수가 아닙니다.
한도는 서로 독립적입니다. 픽셀 한도 이내라도 임시 평면·wavelet·PNG 버퍼의 단일/누적 예약으로
거부될 수 있습니다. 기존 비 JPEG 경로의 8×픽셀 사전 예약과 단일 32 MiB 한도를 함께 적용하면
약 419만 픽셀을 넘는 그림은 먼저 거부될 수 있습니다. A4 200 DPI 스캔은 검증했지만
더 높은 해상도의 수용을 보장하지 않습니다. 원본 해상도를 자동으로 줄여 수용하지 않습니다.

코덱 한도 검증은 전체 parser·font·cache의 할당 상한이나 process RSS·실행 시간의 보장이
아닙니다. `parserAllocationBoundsProven=false`는 유지합니다. CLI의
`--resource-profile standard|constrained`와 `--resource-limits <JSON 파일>`은
[PDF 자원 정책](pdf_resource_policies.md)을 따릅니다. Rust 직접 호출에 CLI 작업자 감독이
자동 적용되지는 않습니다.

ICC는 순수 Rust qcms의 Gray·RGB·CMYK 변환을 실제 이미지·palette·path에 적용하고
alpha를 보존합니다. 유효하지 않거나 지원하지 않는 profile은 기존 대체 색 변환을 쓰며,
자원 한도 초과를 대체 색상의 성공으로 숨기지 않습니다. 복구는 유한한 객체 scan으로
xref와 root를 재구성할 수 있을 때만 성공하며 모호한 stream 경계·암호화 PDF를 우회하지 않습니다.
JPX의 PDF Indexed+내장 mask·CMYK+alpha도 지원 범위를 넓혔지만 모든 JP2 channel 정의와
subsampling 조합을 보장하지 않습니다. Indexed Lab 팔레트의 preblended alpha는 원 성분
범위를 보존하지 못해 명시적으로 거부하며 straight alpha와 구분합니다. OCR·암호 PDF는 계속 미지원입니다.
[보안과 기능 제한](security_feature_limits.md)을 함께 확인합니다.

## 저장본의 다른 앱 출력 확인

RHWP에서 HWP/HWPX로 저장하면 디코딩한 PNG의 색상과 alpha를 보존합니다. 저장한 파일을
다른 앱에서 인쇄할 때는 그 앱의 이미지 출력 방식도 영향을 줍니다. 검증한 한컴 2024
13.0.0.3901의 Print에서는 4×2픽셀 반투명 PNG가 원래 해상도의 1비트 마스크로 출력됐습니다.
한컴이 다시 저장한 HWP도 같은 결과였으며, 저장 파일 안의 8비트 alpha는 유지됐습니다.
작은 반투명 그림의 인쇄가 중요하면 실제 대상 앱의 PDF도 확인합니다. 이 결과를 맞추기 위해
가져온 그림의 alpha를 버리거나 해상도를 자동 변경하지 않습니다.
