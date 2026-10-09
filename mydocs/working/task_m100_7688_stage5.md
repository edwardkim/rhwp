---
kind: working
status: active
last_verified: 2026-10-09
---

# #7688 5단계 — 화살촉의 공통 형상과 실제 소비

crop 단계 `dfeee5b26`을 커밋한 뒤 승인된 순서로 진행한다.
독립 기준은 group-drawing-02.hwp의 한컴 Print PDF와, 해당 원문을 CLI export-hwpx로
저장한 대조군이다. 대조군의 한컴 출력이 원문의 네 화살촉을 보존하는 것을 먼저 확인했다.
저장 줄·그룹·끝점은 유지하고 lineShape의 크기/두께만 바꾼 3개 입력을 Print했다.
짧은 선 대조군은 마지막 저장 scale matrix의 길이만 바꿨다. 임의 저장 LineSeg는 만들지 않았다.
ignored arrow-fix/의 입력, MCP 영수증, PDF와 control-measurements.json을 연결한다.
출력은 한컴 11.0.0.9136, print method 0, one-up이다.

## 독립 관측과 적용 경계

HWP 크기 코드 0..8의 첫 성분은 진행 방향 길이, 둘째는 수직 폭이다.
얇은 선에서 길이/폭의 세 수준은 약 5.757/11.514/17.271 pt이며 두 축은 독립이다.
medium에서 저장 width 32와 100은 같은 최소 크기, 400과 1200은 약 4배 선 두께의
화살촉을 사용한다. 최소 단위는 약 2.88 pt(96dpi에서 3.84px), 수준 배수는 2/4/6이다.
이는 독립 출력에서 관측한 paint 메트릭이며 파일 ID나 화면 위치의 예외가 아니다.
짧은 선 4/8/16pt에서도 medium 길이/폭은 약 11.514pt로 유지된다.
따라서 현재의 선 길이 30% cap과 축 순서·3px 최소 높이는 근거가 없다.
화살촉 크기를 줄이거나 선 끝점을 반전시켜 짧은 선을 처리하지 않는다.
다이아몬드/원/사각/오목 모양의 기존 구성은 보존하고 같은 크기·방향의 공통 path로 전달한다.
독립 실제 출력으로 검증하지 않은 모양은 삼각 화살촉의 한컴 일치 근거로 대신하지 않는다.

## 실제 소비와 반례

lineShape bits → LineStyle → 공통 arrow path(끝점·방향·길이·폭·채움) →
SVG marker, WebCanvas path, Native Skia path, paint JSON → CanvasKit path.
Native Line op는 현재 화살촉 없이 shaft만 그리며 Path connector도 별도 확인한다.
SVG marker cache는 두께에 따른 다른 형상을 같은 id로 재사용하지 않아야 한다.
JSON 소비자는 크기를 다시 추측하지 않고 공통 path를 사용한다.
그룹 transform/clip과 앞뒤 plane, 선/화살촉 소유는 유지한다.
반례는 크기 9종, 얇은/굵은 선, 짧은 선, 양끝·비수평 방향, 무화살표 대조군이다.
먼저 영향 영역을 직접 확인하고 Native/fresh WASM 90% 선행 조건 뒤 정식 회귀를 추가한다.

## 구현 및 선행 검증

공통 `src/renderer/arrow.rs`가 크기·끝점·방향·채움 path를 생산한다. 일반 Line의
shaft 끝점은 보존한다. SVG 일반 marker와 복합선/connector, WebCanvas,
Native Skia 및 JSON의 `arrowHeads`를 통한 CanvasKit이 이를 소비한다.
SVG marker id에 stroke width를 포함하여 다른 두께가 같은 형상을 재사용하지 않는다.
CanvasKit은 누락된 공통 형상을 임의 추측하지 않고 unsupported 진단으로 기록한다.

최종 제품 코드로 Native/fresh WASM 6개 전쪽 sweep을 실행했다.
원문 99.87583%, 크기 0..3 99.87570%, 크기 4..7 99.87593%, 크기 8/두께
99.84531%, 짧은 선 99.87554%, 복합선 99.87578%로 두 경로가 동일하다.
모든 입력은 1쪽이며 누락·90% 미만 쪽은 없다. widths·compound review와 원문
화살촉 확대를 직접 판독했다. 점수와 별도로 네 화살촉 및 source shaft 소유를 확인했다.

실제 Native PNG 수정 전에는 네 화살촉이 없었다(전체 점수는 99.78321%였다).
수정 후 screen/print에서 네 화살촉이 표시되고 99.87079%다. 실제 Studio Canvas2D/
CanvasKit software에도 네 화살촉이 표시된다. 명시 CanvasKit의 본문 family가 기본
Noto Sans KR로 바뀌는 별도 결함은 다음 단계 범위로 남겼다.

Docker `scripts/wasm-pack-locked.sh --target web --out-dir pkg`는 7분 50초에 완료했다.
JS 70cde06a… / WASM cc5e9c9d…는 pkg와 public에서 일치하며 실제 초기화 응답도
검사했다. TS noEmit 및 기존 render-backend 검사, Native/WASM Clippy는 통과했다.

90% 선행 조건 뒤 `tests/cases/issue_7688_arrow_heads.rs`를 추가했다. 공개 SVG의
intrinsic head 크기/끝점 소유와 실제 Native PNG의 shaft 밖 화살촉 영역을 screen/print
양쪽에서 검사한다. 기준값은 독립 Print의 ~15.35px head이며 절대 화면 좌표나 hash가 아니다.
검사 컴파일 및 같은 검사 바이너리의 수정 전 FAIL/후 PASS는 현재 실행 중이며 다음 기록에
연결한다. 처음 잘못 지정한 suite 이름은 검사 준비 오류로 결함 증거에서 제외한다.
필수 전체 Rust 묶음은 승인된 후속 단계의 최종 head에서 실행한다.

증적: ignored `output/pr-review/renderer-backend-audit-20261009/arrow-fix/`의
`final-native-sweep/summary.json`, `fresh-wasm-sweep/summary.json`,
`studio-{canvas2d,canvaskit}-provenance.json`, `native-before-after-review.png`.
다른 head 모양·양끝 비수평·curve/arc 접선의 독립 한컴 출력, Native 복합선 shaft 자체는
이번 삼각 화살촉 비교로 검증하지 않았다. 기존 clipping/rotation/layer를 변경하지 않았지만
그 모든 조합의 완결을 주장하지 않는다.
