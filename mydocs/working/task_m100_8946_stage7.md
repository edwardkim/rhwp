# #8946 단계 7 — 최신 devel 재검증과 제출 준비

Issue: #8946. 범위는 명시적 `{ layout: true }`를 통한 Canvas2D의 호스트 원본 프로그램 선택·환경 갱신·수명 처리와 회귀 보호다.
메트릭 보존·사용자 명시 선택·저장 줄 재사용의 기존 계약을 유지한다. 수용 조건별 구현과 회귀의 연결은
[단계 5](task_m100_8946_stage5.md), 독립 문자 Print와 로컬 프로그램 공급 근거는 [단계 6](task_m100_8946_stage6.md)을 따른다.

**현재 PR 제출 보류:** 이 단계의 모든 실행 검증은 통과했으나, 직후 원격 갱신에서 새 devel #9017과 열린 #9012의 병합 충돌을 확인했다. 아래 결과를 새 base 검증으로 대신하지 않는다.

## 검증 기준

- base: `1a3ed4a7cd76701085a24ac81d30bc5fb31942c1`.
- 검증 코드 head: `cec288ae2fd84b75b884a05ee1ada6ae3a8d2ac7`.
- #8956과 검토 문서를 충돌 없이 반영했다. 추가된 기능은 ByPath 표 조회의 마지막 경로 입력 계약이며 조판 변경은 없다.
- base와 후보의 Native, 최적화한 fresh WASM, 실제 프로덕션 bundle을 별도로 만들고 [출처 원장](../pr/assets/issue8946/stage7-20261112/producer.json)에 해시를 기록했다.
- fresh WASM이 공개 JS에 동기화한 #8956 설명 주석 31줄은 [AST 대조](../pr/assets/issue8946/stage7-20261112/generated-glue-comment-parity.json)에서 실행 코드가 동일하다. 제출 시 함께 보존한다.

## 수용 조건의 최종 대조

| 조건 | 실행 근거와 판정 |
| --- | --- |
| 옵션 생략·false·공급자 없음의 보존 | 원본 HWPX·그림 포함 HWPX·HWP3 × 세 모드의 base/head 각 763쪽 PNG·tree 동일. 실제 DOM 합성 첫 화면도 9조건 모두 RGB 동일. |
| 첫 화면 원본 선택·실제 파일·face | HWPX/HWP3에서 딸기 원본 SHA·faceIndex 0·indexed program ID가 환경 선택과 메트릭 준비에 연결되고 paint보다 먼저 완료됨을 trace와 실제 화면으로 확인. |
| 파일 A→B·revision·TTC·해제·늦은 응답 | Canvas2D/CanvasKit E2E 통과. #8946의 조판 환경·수명 계약 97개 모두 true. |
| 메트릭 정책·명시 환경·내장 우선순위·저장 | 자동 원본 보존과 명시적 교체를 분리한 E2E 및 기존 테스트 통과. 원본 family, dirty/undo/redo, portable 출력과 문서 교체 계약을 유지. |
| 저장 줄·편집 후 재조판 | 실제 두 입력의 삽입·삭제·저장 후 재열기에서 내용 순서·소유·포함·겹침·복원 관계 각 8개 통과. |
| 최종 Native/fresh WASM·시각 증거 | 아래 전쪽 대조·보호 하한·직접 판독·로컬 검증 완료. |

[전쪽 기본 동작 비교](../pr/assets/issue8946/stage7-20261112/comparison-summary.json),
[합성 첫 화면 비교](../pr/assets/issue8946/stage7-20261112/initial-screen-comparison.json),
[HWPX 첫 준비 순서](../pr/assets/issue8946/stage7-20261112/host-hwpx-first-paint.json),
[HWP3 첫 준비 순서](../pr/assets/issue8946/stage7-20261112/host-hwp3-first-paint.json),
[편집 관계 검사](../pr/assets/issue8946/stage7-20261112/edit-verification.json)에 실제 결과를 보존했다.

## 독립 Print와 전쪽 비교

동일 입력·실제 프로그램 SHA·compat·96 DPI에서 각 763쪽을 비교했다. 외부 그림 세 영역을 제외하지 않았다.

| 입력 | Native 최저 raw | fresh WASM 최저 raw | 실제 Host 전쪽 최저 raw |
| --- | ---: | ---: | ---: |
| HWPX | 96.34886% | 99.56218% | 99.35730% |
| HWP3 | 98.77175% | 99.54840% | 99.35495% |

독립 PDF와 쪽수가 일치하고 90% 미달은 없다. [최신 base/후보 3,052행 대조](../pr/assets/issue8946/stage7-20261112/devel-full-comparison.tsv)에서
HWPX는 base의 Native/WASM 각 18쪽 미달을 모두 해소했으며 HWP3는 그대로 보존했다. 원래 정밀 raw를 Decimal로 대조한 최대 하락폭은 네 조합 모두 0%p다.

[정본 보호](../pr/assets/issue8946/stage7-20261112/protected-comparison.json)는 18입력 × Native/fresh WASM, **3,542쪽 전부 통과**다.
누락 0, 공식 두 자리 보호값 하락 0이며 최저 raw는 98.09%다. [전쪽 PNG·tree](../pr/assets/issue8946/stage7-20261112/protected-output-parity.json)도
직전 검증본과 모두 동일하다. 명시 글꼴 환경·원 프로그램 공급을 유지했고 기준값과 제품 코드를 점수에 맞춰 바꾸지 않았다.
필수 [편람 Native gate](../pr/assets/issue8946/stage7-20261112/handbook-gate.json) 383쪽도 통과했다.

후보의 두 입력 Native/WASM 네 묶음과 실제 Host 두 묶음 역시 이전 확대 판독본의 전쪽 PNG·tree와 동일하다.
그 동일성이 확인된 범위에서 단계 3~6의 제목 23곳·608/686쪽 문장부호·729쪽 및 보호 문서의 기호 판독을 재사용했다.
이번에도 [350쪽 문장부호](../pr/assets/issue8946/stage7-20261112/reviews/zoom/punctuation-350.png)와
[표지 글꼴·그림](../pr/assets/issue8946/stage7-20261112/reviews/zoom/cover-footer.png)을 독립 Print·새 base·후보 두 backend의 같은 영역·배율에서 직접 확인했다.
괄호·물음표의 모양, 앞뒤 글자 간격, 제목의 줄 끝과 다음 줄 관계, 표지 글자와 그림의 배치를 확인했다. 이 확대는 96 DPI raster의 3배 표시이며 300 DPI 재측정으로 표시하지 않는다.
raw 100%를 RGB 완전 동일로 해석하지 않는다.

HWPX의 90,804,612바이트 기준 PDF는 기존 50 MiB 이상 PDF 보관 계약을 유지한다. 동일 그림을 포함한 운반본과 독립 Print 출처·SHA는
[단계 4](task_m100_8946_stage4.md)에 있다. 사용자 제공 글꼴과 독립 표본에서 복원한 프로그램은 로컬 검증에만 사용했다.

## 첫 화면 증거와 검증 스크립트 보완

직접 판독에서 기존 `open.png`가 DOM의 기본 canvas만 저장해 합성 화면을 대표하지 못함을 확인했다.
이전 첫 PNG의 동일성만으로 실제 화면 보존을 판정하지 않는다. 화면 전체가 보이는 동일 viewport에서 세 그림의 decode 성공을 확인한 뒤
DOM 합성 screenshot을 새로 저장했다. 기본 9조건의 base/head는 모두 RGB 동일하다. 명시 활성 화면은 독립 Print와
HWPX **99.12%**, HWP3 **99.17%**이며 세 그림을 모두 포함한다.

또한 그림 포함본 대조에서 환경변수 문자열 `0`이 참으로 평가되어 불필요한 외부 그림 재주입을 시도하던 진단 스크립트 조건을 수정했다.
이전 해당 실행은 완료 증거에서 제외하고 base/head 세 기본 모드의 763쪽 전체를 각각 다시 실행했다.
파일 열기 실패 메시지가 없는지 확인했으며 전쪽 PNG·tree 동일성을 재확인했다. HWP3 omitted/false는 전쪽 글꼴 준비 후
각각 두 번 수집한 결과가 같음을 확인했다. 이 보완은 제품 변경이 아니다.

## 리베이스 기능 보존과 로컬 게이트

#8956은 base/head 각각 Native 4개와 fresh WASM 실제 API 검사로 확인했다. 마지막 단계의 controlIndex-only·부분 필드·사용하지 않는 셀 필드,
잘못된 입력 거부, 실제 6칸 bbox 동일성, 중첩 3×4 표와 바깥 경로 필수 필드가 모두 통과했다.
[WASM 재현 스크립트](../pr/assets/issue8946/stage7-20261112/bypath-api-probe.mjs)는 완성된 pkg 경로를 첫 인자로 받는다.
#8958·#8984 등의 원 승인 범위는 단계 6에서 검증한 기록을 유지하며, 이번 새 base 변경과 전쪽 출력 동일성의 근거를 구분한다.

[최종 실행 원장](../pr/assets/issue8946/stage7-20261112/local-verification-results.json)에 각 로그 해시·실제 종료값을 기록했다.
Rust 전체 13,324 PASS, 기존 suite 제외 45; Native Skia lib·placeholder·PDF 및 HFT 공통 회귀;
fmt·Native/WASM/workspace Clippy·workspace build·suite/unit-tier 정책; TypeScript·Studio 2,789 PASS(기존 skip 2)·frontend 계약;
fresh WASM·E2E·전쪽 시각 검증을 완료했다.
최소 글꼴 격리에서 개인 공급 프로그램 71개는 모두 읽기 차단되고 확인한 공개·OS 프로그램 394개는 읽혔다.
시각 측정의 실제 글꼴 공급과 회귀의 최소 환경을 구분했다. 이동된 HFT 387개는 모두 기존 SHA와 일치함을 확인해 새 로컬 경로만 반영했다.

## 제출 직전 통합 검사와 남은 작업

[통합 검사](../pr/assets/issue8946/stage7-20261112/submission-integration-check.json)에서 #9012는 head `71c0024956`, 리뷰어·리뷰 없음, 개별 병합 가능이다. 후보와의 `git merge-tree --write-tree`는 종료 1이며 `src/renderer/legacy_font_advances.rs` 한 곳이 충돌한다.
두 PR이 같은 공통 helper를 도입하면서 #9012는 딸기, #8946은 독립 Print로 확인한 세나루·강낭콩·딸기 계약을 추가했다. `native_pua_fonts.rs`의 공통 helper 소비 변경은 자동 병합된다. 충돌은 새 PR 생성 전 해소해야 하며 다른 PR의 실제 merge 승인을 뜻하지 않는다.

이후 `upstream/devel`이 `a85d3265dd`로 전진했다. #9017은 #8982/#8983의 HTML 표 border 및 첫 서식 전 평문 보존과 속성 경계 보정을 통합했다. 이 단계를 커밋한 뒤 새 base의 기능·필수 검증을 진행한다.
[제출 준비 판정](../pr/assets/issue8946/stage7-20261112/submission-readiness.json)은 이를 반영해 `rebase_and_integration_validation_required`로 기록했다. 원격 push·PR 생성·CI·merge는 실행하지 않았다.
