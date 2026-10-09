---
kind: working
status: completed
last_verified: 2026-10-09
---

# #7688 7단계 — 최종 회귀와 검증 기록

제품 구현은 crop `dfeee5b26`, 화살촉 `0ec7eace9`, Studio font/style `f17a62908`다.
고정 base는 `6f66932a73ee6fd05a74a83bdc438c9499a89a12`, 공용 target은
`/home/edward/mygithub/rhwp/target/pr-review`다. 결과는 ignored
`output/pr-review/renderer-backend-audit-20261009/final/`과 각 단계 증적에 잇는다.

## 기존 crop 검사 정정

첫 전체 nextest 실행에서 두 기존 SVG unit이 실패했다. 4단계 기대값 수정 시
partial fallback의 기대값을 explicit full-image 검사에 잘못 연결하고,
fallback 검사에는 이전 full-width 기대값을 남긴 것이 원인이다.
실행 실패와 테스트 준비 오류를 구별하며 최초 전체 실행 로그를 보존한다.

| 입력 계약 | 독립 기대값 | 최종 검사 |
| --- | --- | --- |
| crop (0,0,174000,26580), imgDim이 같은 명시 전체 크기, pixels 2320×354 | reference 전체가 source 전체와 대응 | width=2320, height=354 |
| crop (0,0,102366,26580), reference 없음, pixels 2320×354 | 동일 HWPUNIT 두 축의 source window 비율 보존; 끝점=전체라는 근거 없음 | window ratio=102366/26580, width<2320, height=354 |

단위 검사의 명시 reference는 독립 명세의 전체 좌표 대응을 검사한다. reference 부재 입력의
실물 부분 선택 계약은 동일 원문의 한컴 PDF/Native/fresh WASM과 공개 CLI crop 회귀가 입증한다.
합성 검사의 임의 전체 reference를 완화하거나 보정된 출력만 새 golden으로 고정하지 않았다.
제품 source·WASM은 변경하지 않고 `src/renderer/svg/tests.rs`의 두 조건만 바로잡았다.
수정 후 crop 경계 검사와 전체 필수 검증을 재실행했다. 최초 실패 결과와 아래 최종
통과 결과를 구분해 보존했다.

## 기존 #7333 화살표 검사와 독립 출력 대조

첫 전체 실행은 10,553건 중 10,548 PASS / 5 FAIL / 50 skipped였다. 위 crop 2건 외
3건은 #7333의 화살표 검사였다. 최초 로그를 ignored `issue7333-arrow/initial-full-integration.log`에
보존했다. 변경한 제품 source가 아니라 기존 검사가 무엇을 고정했는지 독립 출력으로 확인했다.

`samples/issue7333/aaaaaa.hwp`와 대응하는 `pdf/issue7333/aaaaaa-2020.pdf`에서
19·31·41·43쪽을 먼저 Native/fresh WASM sweep하고 review PNG를 직접 판독했다.
화살촉 소속·방향·크기는 보존되며 4쪽 최저 실루엣은 양쪽 모두 99.796%다.
이 문서의 점선 패턴·스크린샷 raster 색상 차이는 여전히 존재하며 화살촉 회귀로 분류하지 않는다.

| 실패 검사 | 기존 조건의 문제 | 독립 근거와 정정 |
| --- | --- | --- |
| 31쪽 signed transform | shaft를 줄인 좌표 (445.9,701.2)를 저장 끝점으로 고정 | PDF triangle tip=(331.140625,532.097656)pt, 96dpi=(441.52,709.46)px. 실제 보존 끝점=(441.89,710.25)px와 대응; 방향·원래 시작점 조건 유지 |
| 19쪽 connector | 보이지 않는 `line stroke=none marker-end`라는 이전 SVG 표현을 요구 | PDF의 오른쪽 아래 방향 유지. 실제 shaft와 화살촉 tip의 동일 소속·유일성·중심축 정렬을 검사 |
| 41·43쪽 screenshot connector | 같은 보조 marker 표현을 요구 | 두 PDF의 오른쪽 위 방향 유지. 실제 shaft의 끝에 소속된 화살촉·정렬을 검사; source connector 파싱 조건 유지 |

PDF 측정은 `pdftocairo -svg -f <쪽> -l <쪽>`의 독립 vector path에서 읽었다.
19쪽 tip=(485.378906,343.542969)pt, 41쪽=(227.156250,446.273438)pt,
43쪽=(216.125000,441.718750)pt다. connector 검사는 이 화면 절대 좌표를 새 golden으로
고정하지 않고 endpoint 소속과 PDF에서 확인한 방향의 관계를 검사한다.

실제 호출은 저장 signed renderingInfo → Line/Path의 끝점·commands →
`arrow::connector_heads`의 world head → SVG `draw_arrow_heads`, Skia/WebCanvas,
paint JSON `arrowHeads` → CanvasKit이다. 일반 Line은 SVG marker의 tip을 보존한다.
이 대조 후 기존 검사만 정정했다. 제품 Rust·TS·Docker WASM은 다시 변경하지 않았다.
curve/arc의 한컴 접선은 이 straight connector 검사의 근거로 대신하지 않는다.

## 실제 출력과 수치 게이트

| 실행 범위 | 결과 | 증적 |
| --- | --- | --- |
| 대표 6문서 전체 8쪽 Native/fresh WASM | 두 경로 각각 최저 95.89322%, full review gate 모두 passed; 누락 없음 | `final/{native,wasm}-review/summary.json`, 초기 TSV `final/{native,wasm}-sweep/` |
| 실제 Native PNG screen/print 전체 8쪽 | 두 profile 각각 최저 96.18681% | `final/native-png/{screen,print}/` |
| 실제 Studio 두 backend, 대표 6문서의 첫쪽 | Canvas2D 최저 95.53281%, CanvasKit software 최저 94.58234% | `final/studio/`; 표 문서의 Studio 2·3쪽은 이 검사 범위가 아님 |
| #7333 전체 50쪽 Native/fresh WASM | 양쪽 최저 95.41718%, 90% 미만·누락 없음 | `issue7333-arrow/{native,wasm}-all-pages/issue7333/silhouette.tsv` |
| #7333 영향 4쪽 실제 Native PNG | 최저 99.77949% | `issue7333-arrow/native-png/` |
| #7333 영향 4쪽 실제 Studio | Canvas2D 최저 99.52841%, CanvasKit software 최저 99.07962% | `issue7333-arrow/studio/` |

새 full review를 실제 출력으로 다시 실행하여 TSV-only의 `not_evaluated`와 구별했다.
full review의 crop·화살촉·수식 대표 PNG 및 #7333의 네 review를 직접 확인했다.
실제 Native 31쪽, Studio CanvasKit 19·41쪽의 화살촉도 직접 읽었다. 실제 화면의 전체
paint 의미를 SVG 존재 여부나 높은 실루엣 점수만으로 승인하지 않는다.

Docker JS/WASM은 `70cde06a…` / `cc5e9c9d…`이며 pkg/public과 실제 초기화 응답이 대응한다.
Studio main/renderer/style module의 source map hash까지 원본 TS와 대조했다.
6단계 TS 이후 이번 Rust 수정은 cfg(test) unit과 integration 검사뿐이므로 제품 WASM을
다시 빌드하지 않는다. `final/provenance.json`의 source/test/artifact hash로 재사용 범위를 고정한다.

## 잔여 차이와 자동 진단의 한계

기존 browser baseline을 6문서 × screen/print × Canvas2D/CanvasKit compat/default의
36캡처로 다시 실행했다. runtime hard gate 0, render error·unexpected unsupported·image failure 0이다.
엄격한 backend parity는 **report-only**이며 24비교 중 16 PASS / 8 FAIL이다. 탭과 수식의
각 profile/CanvasKit mode 4건씩이 여전히 기준을 넘는다. 실패 건수만으로 무회귀라 판정하지 않았다.
탭의 ink-mask 차이는 0.007237→0.005945, 수식은 0.007808→0.006411로 감소했으며
각 비교의 실제 차이 크기를 `final/browser-parity-before-after.json`에 보존했다.
basic의 tolerant 차이는 0.013529→0.013996으로 늘지만 ink-mask 차이는
0.001956→0.001631로 줄었다. 이 backend raster 지표와 독립 Print의 90% 시각 기준을 구별하며,
backend 완전 동일 또는 엄격 parity 전체 통과를 주장하지 않는다. 허용치는 바꾸지 않았다.

software 캡처는 auto/GPU surface readiness gate를 실행하지 않는다(evaluated=0).
Windows/macOS·실제 GPU, 모든 italic 원 face, 단일 bold-only 공급의 합성 정책,
모든 화살촉 모양·curve/arc 및 원본 크기가 완전히 불명인 모든 crop을 완료로 승격하지 않는다.
이번 crop·삼각 화살촉·문서 font 준비/복수 face 선택의 실제 해결 범위와 구별해 남긴다.

후속 작업자가 같은 소비 경계를 확인할 수 있도록 canonical
[backend 검증 가이드](../manual/verification/renderer_backend_verification.md#구현된-소비-경계-회귀)에
정식 CLI/Studio 회귀의 위치와 실행 방법을 연결했다. 브라우저의 실제 글꼴 style 소비 검사는
독립 face 공급 대조군이며 그 합성 paint tree를 별도 한컴 문서의 조판 golden으로 등록하지 않았다.

## 최종 로컬 검증과 전달 상태

모든 Rust lint를 순차로 통과했다: fmt, Native root Clippy, WASM32 Clippy,
workspace build/all-target Clippy, 추가 Native Skia Clippy. suite manifest와 source-side
unit-tier 정책도 고정 base `6f66932a…`에 대해 통과했다. 추가 helper의 Clippy
`manual_is_multiple_of`를 suppression 없이 `is_multiple_of`로 정정하고 해당 focused 검사와
all-target lint를 다시 실행했다. 최초 실패 로그와 최종 통과 로그를 분리해 보존했다.

| 검사 | 최종 결과 |
| --- | --- |
| crop source 경계 8건 | 8 PASS |
| #7333 문서 회귀 | 14 PASS |
| release-test 전체 nextest | 10,553 PASS / 0 FAIL / 50 skipped; 실행 632.284초 |
| Native Skia lib와 workspace lib | 4,109 PASS / 0 FAIL / 13 ignored |
| Native missing-picture placeholder | 2 PASS |
| Native 직접 PDF 내보내기 | 4 PASS |
| 최종 Native release-test build | PASS |
| Studio TypeScript·전체 unit | noEmit PASS; 1,825 PASS / 0 FAIL / 2 skipped |
| 새 font preparation/style 실제 E2E | 수정 전 FAIL / 수정 후 PASS; font URL 차단 fallback 포함 |
| 변경 가이드·6/7단계 내부 링크 | 3문서 이상 없음 |

15개 Rust 실행 기록·argv·exit code는 `final/checks-results.json`과 같은 이름의 로그에 있다.
Studio의 기존 단위 검사와 최종 E2E는 6단계 source hash에 대응하며 그 뒤 TS/E2E source를
변경하지 않았다. 전체 비교 범위와 미실행 surface는 위 표와 잔여 범위대로 구별한다.

최종 새 Native 바이너리를 `final/rhwp-final`에 보존하고 대표 전쪽 screen/print 16캡처와
#7333 영향쪽 print 4캡처를 다시 출력했다. **20개 PNG 모두 이전 실제 캡처와 byte 단위로 동일**하다.
`final/native-final-output-identity.json`은 새/이전 바이너리와 실제 PNG의 hash를 연결한다.
따라서 이전 실물 PNG 비교를 다른 바이너리 신원으로 바꿔 쓰지 않고, 새 실행과 동일성 증거를 잇는다.
원본·독립 Print 및 full review는 별도로 보존하며 이 동일성을 한컴 일치의 근거로 대신하지 않는다.

`final/validation-inputs.json`은 검증한 Rust/TS/E2E 21개 파일의 hash를 고정한다.
기록 commit 뒤 동일성을 다시 확인한다. code 변경 없는 기록 commit에 대해 대형 검사나
동일 WASM을 불필요하게 반복하지 않는다. generated suite·manifest와 output은 stage하지 않는다.

검증용 Studio는 `http://localhost:7788/`이다. ignored `final/review.html`은 crop·화살촉·
수식과 #7333 영향쪽의 Print/실제 Native PNG/Studio Canvas2D/CanvasKit을 나란히 보여 준다.
메인테이너의 기존 시각 통과는 3단계 Native 수식에 적용하며 이번 추가 수정의 사용자 판정으로
확대하지 않는다. 이번 결과는 승인된 순차 로컬 구현·검증의 완료다. 후속 시각 판정과 제출은
최신 비교 자료에서 이어 진행하며, #7688 전체 종료·원격 push·PR 생성·merge는 수행하지 않았다.
