# #7751·#7494 일반 그림 색조의 Studio DOM 지연 로딩 — 13단계

시작 HEAD `2afc3bf8b04a9d85af01b8c3dbbd06610dbdae04`.
12단계 Native/fresh WASM 전383쪽은 통과했지만, 실제 DOM 소비 경로에서
다음 차이를 재현했다. 이 차이를 남긴 채 PR을 생성하지 않는다.

## 독립 기대값과 실제 실행

같은 실제 한컴 저장 입력 `hancom-color-probe-2024.hwp`와 대응 Print를 사용한다.
밝기20·대비0의 원본 RGB(80,100,120)은 Print에서(131,151,171)이다.
최종 fresh WASM의 `getPageFlowImageOps`·`getSourceImageBytes`와 실제
Studio `flowImageOpsFromNarrowQuery`를 Chrome에서 호출해 DOM `<img>`를
표시하고 브라우저 screenshot의 픽셀을 읽었다. 결과는(96,120,144)이었다.
`brightness(1.2000)` CSS 승산이 독립 Print의 가산 색조를 대신하고 있었다.
진단 첫 실행은 TypeScript shim의 transpile API가 없어 실패했다. 실제 Node24의
타입 제거로 재실행한 결과만 위 결함의 실행 증거로 사용한다.

## 생산 결과와 소비 경로

기존 일반 그림의 색조는 Native·Canvas·SVG 각 backend 직전에 bake된다.
반면 공통 layer의 resolved payload와 source key에는 원본 바이트·B/C가 남아,
JSON·좁은 질의·DOM은 같은 결과를 받지 못했다. 같은 BinData를 다른 B/C로
사용한10개 행에서 키도 모두같아 CSS에 의존했다.

`원본 image 속성 → 공통 resolved PNG/suppress_effects → layer JSON·좁은 질의의
표시 속성 → 매개변수 포함 source key → 같은 PNG 재현 → 실제 DOM 표시`를 연결한다.
소스 모델의 B/C·효과는 유지하고 표시 속성만 bake 후 중립값으로 만든다.
frontend에 별도 색조식을 복제하지 않는다. 기존 epoch·variant 거부와 캐시 상한도 유지한다.

## 구현과 검증 순서

1. 일반 raster 색조의 적용 조건·원본 포맷 변환을 공통 producer에 두고,
   같은 입력의 inline/lazy가 같은 PNG를 내도록 source variant에 B/C·효과를 넣는다.
   WMF·흑백 무보정·legacy JPEG 워터마크와 RealPic preset의 기존 경로를 보존한다.
2. `tests/cases/`의 공개 API 검사와 기존 실제 브라우저 색조5건을 보강한다.
   독립 Print RGB, 원본 속성 보존, 키 분리, inline/lazy 동일 바이트, 이중 효과 없음과
   실제 DOM screenshot 픽셀을 검사한다. 새 source-side 단위 모듈/함수는 만들지 않는다.
3. 수정 전 실제 RGB assertion FAIL과 수정 후 PASS를 확인한다. Native·Canvas의
   기존 전쪽 출력은 먼저 대표 범위에서 대조한 뒤 최종 전383쪽을 재검증한다.
4. Direct PDF는 실제 bake된 일반 그림을 기존 guard가 허용하는 경로와 unbaked 거부를
   구분해 확인한다. 지원 확대를 추정하지 않고 실제 출력·독립 Print로 판정한다.
5. 최종 코드의 필수 회귀·Native Skia·lint·fresh WASM·실제 Studio 소비를 확인한 뒤
   upstream head 재확인과 승인된 PR·동일 PR trailing review/오늘할일·merge·후속 처리를 진행한다.

## 중단한 검증의 기록

`2afc3bf8b`의 Native Skia lib4,043건·placeholder2건·Direct PDF14건은 통과했다.
기본 전체 nextest는10,466건을 발견하고 실행을 시작했으나, 위 실제 DOM 결함을
확인해 소유한 nextest PID에 SIGINT를 보냈다. 종료코드100이며 전체 PASS로 세지 않는다.
출력 공백이나 시간 경과 때문에 중단한 것이 아니다. 공유 Cargo/Rust 작업 종료를 확인한 뒤
이 단계 코드를 수정한다. 중단 전 로그·단계 결과·screenshot과 RGB는 ignored output에 보존한다.


## 선행 구현 검증

일반 색조는 원본 포맷 변환 뒤 공통 resolved PNG에서 확정하고 suppress_effects를 전달한다.
`pictone_B_C_effect` 신원은 같은 조건으로 같은 PNG를 재현한다. 모델의 원본 B/C·효과는
수정하지 않았다. 실제 저장 fixture의 원본 속성·색조별10개 키·inline/lazy 동일 바이트·
독립 RGB·중립 표시 속성·오래된 epoch와 잘못된 variant 거부를 공개 API로 검사했다.
수정 전 compile0/test101로 B8 lazy(80,100,120) 대 Print(100,120,140)에서 실패했고,
수정 후 compile0/test0으로 통과했다. 첫 after 진단의 serde_json 이중 --extern 오류는
실행 증거로 세지 않고 중복 인자를 제거한 after-r2만 사용했다.

Native 빌드는4분56초, fresh WASM은4분30초에 종료코드0이었다. Native library는
별도 --lib warm build로 실제 산출물을 고정했다. 오래된 top-level rlib를 새 빌드로
간주하지 않았다. 공용 target·jobs8·WASM codegen16/no-opt host 경로를 유지했다.
Native/fresh WASM의 실제 저장 색조5개는 각각전1쪽 모두raw100%를 재확인했다.
그 뒤 `tests/cases/issue_7751_picture_tone_source.rs`에 정식 공개 API 검사를 등록했다.
기존 브라우저5건도 실제 Studio 함수와 좁은 질의·source bytes·DOM screenshot까지
보강했다. 전체브라우저8건은8PASS였고, Canvas550패치와 DOM550패치가 각각
독립Print와±1 이내였다. Native 단독 통과로 DOM을 통과 처리하지 않았다.

현재 제품 Native383쪽도 새로 출력해 모든 쪽의raw가12단계와 같았다.
최저93.07066%,90%미만·누락·devel대비0.01%p이상하락0쪽이다.
수동 export orchestration에서 처음 Path 대신 문자열을 넘긴 오류, 그 뒤 출력 목적
디렉터리를 만들지 않은 오류는 제품 회귀로 세지 않았다. 네 실제CLI는96/96/96/95쪽을
모두 출력했으므로 manifest·페이지·바이트·PNG·tree를 전부 확인해 같은 새 산출물을
집계해 복구했다. 기존 renderer 출력을 다시 실행한 것으로 기록하지 않았다.
기준 PDF의 캐시383쪽도 앞 단계SHA를 다시 대조했으며 새로운 한컴 변환은 하지 않았다.

CI의 Node22에서도 실제 TypeScript 함수 로딩을 지원하는
[stripTypeScriptTypes API](https://nodejs.org/download/release/v22.14.0/docs/api/module.html#modulestriptypescripttypescode-options)의
22.13.0 도입을 확인했다. 타입만 제거해 같은 함수를 실행하며 색조 구현을 테스트에 복제하지 않는다.
fresh WASM383쪽도 새로 출력해 최저92.98362%,90%미만·누락·고정devel대비
0.01%p이상하락0쪽을 확인했다. Native Skia lib는4,043통과·12기존ignore,
공식 focused26건도 모두 통과했다. 실제SVG 브라우저3/130/375쪽은 각각
raw99.99690/98.61065/99.46269%였다. 실제Direct PDF 색조5개도 모두raw100%였다.

최종 전체 nextest를 준비하던 중 upstream/devel에 #7953이 들어왔다.
이전 기준의 실행 중 nextest 소유 PID에 SIGINT를 보내 종료코드−2로 끝냈다.
이는 전체 통과가 아니며, 출력 공백이나 관찰 timeout 때문에 종료한 것이 아니다.
clean 작업 트리와 Cargo/Rust 종료를 확인한 뒤 새devel 위로30커밋을 충돌 없이
rebase했다. 이어지는 검증은 [14단계](task_m100_7751_stage14.md)에 기록한다.
