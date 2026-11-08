---
kind: guide
status: active
canonical: mydocs/manual/pdf_developer_onboarding.md
last_verified: 2026-11-08
---

# PDF 가져오기: 0.9.0 개발자 시작 안내

[English](pdf_developer_onboarding_EN.md). 이 문서는 **0.9.0**의 API·소비 계약이다.
registry·스토어의 버전별 설치 가능 상태는 해당 채널에서 확인한다. 소스에서 빌드할 때도
CLI·JS glue·WASM·Worker·Studio를 같은 릴리즈로 고정한다.

PDF를 편집 가능한 Document IR로 가져와 HWP/HWPX로 저장한다. PDF 뷰어 임베드나 OCR API가
아니며 PDF 원저작 앱의 문단·표 구조를 그대로 복원한다고 보장하지 않는다. HWP/HWPX에서 PDF로
내보내는 역방향은 기존 `export-pdf` 명령이다.

## 어떤 진입점을 선택할까

| 목적 | 진입점 | 실행 경계 |
| --- | --- | --- |
| 배치 변환·서버 작업자 | CLI `import-pdf` | 별도 자식 프로세스, 시간·출력 감독; Linux는 OS 격리 적용 |
| Rust 앱에 통합 | `DocumentCore::from_pdf_bytes_with_policy` | 동기 API; 비신뢰 업로드의 프로세스 격리·종료는 호스트가 구성 |
| WASM 변환을 직접 통합 | `HwpDocument.importPdf` | 동기 API; UI 스레드에서 대형 PDF를 직접 호출하지 않음 |
| 취소·진행률·재시도 | 저장소 `PdfImportSession` 모듈 Worker | 변환·첫 저장의 공통 요청 예산, 실제 Worker 종료·재생성 |
| 완성된 편집 UI | Studio 또는 `@rhwp/editor` | 가져오기 설정·암호·쪽별 진행률·취소·편집·저장 |

## 1. CLI로 첫 문서 변환

Rust **1.93.1**과 저장소의 잠긴 의존성으로 빌드한다. `native-skia`를 더하면 PNG·direct PDF·
Sixel·Kitty 출력도 사용할 수 있다. PDF **입력**은 기본 feature이므로 별도 추가가 필요 없다.

```sh
cargo build --locked --release --bin rhwp --features native-skia
./target/release/rhwp --version
./target/release/rhwp capabilities
./target/release/rhwp import-pdf input.pdf output.hwpx --json
./target/release/rhwp info output.hwpx --json
```

공용 target을 쓰는 저장소 검토 환경에서는 `--target-dir /절대/경로/target/pr-review`를 주고
실행 파일 경로도 그에 맞춘다. 아래 예제의 `rhwp`는 이 바이너리 또는 공식 설치본이다.

```sh
# 원본의 1·3·4쪽. 0-based이며 범위 양 끝을 포함한다.
rhwp import-pdf input.pdf selected.hwpx --source-pages 0,2-3 --json
# 문단 재편집을 우선한다. 기본값 fidelity와 결과 배치가 다르다.
rhwp import-pdf input.pdf editable.hwpx --layout editable --json
# 열기 암호: 명령줄 인자나 로그에 암호를 쓰지 않는다.
rhwp import-pdf protected.pdf opened.hwpx --password-stdin < password.txt
# 가져온 결과를 확인한다. 소유·사용 권한이 있는 실제 글꼴을 공급한다.
rhwp export-png output.hwpx -p 0 -o preview-png --font-path /path/to/fonts --profile print
rhwp export-pdf output.hwpx -o preview.pdf --font-path /path/to/fonts
```

`export-png -o`는 디렉터리를 받으며 그 안에 쪽별 PNG를 만듭니다. `export-pdf -o`는 파일을 받습니다.

`password.txt`는 사용자가 안전하게 관리하는 로컬 입력이며 소스·배포물에 넣지 않는다.
Standard Security R2–R6를 인증하며 빈 열기 암호이면 자동으로 연다. 인증된 사용자/소유자
암호를 허용하고 복사·편집 플래그로 추가 차단하지 않는다. 결과 HWP/HWPX는 **새 문서**로,
입력 PDF 암호를 자동 승계하지 않는다. 출력 암호는 후속 `convert`/`export-hwpx`에서 설정한다.

`fidelity`는 인식한 본문 영역을 위치 고정 글상자로, `editable`은 본문 문단을 구역 흐름과
단·쪽 나누기로 가져온다. 그림·표·특수 영역은 고정 개체로 남을 수 있다. `--table-inference`는
`--json`과 함께 상세 추론 근거를 보여주는 옵션이지 모든 표를 정확히 복원하는 스위치가 아니다.
상세 진단에는 원문이 포함될 수 있으므로 공개 로그로 보내지 않는다.

## 2. Rust 라이브러리

0.9.0부터 crates.io 배포를 시작한다. 아래는 게시 후 사용할 Rust 의존성이다.

```toml
[dependencies]
rhwp = { version = "0.9.0", default-features = false, features = ["pdf-import", "pdf-jbig2", "pdf-jpeg2000", "pdf-icc"] }
```

게시 준비 중에는 `version = "0.9.0"`을 `path = "/path/to/rhwp"`로 바꾸어 같은 버전의 checkout을 사용한다.
실제 공개 버전과 설치 확인은 [crates.io 소비 안내](../../packaging/crates-io/README_PACKAGE.md)를 따른다.
`wasm-api`를 제외해 JS 자동 초기화를 연결하지 않는 구성이다. parser-only 빌드는 아니다.

```rust
use rhwp::{DocumentCore, parser::pdf::{PdfImportOptions, PdfImportPolicy, PdfImportProfile}};

fn import_pdf(bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let options = PdfImportOptions { source_pages: Some(vec![0]), ..Default::default() };
    let policy = PdfImportPolicy::for_profile(PdfImportProfile::Constrained)
        .with_limits_json(r#"{"maxSelectedPages":8}"#)?;
    let core = DocumentCore::from_pdf_bytes_with_policy(bytes, &options, policy)?;
    Ok(rhwp::serializer::serialize_hwpx(core.document())?)
}
```

열기 암호는 `PdfImportOptions.password`에 전달한다. 이 호출에 CLI의 OS 격리가 자동 추가되지는
않는다. 변환부터 첫 저장까지 하나의 예산 수명이 필요하면 `rhwp::document_core::PdfImportSession`과
동봉한 [`pdf_isolation_host` 예제](../../packaging/crates-io/examples/pdf_isolation_host.rs)를 사용한다.
[공식 소비 계약](crates_io_guide.md)과 [저장 손실 보고](../../packaging/crates-io/README_PACKAGE.md)를 함께 확인한다.

## 3. npm core / WASM

출시 후 `npm install @rhwp/core@0.9.0`으로 버전을 고정하고 `rhwp_bg.wasm`을 서버의
`/rhwp_bg.wasm`에 복사한다. 출시 전에는 [locked WASM 빌드](dev_environment_guide.md#wasm-빌드)의
`pkg/`를 사용한다. JS와 WASM은 같은 빌드여야 한다.

```js
import init, { HwpDocument } from '@rhwp/core';
await init({ module_or_path: '/rhwp_bg.wasm' });
const support = JSON.parse(HwpDocument.getPdfImportSupport());
if (!support.enabled) throw new Error('PDF_IMPORT_DISABLED');
const pdfBytes = new Uint8Array(await (await fetch('/input.pdf')).arrayBuffer());
const doc = HwpDocument.importPdf(pdfBytes, {
  sourcePages: [0], layout: 'fidelity', resourceProfile: 'constrained',
  resourceLimits: { maxSelectedPages: 8 },
});
try {
  const diagnostics = JSON.parse(doc.getPdfImportDiagnostics());
  const hwpx = doc.exportHwpx(); // Uint8Array; 호스트가 다운로드/저장한다.
} finally {
  doc.free();
}
```

위 최소 예제는 동기 호출이다. 암호는 options의 `password` 문자열로 전달하고 필요 이상 유지하거나
로그에 기록하지 않는다. 동일 스레드의 `setTimeout`/`AbortSignal`로 동기 WASM을 선점할 수 없다.

### 취소와 쪽별 진행률이 필요한 앱

저장소의 `rhwp-shared/pdf/`와 `pkg/`를 **동일 출처·상대 경로 그대로** 제공한다. 이 Worker는
`@rhwp/core`의 npm export가 아니며, `@rhwp/core` 설치만으로 아래 import 경로가 생기지 않는다.
module Worker·JS MIME·WASM MIME(`application/wasm`)과 호스트 CSP의 Worker 실행을 허용한다.

```js
import { PdfImportSession } from './rhwp-shared/pdf/import-session.js';
const session = new PdfImportSession();
const controller = new AbortController(); // 취소 버튼에서 controller.abort()
try {
  await session.importPdf(pdfBytes, { resourceProfile: 'standard' }, {
    signal: controller.signal, timeoutMs: 60_000,
    onProgress: (phase, page) => {
      if (page) console.log(`원본 ${page.sourcePage}/${page.sourcePageCount}쪽`,
        `${Math.floor(100 * page.completedPages / page.totalPages)}%`);
    },
  });
  const hwpx = await session.exportDocument('hwpx');
} finally {
  session.dispose();
}
```

선택 인덱스는 0-based, `sourcePage` 표시는 **1-based**다. 백분율의 분모 `totalPages`는
가져올 쪽 수이며 `sourcePageCount`는 원본 총 쪽수다. 100%는 쪽 처리 완료이고 Core 구성·저장이
뒤따를 수 있다. 오류·취소에서는 현재 문서를 보존하고 후보만 폐기한다.
[Worker 계약](../../rhwp-shared/pdf/README.md)에 따라 다음 요청에서 새 후보를 사용한다.

## 4. Studio / 임베디드 편집기

Studio에서 PDF를 열면 가져오기 대화창에서 프로필·고급 설정·필요한 열기 암호를 지정한다.
변환 중 원본 현재/전체 쪽과 완료율을 표시하며 취소 후 재시도할 수 있다. 완료 후 HWP/HWPX로 저장한다.

`@rhwp/editor`는 iframe wrapper이므로 0.9.0 Studio를 함께 배포하고 `viewerUrl`로 지정한다.
패키지 번호만 바꾸고 오래된 호스팅 Studio를 그대로 사용하지 않는다.

```js
import { createEditor } from '@rhwp/editor';
const editor = await createEditor(document.getElementById('editor'), {
  viewerUrl: '/studio/index.html', requestTimeoutMs: 300_000,
});
await editor.loadFile(pdfBytes, 'input.pdf', { suppressDialogs: false });
const hwpx = await editor.exportHwpx();
```

PDF 설정·암호 선택은 사용자 응답을 기다린다. `suppressDialogs`는 PDF 선택창을 없애거나 암호를
전달하는 옵션이 아니다. wrapper의 요청 시간과 PDF Worker의 제한 시간은 별개다.
무인 변환에는 CLI/Worker API를 사용한다. VS Code의 기본 파일 연결은 `.hwp`·`.hwpx`이며
PDF 변환 결과를 열 수 있다. 모든 확장에 PDF 직접 파일 연결이 있다고 가정하지 않는다.

## 기본 기능, 한도와 오류 대응

JBIG2·JPEG 2000·ICC 색 변환·예산 있는 xref 복구는 기본 포함이다. 개발용 최소 빌드에서만
선택적으로 제외한다. 실제 지원은 CLI `capabilities`/WASM `getPdfImportSupport()`로 조회한다.

표준 프로필은 입력 64 MiB·선택 1,024쪽·content operators 최대 8,000,000·단일 그림
64,000,000픽셀을 허용한다. 다른 누적/단일 버퍼 예산과 OS/Worker 한도도 동시에 적용된다.
`constrained`는 더 낮은 한도를 사용한다. JSON은 **프로필 상한 안에서 낮추는** 설정이다.

| 상황 | 대응 |
| --- | --- |
| `PDF_PASSWORD_REQUIRED` / `PDF_PASSWORD_INVALID` | 열기 암호를 전달하거나 재입력한다. |
| `PDF_ENCRYPTION_UNSUPPORTED` | 공개키·미지원 crypt filter 등 인증 방식 문제다. 지원 도구로 인증·재저장 후 다시 시도한다. |
| `PDF_RESOURCE_LIMIT` | 먼저 `standard`와 기본 한도로 복원한다. 계속 초과하면 선택 쪽을 나누고 실제 `resourcePolicy`/`resourceUsage`를 확인한다. |
| `PDF_PROCESS_TIMEOUT` / `PDF_IMPORT_TIMEOUT` | CLI 실행 한도와 Worker/UI 제한 시간을 구분한다. Studio의 제한 시간은 1~300초, 기본 30초다. |
| `PDF_IMAGE_UNSUPPORTED` / `PDF_PATTERN_UNSUPPORTED` / `PDF_TEXT_UNSUPPORTED` | 지원하지 않는 표현이다. 예산을 올려도 해결되지 않으며 원본과 진단으로 재현한다. |
| 빈 글자·다른 줄바꿈 | 렌더링하는 서버/클라이언트에 실제 글꼴을 공급한다. 터미널 글꼴 교체만으로 문서 이미지 글꼴이 바뀌지 않는다. |

CLI `--resource-limits` 파일은 `{ "policy": { ... }, "execution": { ... } }`이고,
WASM `resourceLimits`는 **policy 안쪽 객체만** 받는다. 구체적인 필드·예시는
[PDF 자원 정책](pdf_resource_policies.md)을 따른다. 전체 parser의 할당 상한은 아직 입증되지 않았다
(`parserAllocationBoundsProven=false`). 누적 작업 카운터가 전체 메모리/RSS 제한을 보장하지 않는다.

스캔 글자는 그림으로 보존하며 OCR하지 않는다. 복잡한 clip·mask·회전/세로쓰기·표 추론의 모든 조합을
지원하지는 않는다. PDF 문자 획·색·투명도의 RHWP 확장 메타데이터를 다른 편집기가 재현한다고
보장하지 않는다. [코덱과 저장 호환성](pdf_image_codecs.md), [보안·기능 경계](security_feature_limits.md)를
따라 실제 대상 앱에서도 결과를 확인한다. 임의 글꼴을 배포물에 넣지 말고 재배포 권한을 확인한다.
