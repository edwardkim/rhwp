# PR #9198 검토 — 대량 증적 뒤 문서 후행 CI 재사용

[원 PR #9198](https://github.com/edwardkim/rhwp/pull/9198)의 작성자는 `jangster77`이다. 검증한 코드 head `12361e8ebedca625e79b873bf911b9c28f3b30da`, 비교 base `eb9311d9d67ae2ae2b45e28c5a911d9f0e16127d`. 정확한 head의 필수 검사를 확인하고 2026-11-19 21:32:35 KST에 [`b9132685a4`로 devel에 병합](https://github.com/edwardkim/rhwp/commit/b9132685a41d9d335abc5026d3ad25a59effa8e3)했다. 관련 이슈는 [#9197](https://github.com/edwardkim/rhwp/issues/9197)이다.

## 원인과 변경

#9195에서 녹색 코드 커밋 `bd20cae045`의 파일 목록이 315개였으나 커밋 API 첫 응답의 상한인 300개에서 후보 탐색을 중단했다. [#9195의 실제 preflight](https://github.com/edwardkim/rhwp/actions/runs/44513773166/job/135524480932)

CI, CodeQL, Render Diff, Adapter, Proptest와 trusted controller가 커밋 파일 목록의 다음 페이지를 읽고 응답 SHA, 빈 목록과 중복을 확인한다. 3,000개 한도, 조회 실패, 코드 경로가 포함된 이동 또는 녹색 증거가 없는 경우 Full로 처리한다. 변경 범위는 CI 분류와 계약 검사이며 제품 조판과 보호문서 검사는 비해당이다.

## 검증

- #9195의 PR 파일 목록, 커밋, workflow·job·CodeQL 응답으로 다섯 preflight를 읽기 전용 재실행했다. 보정 전에는 모두 `unusable-file-list`였고, 보정 후에는 `100+100+100+15`개를 읽어 다섯 경로 모두 `bd20cae04547db2053a541d259748eff6dc898e0`를 재사용 후보로 선택했다.
- Node 계약 622건, Python 계약 120건, workflow 6개 actionlint 및 공백·문서 링크 검사를 통과했다. 첫 원격 Lint에서 실패한 기준선 병합 테스트는 페이지 조회 모형을 실제 요청에 맞춰 수정한 뒤 최종 head에서 다시 통과했다.
- 정확한 head의 검사 35건 성공, 조건부 건너뜀 6건, 실패 0건. [Full CI](https://github.com/edwardkim/rhwp/actions/runs/44519001690)의 Native Skia, 회귀 A–E, Lint, Frontend package와 필수 Build & Test를 확인했다. [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44519001610) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44519001679) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44519001657) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44519001696) · [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/44519000465)

## 병합 후 확인

CI 실행 정책을 바꾼 PR이므로 이 검토와 작업 기록은 옵션 2로 devel 기반의 별도 문서 PR에 반영한다. 실행 시간 데이터는 대상 44개의 측정값을 갱신해 커밋 `7b12f704a8203da2ee5290a5ba3f25533d1aa8fb`으로 게시했다. [갱신 결과](https://github.com/edwardkim/rhwp/actions/runs/44524856505)

기본 브랜치의 이벤트로 동작하는 trusted controller는 정상 `devel → main` 승격 이후 새 동작을 확인한다. #9195의 읽기 전용 재현과 이 후속 기록 PR의 전체 문서 fast-pass를 각각 독립 근거로 사용한다.
