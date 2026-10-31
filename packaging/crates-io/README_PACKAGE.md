# rhwp — Rust 문서 엔진과 CLI

HWP/HWPX 문서의 파싱·편집·저장·렌더링을 Rust 애플리케이션과 자동화 작업에 연결합니다.
주 사용자는 문서 처리 기능을 자신의 프로그램에 포함하는 Rust 개발자와 CLI 자동화 사용자입니다.

**현재 crates.io 게시 준비 중입니다.** 아래 0.9.0 registry 의존·설치 명령은 게시 후의 사용 안내이며,
현재 게시 완료를 뜻하지 않습니다. 이번 로컬 검증 후보는 0.9.0입니다. 제품 릴리즈와 crates.io 게시 일정은 별도로 결정합니다.

## Rust에서 구조 읽기와 저장

게시 후에는 Cargo 의존성으로 사용합니다.

```toml
[dependencies]
rhwp = "0.9.0"
```

구조만 필요하면 core 세션을 만들지 않고 parser가 반환한 공통 Document IR을 사용합니다.

```rust
use rhwp::{parse_document, serializer::serialize_hwpx_with_report};

fn read_and_save(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let document = parse_document(bytes)?;
    for section in &document.sections {
        for paragraph in &section.paragraphs {
            println!("{}", paragraph.text);
        }
    }
    let saved = serialize_hwpx_with_report(&document)?;
    // 보고 대상 손실이 있으면 호스트가 저장 수용 여부를 결정합니다.
    if !saved.content_loss().is_empty() {
        eprintln!("{}", saved.content_loss().to_json());
    }
    Ok(saved.into_bytes())
}
```

위 루프는 최상위 문단의 본문을 읽는 예제입니다. 셀·각주·머리말 등 모든 story를 평탄화하지 않습니다.
표 구조는 기존 `document_core::queries::table_extract::extract_tables(&document)`를 사용할 수 있습니다.

- `parse_document`는 형식을 감지해 IR을 반환합니다. 비밀번호는 `parse_document_with_password`에 전달합니다.
- `serialize_document`/`serialize_hwp`는 **HWP 출력**이며 입력 형식을 자동 유지하지 않습니다.
  HWPX 출력에는 `serializer::serialize_hwpx`를 선택합니다.
- `*_with_report`는 저장 바이트와 해당 경로에서 수집하는 `ContentLossReport`를 반환합니다.
  보고서가 비었다고 모든 형식 변환의 무손실을 보장하지 않습니다. HWP5에 표현이 없는 필드
  파라미터 등 별도 진단이 있는 범위도 있습니다.

## 쪽 기반 조회와 편집

기존 `service`는 열기 옵션·형식·조회 결과와 `ServiceError`를 제공합니다. 편집은 열린 세션에서
`DocumentCore`를 가져가 수행합니다.

```rust
use rhwp::service::{DocumentService, OpenOptions};

fn edit(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let opened = DocumentService::new()
        .with_title_scan(false)
        .open_bytes(bytes, &OpenOptions::new())?;
    let mut core = opened.into_core();
    core.insert_text_native(0, 0, 0, "추가 문구 ")?;
    Ok(rhwp::serializer::serialize_hwpx(core.document())?)
}
```

core를 열면 스타일 해소·줄 구성·페이지 계산을 수행합니다. `with_title_scan(false)`는 제목 추정을
끄며 이 조판 비용을 제거하지 않습니다. 단순 수집·색인에는 parser/IR 경로부터 검토하세요.
`HwpError`·`ParseError`·`SerializeError`·`ServiceError`는 Rust의 표준 오류 반환 경로에 연결할 수 있습니다.
이 예제가 모든 공개 `pub` 항목의 장기 호환성을 약속하는 것은 아닙니다.

## feature와 빌드 범위

| 선택 | 의미 |
| --- | --- |
| 기본값 | `console_error_panic_hook`, `pdf-import`, `pdf-jbig2`, `pdf-jpeg2000`, `pdf-icc`, `wasm-api` |
| `default-features = false` | 기본 PDF 입력·JS facade/start를 제외. renderer와 Native PDF 출력 의존성은 남으므로 parser-only 빌드가 아닙니다. |
| `pdf-import` | 현재 보안 제한과 자원 정책을 적용하는 PDF 가져오기 |
| `pdf-jbig2` / `pdf-jpeg2000` | PDF 입력과 해당 이미지 코덱. 기본 포함 |
| `pdf-icc` | PDF ICC 색 변환. 기본 포함 |
| `wasm-api` | JS facade·export·자동 초기화. 현재 주 CLI도 필요로 합니다. |
| `native-skia` | Native PNG·direct PDF·Sixel·Kitty 출력 |
| `dev-tools` | 저장소 개발 도구 `font-metric-gen` 빌드. 일반 제품 CLI 설치에는 필요하지 않습니다. |

다른 Rust WASM 앱이 자신의 진입점을 소유할 때는 `default-features = false`로 링크하고,
필요할 때만 `wasm-api`를 선택합니다. Cargo feature는 합쳐지므로 다른 의존성이 켜면 다시 포함됩니다.
공식 최소 Rust 버전은 1.93.1입니다. 실제 검증한 대상은 Linux/WSL2의
`x86_64-unknown-linux-gnu`와 브라우저의 `wasm32-unknown-unknown`입니다. Windows·macOS CLI
릴리즈 경로와 crates.io 소비 검증은 구분하며, 그 밖의 target과 하위 Rust 버전을 약속하지 않습니다.

## CLI 설치와 자동화

게시 후 주 CLI만 설치하는 명령은 다음과 같습니다.

```sh
cargo install rhwp --version 0.9.0 --locked --bin rhwp
rhwp info input.hwpx --json
rhwp export-text input.hwpx --json
rhwp import-pdf input.pdf output.hwpx --resource-profile constrained --json
rhwp capabilities
rhwp mcp-serve
```

`rhwp-q`와 `rhwp-agent`는 별도 제품 bin입니다. 필요한 bin은 `--bin`으로 명시하세요.
PNG·direct PDF·Sixel·Kitty가 필요하면 설치 시 `--features native-skia`를 추가합니다.
PDF 입력을 제외한 주 CLI에는 `--no-default-features --features wasm-api`를 사용합니다.
한글 폰트 등 출력 환경은 `--font-path`와
[CLI 매뉴얼](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/cli_commands.md)을 따릅니다.

소스 작업공간에서 개발 도구는 다음처럼 빌드합니다.

```sh
cargo build --features dev-tools --bin font-metric-gen
```

## 입력과 보안 지원 범위

PDF 가져오기·JBIG2·JPEG 2000·ICC 색 변환을 기본 포함하며, 손상 xref는 요청 예산 안에서
복구합니다. 지원하지 않는 profile·JPX 조합과 복구 불가 입력은 남아 있습니다.
비신뢰 PDF는 아래 요청 정책과 격리 host 계약을 지킨 환경에서 처리하는 지원 대상입니다.
직접 Rust 호출만으로 서비스 격리가 구성되거나 전체 heap·CPU 상한이 보장되지는 않습니다.
파서·제품 요청·host 각각의 한도와 책임을 구분해야 합니다.
현행 제한과 일반 미지원의 차이는
[보안 제한 현황](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/security_feature_limits.md)에 있습니다.
Rust에서 직접 PDF를 가져올 때도 같은 요청 정책을 전달할 수 있습니다.

```rust
use rhwp::{DocumentCore, parser::pdf::{PdfImportOptions, PdfImportPolicy, PdfImportProfile}};

fn import_pdf(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let policy = PdfImportPolicy::for_profile(PdfImportProfile::Constrained)
        .with_limits_json(r#"{"maxSelectedPages":16}"#)?;
    let core = DocumentCore::from_pdf_bytes_with_policy(
        bytes, &PdfImportOptions::default(), policy,
    )?;
    Ok(rhwp::serializer::serialize_hwpx(core.document())?)
}
```

이 Rust 호출은 동기 실행입니다. CLI의 `import-pdf`는 별도 작업자의 실행 시간·주소 공간·출력 크기를
감독하지만, 라이브러리 호출에 프로세스 격리를 자동으로 추가하지 않습니다. 서버는 별도
작업자·시간 제한·파일 및 네트워크 접근 경계를 구성해야 합니다. 패키지에 포함된
`cargo run --example pdf_isolation_host -- input.pdf output.hwpx --verify`는 Linux에서 새 자식의
Landlock·seccomp 제한과 합성 비공개 파일·네트워크 차단을 직접 확인하는 출발점입니다.
해당 Linux `import-pdf`도 같은 confinement를 사용하며 필요한 커널 기능이 없으면 실패합니다.
한도 항목과 CLI 옵션,
WASM의 `resourceProfile`·`resourceLimits` 및 중단 가능한 Worker 사용은
[PDF 자원 정책 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_resource_policies.md)를 따릅니다.

F07 명령 단추의 제한된 실행기와 F08 암호화 복구 저장소는 Studio의 브라우저 기능입니다.
Rust crate에는 양식·스크립트의 보존과 주소 기반 조회·갱신 경로가 있으며, 일반 JScript 실행기를
제공하는 것은 아닙니다. Studio의 실행 허용·세션 키·복구와 host 책임은
[Studio 문서 기능 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/studio_document_features.md)에 있습니다.

이 README는 #8174 v2의 게시 후보 source를 설명하며 실제 registry 게시 완료를 뜻하지 않습니다.

MIT 라이선스와 포함된 font·fork의 고지를 함께 확인하세요. 본체와 보정 fork의 source 출처·고지를
유지하며, 공식 upstream 패키지와 rhwp의 보정 패키지를 구분합니다.

로컬 source CLI 설치는 `cargo install --path . --locked --bin rhwp`로 기본 기능을 포함합니다.
Rust 앱은 `rhwp = { path = "/path/to/rhwp" }`로 소비할 수 있습니다. 해당 버전의 실제
registry 게시 뒤에만 `path`를 게시 버전으로 대체합니다. 명시적인 개발용 제외 구성은
`default-features = false`와 `pdf-import`·`pdf-jbig2`·`pdf-jpeg2000`·`pdf-icc`의 조합으로 정합니다.
CLI의 제외 빌드는 `wasm-api`도 필요합니다. Cargo feature는 의존성 사이에서 합산됩니다.
`parserAllocationBoundsProven=false`는 유지하며 자세한 설치·WASM·오류 안내는
[PDF 이미지 코덱 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/pdf_image_codecs.md)를 참고하세요.

## 0.9.x Rust 호환 계약

공식 소비 진입점은 `parse_document`·`parse_document_with_password`, `model::Document`와
그 section/paragraph/table IR, `DocumentService`/`OpenOptions`·`DocumentCore`의 공개
Native 조회·편집, `serializer`의 HWP/HWPX 저장과 `*_with_report`, 표준 오류 연동입니다.
PDF의 `PdfImportOptions`·`PdfImportPolicy`/`PdfImportProfile`과 지원 조회도 이 계약에 포함합니다.
0.9.x 패치에서는 이 경로의 소스 호환과 문서 의미·저장 보존을 유지하며, 깨지는 변경은
0.10 이상의 minor에서 이행 안내와 함께 제공합니다. 버그 보정으로 결과가 달라질 수 있으므로
정확한 픽셀·쪽수·진단 문구나 비공개 저장 형식까지 고정하는 계약은 아닙니다.

`renderer`/`layout`/`typeset` 내부 구조, `wasm_api`의 제품 facade, 내부 보정 fork의 직접 API,
개발 도구와 숨겨진 worker 프로토콜은 이 Rust 소스 호환 계약의 대상이 아닙니다.
공개 `pub`라는 이유만으로 전체 표면의 안정성을 약속하지 않습니다. JS 앱은 별도의
WASM facade 계약을 사용하고, Rust WASM 소비자는 자신의 start를 소유할 수 있습니다.
지원표·공개 전 점검·부분 실패 복구는
[crates.io 수동 운영 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/crates_io_guide.md)를 따릅니다.
