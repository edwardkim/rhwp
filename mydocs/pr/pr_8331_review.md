# PR #8331 검토 — perf(web_canvas): 회색 JPEG 판정이 디코드한 픽셀을 첫 캔버스 그리기에 넘긴다

## 최종 판정

**머지 보류 — 기존 성능 문서의 조판 차이 처리 결정 대기.** 코드 보정·성능/출력 보존 검증·전체 로컬 gate는 완료했다. 아래 두 성능 문서의 기존 미달 범위에 대한 수용 예외는 아직 승인되지 않았다. [정확한 기존 차이·최종 실측](assets/semanticist21-20261105/performance-control-review.md).

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8331, 작성자 `semanticist21`, Open·non-draft, base `devel`.
- 원 head: `5b36dbd078acd5add18592511682ce268dd66e82`, reviewer `edwardkim`, 지정 시각 `2026-11-05T00:26:46.696813+09:00`; API 재확인 완료.
- 실제 검증 base: `22947f6a90c998e95fa51238d5b907a0ebd37162`. 충돌 simulation: clean.
- [공통 수행 계획](semanticist21_20261105_review_plan.md). CI 참고값: Canvas visual diff: cancelled.
- 기여자의 주장과 독립 측정·메인터너 보정 결과는 별도로 기록한다. 조판 원칙·입력 commit·전쪽 Native/fresh WASM·실행/반례와 로컬 gate는 아래 기록 및 [공통 최종 결과](assets/semanticist21-20261105/final-validation-summary.json)에 연결했다. 원 head의 CI 상태와 새 보정 후보의 실제 검증 결과를 혼동하지 않는다.

## 메인터너 보정과 실측

- 픽셀 인계의 다음 그림 예산 비교가 `u64` 곱셈으로 넘칠 수 있어 `u128`으로 계산한다. 기존 source unit 함수 안에 실제 `take_decoded` 경로의 거대 헤더 반례를 보강했으며 source unit 총량은 4099개로 유지했다.
- 실제 production의 handoff 함수 본체와 PNG header parser를 격리한 음성 대조를 실행했다. PNG 픽셀을 맡긴 뒤 거대 metadata를 읽는 순서에서 원 PR의 `fits_beside`는 픽셀을 남겨 FAIL(exit 101), 보정식은 버려 PASS(exit 0)였다. 별도 거대 pixel buffer를 할당하지 않았다. 함수 대역이나 Python 산술 결과로 제품 경로 실행을 대신하지 않았다. 원래 header 모듈의 자식 경로만 명시해 같은 코드를 컴파일했고 release처럼 overflow 검사를 끈 조건이다. source·명령·해시는 `assets/semanticist21-20261105/negative-handoff-*`에 보존했다.
- 동일한 최적화 WASM·Chrome에서 JPEG 11쪽 첫 그리기 중앙값이 1772.8→1118.6ms였다. 최초 출력 이후 WASM linear memory 최고 크기는 443,482,112→315,555,840바이트였다. 3회 교대 관측의 범위와 전체 출력 동일성은 [성능 검토](assets/semanticist21-20261105/performance-control-review.md)에 구분해 기록했다.
- 원 head Render Diff는 checkout 36분 51초 후 45분 상한에 도달해 Canvas 검사를 시작하지 못했다. 제품 오류로 판정하거나 기여자 재작업을 요청하지 않으며 현재 메인터너 후보의 실제 검증 경로로 처리한다.
- 전17쪽 Native/fresh WASM PNG는 고정 devel과 각각 바이트 동일하다. 독립 Print의 기존 7·15쪽 차이에 대한 수용 예외는 작업지시자 결정 대기다. 최종 runtime의 필수 전체 검증은 완료했다.

## 공통 최종 검증과 원격 조건

최종 runtime `2f44d7d0ab`에서 전체 12,499개·Native 공식 3종 4,064개·집중 21개·Studio 2,720개를 통과했다. Rust lint 묶음·TypeScript·production build·문서 예제·suite/unit 정책도 통과했다. [전체 검증 요약](assets/semanticist21-20261105/final-validation-summary.json), [최종 PNG와 해시](assets/semanticist21-20261105/representative-final-pngs.json), [원 기여 보존](assets/semanticist21-20261105/contribution-ancestry.json).

원격 push·새 통합 PR·merge·후속 댓글은 현재 아직 실행하지 않았다. 승인 뒤 해당 제출 head의 CI와 최신 base의 충돌 여부를 다시 확인한다.
