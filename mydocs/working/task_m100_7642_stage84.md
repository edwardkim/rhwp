# #7642 Stage 84 — CanvasKit 수식의 표시 face와 자연 전진

Issue: #7642

## 독립 기준과 원인

Stage 83의 실제 CanvasKit software surface는 NOT 사선 20개와
공개 중괄호 윤곽을 정상 replay했지만 300dpi Print 대조는
88.24930%다. 같은 소스의 Native SVG는 94.57865%다. 화면의
렌더 오류 0건을 시각 일치의 증거로 사용하지 않는다.

Native SVG는 수학 알파벳을 Latin Modern Math에서 표시하고,
문자·숫자는 그 글꼴의 자연 전진으로 그리며 논리 상자의 폭은
다음 개체의 원점 예약에 사용한다. CanvasKit은 CJK 기본 face를
먼저 선택하고 수학 fallback은 Noto Sans Math이며, 논리 예약 폭에
맞춰 글리프를 다시 수평 확대한다. 실제 소비 계약이 서로 다르다.

`renderEquationBox → drawEquationText → face 선택 → setScaleX →
drawText`를 SVG Text/Number/Symbol의 실제 호출과 대조했다.
두 경로의 표시 face와 자연 글리프 전진을 일치시킨다. 가운데
기호는 논리 상자의 가운데와 자연 표시 폭으로 정렬한다.
논리 상자의 측정·다음 원점·NOT 끝점은 변경하지 않는다.

## 구현 범위

CanvasKit의 기본 수학 fallback 공급을 동봉 Latin Modern Math로
정하고 수학 glyph replay에서 먼저 선택한다. 사용자가 지정한
math fallback URL은 그대로 소비한다. 수학 face에 없는 CJK 등은
기존 본문·기호 face 순서로 공급한다. 표시 함수는 별도 1000줄
미만 모듈로 옮긴다. 기존 대형 renderer는 공급 경로와 호출부만
바꾼다. 공개 글꼴 자산은 기존 것을 재사용한다.

## 검증과 남은 범위

같은 fresh WASM·입력·Print와 실제 CanvasKit software surface의
수정 전후를 대조하고 수식 영역을 확대 판독한다. tsc와 관련
기존 Studio 계약·글꼴 검사를 실행한다. 새 정식 회귀는 시각
선행 조건을 충족한 뒤 추가한다. Rust source tree가 같으면
Stage 83의 Native/fresh WASM 빌드 해시와 현재 frontend SHA를
구분해 기록한다. 전체 원본 55개·853쪽 TSV 및 기존 px 검사의
관계형 전환 범위는 유지한다.

`W`의 Print는 같은 HYhwpEQ 글리프에도 text matrix의 shear
0.333을 사용한다. 네 크기의 Print 연산에서 확인했으며 Linux
설치본과 PDF subset의 윤곽 해시도 같다. 이 독립 변환과 글꼴
공급 문제는 아직 해결하지 않았다. 해당 차이는 별도 잔여로
기록하고 이번 CanvasKit 소비 계약 보정으로 완료 판정하지 않는다.

## 후보 구현과 실행 증거

표시 함수는 `canvaskit/equation-text-replay.ts`로 분리했다.
CanvasKit은 Latin Modern Math를 먼저 선택하고 표시 글리프의
자연 전진을 유지한다. 가운데 기호만 예약 폭의 가운데에서
정렬한다. 기존 대형 renderer의 호출·공급 경로를 바꿨으며
Rust 레이아웃과 NOT 기하는 변경하지 않았다.

`tsc --noEmit --pretty false` exit 0, 기존 preflight·font-plan
검사 10개 PASS, `node e2e/renderer-contract.test.mjs` exit 0이다.
renderer 계약 검사의 기존 분수 대역에 필수 `axisHeight`가
없어 실패했다. 실제 layout JSON 계약에 맞춰 글자 크기의
1/4 축을 공급했고 기존 표시 수량·미지원 연산 assertion은
유지했다. 정식 실물 문서 회귀를 새로 추가한 것은 아니다.

실제 CanvasKit software surface의 후보 출력도 렌더 완료,
오류·미지원 연산 0개다. 본문 face를 공급하지 않은 대조는
300dpi에서 87.74143%이며 함초롬돋움이 Noto Sans KR로
대체됐다. 이를 한컴과 같은 글꼴 환경의 완료 증거로 사용하지 않는다.
공식 설치 `HANDotum.ttf`를 기존 host font provider로 공급한
별도 캡처에서는 face 1개 적재, 대체 0개, 실패 0개를 확인했다.
이 캡처의 Print 점수·확대 판독은 진행 중이다.

Rust source tree `0b3c6d19a743beefdcde83b562f9576d755febae`의
Stage 83 fresh WASM 빌드는 exit 0, 903.50초다. WASM SHA-256
`9075485e35b5ba9dc847b9fa4da0d2b05001f6250b0a9700b6a3c969a0770608`과
JS SHA-256 `a043bdc9047d3ff5d89f55ccc21846f017ac109dad6de25b2885e43c44ee08da`가
Studio public과 각각 일치한다. 같은 Rust source의 NOT 경계
8개는 Native/fresh WASM 모두 99.29469% 이상이다. frontend
후보의 캡처는 위 Rust source와 구분해 기록한다.

전체 Native 원본은 4개·81쪽까지 최저 90.76899% 이상이고
전체 fresh WASM은 1개·15쪽까지 95.12426%다. 나머지 원본의
Native/fresh WASM TSV 산출을 계속한다. 전체 관련 회귀의
관계형 전환과 최종 필수 게이트는 미완료다.

상기 증거는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`
아래의 `stage84-*` 로그·PNG·receipt와 `not-glyph-frame-*` 원장이다.

### 공식 본문 face 공급과 인쇄 안내선

같은 한컴 저장 HWP·Print PDF에 공식 함초롬돋움을 공급한
CanvasKit의 300dpi 일치율은 93.93565%다. 글꼴 미공급 대조와
환경이 다르므로 이를 코드 보정만의 개선율로 보고하지 않는다.
수식의 표시 face·자연 전진은 SVG 경로와 대조했고 `W`의
독립 Print 변환 차이는 아직 남는다.

실제 PNG에서 편집용 여백 안내선도 검출했다. `renderPage`의
tree replay 뒤 `pageInfo`가 있으면 이 선을 무조건 덧그리므로
print profile을 소비하지 못했다. screen에서는 보존하고
print에서 제외하도록 소비 조건을 보정했다. 이 후속 변경의
타입·계약 검사와 실제 인쇄 캡처를 다시 실행한다. 위 93.93565%
결과는 이 안내선 보정 전 frontend `38f571c67`의 결과다.

후속 보정의 tsc와 renderer 계약 검사는 모두 exit 0이다.
공식 본문 face를 다시 공급한 실제 CanvasKit 캡처도 대체·실패·
미지원 연산 0개다. 첫 캡처 경로를 재사용해 보정 전 PNG가
덮어써졌으므로 이전 수치만 `prior-guide-comparison.json`으로
구분해 보존한다. 이를 수정 전후 이미지 증거로 사용하지 않는다.
현재 PNG는 안내선 보정 후 출력이며 새 점수와 확대 판독은
해당 현재 파일로 다시 산출한다.

### 안내선 보정 후 판독

frontend `8c88db70a`의 공식 본문 face 공급 캡처를 다시 대조해
300dpi 일치율 94.58213%를 확인했다. 실측 입력은 한컴 저장
`not-glyph-frame-2024.hwp`이고 동일 저장본의 한컴 Print를 사용한다.
현재 PNG·frontend 파일 해시·fresh WASM 해시는
`stage84-canvaskit-host-font-after-guides/receipt.json`에 고정했다.
새 확대 그림은 `stage84-canvaskit-current-selected-detail-review.png`다.

사선과 중괄호 replay를 확인했으나 `W`의 표시 모양과 등호의
가로선 높이 차이는 남는다. 따라서 점수 통과만으로 수식의
확대 판독을 충족했다고 판정하지 않는다. 이 표시 변환 보정과
전체 원본의 검증, 관계형 회귀 전환을 계속한다.
전체 Native는 현재 5개·101쪽, 최저 90.76899%이고 전체 fresh
WASM은 1개·15쪽, 최저 95.12426%다. 실행 중인 나머지 원본은
완료 건수에 포함하지 않는다.
