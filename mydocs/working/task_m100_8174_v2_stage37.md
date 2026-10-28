# #8174 v2-S37 — WASM Worker의 첫 저장·진단 요청 수명

Issue: #8174. 2026-10-26, T04/B04와 T05/B07의 WASM 연결 절편이다.
기준은 S36 로컬 커밋 `59a11ad895`이며 기존 순차 진행 승인으로 계속했다.

## 변경

기존 Worker는 `HwpDocument.importPdf`로 문서를 만든 뒤 일반 저장 API를 호출했다.
가져오기 요청의 누적 카운터가 첫 저장까지 이어지지 않는 경계였다. 새 WASM
[`PdfImportRequest`](../../src/wasm_api/pdf_request.rs)는 Native에서 사용하는 Rust
`PdfImportSession`과 같은 Core를 소유한다. 정책/입력 view 검사 → 가져오기·Core 구성 →
진단 JSON → 첫 HWP/HWPX 저장을 같은 요청으로 처리한다. 렌더링은 기존 HwpDocument를
소비하며 별도 조판 규칙을 추가하지 않았다. 일반 HwpDocument API의 수명은 유지한다.

공통 [`PdfTransferLimits`](../../src/pdf_runtime.rs)에서 Native와 WASM의 출력/진단 byte
상한을 정한다. standard 128/32 MiB, constrained 32/8 MiB로 기존 Native 값과 같다.
Worker export의 `outputBytes`는 이 출력 상한을 낮출 수 있다. 진단은 길이를 센 뒤 정확한
backing을 예약하며 첫 저장도 같은 요청의 bounded writer를 사용한다. 실패한 직렬화는
Worker를 폐기해 실패한 요청을 재사용하지 않는다. 다음 가져오기는 새 Worker로 복구한다.

WASM 내부 OOM·전체 브라우저 RSS를 byte 카운터로 보증하거나 Studio의 동기 main thread
재열기를 중간에 선점했다고 주장하지 않는다. 동기 재열기의 한도·중단 경계와 나머지 Core/
serializer 내부 감사는 남는다.

## 실제 검증

- 최적화 root WASM: `scripts/wasm-pack-locked.sh --target web --out-dir pkg`,
  release codegen-units 16, **10분 53초**. 실제 wasm-opt를 포함한 빌드다.
  WASM **24,175,480 bytes**, SHA-256
  `a44597495da860114b90e3c835916ce123d29bd1f3c2adcf2349a8a85860183a`.
  보존한 직전 제품은 S19의 23,715,663 bytes이며 현재 제품으로 덮어 보고하지 않는다.
- 공통 Worker Node **15 PASS**, Studio PDF 계약 **8 PASS**, Studio TypeScript 통과.
  새 output cap의 사전 거절과 전달을 검사했다. source unit을 추가하지 않았다.
- 현재 최적화 제품의 실제 브라우저 **249개 assertion PASS / 실제 Worker 14개**.
  같은 요청에서 저장 후 decoded 계상이 증가하며 bounded/일반 HWPX bytes가 같은지 확인했다.
  출력 1 byte 거절·실패한 요청 재사용 거절·새 요청 복구, 실제 Worker의 fatal 출력 거절과
  재생성, 기존 입력/옵션/selection/코덱/메모리 실패·취소·deadline 복구를 통과했다.
  12쪽 실물의 전쪽 렌더링과 HWP/HWPX 저장·재열기·그림 보존 검사도 통과했다.
- 실제 Studio: 공용 Vite runner가 이 검사 전용 서버를 띄우고 종료했다. 800만 초과·낮춘 cap·
  집중 경로 실패와 취소가 기존 문서·draft·최근 문서·저장 핸들을 유지했다. 다음 15쪽 PDF는
  가져오기·편집기 열기·HWP/HWPX 저장에 성공했다. HTTP WASM 응답 hash도 위 제품과 같다.
- 최신 WASM CanvasKit CFF **100%**, Type3 11쪽 **99.98717%**. 독립 PDF와 현재 compare/
  overlay/review를 직접 대조해 문자·간격·표 외곽·그림/캡션 소속을 확인했다. 이전 PNG를
  현재 캡처로 재사용하거나 새 렌더링 baseline을 등록하지 않았다.
- Native library/CLI와 WASM library Clippy(`-D warnings`), feature-off library 컴파일 통과.
  자동 수집 스크립트 **23 PASS**, S36 대비 unit-tier 검사 통과. 이전 절편에서 추가한 source
  unit의 브랜치 전체 정책 실패는 아직 남는다. 최신 Native 제품의 전체 실행은 S36이며,
  이번 Native compile 검사와 구분한다.

source·제품 hash와 실제 명령·원시 로그·전송 오류 코드의 JS 변경 delta, 현재 TSV·PNG는
ignored `output/pr-review/issue8174/v2-s37-20261026/`에 보존한다. Cargo compile 이후 Worker의
직렬화 오류 코드만 JS에서 `PDF_RESOURCE_LIMIT`로 정리했고 Rust 생산 source는 바꾸지 않았다.
생산 source hash를 확인한 뒤 현재 브라우저 검사를 실행했다. 검증 뒤 공통 Worker README만
변경한 차이는 `documentation-after-validation-delta.json`으로 구분했다. 기존 memory/entry instrumentation은
새 제품 요청 진입점을 관측하도록 바꿨으며 실패·복구 조건을 완화하지 않았다.

이 결과는 T04·T05 전체나 PR 제출 준비 완료가 아니다. 다음에는 잔여 할당/편집기 경계와
이전 절편 테스트 정책 문제를 보정하고 T06–T09를 이어간다. 원격 push·PR 생성·registry
업로드·인증 정보 열람은 수행하지 않았다.
