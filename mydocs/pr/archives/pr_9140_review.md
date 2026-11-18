# PR #9140 리뷰 — planet6897 표·캡션·SQUEEZE·HFT 전진 통합

## 최종 판정

**승인 — 원 PR 네 건의 주장 범위와 필요한 메인터너 보완을 검증했습니다.** 정확한 code head의 Full CI와 필수 로컬 검증을 완료했습니다. 문서 전용 trailing head의 실제 CI·mergeability를 확인한 뒤 승인된 병합·후속 처리를 진행합니다.

검토 갱신: 2026-11-16 23:15:03 KST.

## 접수 정보

- [통합 PR #9140](https://github.com/edwardkim/rhwp/pull/9140), 작성자 `jangster77`, 대상 `devel`, branch `review/integration-20261116-planet6897`. 통합 PR reviewer는 지정하지 않았습니다.
- code candidate `c6996300203fec553a57b0c16b2e48329ebc083a`, base `27ba0333b358758ac71e976849a663e8a86fe6ff`. 초기 base `ae9277dda0ca17be279cba89c4deea74fb75dcae` 뒤 두 번 리베이스했습니다. 충돌 없음과 별도로 일반 소비 경로 동일성 및 유입 기능 검증을 기록했습니다.
- 원 PR [#9078](https://github.com/edwardkim/rhwp/pull/9078)·[#9080](https://github.com/edwardkim/rhwp/pull/9080)·[#9086](https://github.com/edwardkim/rhwp/pull/9086)·[#9098](https://github.com/edwardkim/rhwp/pull/9098), 작성자 `planet6897`. 중복 공유 커밋을 제외한 8개 기여 SHA를 `cherry-pick -x`로 보존했습니다. 원 contributor branch는 수정하지 않았습니다.
- 해결 대상 이슈 [#9073](https://github.com/edwardkim/rhwp/issues/9073)·[#9079](https://github.com/edwardkim/rhwp/issues/9079)·[#9085](https://github.com/edwardkim/rhwp/issues/9085)·[#9087](https://github.com/edwardkim/rhwp/issues/9087). [접수·기여 계보](../assets/planet6897_20261116/intake-provenance.json).

## 변경과 조판 원칙 검토

단독 저장 줄을 소유한 표의 전진, 글자처럼 취급 도형의 옆 캡션 예약·재조판, 저장 줄 없는 SQUEEZE 문단, 신명 순명조 HFT Type3 라틴·숫자 및 굵기 전진을 통합했습니다. 원본 속성→측정→글줄·행·표→paint가 같은 계약을 사용하도록 검토했습니다. 문서명·페이지별 임의 이동이나 통과를 위한 기대값·문턱 완화는 추가하지 않았습니다.

메인터너 보완은 CJK 결측 기호의 Unicode full-width 전진과 원본 한양중고딕 `H2GTRM.TTF` 별칭 연결입니다. 실제 메트릭·반각·결합 문자는 보존했으며 원본 TTF→TTC→OTF/CFF를 일반 대체보다 먼저 선택합니다. 설치 글꼴이 없는 환경에서도 관계형 합성 프로그램 검사가 실행됩니다. 생성했던 기호 하나용 폰트는 제거했고 비공개 원본 바이너리를 Git에 넣지 않았습니다.

[선행 원 PR 분석](../assets/planet6897_20261116/intake-analysis.md) · [기호·원본 TTF 근거](../assets/planet6897_20261116/font-order/current-selection/README.md). CJK 독립 원점 검사는 전 2FAIL→후 2PASS이며, 해당 3쪽 양쪽 99.66%·독립 PDF 판독으로 ±2px 조건을 충족했습니다. 다른 검사는 소유·순서·내용 보존 관계를 보호합니다.

## 검증 입력과 로컬 결과

기준은 원 PR의 **데스크톱 한컴 2022 12.0.0.4547 Print**입니다. 웹한글기안기 PDF를 판정에 사용하지 않았습니다. [원본·PDF 해시·출처](../assets/planet6897_20261116/claim-input-audit.json), [실제 사용 입력 57개 커밋 확인](../assets/planet6897_20261116/final/committed-input-audit.json).

- 주장 6개 입력 Native/fresh WASM 각 18쪽의 독립 PDF 쪽수 일치·raw 90% 이상·직접 판독을 완료했습니다. 초기 base와 후보 36개 페이지 대응에서 2%p 초과 하락은 0입니다. base의 서식·SQUEEZE 추가 2쪽은 원 결함이며 독립 PDF에 없는 쪽을 채점하거나 base 전쪽 통과로 표현하지 않았습니다. [전체 TSV·생산자](../assets/planet6897_20261116/claim-evidence-index.json) · [base/head 대조](../assets/planet6897_20261116/final/base-head-comparison.json).
- 검증된 macOS 최소 글꼴 sandbox에서 전체 nextest **13,634 PASS·0 FAIL·45 skip**, 821.026초. 실제 실행 head `2e3e6698d198cf09189105746a2ed8e988ac4b0d`와 현재 head를 구분합니다. fmt·Native/WASM/all-target Clippy·workspace build·suite 정책 PASS, Native Skia 3종 **4,072 PASS**. [원장](../assets/planet6897_20261116/final/rust-validation.json).
- 리베이스 후 일반 렌더·Cargo 읽기 경로는 불변이며 유입 #9136 dialog·#9132 선택적 GPU를 별도로 확인했습니다. 최신 집중 **30 PASS**, Native GPU **5 PASS**, suite/unit 정책 PASS. Studio **2,805 PASS·0 FAIL·2 skip**, TypeScript·dialog viewport·autosave E2E PASS. [리베이스](../assets/planet6897_20261116/final/rebase-validation.json) · [프런트엔드](../assets/planet6897_20261116/final/frontend-validation.json).
- 일반 원 글꼴 환경에서 정본 보호 18문서 Native/fresh WASM 각 1,771쪽, **3,542행·하락 0·상승 0**. 보호 baseline은 그대로입니다. 편람 HWPX Native 383쪽 `--handbook-gate` PASS. [원문 보호 TSV와 판정](../assets/planet6897_20261116/final/protected-validation.json).
- 최초 제출 뒤 필수 항목 재대조에서 공개 공통 4문서 기록 누락을 발견해 같은 코드에서 보완했습니다. 8건 모두 공식 raw **100.00%**, 하락·누락 0이며 각각 review·overlay를 판독했습니다. 제출 전 완료로 소급하지 않습니다. [추가 공통·SVG 원장](../assets/planet6897_20261116/final/supplemental/README.md).
- SVG 대표 HFT 3쪽 Native **99.68%** / fresh WASM **98.44%**, nested 5쪽 **99.99% / 100.00%**. 기호 전진·표·주석·내용 소유를 판독했습니다. 대표 4건의 SVG 소비 경로 확인이며 전쪽 SVG 수용으로 일반화하지 않습니다.
- 실제 Studio 대표 7쪽×screen/print×Canvas2D/CK compat/default **42상태**에서 backend·자원·프로필·내용 소유를 확인했습니다. CK는 software surface이며, 초기 nested 준비 fallback을 성공으로 세지 않고 설치된 전체 원형 글꼴 공급으로 해당 6상태를 보완했습니다. report-only 픽셀 parity **28 FAIL**은 보존했고 픽셀 동일성 PASS로 표시하지 않았습니다. [소비 범위·원 제약](../assets/planet6897_20261116/final/actual/validation.json).
- [실제 편집·HWP/HWPX 반복 저장](../assets/planet6897_20261116/final/repeated-save.json)과 [Type3 폭 2종 각 100자 독립 재감사](../assets/planet6897_20261116/final/source-type3-width-oracle-audit.json)를 완료했습니다. Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화가 아닙니다. root pkg·Studio public·실제 초기화 JS/WASM hash를 확인했습니다.

## Exact-head GitHub Actions

`c6996300203fec553a57b0c16b2e48329ebc083a`의 [CI](https://github.com/edwardkim/rhwp/actions/runs/43877624194)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/43877624197)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/43877624211)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/43877624254)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/43877624287) 모두 success입니다. Native Skia·Lint·frontend package와 Archive A–E 실제 실행을 확인했습니다. 기본 회귀는 **13,465 PASS·0 FAIL·45 skip**이며 macOS 로컬 수와 다릅니다. trusted policy는 `mode=full;rfp=0`입니다. classifier 비대상 skip을 실제 실행 성공으로 세지 않습니다. [job·head·집계·정책](../assets/planet6897_20261116/final/code-ci.json).

이 녹색 code candidate 뒤에 source·test·workflow·baseline 변경 없이 본 review·오늘할일·추가 증적만 trailing으로 반영합니다. 최신 head의 실제 preflight·required aggregate·mergeability 확인은 별도 완료 조건입니다.

## 시각 증적과 남은 차이

[개별 #9078](pr_9078_review.md) · [#9080](pr_9080_review.md) · [#9086](pr_9086_review.md) · [#9098](pr_9098_review.md)에 해당 원 PR 주장 쪽의 Native/WASM raw·review·overlay를 보이게 연결했습니다. [직접 판독](../assets/planet6897_20261116/final/direct-reading.json), [대표 이미지 바이트 동일성](../assets/planet6897_20261116/final/main-representative-reuse-audit.json), [게시 이미지 실제 표시](../assets/planet6897_20261116/final/published-image-audit-code-head.json)를 구분합니다.

HFT 탐침 전쪽 최저는 일반 Native 97.42%/WASM 97.39%, 굵게 97.60%/97.51%로 글리프·래스터 잔차가 남습니다. 원 Type3 폭·줄 소유·프로그램 선택의 주장 범위를 충족하는 것으로 수용하며 모든 픽셀 100%라고 하지 않습니다. 문서 전쪽 수용은 canonical PNG raw로 판정했고 실제 backend 간 엄격 픽셀 동일성과 구분했습니다. 원 PR의 외부 대규모 코퍼스를 이번 실측이라고 인용하지 않습니다.

## Merge 후 contributor PR comment 계획

문서 trailing의 exact-head CI·정상 merge·devel asset 반영을 확인한 뒤 원 4개 이슈의 completed 상태를 확인하고 실제 해결 근거를 존댓말로 게시합니다. 이어 원 PR별 개별 review의 계획대로 **그 PR이 주장한 같은 입력·같은 쪽의 raw/review/overlay**를 최종 merge SHA URL의 Markdown 이미지로 게시하고 supersede close합니다. 직접 merge라고 표현하거나 다른 PR의 이미지를 쓰지 않습니다. API 본문과 원격 이미지 바이트·실제 표시를 재확인합니다.

[후속 정본](../../manual/pr_review/post_merge.md)에 따라 duration 자동 갱신 성공 또는 증거 부족 보류, 최종 devel 동기화, 댓글·이슈·원 PR 상태, 소유 local/upstream branch·worktree·output 전체·사용 종료한 shared debug 정리를 순서대로 확인합니다. 병합 후 제품 검증 workflow를 새로 시작하지 않으며 contributor fork·개인 글꼴·원본 PDF와 다른 작업은 보존합니다.
