# #8946 시각 미달 원인 추적과 보정

범위는 Canvas2D 호스트 원본 프로그램 선택·수명과 기존 메트릭 정책 보존이다.
2026-11-11 추가 지시에 따라 수용 기준을 낮추지 않고 보정·재검증을 계속한다.

## 확인한 경계

전체 글꼴 목록의 과거 캡처는 `prepareCanvasMetrics.start` 뒤
`outline font session byte limit exceeded`가 발생한 대체 화면이었다.
실제 `prepareCanvasMetrics.end`와 후속 paint가 없는 실행을 완료로 잘못 취급했다.
단일 딸기 공급의 기존 실행은 준비 종료를 확인했으며 둘을 구분한다.
진단 스크립트는 현재 문서의 메트릭 준비 종료와 그 이후 paint를 기다리도록 보정했다.
원 실패 증거는 보존한다.

또한 새 `host-<SHA>-face-<index>` ID는 기존 Rust의 프로그램 신원 판독 형식이 아니다.
문자열이 환경 JSON에 저장되는 검사만으로 실제 신원 소비를 입증하지 못했다.
기존 Native 계약인 SHA-256(원본 바이트 + big-endian u32 faceIndex)을 사용하고,
TTC를 Rust에 공급할 때 원본 컨테이너와 선택 index를 함께 보존한다.

## 보정과 검증 순서

1. 원본 프로그램 신원을 기존 공통 선택·측정 계약에 연결한다. TTC 전송에서도
   선택 face를 잃지 않는지 실제 WASM으로 검증한다.
2. 큰 문서의 윤곽·메트릭 자원 사용을 추적한다. 기존 상한을 올리거나 실패를 숨기지 않고
   필요한 표·쪽 자원과 중복 보관을 구분한다.
3. 준비 완료 이후의 실제 화면과 동일 환경 Native/fresh WASM을 재산출한다.
   기존 기본 모드·수명·저장·이력 대조도 영향 범위에 맞게 다시 실행한다.

편람 368쪽의 공개 HANBatang U+2024는 1000em에서 y=-16..73의 실제 윤곽이다.
독립 Print의 점 위치와 다르므로 선택한 glyph 공급자의 선행 해결 근거를 추적한다.
이 관찰만으로 글꼴 파일이나 문서 좌표를 수정하지 않는다.

## 원본 프로그램 신원 보정 검증

Native와 같은 SHA-256(원본 바이트 + big-endian faceIndex)을 채택했다.
새 WASM `prepareNativeOtfFontFaces`는 TTC 원본과 index를 함께 전달하며 기존
Uint8Array 공급 API도 유지한다. 합성 TTC의 두 face는 실제 Native TrueType
측정 경로에서 각각 선택되고, 저장된 900/1080 전진 단위에 따른 1.2 비율을 확인했다.
CFF는 불명확한 ID에서 등록되지 않고 정규 신원에서 등록되는 실제 소비도 확인했다.

2026-11-11 로컬 검증: 집중 Node 30개, TypeScript 검사, 최적화 fresh WASM 빌드,
Canvas2D/CanvasKit #7403 E2E 모두 통과했다. E2E는 기존 기본 동작·교체·제거·
문서 전환·저장·undo/redo 검사도 포함한다. 이는 프로토콜 검증이며 실물 문서의
시각 통과를 대신하지 않는다. 용량 경계와 전쪽 시각 검증은 계속 진행 중이다.

## 대용량 문서 자원 경계 조사

실제 전체 호스트 목록에서 읽은 고유 CFF 71개의 원본은 103,578,824바이트다.
측정용 SFNT 표 합계는 1,249,794바이트다. 동일 바이트 중복은 기존 저장소가
이미 공유하므로 중복 제거만으로 64 MiB 윤곽 상한을 해결할 수 없다.

측정의 소비는 `NativeOtfMetrics::glyph_metric`의 cmap/hmtx이며, 윤곽을 사용하지
않는다. 정적 CFF도 기존 TrueType과 같은 측정 전용 SFNT를 보관하되 원본 SHA·
collection index·CFF 형식·faux bold 판정은 별도로 보존한다. 가변 글꼴은 기존
원본 경로를 유지한다. 계산 규칙이나 한도 값은 변경하지 않는다.

호스트 조판 opt-in의 paint는 전체/filtered/patch 및 머리말·꼬리말 preview가
같은 `withPageOutlines`를 거친다. 해당 트리의 실제 글꼴만 원본 bytes/index로
등록하고 finally에서 윤곽만 해제한다. coverage·측정·저장 상태는 유지한다.
일반 옵션 생략/비활성 경로는 기존 준비 계약을 유지한다. 새 공개 WASM 기능이
없는 runtime에서는 기존 등록 경로를 사용한다.

집중 Node 40개와 Studio 전체 2,789개 통과(기존 skip 2개), TypeScript 검사를
확인했다. 현재 보정의 Native 단위 검사와 fresh WASM/실물 출력 검증은 진행 중이다.

## 편람 검증 환경 재진단

기존 Native producer SHA-256 `cb63fce663d2c2e930cab9c371ea6dc0aa203fb528fe4992f0d3e1530057dd3c`를
보존하고, 글꼴 준비부터 출력까지 같은 최소 글꼴 sandbox에서 실행했다. 자동 웹폰트
캐시도 T7 전용 경로로 지정했다. `--handbook-gate`는 383쪽 전부 기준을 유지해
passed였다. 130쪽 표시값 99.96%, 368쪽과 나머지 쪽 표시값 100.00%다.

기존 환경 `f09544a8…`와 새 환경 `d68b9023…`의 실제 차이는 KoPub바탕체 Light
언어 슬롯 5/font ID 13의 문자 fallback이다. 기존은 HCRBatang, 새 실행은
Noto Serif → HCRBatang이다. 설치 글꼴 선택 전체가 바뀐 것으로 일반화하지 않는다.
현재 결과는 기존 producer의 환경 대조이며 새 코드의 최종 gate를 대신하지 않는다.
원 기대값·점 좌표·글꼴 규칙을 바꾸지 않고 새 producer에서도 같은 절차를 실행한다.

## 자원 보정 후 실행 결과

정적 CFF의 cmap·unitsPerEm·모든 hmtx 전진을 원본과 대조하는 Native 단위 검사,
최적화 fresh WASM, Canvas2D/CanvasKit #7403 E2E가 통과했다. 조판 환경 E2E의
45개 계약은 CFF 실제 등록, 전체/filtered/patch paint 후 해제, HF preview의
기존 쪽 트리 보존을 포함한다. 단순 JSON 식별자 확인과 실제 소비 검사를 구분했다.

새 프로덕션 번들에서 HWPX 763쪽·전체 호스트 목록 521개를 열었을 때 필요한
face 67개가 loaded, pending/failed는 각각 0이었다. Native PUA 준비와
`prepareCanvasMetrics`가 모두 true로 끝난 뒤 paint를 확인했다. 608쪽 #세나루의
폭 출처는 이전 미완료 실행의 heuristicHalfwidth에서 nativeOtf로 바뀌었다.

마스크 없는 화면/독립 Print 집중 비교는 표지 72.23324%, 608쪽 86.20868%,
757쪽 96.55515%다. 표지는 사용자 Print에 없는 외부 그림 placeholder가 있고,
608쪽은 제목 폭 차이가 남는다. 준비 실패 해소를 시각 완료로 간주하지 않는다.
전쪽 출력·HWP3 대조·새 Native/fresh WASM 인쇄 검증을 계속한다.
