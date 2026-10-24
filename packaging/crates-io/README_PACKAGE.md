# rhwp — Rust 문서 엔진과 CLI

HWP/HWPX 문서의 파싱·편집·저장·렌더링을 Rust 애플리케이션과 자동화 작업에 연결합니다.
주 사용자는 문서 처리 기능을 자신의 프로그램에 포함하는 Rust 개발자와 CLI 자동화 사용자입니다.

**현재 crates.io 게시 준비 중입니다.** 아래 0.9.0 registry 의존·설치 명령은 게시 후의 사용 안내이며,
현재 게시 완료를 뜻하지 않습니다. 준비 검증 패키지는 현 제품 버전 0.8.7과 구분해 기록합니다.

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
| 기본값 | `console_error_panic_hook`, `pdf-import`, `wasm-api` |
| `default-features = false` | 기본 PDF 입력·JS facade/start를 제외. renderer와 Native PDF 출력 의존성은 남으므로 parser-only 빌드가 아닙니다. |
| `pdf-import` | 현재 보안 제한과 자원 정책을 적용하는 PDF 가져오기 |
| `pdf-jbig2` / `pdf-jpeg2000` | PDF 입력과 해당 이미지 코덱을 개별 opt-in. 기본 비활성 |
| `wasm-api` | JS facade·export·자동 초기화. 현재 주 CLI도 필요로 합니다. |
| `native-skia` | Native PNG·direct PDF·Sixel·Kitty 출력 |
| `dev-tools` | 저장소 개발 도구 `font-metric-gen` 빌드. 일반 제품 CLI 설치에는 필요하지 않습니다. |

다른 Rust WASM 앱이 자신의 진입점을 소유할 때는 `default-features = false`로 링크하고,
필요할 때만 `wasm-api`를 선택합니다. Cargo feature는 합쳐지므로 다른 의존성이 켜면 다시 포함됩니다.
현재 준비 검증은 Rust 1.93.1을 기준으로 합니다. 검증하지 않은 하위 MSRV나 모든 target을 보장하지 않습니다.

## CLI 설치와 자동화

게시 후 주 CLI만 설치하는 명령은 다음과 같습니다.

```sh
cargo install rhwp --version 0.9.0 --locked --bin rhwp
rhwp info input.hwpx --json
rhwp export-text input.hwpx --json
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

PDF 입력은 기본 포함이지만 일부 codec·손상 구조 복구·색 변환은 제한돼 있습니다.
전체 heap·CPU 상한이나 비신뢰 업로드 서비스의 안전성을 보장하는 계약으로 해석하지 마세요.
파서·제품 요청·host 각각의 한도와 책임을 구분해야 합니다.
현행 제한과 일반 미지원의 차이는
[보안 제한 현황](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/security_feature_limits.md)에 있습니다.
게시 준비에서 현재 제한을 해제하거나 새 보안 지원을 선언하지 않습니다.

MIT 라이선스와 포함된 font·fork의 고지를 함께 확인하세요. 본체와 보정 fork의 source 출처·고지를
유지하며, 공식 upstream 패키지와 rhwp의 보정 패키지를 구분합니다.

PDF 이미지 코덱은 필요한 것만 선택합니다. 로컬 source CLI 설치 예는
`cargo install --path . --locked --features pdf-jbig2`입니다.
Rust 앱의 로컬 의존은 `rhwp = { path = "/path/to/rhwp", default-features = false, features = ["pdf-jpeg2000"] }`처럼
선택합니다. 해당 feature를 포함한 버전의 실제 registry 게시 뒤에만 `path`를 게시 버전으로
대체합니다. 코덱 선택은 기존 요청 한도와 `parserAllocationBoundsProven=false`를 바꾸지 않습니다.
자세한 설치·WASM·오류 안내는 [PDF 이미지 코덱 가이드](../../mydocs/manual/pdf_image_codecs.md)를 참고하세요.
