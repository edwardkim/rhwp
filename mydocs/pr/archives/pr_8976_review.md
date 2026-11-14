# PR #8976 리뷰 — 다중 TAC 저장 줄 소유와 바깥 여백

## 최종 판정

**메인터너 보정 후 수용 가능 — 원 PR 주장과 최종 통합 후보의 필수 로컬 검증 완료.** 원격 CI·push·병합은 아래 로컬 판정과 구분하며 exact-head CI 확인 후 진행한다.

## 접수와 적용

- 원 PR: [#8976](https://github.com/edwardkim/rhwp/pull/8976), 작성자 `planet6897`, head `e725859e24f9bd0d4631a95ad2d4d37e521193c5`.
- 원 브랜치: `task/8969-owned-tac-line-flow`. reviewer `jangster77`을 API로 확인한 뒤 fetch·체리픽했다.
- 현재 고정 devel: `88170fd75f9d0ce156fe49d79aeabea9bb3a4bcc`. 이전 검증 base `dc3d0ff15e6e7cdb39138c72a5466994dea0e0d4`에서 충돌 없이 리베이스했으며, 기존 증거를 새 head의 실행으로 표시하지 않는다.
- 통합 순서: #8976 → #9064 → #9065 → #9067 → #9068, 원 15개 commit을 `-x`로 누적 적용했다. 충돌은 없었다.
- 통합 후보는 `review/integration-20261114-planet6897-open`이다. 원 fork 이력과 head는 수정하지 않았다.
- [원 SHA·적용 SHA](../assets/planet6897_integration_20261114/integration-provenance.json), [devel/PR·PR 간·누적 병합 시뮬레이션](../assets/planet6897_integration_20261114/merge-simulation.json)을 보존했다.
- #9068은 사용자의 명시적 추가 지시에 따라 최신 green CI head의 5개 커밋을 누적 적용했다. 초기 CI 오류 제외 분류를 현재 범위로 사용하지 않는다.

## 주장과 공통 소비 경로

저장 줄을 혼자 소유한 둘째 이후 TAC 표의 흐름 전진과 앞 글줄·바깥 여백을 보존한다.

`stored_owned_tac_line → is_effective_tac_table/줄 높이 → type/layout의 같은 저장 줄 소유와 바깥 여백 전진`

문서 ID나 PDF 목표 수치로 맞추는 clamp를 추가하지 않았다. 유효한 저장 줄·HWP/HWPX 속성과 원본 제어 주소를 근거로 type/layout이 같은 흐름을 소비하는지 확인했다. 원 PR의 추가 10k 코퍼스 수치는 작성자의 선행 기록이며 이번 통합 head에서 실행한 전수 검사로 표시하지 않는다.

## 검증 입력·독립 Print

- 원문: [`samples/issue8969/36400485-owned-tac-lines.hwpx`](../../../samples/issue8969/36400485-owned-tac-lines.hwpx), SHA-256 `6c74234bd60208a64d04e4b00cb6b6f09aac0fe323734c05e00f6d0b1526d399`.
- 독립 PDF: [`pdf/issue8969/36400485-2020.pdf`](../../../pdf/issue8969/36400485-2020.pdf), 11쪽, SHA-256 `d52da3db4784ea987ee61ea42df8ecd414befbece3a8bdcb9c86e54ecd05ad32`.
- PDF Creator는 Hwp2020 0.0.0.0, Producer는 HancomPDF 1.3.0.550이다. 실제 데스크톱 한컴 2020 Print를 사용했다.
- 웹한글기안기/Cairo PDF로 판정하지 않았다. 유효한 기존 독립 Print를 재사용했으며 기준 PDF·점수·문턱을 바꾸지 않았다.
- canonical Visual Sweep, 96 DPI·compat2022·2px raw를 사용했다. 최소 글꼴에서 실행한 것은 회귀검사이며 Visual Sweep은 정상 설치 글꼴 환경이다.

## 시각 검증과 직접 판독

2·4쪽에서 제목 표와 후속 본문의 위치·줄 소속을 독립 Print와 대조했다. 같은 원본의 기존 #8973 10쪽은 원인이 달라 공통 측정 helper의 비양수 오프셋 누락 한 곳만 추가 보정했다.

- [전쪽 정밀값·입력/producer](../assets/planet6897_integration_20261114/claim-validation.json).
- [같은 입력/PDF의 devel 대비 페이지별 비교](../assets/planet6897_integration_20261114/base-candidate-comparison.json): 같은 PDF 쪽의 2%p 초과 하락은 없다. #9064의 base 3/Print 2쪽, #9067의 base 11/Print 10쪽은 알려진 쪽수 결함으로, 여분 쪽에 임의 PDF 점수를 만들지 않았다. 후보는 독립 PDF 전체 쪽수와 일치한다.
- 최종 코드 `5fa2645186`는 중첩 host·표 이어받기 뒤 간격·선행 공백 줄의 종료 글자 높이를 공통 경로에서 보정했다. 기존 [재사용 감사](../assets/planet6897_integration_20261114/claim-reuse-audit.json)는 이전 단계 기록이다. 현재 후보의 Native/fresh WASM 전체 재출력·보호 검증 결과로 최종 증적을 갱신했다. 과거 결과와 새 실행을 producer별로 구분한다.

- native: [전쪽 TSV](../assets/planet6897_integration_20261114/claims/owned-tac/native/silhouette.tsv) · 2쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/native/raw-002.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/native/pdf-002.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/native/review-002.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/native/overlay-002.png) · 4쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/native/raw-004.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/native/pdf-004.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/native/review-004.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/native/overlay-004.png) · 10쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/native/raw-010.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/native/pdf-010.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/native/review-010.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/native/overlay-010.png)
- wasm: [전쪽 TSV](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/silhouette.tsv) · 2쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/raw-002.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/pdf-002.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/review-002.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/overlay-002.png) · 4쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/raw-004.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/pdf-004.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/review-004.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/overlay-004.png) · 10쪽 [raw](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/raw-010.png)/[Print PNG](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/pdf-010.png)/[review](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/review-010.png)/[overlay](../assets/planet6897_integration_20261114/claims/owned-tac/wasm/overlay-010.png)

## 필수 게이트·보호 검증

[최소 글꼴 전체 회귀](../assets/planet6897_integration_20261114/full-nextest-result.json)는 13,541통과·0실패·45skip이다. [Rust 검증](../assets/planet6897_integration_20261114/rust-validation.json)의 단계별 명령·exit code·요약·로그 SHA에서 fmt·manifest·unit tiers·집중 회귀·전체 회귀·Native Skia lib/PNG/PDF·Native/WASM/all-target Clippy·workspace build·관련 프런트엔드 검사를 확인했다.

[정본 보호 전쪽 검증](../assets/planet6897_integration_20261114/protected-validation.json)은 Native/fresh WASM 각각 18종·1,771쪽, 총3,542행이며 공식 두 자리 표시값 하락0쪽이다. Native 편람 HWPX383쪽 `--handbook-gate`도 통과했다. [상승 버전 수용 영수증](../assets/planet6897_integration_20261114/protected-baseline-update.json)과 정본 manifest는 실제 커밋한 최종 TSV를 가리키며 페이지별 최대값을 혼합하지 않았다.

회귀는 macOS 최소 글꼴 sandbox에서 실행했고 사용자 글꼴을 삭제하거나 비활성화하지 않았다. Visual Sweep은 정상 설치 글꼴 환경이다. Cargo 순차·jobs8·threads8·캡처 총8을 유지했다. fresh WASM은 Mac locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드다. Docker 최적화 통과로 표시하지 않는다. 원 PR의 별도 대량 코퍼스 수치는 작성자의 기록으로 구분한다.

## 리베이스 기능 보존과 실제 소비

현재 base는 `88170fd75f9d0ce156fe49d79aeabea9bb3a4bcc`, 실제 검증한 구현은 `5fa2645186fc8bec4e53074aa6ff0af7ef67f0ca`다. 빌드 당시 parent와 미커밋 patch, 이후 같은 바이트를 커밋한 구현 SHA를 [producer](../assets/planet6897_integration_20261114/final-claim-producer.json)에서 구분한다. 리베이스는 충돌 없이 32개 커밋의 range-diff가 모두 같았다. 반영 #9007의 HWP/HWPX 무늬 대조 2종을 Native/fresh WASM으로 다시 출력해 각각 99.97%/99.99%를 확인했고, 무늬 6건·숨김 23건 집중 회귀도 통과했다. [현재 base 경로 감사](../assets/planet6897_integration_20261114/rebase-base-read-path-audit.json)는 이전 고정 base 결과의 재사용 범위와 실제 입력 바이트·XML 근거를 기록한다. base 쪽수 결함을 가상의 0% 결과로 만들지 않는다.

[실제 소비](../assets/planet6897_integration_20261114/actual-current-validation.json)는 기존 9개 쪽 경계의 화면/인쇄 × Canvas2D/CanvasKit compat/default 54건(새 30건·변경 없는 ws-host/orphan 24건 재사용), SVG 18건(새 10건·재사용 8건)을 구분한다. owned-tac 4·10쪽, page-band 2쪽, title-mark 9쪽은 현재 구현으로 Studio 24건과 SVG 8건을 다시 캡처했다. 실제 요청 backend/profile이 모두 일치하고 fallback은 0건이며, 같은 소비 경로의 이전 PNG 24건과도 바이트가 일치한다. [새 producer와 판독](../assets/planet6897_integration_20261114/actual-consumer-completion/validation.json)에 현재 JS/WASM 응답 해시와 Print review/overlay 8쌍 직접 판독, default 4쌍의 compat PNG 동일성을 구분했다. 이전 5만 한도 fallback·실패 report는 현재 CanvasKit 통과에 합산하지 않는다.

[10만 정책](../assets/planet6897_integration_20261114/budget100k/validation.json)은 문서 전체 77,079작업·5쪽 완료와 실제 CanvasKit 인쇄 두 모드 raw100.00%를 확인한다. 128쪽·글꼴·payload·깊이 제한을 유지했고 합성 작업 예산 회귀 2건은 최소 글꼴 환경에서 통과했다. 독립 Print와 같은 기준·2px 허용을 유지한다.

## Merge 후 contributor PR comment 계획

통합 PR의 exact-head CI 성공·정상 병합을 확인한 뒤 원 PR에 한국어 존댓말로 감사와 통합 PR·merge SHA·CI를 남긴다. 위 원문·주장 페이지와 같은 raw/독립 Print PNG/review/overlay를 merge SHA 고정 raw URL의 **인라인 이미지**로 게시한다. 링크만으로 대신하지 않으며 게시 전에 응답 이미지 SHA를 검증하고 게시 후 API 본문을 재확인한다. 원 기여가 해결한 내용과 필요한 별도 보정을 구분한다. 유효한 데스크톱 Print의 실제 제품·버전과 전쪽 결과를 적고, 병합 전에는 원 PR·연결 이슈를 닫지 않는다. 원 fork branch는 보존한다.

## 대표 review·overlay

### owned-tac 2쪽

![Native review 2쪽](../assets/planet6897_integration_20261114/claims/owned-tac/native/review-002.png)

![Native overlay 2쪽](../assets/planet6897_integration_20261114/claims/owned-tac/native/overlay-002.png)

추가 보정 #8973의 [원인·수정 전 관계 실패1건 → 수정 후 통과1건](../assets/planet6897_integration_20261114/row8973/cause.md)은 원 #8976의 2·4쪽 주장과 구분한다.

## 통합 코드 CI와 현재 캡처

[PR #9074 코드 CI](../assets/planet6897_integration_20261114/code-ci.json)는 `f3446e4bbaae57b70c3da0288a27f019aa897c4f`에서 필요한 5개 workflow 모두 성공했다. 실제 구현 source는 `5fa2645186fc8bec4e53074aa6ff0af7ef67f0ca`이며 이후는 증적·리뷰 기록이다. trailing head CI·병합은 아직 완료로 표시하지 않는다.

현재 Studio canvas2d 4쪽:

![현재 4쪽 review](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p4/canvas2d/review-004.png)

![현재 4쪽 overlay](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p4/canvas2d/overlay-004.png)

현재 Studio canvaskit-compat 4쪽:

![현재 4쪽 review](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p4/canvaskit-compat/review-004.png)

![현재 4쪽 overlay](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p4/canvaskit-compat/overlay-004.png)

현재 Studio canvas2d 10쪽:

![현재 10쪽 review](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p10/canvas2d/review-010.png)

![현재 10쪽 overlay](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p10/canvas2d/overlay-010.png)

현재 Studio canvaskit-compat 10쪽:

![현재 10쪽 review](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p10/canvaskit-compat/review-010.png)

![현재 10쪽 overlay](../assets/planet6897_integration_20261114/actual-consumer-completion/owned-tac-p10/canvaskit-compat/overlay-010.png)
