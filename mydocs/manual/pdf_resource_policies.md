---
kind: guide
status: active
canonical: mydocs/manual/pdf_resource_policies.md
last_verified: 2026-11-01
---

# PDF 가져오기 자원 정책

`rhwp import-pdf`는 요청 정책과 별도 Native 작업자를 사용한다. CLI가 프로필·설정과 쪽 목록을
검증하고, 작업자가 PDF를 읽어 IR 생성·Core 조판·직렬화를 수행한다. 부모는 작업자의 성공과
응답 크기를 확인한 뒤 출력 파일을 원자적으로 교체한다. 초과·실패·시간 초과에서는 기존 출력이
유지된다. Native는 [#8174 v2-S01](../working/task_m100_8174_v2_stage1.md), WASM·공통 Worker는
[v2-S03](../working/task_m100_8174_v2_stage3.md)의 로컬 구현 범위이며,
기본 코덱·ICC 등 F01–F08 전체의 완료 상태는 [보안 현황](security_feature_limits.md)을 따른다.
아래의 표준 한도 확대는 [#8413–#8416 작업](../working/task_m100_8413_implementation.md)의
로컬 변경이며, 이 변경의 `devel`·registry 배포 완료를 뜻하지 않는다.

## 프로필과 명시적 조정

```bash
# 일반 문서 처리. 생략해도 standard다.
rhwp import-pdf input.pdf result.hwpx --resource-profile standard --json

# 작은 자원 환경. 원본에서 필요한 쪽만 선택할 수 있다.
rhwp import-pdf input.pdf selected.hwp --source-pages 0,6-7 \
  --resource-profile constrained --json

# 선택한 프로필 이내로 한도를 낮춘다.
rhwp import-pdf input.pdf result.hwpx --resource-limits limits.json --json
```

`limits.json` 예시:

```json
{
  "policy": {
    "maxSelectedPages": 16,
    "maxDocumentImagePixels": 16000000,
    "backend": {
      "inputBytes": 16777216,
      "decodedBytes": 268435456,
      "codecWorkUnits": 100000000
    }
  },
  "execution": {
    "wallTimeoutMs": 30000,
    "cpuSeconds": 20,
    "addressSpaceBytes": 1073741824,
    "outputBytes": 33554432,
    "diagnosticsBytes": 8388608
  }
}
```

두 설정 객체와 각 필드는 생략할 수 있다. 생략한 값은 선택한 프로필을 유지한다. 설정 파일은
64 KiB 이하의 JSON 객체이며, 미지·중복 키·`null`·음수·0·실수·overflow·프로필보다 큰 값은
입력을 열기 전에 exit 2로 거부한다. 프로필이나 파일 옵션을 중복 지정해도 exit 2다.
출력 렌더링의 `--profile print`와 `--resource-profile`은 다른 옵션이다.

`PDF_RESOURCE_LIMIT: content operators`·`parsed object count`·그림 픽셀/버퍼 오류가 나면
우선 표준 프로필을 선택하고 이전에 입력한 낮은 개별 한도를 지운다. Studio는 「표준」과
「기본 한도 사용」을 선택하고, CLI는 `--resource-profile standard`로 실행하면서 불필요한
`--resource-limits` 파일을 제외한다. JSON은 선택한 프로필보다 높이는 수단이 아니다.
아래 값은 이 브랜치에서 확대한 정책이며 이전 WASM/CLI에는 적용되지 않는다. Studio도
새 WASM을 적용해야 한다. 표준에서도 다른 한도가 소진되면 원본 쪽 선택과 실제 사용량을
확인한다. `*_UNSUPPORTED`는 자원 부족과 다른 기능 오류이므로 한도를 올려 해결하지 않는다.

Studio에서 `PDF_IMPORT_TIMEOUT`이 발생하면 「고급 설정 → 제한 시간 (초)」를 기본 30초에서
60초로 늘려 다시 시도할 수 있다(허용 범위 1~300초). 이미지가 반복되는 37쪽 검증 PDF는
확대 정책과 공유 디코딩을 적용한 WASM에서 60초 설정으로 열렸다. 한 실행은 약 27초였지만
다른 실행은 30초를 초과했으므로 모든 환경에서 30초 이내 완료를 보장하지 않는다.
이 UI 시간 제한은 아래 CLI supervisor의 기본 120초와 별개다.

이 옵션은 `import-pdf` 전용이다. HWP5 압축 stream·HWPX ZIP/XML의 기존 읽기 한도를
조정하지 않으며, 해당 형식의 한도는 [보안 현황](security_feature_limits.md#자원-한도로-거부하는-입력)을
따른다.

| 한도 | standard | constrained | 의미 |
| --- | ---: | ---: | --- |
| 입력 | 64 MiB | 16 MiB | 작업자가 이 값 + 1바이트까지만 읽고 초과를 검사 |
| 선택한 쪽 | 1,024 | 64 | 전체 원본 쪽수와 분리하며 범위 확장 전에 검사 |
| content operators | 최대 8,000,000 | 250,000 | 어휘 파싱의 누적 명령 수. 선택 쪽수로 축소하지 않으며 더 낮은 host cap 유지 |
| 파싱 객체 값 | 200,000 | 50,000 | 간접 객체뿐 아니라 사전·배열 안의 scalar도 포함 |
| 단일 stream 버퍼 | 256 MiB | 32 MiB | 압축 해제·operator 수집 등의 개별 출력 backing |
| 그림 한 개 | 64,000,000 px | 4,000,000 px | 선언·합성된 이미지 면적 검사 |
| 문서 누적 그림 | 2,000,000,000 px | 16,000,000 px | 선택한 쪽의 그림 배치별 픽셀 합계 |
| decoded 작업 | 최대 8 GiB | 256 MiB | 누적 예약 카운터이며 live heap이 아님 |
| collection entries | 400,000 | 100,000 | xref·객체 컬렉션 등의 누적 항목 |
| font entries | 1,000,000 | 100,000 | Type1/CFF/TrueType의 map·glyph name·Unicode memo 등 누적 확장 작업 |
| codec 단일 예약 | 256 MiB | 32 MiB | decoder 예약과 누적 decoded 예산에 함께 적용 |
| codec 작업량 | 2,000,000,000 | 1,000,000,000 | 논리 작업 단위이며 실행 시간이 아님 |
| wall timeout | 120초 | 30초 | 부모가 감시하는 작업 수명 |
| CPU 시간 | 90초 | 20초 | Linux 작업자 프로세스 CPU 제한 |
| 주소 공간 | 4 GiB | 1 GiB | Linux 작업자 가상 주소 공간 제한 |
| 직렬화 출력 | 128 MiB | 32 MiB | 부모가 받을 최종 HWP/HWPX 바이트 |
| JSON 진단 | 32 MiB | 8 MiB | 작업자 응답의 메타데이터 크기 |

선택한 쪽을 알게 되면 공통 parser가 span·참조 예산을 추가로 좁힌다. 명령·경로 복잡도는
쪽수로 예측할 수 없어 operator와 문서 path 작업량은 프로필/host의 요청 상한을 유지한다.
문서 path 상한은 standard 1,000,000, constrained 80,000이며 개별 경로의 20,000 상한도 유지한다.
operator 수집 Vec도 capacity를 늘리기 전에 단일 `streamBytes` 한도와 같은 요청의 decoded
예산을 검사한다. 따라서 한 stream에 명령을 집중해도 모두 모은 뒤에야 복사 한도로 거절하지
않는다. 800만은 선택 문서 전체의 작업량 상한이며 단일 버퍼의 허용량을 보장하지 않는다.
실제 유효값은 성공 봉투의 `diagnostics.resourcePolicy`, 사용량은 `diagnostics.resourceUsage`로
확인한다. 표준의 큰 이미지/버퍼 한도는 원본 해상도를 유지하기 위한 유한한 상한이다.
이미지 재배치도 픽셀 작업량에 누적하며, 개별 값이 남아 있어도 decoded 작업·코덱 작업량·
OS/Worker 제한에서 실패할 수 있다. 자동 축소하거나 한도 초과를 성공으로 처리하지 않는다.
같은 요청에서 재사용하는 self-contained XObject는 최대 8개 원본의 픽셀·PNG 결과를
공유한다. 캐시는 요청 종료와 함께 해제되며 다른 PDF와 공유하지 않는다. 배치별 픽셀 합계는
계속 검사하고, 투명도·clip 등으로 픽셀을 바꿀 때의 복사와 압축 바이트 반환도 예산에 포함한다.
decoded 작업은 공유 폰트·paint와 Core 초기화·첫 저장·진단 인코딩까지 같은 요청에서
누적하므로 선택 쪽수로 축소하지 않는다. 한 쪽도 standard의 최대 8 GiB 또는 더 낮게 지정한
host cap을 사용한다. constrained의 256 MiB 상한도 유지된다. Rust의 누적 decoded/codec
예약 카운터는 `u64`이며, 실제 단일 버퍼의 길이는 해당 target의 `usize`로 검사한다.
WASM32에서도 누적 카운터는 64-bit 정수를 유지하고 JS에는 안전한 정수 `Number`로
재열기에 남은 예산을 전달한다. 이는 누적 예약 작업량이며
실제 메모리 사용량을 허용하는 보장이 아니다. 한 값이 남아 있어도 독립된 다른 한도가 먼저
소진되면 요청 전체가 실패한다. 원본 쪽수·경로/문자 수집 한도 등
전체 필드 이름은 [`PdfImportPolicy`](../../src/parser/pdf/policy.rs)와
[`PdfImportLimits`](../../vendor/rhwp-pdf-import-backend/src/import_limits.rs)를 따른다.

`backend.fontEntries`는 폰트 파일 개수가 아니다. Type1 encoding의 삽입·덮어쓰기,
CFF INDEX·charset·출력 map, glyph-name 슬롯과 Unicode memo의 양성·음성 항목도 작업량에
포함한다. 같은 글자를 memo에서 다시 읽을 때는 새 entry를 추가하지 않지만, 반환 문자열의
복사는 `decodedBytes`를 소비한다. 문자열 예약에는 `stringBytes`도 적용된다. 따라서 폰트나
문자 종류가 적어도 너무 낮게 설정한 한도는 초과할 수 있다. 작은 사용자 정의 값을 적용했다면
해당 override를 제거한 뒤 다시 실행한다. 필요하면 `constrained` 대신 `standard` 프로필을
선택한다. 프로필만 바꾸면 명시한 override가 함께 제거되는 것은 아니다.
이 카운터는 누적 작업량이며 실제 allocator의 부가 비용이나 live heap을 나타내지 않는다.
적용 경계·실행 근거는 [S07 기록](../working/task_m100_8174_v2_stage7.md)을 따른다.

## 결과와 실패 후 복구

성공한 `--json` 봉투에는 기존 필드 외에 `resourceProfile`과 `execution`이 있다.
`execution.limits`는 감독에 요청한 실행 한도다. Linux의 상속된 hard limit가 더 작으면
그 낮은 값을 유지한다. wall 시간, 관측한 최대 RSS, 실제 적용 가능한 OS 제한도 구분한다.

- 잘못된 설정·쪽 목록은 exit 2다. 입력/요청 예산 초과는 `PDF_RESOURCE_LIMIT`, 작업 시간
  초과는 `PDF_PROCESS_TIMEOUT`, 출력·진단 초과는 `PDF_PROCESS_OUTPUT_LIMIT`로 exit 1이다.
- 작업자의 비정상 종료는 실패다. 부분 결과를 성공으로 내보내지 않으며 실패 시 stdout은 비어 있다.
- 기존 출력이 유지되므로 설정·쪽 선택을 조정하고 다시 실행할 수 있다. 새 요청은 새 작업자와
  새 parser 예산을 사용한다. 다음 요청의 성공으로 이전 실패를 성공으로 기록하지 않는다.
- 입력 PDF의 암호·지원하지 않는 표현에 대한 기존 오류 계약은 그대로 적용한다.

## WASM과 공통 Worker

WASM도 `resourceProfile: 'standard' | 'constrained'`와 `resourceLimits`를 받는다.
`resourceLimits`는 위 CLI 설정의 **`policy` 안쪽 객체**다. CLI의 `execution` 객체나 파일 경로를
넘기지 않는다. 생략하면 기존 standard 정책이며 선택한 프로필보다 큰 값·미지 필드·`null`·
0·음수·소수·안전한 정수 범위 밖의 수를 거부한다. 한도 해석은 CLI와 같은 Rust 정책을 사용한다.

```js
const options = {
  resourceProfile: 'constrained',
  sourcePages: [0],
  resourceLimits: {
    maxSelectedPages: 8,
    backend: { inputBytes: 8388608, codecWorkUnits: 100000000 },
  },
};
const policy = JSON.parse(HwpDocument.getPdfImportPolicyWithOptions(options));
const document = HwpDocument.importPdf(pdfBytes, options);
console.log(policy, JSON.parse(document.getPdfImportDiagnostics()).resourceUsage);
document.free();
```

기존 `getPdfImportPolicy()`는 standard 조회로 유지한다. `importPdf`는 JS의 Uint8Array 참조로
들어온 입력을 검사한 뒤 보이는 view만 WASM 메모리에 복사한다. 직접 호출은 여전히 동기식이며,
같은 스레드의 timer·AbortSignal로 실행 도중 중단할 수 없다. 전체 형식 자동 감지용 기존
생성자에 사용자 정의 PDF 정책을 전달하는 API는 아니다.

중단이 필요한 host는 [공통 Worker 세션](../../rhwp-shared/pdf/README.md)을 사용한다.

```js
import { PdfImportSession } from './rhwp-shared/pdf/import-session.js';
const session = new PdfImportSession();
const abort = new AbortController();
try {
  const result = await session.importPdf(pdfBytes, options, {
    signal: abort.signal,
    timeoutMs: 30000,
    onProgress: phase => console.log(phase),
  });
  const saved = await session.exportDocument('hwpx', { outputBytes: 8 * 1024 * 1024 });
} finally {
  session.dispose();
}
```

호출자 options는 첫 await 전에 고정한다. 후보 Worker가 그 options의 실제 정책을 조회한 뒤
입력 view의 private 사본을 `reserveInput`으로 예약한 다음 복사·전송한다. WASM 입력 사본도
생성 전에 같은 요청으로 예약한다. 이전 요청의 프로필·한도를 cache해 새 요청에 재사용하지 않는다.
`initializing`·`importing`·`ready`는 단계 통지이며 진행률 백분율이 아니다. 성공 응답까지 받아야
현재 문서를 교체한다. 기존 문서는 후보의 일반 오류·예산 초과·취소·deadline·메모리 실패 동안
유지된다. `PDF_WORKER_MEMORY`·WASM trap은 후보를 폐기하고 다음 가져오기에서 새로 만든다.

Worker는 `PdfImportRequest`를 통해 Native와 같은 Rust `PdfImportSession`을 유지한다.
가져오기·Core 구성·진단 JSON·첫 저장을 같은 누적 요청으로 처리하며, 저장 backing과 진단
backing을 할당하기 전에 한도를 확인한다. SVG 출력 backing도 성장 전에 제한하고 UTF-16
응답 사본을 예약한다. owner는 진단 JSON·SVG·binary의 길이와 schema를 검사한 뒤 사용하며
전송 오류도 Worker 폐기로 처리한다. transfer 한도는 standard에서 출력 **128 MiB**·
진단 **32 MiB**, constrained에서 **32 MiB**·**8 MiB**다. `exportDocument`의 `outputBytes`는
출력 한도를 더 낮출 수 있고 프로필 상한을 올릴 수 없다. 직렬화 중 한도 초과는
`PDF_RESOURCE_LIMIT`로 Worker를 폐기하며 다음 `importPdf`는 새 Worker에서 시작한다.
잘못된 옵션·형식 거절은 문서를 소비하지 않는다. 이미 표시 중인 Studio 문서는 후보의
실패로 교체하지 않는다.

직접 `HwpDocument.importPdf()` 뒤 일반 `exportHwp()`/`exportHwpx()`를 호출하는 API에는
이 Worker 요청 수명을 자동 추가하지 않는다. 가져오기부터 첫 저장까지의 공통 요청 예산이
필요하면 공통 Worker 또는 Rust의 `PdfImportSession`을 사용한다.

Worker의 기본 deadline은 초기화·정책 조회를 포함한 30초이며 `timeoutMs`로 정한다.
CLI의 Linux CPU/주소 공간 제한을 브라우저에 적용했다고 표시하지 않는다.
`getPdfImportRuntimeCapabilities()`는 동기 실행·복사 전 검사·host Worker 종료 요구와
`processMemoryLimitEnforced=false`를 구분한다. 로컬 S04 Studio는 같은 정책·중단 경계를
실제 열기 UI에 연결한다. Worker 세션은 inspect/render/export 수명이며 전체 편집기를 대신하지 않는다.

## Studio의 실제 열기

로컬 S04는 파일·드롭·URL·host 입력의 공통 열기 경로에서 PDF를 전용 Worker로 가져온다.
「표준」·「자원 절약」과 개별 한도는 같은 Rust 정책을 조회하며 입력 크기는 전체 파일 읽기 전에
확인한다. 확장자 이외의 형식 확인은 1 KiB header만 먼저 읽는다. 지원 확장자의 파일 선택 규칙은 유지한다.
실제 UI 절차는 [Studio 가이드](studio_document_features.md#pdf를-편집-문서로-열기)를 따른다.

T05는 HWPX export 뒤의 실제 남은 decoded 예산을 `PdfEditorReopen`에 전달한다. 입력·ZIP 사본,
압축 해제 backing의 성장과 Core 초기화를 먼저 예약하며, lazy reader는 재열기 종료 후 일반
편집 수명으로 분리한다. `diagnostics.resourceUsage`는 가져오기/저장의 진단 시점이며,
`PdfEditorReopen.resourceUsage()`는 별도 재열기 사용량이다. 두 값이나 현재 heap을 혼동하지 않는다.
개별 XML 객체·ZIP 내부 할당 전체가 증명됐다는 뜻은 아니다.

기본 30초의 deadline은 입력 읽기·초기화·변환·직렬화·동기 재열기 후의 교체 전 검사까지 공유한다.
파일 공급자가 늦게 끝나도 취소/시간 초과된 읽기의 결과는 사용하지 않는다. 대화상자는 최종
교체까지 유지하고 재열기 예약·읽기 경계에서 협력적으로 시간을 검사한다. 동기 작업 뒤에는
이벤트 루프에 양보해 취소·deadline·문서 generation을 다시 확인한 후 원자적으로 교체한다.
취소·실패·초과는 후보 Worker를 종료하고 현재 문서·draft·최근 문서·저장 핸들을 유지한다.
성공 때만 편집 문서를 교체한다. main thread의 재열기를 중간에 선점하는 기능이나
브라우저 전체 메모리 제한으로 설명하지 않는다.

## Rust 소비와 host의 실행 경계

```rust
use rhwp::{DocumentCore, parser::pdf::{PdfImportPolicy, PdfImportProfile}};

let policy = PdfImportPolicy::for_profile(PdfImportProfile::Constrained)
    .with_limits_json(r#"{"backend":{"inputBytes":8388608}}"#)?;
let core = DocumentCore::from_pdf_bytes_with_policy(&bytes, &Default::default(), policy)?;
```

Rust의 `parse_pdf_with_metadata_and_policy`도 자동 감지와 같은 provenance 봉인을 적용한다.
명시적 Rust policy는 유한한 값인지 검사하며, 제품 프로필의 JSON override는 해당 프로필을
올릴 수 없다. 기존 `from_pdf_bytes`·`parse_pdf_with_options` 호출은 standard 정책을 유지한다.
직접 Core 호출의 조판·저장에 프로세스 감독이 자동 생기지는 않는다.
`info input.pdf`·`export-svg input.pdf`·`convert input.pdf` 등 기존 공통 로더 명령도
현재 프로세스 안에서 실행한다. 별도 작업자의 실행 한도는 이 절편의 `import-pdf`에 적용되며,
다른 CLI 명령으로 같은 보호를 얻었다고 해석하지 않는다.

Native host용 [`resource_supervisor`](../../src/resource_supervisor.rs)는 별도 실행 프로그램의
stdin/stdout/stderr·wall 수명과 자식 정리를 연결하는 실험적 Rust 진입점이다. Linux에서는
`RLIMIT_AS`·`RLIMIT_CPU`·`RLIMIT_CORE=0`과 `no_new_privs`를 적용하고, Unix에서는 private
process group을 종료한다. 다른 host에서 AS/CPU·자손 전체 종료를 적용했다고 표시하지 않으며
실제 지원 여부는 결과의 `addressSpaceLimitEnforced`·`cpuLimitEnforced`·`descendantGroupCleanup`에
담는다. host에서는 요청마다 새 `Command`를 구성한다. 현재 실행 증거는 Linux/WSL2이며
다른 OS의 검증 완료를 뜻하지 않는다.

Linux의 [RLIMIT_AS](https://man7.org/linux/man-pages/man2/getrlimit.2.html)는 주소 공간 제한이다.
RSS와 다르며, `observedPeakRssBytes`도 주기적으로 읽은 관측치다. 부모·동시 요청·브라우저 전체의
메모리 상한으로 사용하지 않는다. 일반 `resource_supervisor::run` 자체는 파일·네트워크
sandbox가 아니므로 `filesystemNetworkSandbox=false`를 보고한다. Linux x86_64/aarch64의
`import-pdf`는 아래 별도 confinement까지 성공한 작업자 응답에서만 이 값을 `true`로 보고한다.
#7911의 남은 할당 감사와 MD05-B의 보안 경계 검증은 [v2 단계 기록](../working/task_m100_8174_v2_stage1.md)에 연결하고 완료 전에는 비신뢰
업로드 공식 지원 완료를 발표하지 않는다. WASM의 공통 정책 연결은 위 S03 계약을 따르며
Studio의 실제 열기·편집·복구 연결은 [S04 사용자 가이드](studio_document_features.md)를 따른다.
호스트가 core dump를 별도 프로그램으로 pipe하면 `RLIMIT_CORE`를 적용하지 않는
[Linux 동작](https://man7.org/linux/man-pages/man5/core.5.html)도 있어, 해당 설정을 모든
호스트의 진단 저장 차단으로 해석하지 않는다.


### Linux PDF 작업자의 파일·네트워크 제한

Linux x86_64/aarch64의 `import-pdf`는 요청을 읽은 새 단일 스레드 작업자에서
[`confine_pdf_worker`](../../src/resource_supervisor/pdf_confinement.rs)를 적용한 뒤 PDF를 연다.
Landlock ABI 3 이상·seccomp를 사용할 수 없거나 허용 경로가 잘못되면
`PDF_PROCESS_SANDBOX_UNAVAILABLE`로 실패하며 기존 출력 파일을 보존한다. 지원되지 않는
다른 OS/아키텍처에서는 이 제한을 적용했다고 표시하지 않는다.

허용하는 파일 내용은 입력 PDF와 실행자가 지정한 글꼴 자원이다. 기본 글꼴 디렉터리,
`/etc/fonts`·`/usr/share/fontconfig`, 사용자 글꼴과 명시적 글꼴 캐시를 읽기 전용으로 허용한다.
`RHWP_FONT_PATH`는 신뢰하는 호스트 설정이며 입력 문서에서 가져오지 않는다. `/`는 거절한다.
호스트는 필요한 글꼴 디렉터리만 지정하고 민감한 파일을 함께 넣지 않는다. 작업자는 새 파일
생성·수정과 네트워크 소켓·외부 실행·별도 프로세스 생성을 허용하지 않는다. 내부 계산 스레드는
같은 제한을 상속한다. 표준 입출력은 supervisor가 만든 파이프만 유지하고 다른 상속 fd를 닫는다.

이것은 전체 컨테이너나 경로 메타데이터 은닉을 보장하는 기능이 아니다. 허용 파일의 내용,
부모 프로세스와 동시 요청의 자원, 호스트·커널의 설정과 보안 업데이트는 운영자가 관리한다.
[Landlock](https://docs.kernel.org/userspace-api/landlock.html)과
[seccomp filter](https://docs.kernel.org/userspace-api/seccomp_filter.html)의 경계를 따른다.

Rust 서버는 같은 별도 프로세스 구성을
[게시 패키지의 예제](../../packaging/crates-io/examples/pdf_isolation_host.rs)에서 시작할 수 있다.
예제는 요청마다 새 작업자에서 제한·Session 가져오기·첫 저장을 수행하고 부모가 결과를 저장한다. 입력 PDF는 제한 적용 전
읽기 전용으로 연 핸들로만 전달한다. `/mnt/e` 같은 WSL 마운트도 같은 경로를 사용하며
입력 bytes 해석은 제한 적용 뒤에 시작한다. Rust host는 반환된 핸들을 사용한다.
현재 프로세스 안에서 `confine_pdf_worker`를 호출하지 않는다. 제한은 되돌릴 수 없다.

```bash
cargo run --locked --example pdf_isolation_host -- input.pdf output.hwpx
# 입력 읽기·계산 스레드는 허용하고 합성 비공개 파일·네트워크·외부 실행을 차단하는지 검사
cargo run --locked --example pdf_isolation_host -- input.pdf verification.hwpx --verify
```

`--verify`의 합성 파일은 출력 파일 옆에 잠깐 생성되므로 쓰기 가능한 검증 디렉터리를 사용한다.
이 예제와 `import-pdf`의 보호를 다른 CLI 명령이나 동기 라이브러리 호출의 자동 보호로
확대하지 않는다. 플랫폼별 최종 검증 범위는 작업 보고서와 게시 지원표를 확인한다.

`layout: "fidelity" | "editable"`과 CLI `--layout`은 같은 요청 정책을 사용한다. Studio 환경 설정의 PDF 가져오기 배치도 Worker에 전달한다. 편집 우선 모드의 문단 병합·다단 재배치·진단 자료는 요청 예산에 포함하며, 일반 편집 수명과 최초 가져오기 수명을 구분한다.
