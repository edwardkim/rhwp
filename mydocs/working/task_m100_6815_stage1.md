# #6815 1단계: 문서 merge 뒤 post-merge 재사용 누락 분석

Issue: [#6815](https://github.com/edwardkim/rhwp/issues/6815)

## 범위와 승인

- 기준 `upstream/devel`: `6a193a648dba3df6d5c4cffa0182bc02f3e011ff`.
- 작업지시: 이슈 등록 후 개선. 로컬 구현·검증·단계별 커밋까지 진행하고 push/PR은 별도 승인한다.
- GitHub 운영 O3: CI/CodeQL 재사용의 신뢰 계약 변경이다. 제품 Rust/Studio 소스는 변경하지 않는다.
- 유사 열린 이슈/PR 검색 결과 동일 작업 없음. 선행 #6779는 frontend-only 계약으로 별도 경로다.

## 관찰 및 원인

[CI](https://github.com/edwardkim/rhwp/actions/runs/34030385295)는 14분 13초,
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34030385431)은 17분 3초의 Full 재검증 뒤 성공했다.
두 실행의 거부 사유는 `candidate-full-lane-evidence-unavailable`이다.

- 초기 Full head `964ef10a6`, 문서 merge `359f4d3ea`, 최종 head `42949b966`.
- `classifyReviewOnlyCommit`이 부모 2개를 code로 취급해 collector와 selector가 함께 탐색을 중단한다.
- PR preflight는 이미 current-base merge tree/remerge 검증을 지원하지만 post-merge에는 연결되지 않았다.
- 초기 검사 merge `c0687162210c4277d44a506bdff325ec3e548861` 대비 최종 head의 mydocs 밖 diff는 0이다.
- 초기 CI B/C/D duration 및 merge-tree artifact는 만료되지 않았고 CodeQL 3개 언어 분석도 성공했다.
- 원시 로그·개인 환경 정보는 커밋하지 않는다. 재현 가능한 합성 계보와 실제 run 링크만 기록한다.

## 단계 계획

1. 현재 실패 계보를 테스트로 고정하고 분석과 함께 커밋한다.
2. current-base bridge 후보 탐색과 Git 객체 기반 tree 검증을 구현한다. base/계보/증거를 결합한
   positive/negative 계약을 통과시킨 뒤 코드와 분석 결과를 커밋한다.
3. reusable workflow에 collector, 신뢰 base 코드에 의한 객체 검증, CI 계약 테스트를 연결한다.
   운영 문서를 갱신하고 집중 검증 후 커밋한다.

허용 조건은 same-repository PR, 정확한 base의 bridge 한 개, 성공한 Full candidate, 후보의 실제
검사 merge tree와 최종 tree 사이의 허용된 review 문서 차이뿐이다. 실행 계약 문서는 제외한다.
코드 충돌 해소, stale base, 복수 bridge, 누락/실패/진행 중 증거는 Full로 유지한다.
PR 코드를 checkout/실행하거나 권한을 늘리지 않는다. B/C/D duration 소비와 required context는 유지한다.

## 검증 결과

- 기존 Node verifier/squash 34개, Python workflow 8개 통과(이전 분석 단계).
- 신규 합성 계보 테스트는 Full 성공 + 문서 merge + fast-pass tail에서도 증거 없이는 재사용을
  거부함을 고정한다. 구현 뒤에도 독립 tree 증거 없이는 이 안전 경계를 유지한다.
- 신규 Node 계보 테스트 1개 통과, `git diff --check` exit 0.

## 적용 후 확인과 복구

이번 개선 PR은 실행 정책 변경이므로 Full 검증 대상이다. 이후 별도 제품 PR에서 동일한 문서 merge
계보를 만들고 post-merge CI/CodeQL skip 및 초기 B/C/D duration 재사용을 확인하기 전에는 이슈를 닫지 않는다.
오판 시 이 변경 commit들을 revert하는 독립 PR로 복구한다. 검증 실패를 skip으로 바꾸지 않는다.
