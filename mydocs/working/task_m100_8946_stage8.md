# #8946 단계 8 — 최종 devel 보존과 제출 검증

검증 기간: 2026-11-12~13 KST. 증적 디렉터리 날짜는 실행을 시작한 날짜다.

이 기록은 [단계 7](task_m100_8946_stage7.md) 뒤 추가된 #9017을 반영하고 공통 계약의 병합 충돌을 정리한 최종 실행을 다룬다. #8946의 명시적 Canvas2D 호스트 원본 프로그램 선택·환경 갱신·수명 처리 범위를 유지한다. 수용 조건별 구현·회귀의 연결은 [단계 5](task_m100_8946_stage5.md), 독립 프로그램 공급 근거는 [단계 6](task_m100_8946_stage6.md)을 따른다.

## 소스와 산출물

- 기준 devel: `a85d3265dd10b316d1a0ae029ca3f67245b38b77`.
- 최종 코드·회귀 검증 head: `8af32b9d991ee6b6c7bf082aaf6cc6caac1b86ab`.
- 후보 Native·최적화 WASM 생산 head: `22a994602442159ed6b146098733eec255231d9c`. 공통 커밋 수용 전후의 src·Studio·Cargo·scripts Git object는 모두 동일하다. 추가 회귀를 포함한 전체 검사는 수용 후 head에서 실행했다.
- [생산자 원장](../pr/assets/issue8946/stage8-20261112/producer.json)과 [공통 계약 이력](../pr/assets/issue8946/stage8-20261112/shared-contract-integrated.json)에 실제 바이너리·WASM·프로덕션 사이트의 해시와 소스 동일성 근거를 기록했다.
- 이슈 본문은 재확인 시 `2026-11-11T10:17:17Z` 수정본을 유지했다. 조판 참여와 메트릭 교체를 분리하며, 옵션 생략·false의 기존 동작, 명시 환경·내장 우선순위·저장 줄 재사용·편집 후 재조판 계약을 유지한다.

## 전쪽 시각 검증과 정본 보호

각 입력은 동일 글꼴 환경의 독립 한컴 Print PDF와 763쪽 전체를 대조했다. 외부 그림 세 영역을 모두 포함한다.

| 입력 | Native 최저 raw | fresh WASM 최저 raw | 실제 Host 최저 raw |
| --- | ---: | ---: | ---: |
| HWPX | 96.35% | 99.56% | 99.36% |
| HWP3 | 98.77% | 99.55% | 99.35% |

전쪽 90% 미달·누락·쪽수 불일치가 없다. [최신 base/후보 대조](../pr/assets/issue8946/stage8-20261112/devel-full-comparison.json)의 네 조합은 최대 하락 0%p다. HWPX는 기준 Native/WASM의 각 18쪽 미달을 해소하고, HWP3는 기존 출력을 보존했다.

[정본 보호](../pr/assets/issue8946/stage8-20261112/protected-comparison.json)는 18개 입력 × Native/WASM **3,542쪽**, 누락 0·공식 보호값 하락 0으로 통과했다. [36개 전쪽 PNG·tree 묶음](../pr/assets/issue8946/stage8-20261112/protected-output-parity.json)도 단계 7과 모두 동일하다. 필수 [편람 Native gate](../pr/assets/issue8946/stage8-20261112/handbook-gate.json) 383쪽을 통과했다. 보호 baseline과 기대값은 변경하지 않았다.

후보의 두 입력 Native/WASM 및 실제 Host 전쪽 PNG·tree가 단계 7과 동일함을 확인했다. 이 동일성이 성립하는 범위에서 기존 확대 판독을 연결하며, 새로운 공통 딸기 표본과 HTML 붙여넣기 영역은 아래처럼 직접 판독했다. 100% raw는 RGB 완전 동일을 뜻하지 않는다.

## 실제 화면·수명·편집

- 원본 HWPX·그림 포함 HWPX·HWP3 × 공급자 없음·옵션 생략·false의 **9조건**, base/head 각 763쪽 PNG·tree 동일.
- 같은 viewport의 실제 DOM 합성 첫 화면 9조건 모두 RGB 동일. 세 외부 그림의 decode 성공을 확인했다. 기본 canvas만 캡처한 이전 진단은 첫 화면 증거로 사용하지 않았다.
- 명시 활성 첫 화면은 독립 Print와 HWPX 99.12%, HWP3 99.17%다. 전쪽 비교와 별도로 판정했다.
- 실제 파일 SHA·faceIndex·indexed program ID가 환경 선택과 메트릭 준비에 연결되고 첫 paint 전에 준비됐다. Canvas2D/CanvasKit E2E 및 조판 환경·수명 97조건을 통과했다.
- 두 입력의 실제 삽입·삭제·저장·재열기에서 내용·순서·소유·포함·겹침·복원 관계 각 8개를 통과했다. 빈 후속 문단의 정당한 재조판 차이를 내용 누락이나 임의 좌표 고정으로 처리하지 않았다.

근거는 [기본 모드](../pr/assets/issue8946/stage8-20261112/comparison-summary.json), [합성 첫 화면](../pr/assets/issue8946/stage8-20261112/initial-screen-comparison.json), [편집 관계](../pr/assets/issue8946/stage8-20261112/edit-verification.json), [HWPX 준비 순서](../pr/assets/issue8946/stage8-20261112/host-hwpx-first-paint.json)에 보존했다.

## 새 devel #9017 기능 보존

#8982/#8983의 표 border와 첫 서식 전 평문 보존, 인용된 HTML 속성 경계 보정은 관련 Native 계약 및 실제 Studio에서 확인했다. 기본 붙여넣기·표·혼합·본문/셀 서식 커서·유사 속성의 6조건을 base/head에서 각각 실행하고 HWP/HWPX 두 형식으로 두 번씩 저장·재열기했다. 실제 수신 WASM body SHA도 각 패키지와 일치했다.

원 승인 12개 표본과 #7521 대조를 합친 13개 입력 × base/head × Native/WASM **52쪽**은 모두 raw 100.00%이며 같은 backend의 base/head PNG가 동일하다. 결합 표본의 앞 평문·굵게·표 테두리·셀 내용·뒤 본문을 독립 Print와 직접 확인했다. [보존 원장](../pr/assets/issue8946/stage8-20261112/incoming-preservation.json)과 [원 승인 근거](../pr/assets/bongseop-kim-20261112/README.md)를 구분해 연결한다.

## #9012와 공유하는 계약

제출 후보와 리뷰어 미지정 #9012의 최초 시뮬레이션은 `legacy_font_advances.rs`의 공통 helper 추가에서 충돌했다. #9012의 원 커밋 `4e66e76981`은 이미 #8946에 포함된 딸기 굵기 전진 계약만 구현한다. 이 커밋·기존 회귀·정식 증적을 이력에 수용했으며, #8946 제품 소스는 전후 동일하다. #9012의 연결 그림 인쇄 처리는 수용하지 않았다.

원 딸기 표본은 현재 Native/fresh WASM 전체 1쪽 각각 raw 100.00%다. 독립 Print와 같은 배율에서 12/18/24pt 보통·굵게 여섯 행의 글자 형태·전진·공백·중앙 정렬·행 간격을 직접 확인한 뒤 기존 관계형 회귀를 수용했다. 전체 회귀에서도 두 검사가 실행돼 통과했다. 원 커밋의 기존 추적 증적은 그대로 보존했다. 이후 #9012의 로그 정리 변경을 먼저 적용하자 #9017의 다른 로그와 잘못된 rename/delete 추론이 발생해, 해당 정리는 원 PR에 남겼다. 새 비공개 실행 로그를 추적 경로에 추가하지 않았다.

정리 후 `git merge-tree --write-tree`로 #9012 head `71c002495603a34af43f238dd2c98354d942a589`와 충돌 없이 병합됨을 확인했다. 다른 PR의 branch·리뷰·원격 상태는 변경하지 않았다.

## 기준 WASM 캐시 재사용의 발견과 재검증

최초 기준 WASM 실행은 Cargo가 0.52초에 기존 산출물을 재사용했다. 실제 SHA가 후보와 같고 후보 전용 `hwpdocument_prepareNativeOtfFontFaces` export까지 포함돼 기준 출력으로 사용할 수 없었다. [무효 범위](../pr/assets/issue8946/stage8-20261112/base-wasm-invalidated.json)에 해당 WASM·Studio·기본 모드 결과를 기록하고 최종 판정에서 제외했다. 새로 컴파일한 기준 Native와 후보 검증은 유효하다.

진행 중인 진단은 정상 종료하도록 두었다. 사용 중이 아닌 rhwp WASM release 캐시만 정리한 뒤 기준 소스를 별도 pkg에 재컴파일하고 별도 프로덕션 사이트를 만들었다. 기준 SHA `d5c348428dca32f12a402a55e5e03c60e0cfa9694257d93034c599b07a96e08b`, 후보 SHA `a1596ff11c174bdc52da36f7951d7b3e3abfee5449ee612fd9c85b1e0756dbad`를 구분했다. 기준에는 후보 전용 export가 없고, 실제 #9017 붙여넣기 동작은 유지됨을 확인했다.

무효화한 모든 기준 WASM·브라우저 항목을 다시 실행했다. [소스별 WASM 확인](../pr/assets/issue8946/stage8-20261112/base-wasm-source-check.json), [전쪽 측정 소비 해시](../pr/assets/issue8946/stage8-20261112/source-specific-wasm-receipts.json), Studio 수신 body 및 기본/첫 화면 runtime 자원 해시를 최종 판정의 필수 조건으로 삼았다. 단계 7의 기준·후보 WASM 해시는 서로 달랐으며 이번 무효 실행과 구분한다.

## 필수 로컬 게이트와 제출 상태

[실행 원장](../pr/assets/issue8946/stage8-20261112/local-verification-results.json)에 실제 종료값·로그 SHA를 기록했다.
Rust 전체 13,341 PASS(기존 skip 45), Native Skia library 4,034 PASS(기존 ignore 12), 그림 대체 표시 2·직접 PDF 36·HFT 공통 22 PASS;
fmt·Native/WASM/workspace Clippy·workspace build·suite/unit-tier 정책; Studio 2,789 PASS(기존 skip 2)·TypeScript·frontend 계약·E2E를 완료했다.
전체·집중 Rust 회귀는 기존 최소 글꼴 격리 프로필로 실행했다. 사용자 제공 및 독립 Print에서 준비한 글꼴 프로그램은 로컬 시각 검증에만 사용했다.

[제출 준비 판정](../pr/assets/issue8946/stage8-20261112/submission-readiness.json)은 최종 로컬 결과와 원격 재확인을 구분한다. 이 기록 자체가 원격 CI 또는 merge 승인을 뜻하지 않는다.
