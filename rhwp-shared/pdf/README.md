# PDF 가져오기 Worker 세션 (#7911)

기본 포함 `pdf-import` 제품 경로를 브라우저에서 실행하는 저장소 공용 모듈이다.
`import-worker.js`는 루트 `pkg/rhwp.js`를 직접 사용한다. Studio는 이 Worker에서 PDF를
공통 IR로 가져오고 HWPX로 전달한 뒤, 같은 요청의 남은 decoded 예산으로 main thread의
편집 문서를 재연다. 성공한 문서는 일반 편집·저장의 독립 수명을 갖는다. npm 배포 패키지의
export와 전체 Worker 편집기는 별도 범위다.

Studio 빌드·실행은 [PDF 직접 열기](../../mydocs/manual/dev_environment_guide.md#studio에서-pdf-직접-열기)를 따른다.

## 빌드와 사용

저장소 루트에서 다음 wrapper를 실행한다. Docker 또는 호스트의 최적화 포함 빌드와 공용 target을
유지하는 방법은 [개발 환경 가이드](../../mydocs/manual/dev_environment_guide.md)를 따른다.

```sh
scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

서버에서 `rhwp-shared/pdf/`와 `pkg/`의 상대 경로를 보존해 제공한다. 모듈 Worker를 허용하는
같은 출처에서 실행하며, 이 모듈과 WASM을 함께 버전 고정한다.

```js
import { PdfImportSession } from './rhwp-shared/pdf/import-session.js';
const session = new PdfImportSession();
const abort = new AbortController();
try {
  const result = await session.importPdf(pdfBytes, {
    sourcePages: [0, 6, 7],
    resourceProfile: 'constrained',
    resourceLimits: { maxSelectedPages: 8 },
  }, {
    signal: abort.signal,
    timeoutMs: 30_000,
    onProgress: (phase, page) => {
      if (page) console.log(`${page.sourcePage}/${page.sourcePageCount}`, `${Math.floor(100 * page.completedPages / page.totalPages)}%`);
      else console.log(phase);
    },
  });
  console.log(result.pageCount, result.diagnostics);
  const svg = await session.renderPageSvg(0);
  const hwpx = await session.exportDocument('hwpx', { outputBytes: 8 * 1024 * 1024 }); // 또는 hwp; Uint8Array
  // svg 표시와 저장은 호스트가 담당한다. 가져온 SVG/문서 내용은 비신뢰 콘텐츠다.
} finally {
  session.dispose();
}
```

`sourcePages`는 0-based 엄격 증가 배열이다. 생략하면 전체 쪽, 중복·역순·빈 배열·잘못된
타입·범위는 오류다. 입력 Uint8Array를 detach하지 않으므로 호출자가 재시도할 수 있다.
동시에 하나의 가져오기만 허용하며, 중복 요청은 `PDF_IMPORT_BUSY`다. `inspect()`는 현재
문서의 쪽수와 가져오기 당시 진단을 반환한다. 편집 뒤의 쪽 대응 재계산을 주장하지 않는다.

## 요청 정책과 세션 실패

- 호출자가 전달한 options를 첫 await 전에 복사해 고정한다. options·resourceLimits는 일반 객체의
  자체 enumerable data property를 사용한다. getter·symbol·함수로 정책을 전송하지 않는다.
- 후보 Worker는 WASM 초기화 뒤 `getPdfImportPolicyWithOptions(options)`로 읽은 제품 정책을 먼저 반환한다.
  호스트는 그 후보의 `backend.inputBytes`를 사용해 `bytes.slice()`·structured clone 전에
  입력을 검사한다. JavaScript에 별도의 16 MiB 상수를 복제하지 않는다. 거절 시 현재 문서는
  보존하며 다음 정상 입력은 같은 후보를 재사용할 수 있다. 정책은 요청마다 다시 해석하므로
  이전 요청의 낮거나 높은 한도가 다음 요청에 남지 않는다.
- `resourceProfile`·`resourceLimits`는 [공통 자원 정책](../../mydocs/manual/pdf_resource_policies.md#wasm과-공통-worker)을
  따른다. Worker도 같은 options로 실제 WASM을 호출하며 그 API는 자체 메모리 복사 전에 입력 한도를 검사한다.
- 정책 조회와 실제 가져오기는 한 번의 `timeoutMs`를 공유한다. 초기화·정책 응답 시간이
  길었다는 이유로 가져오기 deadline을 새로 시작하지 않는다. 이 프로토콜의 Worker와 WASM은
  함께 버전 고정해야 하며, 정책이 없거나 잘못된 응답이면 입력을 전송하지 않는다.
- 성공 진단의 `resourcePolicy`·`resourceUsage`는 실제 요청의 정책·연산/복사 계상과 cache 관측이다.
  누적 decoded 비용과 codec 예약을 프로세스의 현재 heap/RSS로 해석하지 않는다.

- 후보 Worker에서 가져오기·진단 생성까지 성공해야 현재 Worker를 교체한다. 기존 문서는
  그동안 조회·렌더·저장할 수 있다. 보통의 입력 오류 뒤에는 후보 Worker를 재사용한다.
- 취소·deadline은 **호출자 스레드에서 Worker를 종료**한다. 동기 WASM 도중 cancel 메시지를
  처리한다고 가정하지 않는다. 후보 종료는 현재 문서에 영향을 주지 않으며 다음 요청에서
  새 Worker를 만든다. 로드 실패·WASM trap·`PDF_WORKER_MEMORY`도 후보를 폐기한다.
- `onProgress(phase, page?)`의 `importing` 단계는 실제 쪽 처리 시작/완료 시
  `{ sourcePage, sourcePageCount, completedPages, totalPages }`도 전달한다. sourcePage는 1-based 원본 쪽,
  totalPages는 선택한 쪽 수다. 완료 쪽 비율은 남은 시간의 추정치가 아니며 100% 뒤에도
  Core 생성과 저장이 남는다. 기존 phase-only callback은 그대로 사용할 수 있다.
- `onProgress`의 `initializing`·`importing`·`ready`는 후보의 단계 통지다. 통지 자체로 문서를
  교체하지 않으며 백분율이나 parser 내부의 중단 지점을 뜻하지 않는다.
- 현재 문서 자체의 렌더/저장 요청에 timeout 또는 abort를 적용하면 그 Worker를 종료하므로
  현재 문서도 해제된다. 다음 가져오기 전에는 `PDF_NO_DOCUMENT`다. 가져오기 후보의 취소와
  구분한다. 문서 교체 시 진행 중이던 이전 문서 요청은 `PDF_SESSION_REPLACED`로 끝난다.
- `dispose()`는 두 Worker와 남은 요청을 정리한다. 성공 응답 이후 `AbortSignal`을 취소해도
  이미 끝난 요청을 되돌리지 않는다. 기본 deadline은 초기 WASM 로드까지 포함한 30초다.
- Worker 종료는 시간 제한·운영 복구일 뿐 전체 heap/RSS 상한의 증명이 아니다.
  `parserAllocationBoundsProven=false`와 미지원 진단은 그대로 노출한다.

## 검증

```sh
node --test rhwp-shared/pdf/import-session.test.js
PDF_IMPORT_DISABLED_PKG=/검증용/명시적-feature-off-pkg \
PDF_IMPORT_EVIDENCE=output/pr-review/issue7911/wasm-runtime \
node --test scripts/tests/pdf-import-wasm.test.mjs
```

두 번째 검사는 실제 브라우저, 루트 제품 JS/WASM, 실제 12쪽 PDF와 합성 API 입력을 사용한다.
`PDF_IMPORT_DISABLED_PKG`에는 `--no-default-features --features console_error_panic_hook`로
만든 별도 bundle을 전달한다. 미지정이면 feature-off 실행을 생략하며 기본 feature-on은 항상 필수다.
생성자·엄격한 선택 옵션·진단·전쪽 SVG·HWP/HWPX 재열기·같은 Worker의 실패 후 복구·후보
취소/시간 초과·재생성을 확인한다. 자동화는 시각 일치율과 사용자 판정을 대신하지 않는다.


### 첫 저장과 전송 한도

제품 Worker는 `PdfImportRequest`에서 Native와 같은 Rust 요청을 첫 저장·진단까지 유지한다.
standard는 출력 128 MiB·진단 32 MiB, constrained는 출력 32 MiB·진단 8 MiB를 상한으로
사용한다. `exportDocument`의 `outputBytes`로 출력 상한을 더 낮출 수 있다. 한도 초과의
`PDF_RESOURCE_LIMIT`나 직렬화 실패는 해당 Worker를 폐기한다. 다음 `importPdf`로 새 요청을
시작한다. 형식·옵션 오류는 저장을 시작하기 전에 거절한다. 이 byte 상한은 전체 브라우저
RSS나 동기 편집기 재열기의 강제 중단을 보장하지 않는다.

## 복사·응답과 Studio 교체 경계 (#8174 T05)

옵션은 허용 필드·값·선택 배열의 data property를 검사한 후 고정한다. 미지 필드나 접근자를
평가하거나 전체 객체를 먼저 복제하지 않는다. 정책 조회 다음 `reserveInput`이 호출자 view의
private 사본을 예약하고, 이후 보이는 view만 복사·전송한다. 원본 buffer는 detach하지 않는다.
WASM 입력 사본도 생성 전에 같은 요청에 예약한다.

진단은 한도가 있는 JSON 문자열로 전송하고 owner에서 schema·UTF-8 길이를 확인한 뒤
파싱한다. SVG도 출력 버퍼의 성장 전에 검사하며 `renderPageSvg(page, { outputBytes })`로
상한을 낮출 수 있다. WASM→Worker 문자열과 owner 응답의 UTF-16 backing을 각각 예약한다.
Binary는 WASM→JS 사본을 예약하고 정확한 buffer를 transfer한다. 잘못된 응답·전송 실패·
자원 거절은 해당 Worker를 폐기한다. 다음 가져오기는 새 Worker에서 복구한다.

`getEditorReopenLimits()`는 성공한 export **이후**의 실제 남은 decoded 예산과 원래 옵션,
입력 상한을 반환한다. `PdfEditorReopen.open`은 그 예산에서 WASM 입력·ZIP 사본·압축 해제
backing·Core 초기화를 처리한다. 초기 재열기를 마치면 lazy asset reader는 해당 예약에서
분리되어 일반 편집과 저장에 이전 요청의 credit을 다시 사용하지 않는다.

Studio 대화상자는 재열기와 최종 교체까지 유지한다. 하나의 deadline과 AbortSignal,
문서 generation을 확인하고 동기 작업 뒤 이벤트 루프에 양보해 대기 중인 취소도 반영한다.
후보가 준비된 경우에만 기존 문서·draft·최근 문서·저장 핸들을 교체한다.

이는 논리 payload와 지정된 backing의 누적 예약이다. XML parser 객체·ZIP 내부·renderer
scratch·브라우저 전체 heap/RSS의 상한 증명이 아니다. main thread의 동기 WASM은 같은
스레드의 timer로 선점하지 못한다. 재열기 예약/읽기 경계의 협력 검사와 최종 교체 검사를
적용하며, Worker의 실제 종료와 구분한다. `parserAllocationBoundsProven=false`를 유지한다.

`layout: 'editable'`을 지정하면 인식한 본문을 구역 본문 흐름으로 가져온다. 생략하거나 `'fidelity'`이면 원본 위치를 우선한다. 두 모드 모두 같은 자원 정책·취소·기존 문서 보존 계약을 적용한다.
