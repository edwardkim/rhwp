# #8153: 원본 연속 글자 그림자의 공통 재생

## 사양·실제 원본·독립 인쇄

HWP5 revision 1.3 표 35의 bit 11~12는 그림자 없음 0, 비연속 1, 연속 2다.
표 33의 두 INT8 간격과 그림자 색도 각각 보존해야 한다. 일반 문자 그림자이며,
그림·도형의 별도 shadow 속성이나 수식 효과로 해석하지 않는다.

두 편람 9쪽의 큰 숫자 `1`은 글자 모양 784, KoPub돋움체 Bold의 영문 슬롯,
58,000HU·장평 100%·흰색·연속 그림자 2·흰 그림자·간격 `(1,1)`이다.
같은 원본 스타일의 장 제목 숫자 `2`~`5`도 다른 구역에 저장되어 있다.
9쪽의 잔여 25픽셀은 숫자 영역의 모서리에 걸린다. 원본 HWP/HWPX와 독립
PDF를 바꾸지 않고 현재 실행 코드가 두 그림자 종류를 구분하는지 확인한다.

현재 `SkiaTextReplay::draw_text`와 Canvas/SVG는 `shadow_type > 0`이면 끝 위치의
그림자를 한 번 그린다. 원본의 비연속·연속 구분을 소비하지 못한다. 독립 한컴
2024 Print PDF는 9쪽의 같은 숫자를 600dpi 장치의 x/y에서 한 단위씩 전진하며
반복하고 마지막에 원 위치의 본문을 그린다. 실제 60회 중 59회가 중간 그림자다.
이 구별이 없는 현재 소비가 사양의 연속 그림자 계약을 지키지 못한다.

현재 원본의 nominal 간격을 장치로 단순 환산한 값과 PDF의 반복 끝은 다르다.
단일 숫자에서 배율을 추정해 상수로 추가하지 않는다. 실제 한컴에서 저장·Print한
작은 대조군으로 종류 0/1/2, 다른 글자 크기, 혼합 축·음수·0 간격을 확인한다.
HWP5가 반복 장치 간격을 명시한다고 주장하지 않고 인쇄에서 확인한 미기술 동작을
별도로 기록한다. 기존 383쪽 PDF는 재출력하지 않는다.

## 공통 생산·소비와 보정 범위

`CharShape bit·원본 간격 → 언어별 TextStyle → 공통 그림자 재생 계획 →
Native·Canvas·SVG의 실제 glyph 그리기`를 추적한다. 각 backend가 종류·보간·끝을
독립적으로 추측하지 않게 한다. 원래 글자 원점·줄/셀 소속·전진 폭·본문 그리기
순서는 유지한다. Screen과 Print의 별도 장치 계약은 확인한 범위에서 구분한다.
쪽·문서 ID·숫자·크기 임계값으로 적용 대상을 좁히지 않는다.

먼저 동결된 기존 라이브러리에서 원본 장 제목과 작은 독립 대조군을 재생해
방향을 확인한다. 사양·독립 기대값으로 정한 정식 `tests/cases/issue_8153_...`
검사를 수정 전 FAIL/수정 후 PASS로 보호한다. 비연속·없음의 정상 출력도 유지한다.
새 source의 Native/fresh WASM 영향 쪽과 전쪽 1,532조건을 실제 검증하며, 집중
진단만으로 목표 raw100·미대응 0이나 최종 필수 검증을 완료 처리하지 않는다.

분석·임시 원본·PDF·실행 로그는 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/continuous-text-shadow/`
아래에 둔다. upstream 동기화·원격 GitHub 쓰기·push·PR 생성은 하지 않는다.

## 실행한 원인 분리와 독립 대조

두 원본을 실제로 연 결과 모두 글자 모양 784의 연속 그림자 2와 간격 `(1,1)`을
보존한다. 9·27·183·251·277쪽의 현재 Native 출력을 동결된 같은 라이브러리로
재생했으며, 진단 전 PNG 10개는 기존 전쪽 산출물과 RGB 전 픽셀이 같다.

| 쪽 | 현재 Native 미대응 | 진단 재생 미대응 | 범위 |
| --- | ---: | ---: | --- |
| 9 | 25 | 0 | HWP·HWPX 각각 |
| 27 | 7 | 0 | HWP·HWPX 각각 |
| 183 | 5 | 0 | HWP·HWPX 각각 |
| 251 | 16 | 0 | HWP·HWPX 각각 |
| 277 | 8 | 0 | HWP·HWPX 각각 |

진단은 실제 원본·글꼴·논리 bbox를 유지하고 그림자 paint만 분리한 것이다.
이 숫자는 제품 보정·새 head의 전쪽 검증이 아니며 기존 1,388/1,532 완료 조건에
합산하지 않는다. 단순히 원본 nominal 간격 사이를 연결한 별도 진단에는
9·27·183쪽 각각 1픽셀, 251쪽 4픽셀이 남았다. 연결과 거리 생산을 따로 확인한다.

작은 대조군은 실제 한컴 2024 13.0.0.3901에서 HWP로 저장한 뒤 1-up Print했다.
최종 대조군은 25쪽이며 그림자 없음·비연속·연속, 양수·음수·혼합 축·0,
작은 크기·소수 거리·구멍이 있는 글자와 서로 다른 세 글꼴을 포함한다.
반환된 HWP의 실제 글자 모양을 다시 읽어 크기·종류·간격 보존도 확인했다.
초기 대조군의 잘못된 `DISCRETE` 토큰은 제외하고 정식 `DROP`으로 다시 저장·인쇄했다.
원본 편람·기존 PDF·글꼴 파일은 변경하거나 복사하지 않았다.

같은 100pt·간격 10%에서 독립 PDF의 연속 그림자 끝은 KoPub돋움체 Bold
`(100,100)`, Times New Roman `(101,101)`, 굴림체 `(83,83)` 장치 단위다.
따라서 한 글꼴에서 관찰한 1.2배를 일반 보정으로 채택하지 않는다.
HWP5 표 33·35는 저장 종류·간격 범위를 명시하지만, 그 표만으로 인쇄의 거리 기준
메트릭·장치 정수화·중간 표본의 계산식을 확정할 수는 없다.

독립 PDF 25쪽의 중간 재생 순서는 모두 대조했다. 끝점을 관측 입력으로 사용한
순서 검증과 끝점 자체를 만들어 내는 생산 규칙의 입증을 구분한다. 글꼴 메트릭과
인쇄 원점의 실제 생산·소비 대조를 마친 후 공통 구현에 연결한다. 음수 그림자의
본문 clip 경계도 함께 확인하며, 그림자를 연결한 뒤의 범위를 과거 bbox로 자르지 않는다.

대표 9쪽의 직접 확대 판독은 현재의 계단 모서리가 진단 재생에서 연결되고 독립
Print의 모서리와 대응함을 확인했다. 그림자 끝점의 글꼴별 생산과 정상 반례가
미완료이므로 구현 완료·회귀 통과·PR 준비 완료로 판정하지 않는다.
독립 API·바이너리 분석 기록은 작업지시자의 범위에 따라 ignored output에만 둔다.

확인할 증거는 위 작업 폴더의 `originals-focused-diagnostic.json`,
`shadow-step-contract-proof.json`·`.log`, `normalized-print-v4-result.json`,
`normalize-print-v4.log`, `windows-font-metrics.json`,
`diagnostic-p9-corner-review.png`다. 최종 제품 결과는 여전히 source `8b960562c8`의
전쪽 1,532조건 중 1,388조건 완료·144조건 잔여다. 코드 수정 후 필수 검증을
다시 완료하기 전에는 진단 자료로 TSV를 교체하지 않는다.

## 공통 재생 연결 단계

독립 Print 대조에서 확인한 연속 그림자의 중간 위치를 공통 paint 결과로 만든다.
실제 공급한 SFNT의 OS/2·VDMX 정보를 읽으며 글꼴 이름별 배율을 추가하지 않는다.
Native·Canvas·SVG는 같은 결과를 소비하고 원문 TextRun·source 수량을 유지한다.
원본 간격은 해소된 px를 다시 추정하지 않고 signed 백분율을 별도로 전달한다.

우선 실제 연속 그림자 경로를 연결해 원본 5쪽의 방향을 확인한다. 비연속 그림자의
남은 절대 원점 경계, 음수 그림자의 본문 clip, 호스트 글꼴 공급·최종 bounds,
전체 Native/fresh WASM 결과가 완료되기 전에는 최종 보정·PR 준비 완료로 판정하지 않는다.
공통 코드의 거리·중간 위치에 관한 기대값은 실제 한컴에서 저장·인쇄한 대조군에서
정한다. 임시 진단과 비공개 분석 기록은 계속 ignored output에만 둔다.

## Native 공통 구현 집중 확인

실제 Native renderer가 선택한 SFNT를 메모리로 공급하고 공통 paint에서 연속 그림자의
거리·중간 위치를 해소했다. Native·Canvas·SVG의 재생 반복은 이 공통 결과를 소비한다.
원문 TextRun이나 source를 복제하지 않으며, 원본 signed 간격을 별도로 전달한다.
본문 clip은 흐름 그림자 범위를 포함하고 셀·글상자·곡선 clip의 소유 경계는 유지한다.
공급 증거가 없는 프로그램으로 거리를 확정하거나 글꼴별 고정 배율을 추가하지 않는다.

Native 집중 실행에서 두 원본의 9·27·183·251·277쪽 10건 모두 raw100·미대응 0을
확인했다. 함께 실행한 8·10쪽은 기존 결과를 유지했고, 14쪽의 논리 RenderTree JSON은
기존 source 8b960562c8과 바이트 단위로 같다. 실제 한컴 저장·Print 대조군 중 연속
그림자 20건도 모두 Native raw100·미대응 0이다. 음수 그림자의 기존 14픽셀 잘림도
본문 clip의 공통 범위 반영으로 0이 됐다. 비연속 대조군의 기존 차이는 개선 완료로
주장하지 않는다. 중간 표본의 혼합 축·음수·0과 종류 구분 계약 검사 2건은 통과했다.

자료는 작업 폴더의 `product-native-v3/focus-result.json`, `controls-result.json`,
`focused-logical-tree-preservation.json`, `common-contract-receipt.json`,
`page009-current-corner-review.png` 및 `product-common-contract-tests.log`다.
전쪽 Native 비교는 766건 모두 완료했으며, raw100·미대응 0은 695건에서 705건으로
늘었다. 미대응 증가나 raw 하락은 없고 전쪽 논리 RenderTree는 기존 source와 바이트
단위로 같다. 이 결과는 source `65af5b612d`의 Native 중간 결과이며, 같은 source의
표준 최적화 fresh WASM 전쪽 결과를 대신하지 않는다.
SVG의 호스트 공급 경로와 직렬화된 paint 계약,
fresh WASM 집중·전쪽 및 최종 필수 검증은 후속 단계이며 이 집중 확인으로 대신하지 않는다.

## 종류 변경과 잉크 범위의 후속 경계

HWP5 표 35의 종류 0·1·2가 재생 계획보다 우선한다. 이미 준비한 연속 계획이
남아 있어도 종류 없음은 그림자를 그리지 않고, 비연속은 끝점 한 번만 소비한다.
출력 투영에서 종류를 바꾸는 실제 Native 재생을 수정 전 라이브러리와 대조하고,
같은 공통 소비 규칙을 계약 검사로 보호한다. 실제 표시 문자열이 공백뿐인 경우
줄 점유는 보존하되 연속 그림자의 잉크로 본문 clip을 늘리지 않는다.

검증은 기존 전쪽 증적을 유지하며 새 source의 집중 재생부터 진행한다. 진단용
dev WASM은 방향 확인에만 사용하고 최종 표준 최적화 WASM 검증과 구분한다.

### 2026-10-27 확인 결과

source `65af5b612d`의 dev WASM 집중 검증에서 두 원본 장 제목 5쪽씩 10건 모두
raw100·미대응 0이며, 함께 확인한 8·10쪽은 기존 결과를 유지했다. 독립 Print
대조군의 연속 그림자 20건도 모두 raw100·미대응 0이다. 이 집중 결과는 최종 표준
최적화 fresh WASM 전쪽 결과에 합산하지 않는다. 같은 좌표에서 확대한 9쪽의
이전 Native·현재 Native·dev WASM·독립 Print 모서리도 직접 대조했다.

종류 0으로 투영한 실제 Native 재생은 이전 연속 계획의 유무에 따라 PNG가
달라지는 문제를 수정 전 라이브러리에서 재현했다. 종류 우선 규칙과 표시 잉크·셀
clip 경계를 보호한 공통 Rust 검사 4건은 통과했다. 새 검사 작성 중의 좌표 비교
타입 오류는 검사에서 좌표 성분을 비교하도록 바로잡은 뒤 전체 선택 검사를 다시
실행했다. 수정 후 실제 Native PNG 재생은 새 라이브러리 빌드 뒤 대조한다.

Studio의 실제 호스트 공급 세션은 WMF 요청만 읽고 있어, 연속 그림자만 있는
문서에서 공급 등록이 누락되는 것도 수정 전 API 호출로 재현했다. 같은 세션이
연속 그림자 요청도 함께 읽도록 연결했다. 실제 바이트는 기존 호스트/local 공급
경로를 재사용하며 글꼴 파일은 복사하지 않는다. 새 공급 계약 3건과 기존 PUA
공급·문서 교체 계약 4건을 함께 실행해 7건 모두 통과했다. 이 검사의 가짜 바이트는
공급·비동기 소유 계약만 검증하며 실제 글꼴·glyph·시각 일치의 증거가 아니다.

새 증적은 같은 작업 폴더의 `diagnostic-wasm-dev/focus-comparison.json`,
`diagnostic-wasm-dev/controls-result.json`,
`diagnostic-wasm-dev/page009-native-wasm-print-corner-review.png`,
`stale-shadow-before/result.json`, `source-kind-guard-contract-tests.log`,
`studio-shadow-supply-before.log`, `studio-shadow-supply-final-tests.log`에 있다.
소스 종류·호스트 공급 보정의 최종 Native/fresh WASM 전쪽·전체 필수 게이트는
계속 진행하며, 현재 목표 전체 완료나 PR 준비 완료를 선언하지 않는다.

## 직렬화된 paint와 내장 SFNT 공급

Rust 내부 소비자는 공통 연속 계획을 받지만 기존 레이어 JSON의 종류·끝점만으로는
CanvasKit 소비자가 중간 표본을 재생할 수 없다. 준비된 공통 계획의 순서 있는
좌표를 `shadowOffsets`로 전달하고, TextRun과 glyph paint 모두 같은 표본을
소비하게 한다. 소비자가 글꼴 메트릭이나 보간을 다시 추측하지 않는다. 종류 0·1은
이전에 남은 연속 표본보다 우선하며, 계획이 없는 이전 전송 형식은 기존 끝점을
유지한다. 잘못된 표본을 임의 좌표로 바꿔 그리지 않는다.

WMF가 없어도 연속 그림자의 실제 글꼴 메트릭은 필요하다. 원본 TextRun의 종류로
필요 여부를 확인하고, 문서 내장 face도 기존의 제한된 메모리 공급 경로에서
등록한다. source family·원본 bytes·본문의 논리 메트릭을 바꾸지 않는다.
Native 실제 재생을 검증하는 source `e5a1e3e04a`는 유지하며, 이 후속 보정은
별도 로컬 검증 worktree에서 준비한다. 최종 source의 공통 Rust 계약·실제 레이어
JSON·CanvasKit 소비와 Native/fresh WASM 시각 증거를 확인하기 전에는 완료로
판정하지 않는다. 별도 SVG 호스트 글꼴 공급 경로도 계속 확인한다.

후속 공통 Rust 검사 6건은 모두 통과했다. 내장 face 검사는 저장소에서 직접 만든
MIT 합성 TTC를 읽어 WMF 없는 페이지의 공급·공통 계획·실제 레이어 JSON 연결과
Screen 분리를 확인했다. 좌표 기대값은 합성 face의 OS/2·UPEM과 원본 signed 비율의
단위 계약이다. 실제 한컴 문서의 시각 검증과 구분한다. JSON 숫자의 정수/실수 표기를
다르게 비교했던 검사는 해석한 숫자 좌표를 비교하도록 고친 뒤 6건을 다시 실행했다.

실제 CanvasKit 제품 `renderGlyphRun`을 호출한 진단은 이전 코드에서 끝점 한 번만
소비하는 실패를 재현했고, 후속 코드에서 독립 혼합 축 표본 6회와 원문 1회의 순서를
확인했다. 이 소비 경계를 종류 0·1·2의 정식 검사로 추가했다. 새 전송·기존 공급
검사 총 13건과 TypeScript 타입 검사가 통과했다. 가짜 canvas/font의 명령 소비
검사이며 실제 글꼴·픽셀·Print 일치의 증거로 대신하지 않는다.

Native source `e5a1e3e04a`의 실제 종류 0 투영은 수정 전 FAIL·수정 후 PASS이며,
원본 영향 14건·독립 대조군 25건의 미대응 증가가 없다. 영향 14건의 논리 RenderTree도
기존 source와 바이트 단위로 같다. 후속 전송 보정의 실제 Native/fresh WASM 출력과
전쪽 목표는 계속 검증한다. 작업 폴더의 `product-native-v4/validation-receipt.json`,
`focused-logical-preservation.json`, `shadow-transport-rust-contract-tests.log`,
`shadow-transport-node-final-tests.log`, `shadow-transport-typecheck.log`,
`shadow-transport-canvaskit-consumer-before.log`·`after.log`를 대조한다.
