# PDF 이미지 코덱 선택 빌드

PDF 가져오기는 기본 빌드에 포함됩니다. JBIG2와 JPEG 2000 그림은 필요한 코덱을 빌드 시
선택합니다. 파일 확장자는 같아도 쪽마다 그림의 압축 방식이 다를 수 있습니다.

이 안내는 #8183을 포함한 source에 적용합니다. 현재 crates.io 게시 완료를 뜻하지 않습니다.
공식 배포 바이너리·Studio의 지원은 그 산출물에 실제 포함한 feature로 결정됩니다.

## 필요한 코덱 확인

```bash
rhwp capabilities
pdfimages -list input.pdf
```

`capabilities`의 `pdfImportSupport`는 `enabled`, `jbig2`, `jpeg2000` 세 boolean입니다.
`enabled`는 PDF 가져오기, 나머지는 해당 decoder를 포함했는지 나타냅니다.
`pdfimages`는 별도 Poppler 도구이며 `jbig2`는 `pdf-jbig2`, `jpx`는 `pdf-jpeg2000`에 대응합니다.
PDF의 `/Filter /JBIG2Decode`·`/JPXDecode`도 각각 같은 의미입니다.

가져오기 성공 봉투의 `diagnostics.pdfImportSupport`도 같은 정보를 담습니다.
이 정보는 입력에 들어 있는 코덱 목록이나 모든 PDF의 호환성 보장이 아닙니다.

## Native CLI 설치와 빌드

source checkout에서 필요한 것만 선택합니다. 아래 명령은 저장소 루트에서 실행합니다.

```bash
# 로컬 source에서 CLI 설치. 기존 기본 feature도 유지합니다.
cargo install --path . --locked --features pdf-jbig2
# JPEG 2000만 필요하면 --features pdf-jpeg2000
# 둘 다 필요하면 --features pdf-jbig2,pdf-jpeg2000

# 검토용 빌드: 기존 공용 target을 재사용합니다.
CARGO_TARGET_DIR="${rhwp_review_target_dir:?공용 target의 절대 경로를 먼저 지정하세요}" \
  cargo build --locked --release --bin rhwp --features pdf-jbig2,pdf-jpeg2000
```

`native-skia`는 Native PNG·PDF·Sixel·Kitty 출력의 별도 선택입니다. 코덱 선택만으로
이 출력 기능을 활성화하지 않습니다. CLI는 기존 `wasm-api` feature도 필요하므로 특별한
이유 없이 위 CLI 명령에 `--no-default-features`를 추가하지 않습니다.

```bash
rhwp capabilities
rhwp import-pdf input.pdf imported.hwpx --json
rhwp info imported.hwpx --json
```

PDF 원본의 선택 쪽은 `--source-pages 0,2-3`처럼 0부터 지정합니다.
저장한 HWP/HWPX에는 복호화한 그림 리소스를 넣으므로 다시 열 때 원 PDF 코덱은 필요하지 않습니다.
변환은 원 저작 문단·복잡한 paint를 완전히 복원하는 기능이 아니므로 결과를 확인합니다.

## Rust 라이브러리

현재 checkout을 소비하는 앱의 예입니다. `path`는 실제 checkout 경로로 바꿉니다.

```toml
[dependencies]
rhwp = { path = "/path/to/rhwp", default-features = false, features = ["pdf-jbig2"] }
```

```rust
let support = rhwp::pdf_support::pdf_import_support();
assert!(support.enabled && support.jbig2);
let result = rhwp::parser::pdf::parse_pdf_with_options(bytes, &Default::default())?;
let saved = rhwp::serializer::serialize_hwpx(&result.document)?;
```

JPEG 2000은 `pdf-jpeg2000`, 양쪽은 두 feature를 함께 지정합니다. 각 코덱은 `pdf-import`도
포함합니다. JS facade·자동 start는 `wasm-api`를 따로 선택한 경우에만 포함합니다.
Cargo feature는 합쳐지므로 앱의 다른 의존성이 선택한 기능도 최종 산출물에 포함될 수 있습니다.
지원 정보는 backend의 실제 합산된 feature를 읽으므로 이 경우도 반영합니다.
registry 의존·설치는 해당 feature가 들어간 버전이 실제 게시된 뒤 그 버전을 지정합니다.

## WASM과 Studio

코덱은 런타임 설정이나 내려받는 플러그인이 아닙니다. WASM을 다시 빌드합니다.
[개발 환경 가이드](dev_environment_guide.md#wasm-빌드)의 도구·공용 target을 준비한 뒤 실행합니다.

```bash
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 \
CARGO_TARGET_DIR="${rhwp_review_target_dir:?공용 target의 절대 경로를 먼저 지정하세요}" \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg --features pdf-jbig2,pdf-jpeg2000
```

기본 feature를 유지하므로 `wasm-api`도 포함됩니다. `--out-dir pkg`가 성공하면 wrapper가
`rhwp-studio/public/`의 JS·WASM을 동기화합니다. 다른 out-dir은 자동 동기화하지 않습니다.
최적화 종료와 pkg/public 파일 해시를 확인하고 실행 중인 Studio에서 캐시를 새로 고칩니다.
실제 HTTP로 받은 JS·WASM 바이트도 같은 해시인지 확인합니다.

초기화한 실제 WASM의 `HwpDocument.getPdfImportSupport()`를 JSON으로 읽으면 문서를 열기 전에
지원 코덱을 알 수 있습니다. `supportsPdfImport()`는 PDF 입력 자체의 활성 여부만 반환합니다.
오래된 JS glue와 새로운 WASM을 섞지 않습니다. 직접 `HwpDocument.importPdf()`와 현재
Studio의 문서 열기는 동기 호출입니다. UI에서 진행 중인 동기 WASM을 timer로 중단할 수 있다고
가정하지 않습니다. 별도 Worker를 쓰는 호스트는 공통 `rhwp-shared/pdf/import-session.js`의
입력 복사 전 정책 조회, 취소·시간 초과 후 Worker 종료·재생성 계약을 사용할 수 있습니다.

## 오류와 자원 한도

| 관측 | 대응 |
| --- | --- |
| `PDF_IMPORT_DISABLED` | PDF 자체를 제외한 빌드입니다. `pdf-import` 또는 필요한 코덱 feature를 포함합니다. |
| `PDF_PARSE_FAILED`와 미활성 filter 진단 | 현재 산출물의 지원 정보를 확인하고 해당 코덱만 선택해 다시 빌드합니다. 손상된 입력도 이 코드일 수 있으므로 코드만으로 미활성을 단정하지 않습니다. |
| `PDF_IMAGE_UNSUPPORTED` | 지원하지 않는 mask·색 공간·성분 조합일 수 있습니다. 진단과 원본을 확인합니다. |
| `PDF_RESOURCE_LIMIT` | 현재 요청 예산을 초과했습니다. 지원 코덱이어도 발생합니다. 원본 도구에서 해상도·그림 수를 줄이거나 선택 쪽을 나눕니다. |

선택 빌드도 기존 단일 예약·누적 decoded·그림 픽셀 한도를 유지하며 decoder 내부 저장소와
작업을 같은 요청 예산으로 계상합니다. `resourcePolicy.backend.codecWorkUnits`와
`resourceUsage.codecWorkUnits`는 논리 작업량이며 밀리초·CPU 명령 수가 아닙니다.
한도는 서로 독립적입니다. 픽셀 한도 이내라도 임시 평면·wavelet·PNG 버퍼의 단일/누적 예약으로
거부될 수 있습니다. 원본 해상도를 자동으로 줄여 수용하지 않습니다.

코덱 한도 검증은 전체 parser·font·cache의 할당 상한이나 process RSS·실행 시간의 보장이
아닙니다. `parserAllocationBoundsProven=false`는 유지합니다. ICC CMM·OCR·암호 PDF의
제한도 해제하지 않습니다. [보안과 기능 제한](security_feature_limits.md)을 함께 확인합니다.
