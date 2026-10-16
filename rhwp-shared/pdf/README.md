# PDF 가져오기 Worker 세션 (#7911)

기본 비활성 `pdf-import` 제품 경로를 브라우저에서 실행하는 저장소 공용 모듈이다.
`import-worker.js`는 루트 `pkg/rhwp.js`를 직접 사용한다. POC 모듈이나 HWPX 중간 변환을
사용하지 않는다. 아직 npm 배포 패키지의 export나 Studio 열기 UI에 자동 연결하지 않는다.
패키지 소비·UI 채택은 별도 통합 범위다.

## 빌드와 사용

저장소 루트의 Docker WASM 서비스에서 다음 wrapper를 실행한다. 공용 Docker target/cache를
유지하는 방법은 [개발 환경 가이드](../../mydocs/manual/dev_environment_guide.md)를 따른다.

```sh
scripts/wasm-pack-locked.sh --target web --out-dir pkg --features pdf-import
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

## 세션과 실패

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
PDF_IMPORT_DEFAULT_PKG=/검증용/기본-pkg \
PDF_IMPORT_EVIDENCE=output/pr-review/issue7911/wasm-runtime \
node --test scripts/tests/pdf-import-wasm.test.mjs
```

두 번째 검사는 실제 브라우저, 루트 제품 JS/WASM, 실제 12쪽 PDF와 합성 API 입력을 사용한다.
`PDF_IMPORT_DEFAULT_PKG`가 없으면 feature-off 실행을 생략하며 feature-on은 항상 필수다.
생성자·엄격한 선택 옵션·진단·전쪽 SVG·HWP/HWPX 재열기·같은 Worker의 실패 후 복구·후보
취소/시간 초과·재생성을 확인한다. 자동화는 시각 일치율과 사용자 판정을 대신하지 않는다.
