# #8109 후속 성능 검토

후속: 아래는 보정 전 검토 기록이다. [4단계](task_m100_8109_stage4.md)에서 성능 보정과
재검증을 완료했고, 같은77쪽 auto 열기 준비는1,284.9 → 626.2ms로 개선됐다.

- Issue: #8109. 날짜: 2026-10-21. 작업지시자의 성능 영향 검토 요청에 따른다.
- 검토 head: `f1ae43d70f5984d8cfe2ab920ced2814f6210325`.
- 결론: **성능 무영향으로 판정할 수 없다. Studio 자동 렌더러 선택에 전체 페이지 순회로 인한
  열기 준비 지연을 확인했다.** 제품 코드와 기존 기능 검증 결과는 변경하지 않았다.

## 확인된 회귀와 호출 경로

`main.ts:525`의 `documentFontRequirements`는 대체 선언 하나만 있어도 모든 쪽의
`WasmBridge.getPageLayerTreeObject`를 호출한다. `host-font-requests.ts:25`는 그 결과의
전체 노드·textRun을 다시 순회한다. Tree는 페이지마다 Rust 출력과 JSON 해석을 거치며,
기존 Tree를 재사용하는 전용 캐시는 이 helper에 없다.

`RendererSession.readPreflight:385`는 이 helper를 호출하는 transform을 실행한 **뒤**에
`complete/eligible`을 판정한다(`renderer-session.ts:281–294`). 따라서 이미 incomplete인
문서도 전체 순회 비용을 지불하고 Canvas2D로 돌아간다. 정상 준비에서는 transform과
`prepareCanvasKitDocument:606`이 각각 호출할 수 있다. host provider가 있으면 기존 host
준비도 별도로 전체 쪽을 순회한다. 같은 helper를 공유하는 것과 결과를 재사용하는 것은 다르다.
편집 후 자동 선택도 300ms 지연 뒤 재진입할 수 있지만, 이번 실측은 문서 열기까지다.

저장된 브라우저 상태가 없는 독립 context에서 같은 최적화 WASM을 사용해
`samples/kps-ai.hwp`(77쪽, 대체 선언8개)를 매번 새로 로드했다. before/after 순서를
번갈아 실행했고 워밍업1쌍을 제외한9쌍의 중앙값은 다음과 같다.

| 실제 경로·구간 | 변경 전 ms | 변경 후 ms | 관측 |
| --- | ---: | ---: | --- |
| `renderer=auto`: 파싱·조판 + `CanvasView.loadDocument()` | 634.8 | 1,279.2 | **+644.4ms, +101.5%** |
| 위 경로의 `loadDocument()` 구간만 | 387.7 | 1,032.3 | 파싱 비용과 분리 |
| 위 경로의 transform 콜백 | 0.1 | 644.4 | 추가 Tree 조회 **0 → 77회** |
| `renderer=canvas2d`: 파싱·조판 + `loadDocument()` | 483.5 | 491.3 | 차이의95% bootstrap 구간 -14.8 ~ +9.4ms |
| 위 Canvas2D의 `loadDocument()` 구간만 | 238.1 | 239.0 | transform·추가 Tree 조회0회 |

auto의 전후 최종 판정은 모두 `autoPreflightIncomplete` / Canvas2D였다.
총 시간 차이의 paired bootstrap 95% 구간은 **+606.1 ~ +664.1ms**로, 작은 실행 변동과
구별된다. 이는 파일 바이트를 메모리에서 전달한 시점부터 view 준비 반환까지의 시간이다.
웹폰트 다운로드·파일 선택 UI·최종 첫 화면 paint 완료 시간으로 확대 해석하지 않는다.

같은 Tree가 이미 워밍업된 콜백 직접 실행에서는77쪽 transform이0.1 → 216.2ms였다.
실제 새 문서 열기의644.4ms와 혼동하지 않는다. 대체 선언 없는 `hwpspec.hwp`182쪽은
0 → 0.5ms이고 추가 Tree 조회는 없었다. 대체 선언 있는1쪽 계약 문서는 transform과
준비에서 합계2회 조회하며 약1.1ms를 사용했다. 직접 실행의 준비 renderer는 글꼴 I/O를
제외하기 위한 stub이며, 위 실제 auto 열기 경로는 실제 제품 콜백·선택을 그대로 실행했다.

## Core·Native·반복 조회와 산출물 크기

Core는 변경 전 `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`의 보존한 최적화 WASM과
현재 WASM을 같은 Node 프로세스의 독립 인스턴스로 초기화했다. 각 입력을3쌍 워밍업한 뒤
17쌍 순서를 교차했다. 작은 PDF/HWPX는 한 표본에50회 실행해 프로세스 시작 비용을 제외했다.
가져오기/생성 뒤 `pageCount()`와 `free()`까지 포함하며 paint·직렬화는 포함하지 않는다.

| WASM Core 작업 | 변경 전 ms | 변경 후 ms | 차이 |
| --- | ---: | ---: | ---: |
| 원본 body-list PDF 가져오기 | 2.701 | 2.697 | -0.14% |
| 같은 PDF를64쪽 반복한 부하 입력 | 166.157 | 159.741 | -3.86% |
| 고밀도 합성 PDF8쪽·4,000회 text-show | 11,754.855 | 11,858.796 | +0.88% |
| 대체 없는 HWP182쪽 읽기·조판 | 687.232 | 681.114 | -0.89% |
| 대체 있는 HWP77쪽 읽기·조판 | 239.662 | 241.965 | +0.96% |
| 명시적 대체 HWPX1쪽 읽기·조판 | 0.896 | 0.903 | +0.77% |

64쪽 입력은 원본 body-list의 동일 쪽을 pypdf로64회 복제한 합성 부하이다.
독립64쪽 실문서의 복잡도를 대표하지 않는다. HWP77쪽의 +2.303ms 차이는 작지만
bootstrap 구간 +0.485 ~ +4.782ms여서 비용이 정확히0이라고 표현하지 않는다.
다른 입력의 개선 수치도 전체 엔진의 보편적인 속도 개선 주장으로 사용하지 않는다.
고밀도 입력은 PyMuPDF로 쪽마다100행×5열의 작은 Helvetica 텍스트를 넣은 별도 합성 부하다.
3쌍 워밍업 후13쌍을 교차했고 총 출력8쪽도 확인했다. +103.940ms 차이의95% 구간은
-33.896 ~ +121.375ms로0을 포함했다. 이 입력 역시 실제 문서의 조판 품질 증거는 아니다.

Native는 같은 `release-test`·native-skia 산출물로2쌍 워밍업 후13쌍을 교차했다.
PDF는 `import-pdf … output.hwpx --json`, HWP는 `info … --json`을 실행했다.
프로세스 시작·읽기·PDF 저장을 포함하므로 위 in-process Core 값과 직접 비교하지 않는다.

| Native CLI 작업 | 변경 전 ms | 변경 후 ms | 차이 |
| --- | ---: | ---: | ---: |
| body-list PDF 가져오기·저장 | 14.321 | 15.957 | +1.636ms, +11.42% |
| 반복64쪽 PDF 가져오기·저장 | 153.251 | 156.066 | +2.815ms, +1.84% |
| HWP182쪽 info | 475.931 | 473.550 | -0.50% |
| HWP77쪽 info | 179.208 | 181.099 | +1.06% |

작은 PDF의 차이는 bootstrap95% 구간 +0.060 ~ +3.798ms였다. 64쪽·두 HWP의 차이
구간에는0이 포함됐다. Native 네 부하의 peak RSS 중앙값은 변경 전후 약23–70MiB로
증가가 관측되지 않았다. WASM의 heap peak·장시간 누수·GPU 메모리를 측정한 결과는 아니다.

CanvasKit의 실제 `findPreparedTypeface` 캐시 적중 경로를 독립 측정했다. 첫 miss의 font
탐색만 stub으로 제외하고 실제 key 생성과 LRU 조회·갱신을200,000회 반복했다.
3쌍 워밍업 후21쌍 중앙값은 단일 family371.6 → 31.7ms, 세 후보386.7 → 33.2ms였다.
기존의 매번 NFC·locale 소문자 정규화보다 새 JSON key 생성이 이번 조건에서 빨랐다.
이는 키/캐시 비용에 관한 결과이며 실제 glyph paint, shaping, 스크롤 FPS 개선 증거는 아니다.
캐시는 기존256개 제한을 유지한다. Canvas2D의 host font 해소는 host provider가 없으면
조기 반환한다. host provider가 있는 전체 paint와 긴 후보 체인의 최악 비용은 별도 측정하지 않았다.

Rust 변경은 font 프로그램을 span마다 깊은 복제하지 않고 Arc를 유지한다. 추가 span vector와
paint-order entry에는 참조를 보존하는 비용이 있으며, PDF lowering은 적용 span마다 대체 선언과
진단 JSON을 만든다. HWP parser는 font record별 대체 이름을 보존한다. 이를 무할당 변경으로
설명해서는 안 된다. 위 실측 범위에서는 Studio의 전체 순회가 가장 큰 지연이었다.

최적화 WASM 파일은21,483,428 → 21,573,866바이트로 **+90,438바이트(+0.421%)**다.
압축 전 파일 크기이며 네트워크 전송량·다운로드 시간의 측정값은 아니다.

## 보정 방향과 검토 범위

1. `complete=false`인 preflight에서 대체 후보를 위한 Tree 순회를 생략한다. 자동 선택의
   실패 이유와 대체 선언의 유효성은 유지해야 한다.
2. 완료된 preflight에서도 font 후보·style의 공통 수집 결과를 준비 단계와 공유해 같은
   전체 문서를 중복 출력·해석하지 않도록 한다. 단순 이름 치환으로 원본만 쓰는 동일 family의
   다른 런이나 bold/italic 선택을 잃지 않아야 한다.
3. 캐시를 사용한다면 문서·편집 revision, host/local font generation, render profile에 맞춰
   무효화하고, 기존 source-only·혼합 후보·스타일·늦은 local 갱신 검증을 유지한다.

이 문서는 성능 검토 결과다. 최적화 구현은 수행하지 않았다. 보정 후 동일 조건의 실제 auto 열기와
Canvas2D 대조군을 다시 측정해야 성능 회귀 해소를 판정할 수 있다. 기능 시각 증거는 기존
[3단계 보고](task_m100_8109_stage3.md)의 동일 source head 결과이며 성능 증거를 대신하지 않는다.

## 실행 신원과 원시 증거

- 브라우저 Chrome155, Node v24.15.0, WSL2 Linux. before/after를 동시에 측정하지 않았다.
- Core/Native before는 [1단계 기록](task_m100_8109_stage1.md)의 보존 산출물이다.
  Native SHA `b493a9ad3ea1dc00b373c7d5733197600dfa325c05a53c121a9e50ce06df09bb` →
  `cfb7cb5f91201212f786a061a0a50e880635dda0ce454cfb39d66bfce29a551e`.
- WASM SHA `ec32880a5e176ded1e282cd59be2c0d5be9fd66030b5159aa35db3177322f7b8` →
  `b8c20fcfe5952ed2a1d3f4188842438275f4a964bceaa55f1f170fc6d640ffe1`.
- Studio before는 `5699a377885e3068947400e69a2336af5759112c`의 변경 전 TS4개를
  Vite7703의 pre-load plugin으로 공급했다. after는 현재 TS4개를7701에서 공급했다.
  양쪽 모두 같은 현재 WASM을 사용해 frontend 비용을 분리했다. 두 서버와 context는 검토용이다.
- `output/pr-review/issue8109/performance/`에 `core.mjs`, `native.py`, `frontend.mjs`,
  `open.mjs`, `summarize.py`와 원시 `core.json`, `native.json`, `frontend.json`, `open.json`,
  `core-dense.mjs`/`core-dense.json`, `cache.json`, 입력 SHA·산출물 신원(`identity.json`),
  `summary.json`을 보존한다. 통계는 seed8109의
  paired bootstrap10,000회로 산출한 중앙값 차이이며 체계적인 환경 차이를 제거하는 보증은 아니다.
- 첫 frontend 시도는 이전 origin의 local-font 저장 상태가 섞였고, 다음182쪽의 incomplete
  report를 준비 callback에 전달해 중단했다. `first-frontend.*`를 보존하고 최종 통계에서는 제외했다.
  독립 browser context와 complete 분기로 보정한 재실행 결과가 위 표다. 캐시·기능 기대값을
  완화한 결과가 아니다. 제품 코드·사용자 문서·기존 서버와 공용 Cargo cache를 보존했다.
