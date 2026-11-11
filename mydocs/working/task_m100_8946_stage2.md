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

## 전쪽 재검증과 남은 굵기 간격

자원 보정 head `f01462c356`에서 기본 공급자 없음·옵션 생략·명시적 false 각각의
763쪽 트리와 PNG 해시는 고정 base와 전부 같았다. Native 계약 44개가 통과했고,
새 producer의 편람 383쪽 필수 gate도 passed였다.

HWPX 전쪽은 Native/fresh WASM 각각 19쪽, 호스트 화면은 20쪽이 90% 미만이다.
표지를 제외한 미달 쪽은 세나루·강낭콩의 굵은 제목을 공통으로 사용한다.
608쪽 세나루의 원본 S/Q/L 폭은 CFF와 PDF /Widths에서 각각 0.542/0.708/0.563em으로
일치한다. 독립 Print의 TJ는 S 뒤 -44.8, Q 뒤 -52.5이며, 다른 제목에서도 글리프
폭과 별도로 약 0.05em의 전진이 있다. 12pt 세나루와 20pt 세나루·강낭콩,
18pt 딸기, 32pt 신명 디나루에서도 확인했다. -4% 자간 제목은 원본 자간을 함께
적용한 값이며 자연 폭·자간·굵기 전진을 하나의 보정값으로 합치지 않는다.

선행 보정 `f70a8c937d3dd56ac942d019dfca40c851e0a6a3`는 미확정 HWP5 유형이나
임의의 HFT/CFF에 굵기 전진을 적용하지 않도록 제한했다. 이 제한을 유지한다.
이번 후보는 누락된 세나루·강낭콩·딸기만 기존의 선언·실제 glyph 공급자
양쪽 확인 계약에 추가한다. 임베디드, 실제 Bold face, TTF, 미확인 프로그램에는
적용하지 않으며, 원본 메트릭 교체·저장 LineSeg 재사용 정책도 변경하지 않는다.

생산은 `legacy_font_advances`의 선언 확인과 `NativePuaFonts::register_otf`의 실제
프로그램 확인이다. `text_measurement::char_width_decision_with_hft_tracking`이
두 결과와 실제 글리프의 faux bold 상태를 확인해 기존 전진을 계산한다. 측정과
배치는 이 공통 글자 폭을 소비하며 paint의 윤곽 크기는 변경하지 않는다.
현재는 보정 후보이며 집중 출력 대조와 전쪽 재검증 전에는 완료로 판정하지 않는다.

신명 디나루는 기존 HFT 측정 계약에서 이미 별도 전진을 적용하므로 정상 대조군으로
유지한다. 이 글꼴을 포함한 PDF의 154개 고유 글리프 /Widths는 공급 CFF의 hmtx와
모두 정확히 일치했다. PDF ToUnicode의 bfchar/bfrange를 각각 해석하고, 고유 폭과
별도 TJ 연산을 구분해 기록했다.

## 집중 보정 검증과 공급 조건

[독립 PDF 연산](../pr/assets/issue8946/stage2-20261111/pdf-title-operators.json)과
[프로그램 고유 폭 대조](../pr/assets/issue8946/stage2-20261111/font-widths.json)를 보존했다.
최적화 fresh WASM의 집중 대조에서 71·608·686쪽은 각각 raw 100.00%이며
미대응 픽셀도 0이다. 그림 영역을 제외한 표지 보조값도 100.00%, 미대응 픽셀 0이다.
이 결과는 전체 문서 완료나 제외 영역의 통과를 뜻하지 않는다.

Native는 공급 폴더의 387개 중 최대 256개를 읽으므로 뒤에 있는 신명 디나루를
실제 paint에서 공급하지 못했다. 호스트가 두 원본에서 실제 읽은 71개 프로그램의
[해시 목록](../pr/assets/issue8946/stage2-20261111/selected-fonts.json)을 기준으로
검증 전용 공급 목록을 만들었다. Native 한도나 선택 정책은 변경하지 않았다.

사용자는 2026-11-11 외부 그림 3개 영역을 제외한 보조 비교를 승인했다.
[제외 증거](../pr/assets/issue8946/stage2-20261111/linked-image-exclusions.json)는
원본 컨트롤·96dpi 렌더 상자·그림 파일 해시를 연결한다. 원본 PNG는 변경하지 않는다.
Native의 빈 그림 placeholder 테두리는 상자 밖으로 반 픽셀 나와, 정해진 상자만
제외하면 그림자처럼 잔류한다. 이 상태의 86.78%를 글자 결함으로 오판하지 않도록
원본 바이트가 같은 검증 복사본 옆에 정식 연결 그림을 공급했다. 같은 상자 제외에서
100.00%가 됐고 글자 PNG는 동일했다. 상자를 임의로 넓히거나 점수를 보정하지 않았다.

Native 최초 보정 실험은 신명 디나루 확인 분기도 포함했으나, 해당 제목의 bbox와
PNG가 보정 전후 동일함을 확인하고 불필요한 분기는 제외했다. 최종 후보는 누락된
세나루·강낭콩·딸기만 추가하며 Native 최종 빌드·전쪽 검증은 별도로 수행한다.
제외 없는 표지는 독립 Print의 그림 부재와 공급 조건이 달라 계속 미달로 기록한다.
