# PR #7833 셀의 최소 물리 폭 회귀 증거

이 증거는 기존 셀 경계 대조군의 **한 쪽**을 정식 회귀로 보존한다. 원래 생성기의 잘못된
저장 프레임을 그대로 둔 다른 입력 전체의 해결이나 통합 PR 준비 완료를 의미하지 않는다.
기존 셀 경계 검사는 제거·완화하지 않는다.

## 입력과 독립 기준

- 입력: [nested.hwpx](../../../../tests/fixtures/inline_cell_physical_width/nested.hwpx),
  12,316바이트, SHA-256 `29bd118673625ab5f773c46fc81cae9ca4672d809e02453e252050a7b2137e96`.
- 독립 기준: [한컴 Print PDF](../../../../pdf/semanticist21-20261019/pr7833-inline-cell-physical-width-2020.pdf),
  20,612바이트, 한 쪽, SHA-256 `54812721daf73c93012b359a669d1bb99bfccdb293494dcb2c5364cff17703bb`.
- MCP job `b19f886f-ece2-45bf-a02b-e8588c3dad99`, 실제 한컴 `11.0.0.9136`, engine `2020`,
  `pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`을 확인했다.
  원본 용지·1-up·100% 인쇄다. 진단용 HWP 저장본을 이 PDF의 대체 기준으로 쓰지 않았다.

출발점은 `tests/common/cell_boundary_object_fixture.rs`의 depth=3·tables=true·before
계약이다. 진단 대조군은 본문에서 셀로 옮긴 문단의 `invalidate_layout_inputs()`를 호출하고,
작은 표의 셀 격자를 개체 크기에 맞추며, 중첩 표의 common/셀 격자를 부모 내용 폭에 맞춰
같은 비율로 만든다. 텍스트·스타일·네 경계 개체와 세 단계 소유 관계는 유지한다.
생성 코드·변경점은 [작업 기록](../../../working/task_semanticist21_20261026_regression_contracts.md)의
4299/4300 및 최소 물리 폭 절에 기록했다. 이번 입력/PDF는 그 대조군의 바이트 동일 사본이다.
원 생성 입력의 실패는 별도로 유지하며 이 대조군으로 통과를 대신하지 않는다.

## 실제 시각 검증

96 DPI·print profile·2px 이웃 raw 실루엣으로 전체 한 쪽을 비교했다. 폰트 예외를 적용하지 않았다.
공급 경로는 `/usr/share/fonts`, `/usr/local/share/fonts`, `/home/tsjang/.local/share/fonts`,
저장소 `assets/fonts`다. Native265와 표준 최적화 fresh WASM078에서 **각각 100%**였다.
양쪽 review·standalone overlay를 직접 열어 셀 경계·텍스트 순서·부모/자손 점유를 확인했다.
100%는 raw 실루엣의 양방향 2px 포함이며 RGB 화소 동일성을 뜻하지 않는다.

| backend | 전쪽 TSV | review | standalone overlay |
| --- | --- | --- | --- |
| Native265 | [원문](native_silhouette.tsv) | [PNG](native_review.png) | [PNG](native_overlay.png) |
| fresh WASM078 | [원문](wasm_silhouette.tsv) | [PNG](wasm_review.png) | [PNG](wasm_overlay.png) |

Native 실행 파일 SHA-256은
`75991e51cddb71d38dd07923dd7b3ee73d0020437e550fe604358a8603f0fc55`,
fresh WASM은 `1ebc656b1b74c2a7e47bf36b9557cb0acd7f9f84b857342da3910debd1cb0e16`,
JS는 `a48acb46a157950724859baea436fda8d5d3b7c9a0fd10665455c952bf4c2434`다.
빌드 시작 head `8e44c0f233142b89c63ace77ba574ae1cc494d75`에 추가한 최소 폭 보정의
정확한 dirty source 해시는 ignored 4352/4356 receipt에 고정했다. Native/fresh는 같은
런타임 소스이며 JS·WASM은 `pkg`와 Studio public에서 해시가 같다.

실제 명령·입력/PDF/글꼴·producer 해시는 ignored
`output/pr-review/semanticist21-20261019/`의 `reflow-visible-host-validation-4354-result.json`,
`reflow-visible-host-validation-4361-result.json`에 보존했다. 로그·중간 JSON은 커밋하지 않는다.

## 독립 기대값과 회귀

[독립 기대값](independent-width-oracle.json)은 PDF `get_drawings()`의 세로 괘선 중심을
96/72배한 좌표다. 같은 네 작은 표의 왼쪽·중간·오른쪽 경계가 각각
127.77067057291667 / 145.04133097330728 / 162.47199503580728 CSSpx다.
모델의 선언 600 HU나 수정 helper의 283 HU를 기대 좌표로 재인용하지 않는다.

[정식 검사](../../../../tests/cases/inline_cell_physical_width_print.rs)는 독립 PDF와 입력의
해시·한 쪽·표/칸 수, 내용의 누락/중복 없음, 칸 안 여백 뒤의 양수 내용 폭, 실제 부모 셀 포함·문단/개체 소속, 열 경계의 연결,
표의 원문 순서·비겹침을 검사한다. 양쪽 raw100과 직접 판독을 마친 페이지의 중요한 가로
경계·폭만 ±2px로 보호하며, 세로 절대 위치는 고정하지 않는다. 글꼴 환경에 따라 달라질 수
있는 줄 수·텍스트 원점을 px 핀으로 바꾸지 않는다.

같은 검사 소스 SHA-256
`3f76a1d68cde30c4c57e7bbc552e9e765ee3e10c15e184f85bb3ebcb0b507872`를
수정 전 Native264와 수정 후 Native265에 각각 실행했다. 둘 다 컴파일에 성공했고,
전자는 실제 폭 16px / 독립 기대 34.7013244628906px로 **1 FAIL·exit101**, 후자는
**1 PASS·exit0**이었다. 빌드 실패를 RED로 세지 않았다.

자동 수집을 prepare한 뒤 `run-rust-test.mjs inline_cell_physical_width_print`로
실제 `regression_suite_023`에서 **1 PASS/0 FAIL/307 필터 제외**, exit0을 확인했다.
공유 target/pr-review·8 build jobs·12 test threads이며 로그는 ignored
`logs/minimum-cell-new-regression-nextest-4372.log`다. 이 자동 실행의 Cargo CLI
SHA-256은 `6e7de7f3c9cd54c22458f3a6ee0c390e6d64f77bb9dc3d3af0b4d718d78b7d8b`다.
같은 런타임 소스이며 별도 테스트 빌드의 실행 파일을 시각 검증의 Native265와 혼동하지 않는다.
기존 109 PASS와 이 새 한 함수의 실제 실행을 구분한다. 최종 검사는 부모 셀 포함·문단/개체 소속을 보강한 같은 소스로 RED/GREEN을 다시 확인했다(4371). 처음 버전의 4369/4370은 과거 기록이다.
