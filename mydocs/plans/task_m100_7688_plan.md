---
kind: plan
status: active
last_verified: 2026-10-09
---

# #7688 Canvas·SVG·PNG 출력 정합성 조사 계획

Issue: [#7688](https://github.com/edwardkim/rhwp/issues/7688). 관련 멀티 렌더러 트래킹은 #536이다.

## 범위와 기준

사용자는 이슈 등록에 이어 조사를 승인했다. 기준은 최신 `upstream/devel`의
`6f66932a73ee6fd05a74a83bdc438c9499a89a12`이며, 별도 작업 브랜치
`work/issue7688-renderer-audit-20261009`에서 진행한다.

현재 제품의 Canvas2D·SVG·Native Skia PNG 소비 경로와 최종 화면을 조사한다.
CanvasKit은 기존 baseline의 대조군으로 유지한다. #536의 미병합 개발 범위를 대신하지 않는다.

## 실행 순서

1. 동일 트리 생성 본체, Canvas 보충 메트릭 선택, portable export 가드와 복원 경로를 추적한다.
2. Docker에서 최종 source SHA의 최적화 WASM을 빌드하고 root `pkg/`와 Studio/public의 해시를 확인한다.
   공용 target은 기본 작업공간의 `/home/edward/mygithub/rhwp/target/pr-review`를 재사용한다.
3. Native Skia 기능을 포함한 동일 바이너리로 SVG·PNG를 출력한다. 기존 baseline manifest의
   문단·탭·표·이미지 crop·수식·그룹 도형 6개 항목에서 screen/print 프로필을 비교한다.
4. 같은 Chrome 실행 파일로 SVG raster와 Canvas 캡처를 확보한다. 실제 Studio 화면에서는
   메트릭 모드 전환 전후의 트리·쪽수·내보내기 후 복원 결과를 별도로 대조한다.
5. 대표 이미지의 직접 판독과 구조 차이로 실행 결함·출력 계약 차이·글꼴 공급 차이·미검증을 구분한다.
   수정 필요성이 확인되면 독립 한컴 출력과 적용 경계를 먼저 확정한다.

## 증적과 판정

입력/source/산출 해시·profile·메트릭 모드·browser/backend/surface·DPI/DPR를 보존한다.
진단 산출물은 기본 작업공간의 ignored
`output/pr-review/renderer-backend-audit-20261009/`에 둔다.
기존 baseline 허용치는 바꾸지 않는다. backend 간 비교 통과를 한컴 출력 일치로 보고하지 않으며,
기준 PDF가 없는 부분은 미검증으로 남긴다. 원격 게시·push·PR·이슈 종료는 이번 조사 결과와 구분한다.

## 1단계 실행 결과

대표 6개 입력·12쪽의 메트릭 조사와 Docker fresh WASM, Native Skia, 독립 한컴 Print PDF 비교를 수행했다.
[1단계 결과](../working/task_m100_7688_stage1.md)에 실행 결함·계약 차이·미검증과 재현 증적을 연결했다.
Native 수식의 한글 네모가 우선 수정 대상이며 공통 crop·화살촉 차이와 구분한다.
zoom/DPR/resize 15조건의 트리 유지와 3개 원래 조건 복원 PNG 일치를 확인했다.
제품 변경·새 baseline·원격 push는 없으며, 이슈는 후속 구현·검증이 남아 active다.

## 2단계 — 원인 수정과 재발 방지 절차

사용자는 잘못된 동작을 고치는 범위에서 같은 문제가 다시 생기지 않는 절차·방법을 마련하도록
범위를 확정했다. 별도 전면 재작성이나 모든 PR의 모든 backend 전수 검증은 요구하지 않는다.

1. 공통 조판/paint 결과와 실제 backend 소비자를 연결하고, font/resource·metric context·합성의
   별도 경로를 확인하는 [출력 백엔드 소비 경로 검증](../manual/verification/renderer_backend_verification.md)을 마련한다.
2. CONTRIBUTING의 참고 항목을 해당 소비 경로의 필수 확인으로 바꾸고 거버넌스·로컬 검증·review에 연결한다.
3. 기존 하네스가 검출하는 것과 수식 glyph/crop/실제 합성 화면에서 남는 검증 공백을 명시한다.
   아직 구현하지 않은 자동 검사를 CI gate로 보고하지 않는다.
4. Native 수식부터 공통 font 소비 계약으로 보정하고, 독립 시각 근거와 수정 전 FAIL/후 PASS를
   확인한 경계부터 정식 회귀로 고정한다. 공통 crop은 입력 좌표 기준을 확정한 후 처리한다.

절차 반영과 제품 구현의 완료는 구분한다. 2단계 문서 변경과 검증은
[절차 반영 기록](../working/task_m100_7688_stage2.md)에 남긴다.

## 3단계 — Native PNG 수식 글꼴 소비 보정

사용자가 구현 진행을 승인했다. 독립 기준은 같은 `samples/eq-01.hwp`의 정상 한컴 Print PDF다.
본문 한글과 Canvas/SVG 수식 한글은 표시되지만 Native PNG 수식에는 네모 glyph가 나온다.
유효한 Batang custom 경로를 공급해도 기존 수식 경로는 그 자원을 받지 않아 문제가 유지된다.

`PaintOp::Equation` → `render_equation` → 재귀 `render_box` → `draw_text`에서
본문의 custom/system/bundled 후보와 glyph coverage 선택을 재사용한다. 수식의 family 순서와
정체/기울임 규칙은 유지한다. 선택된 face별 연속 run의 측정 폭을 실제 paint와 가운데 정렬이
함께 소비하게 하며, LayoutBox의 줄 소속·원점·저장 scale은 바꾸지 않는다.
기존 본문 helper는 같은 후보 수집 본체를 호출하고 기존 우선순위를 유지한다.

반례는 custom Batang 공급, 시스템 CJK fallback, 첫 후보에 한글 glyph가 없는 경우,
한글/수학 기호 혼재와 일반 Latin 수식이다. screen/print와 scale 유무의 실제 호출을 확인한다.
먼저 실제 PNG와 독립 PDF를 비교하고, 시각 선행 조건 충족 후 정식 회귀를 추가한다.
공통 crop·화살촉과 다른 backend의 별도 글꼴 등록은 이 보정의 해결 범위가 아니다.

3단계 보정과 검증을 완료했다. [실행 결과](../working/task_m100_7688_stage3.md)에 수정 전 FAIL/
수정 후 PASS, 전체 회귀, Native Skia 3종, 동일 한컴 PDF의 Native/fresh WASM 및 실제 PNG를
연결했다. #7688 전체는 공통 crop·화살촉·CanvasKit 글꼴 등 후속 범위가 남아 active다.

## 4–6단계 — 순차 구현

사용자가 crop → 화살촉 → CanvasKit 글꼴 공급·style 선택의 순차 구현을 승인했다.
각 단계는 독립 출력의 적용 규칙·반례를 확인하고 실제 Native/Studio 출력 및 Docker fresh WASM을
검증한 뒤 커밋한다. crop의 기존 ‘시작점 0이면 끝점은 원본 전체’ 기대값을 독립 PDF로 재검토한다.
4단계의 원본 기록·좌표 단위·실제 소비는 [crop 보정 기록](../working/task_m100_7688_stage4.md)에 잇는다.


4단계 crop `dfeee5b26` 및 5단계 화살촉 `0ec7eace9`를 커밋했다.
[화살촉 실행 결과](../working/task_m100_7688_stage5.md)에 일반/복합선 실제 소비,
독립 크기 대조군, Native/fresh WASM gate와 수정 전 FAIL/후 PASS를 연결했다.
[6단계 결과](../working/task_m100_7688_stage6.md)는 explicit CanvasKit의 문서 글꼴 준비,
여러 face의 style 선택, 실제 font 실패의 Canvas2D fallback과 consumer 회귀를 연결한다.
승인된 세 구현 단계의 로컬 보정은 완료했으며 필수 전체 Rust 검증은 최종 기록에 잇는다.
이 결과는 #7688 전체의 미검증 backend/변형 완료나 원격 게시·통합 승인을 의미하지 않는다.
