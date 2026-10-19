# PDF 가져오기 Worker 세션 (#7911)

기본 포함 `pdf-import` 제품 경로를 브라우저에서 실행하는 저장소 공용 모듈이다.
`import-worker.js`는 루트 `pkg/rhwp.js`를 직접 사용한다. POC 모듈이나 HWPX 중간 변환을
사용하지 않는다. npm 배포 패키지의 export는 아직 연결하지 않는다.
Studio의 PDF-on 파일 열기·드롭은 제품 WASM 생성자를 통해 공통 IR을 직접 사용한다.
현재 Studio는 동기 문서 API를 사용하므로 이 Worker 세션과는 별도 수명주기다.
전체 Worker 편집기 전환과 패키지 배포는 후속 범위다.

Studio 빌드·실행은 [PDF 직접 열기](../../mydocs/manual/dev_environment_guide.md#studio에서-pdf-직접-열기)를 따른다.

## 빌드와 사용

저장소 루트의 Docker WASM 서비스에서 다음 wrapper를 실행한다. 공용 Docker target/cache를
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
  const result = await session.importPdf(pdfBytes, { sourcePages: [0, 6, 7] }, {
    signal: abort.signal,
    timeoutMs: 30_000,
  });
  console.log(result.pageCount, result.diagnostics);
  const svg = await session.renderPageSvg(0);
  const hwpx = await session.exportDocument('hwpx'); // 또는 hwp; Uint8Array
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

- 후보 Worker는 WASM 초기화 뒤 `getPdfImportPolicy()`로 읽은 제품 정책을 먼저 반환한다.
  호스트는 그 후보의 `backend.inputBytes`를 사용해 `bytes.slice()`·structured clone 전에
  입력을 검사한다. JavaScript에 별도의 16 MiB 상수를 복제하지 않는다. 거절 시 현재 문서는
  보존하며 다음 정상 입력은 같은 후보·정책을 재사용할 수 있다.
- 정책 조회와 실제 가져오기는 한 번의 `timeoutMs`를 공유한다. 초기화·정책 응답 시간이
  길었다는 이유로 가져오기 deadline을 새로 시작하지 않는다. 이 프로토콜의 Worker와 WASM은
  함께 버전 고정해야 하며, 정책이 없거나 잘못된 응답이면 입력을 전송하지 않는다.
- 성공 진단의 `resourcePolicy`·`resourceUsage`는 실제 요청의 정책·연산/복사 계상과 cache 관측이다.
  누적 decoded 비용과 codec 예약을 프로세스의 현재 heap/RSS로 해석하지 않는다.

- 후보 Worker에서 가져오기·진단 생성까지 성공해야 현재 Worker를 교체한다. 기존 문서는
  그동안 조회·렌더·저장할 수 있다. 보통의 입력 오류 뒤에는 후보 Worker를 재사용한다.
- 취소·deadline은 **호출자 스레드에서 Worker를 종료**한다. 동기 WASM 도중 cancel 메시지를
  처리한다고 가정하지 않는다. 후보 종료는 현재 문서에 영향을 주지 않으며 다음 요청에서
  새 Worker를 만든다. 로드 실패·WASM trap도 후보를 폐기한다.
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
