# #6815 2단계: bridge 후보와 독립 tree 증거

Issue: [#6815](https://github.com/edwardkim/rhwp/issues/6815)

## 구현 계획

- 1단계 `feea62e0e`의 재현 계약을 유지한다.
- current-base가 두 번째 부모인 bridge 한 개만 후보 탐색에서 통과한다. 일반 merge나 계보 단절은
  통과시키지 않는다. 단순 candidate 발견은 재사용 승인이 아니다.
- 후보 workflow의 immutable merge-tree artifact를 Git commit/tree와 대조한다. 실제 검사 base가
  최종 base와 같고, 최종 tree와의 차이가 실행 계약을 제외한 mydocs뿐이어야 한다.
- helper는 Git 객체만 읽으며 checkout, shell, PR 코드 실행을 하지 않는다. SHA 입력 검증, 시간/출력
  제한과 Git 외부 diff/textconv 비활성화를 적용한다.
- source/head/base/bridge/tested tree/최종 merge SHA가 모두 일치하는 증거를 evaluator에서 요구한다.
- collector는 아직 바꾸지 않는다. workflow 연결 전에는 기존 Full fallback이 유지된다.

## 검증 결과

- Node verifier/squash/bridge **71개 통과**, diff check exit 0.
- SHA별 증거 불일치, fork, 실패/진행 중 candidate와 최종 head, artifact 누락, stale base,
  복수 bridge, 계보 단절 및 코드/실행 정책 변경은 거부했다.
- 임시 Git 저장소에서 실제 commit/tree 객체를 생성해 문서 변경/동일 tree 수용과
  source/test/Cargo/PDF/실행 계약 문서 변경 거부를 검증했다. 테스트 종료 때 해당 임시 저장소만 제거한다.
- 실제 #6813 객체로 helper를 실행해 초기 검사 tree `d205ad5c53988e3533b91c65fa81ab1930141b23`
  대비 최종 tree `6f613d232cb96bcee2ace0d6785eadd20816ff77`의 허용 문서 차이를 확인했다.
- 아직 GitHub workflow에 collector를 연결하지 않았으므로 원격 skip 성공이라고 기록하지 않는다.
