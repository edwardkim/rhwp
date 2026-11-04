# planet6897 통합 검토 — 현재 증적과 회귀 주석 정리

2026-11-05 KST. 대상은 이미 고정한 원 PR 39개다. 새 PR 접수나 원격 변경은 하지 않는다.
통합 브랜치는 `review/integration-20261103-planet6897-open`이며 PR 생성은 작업지시자의
승인을 기다린다.

## 이번 단계의 완료 범위

최신 제품 코드는 `a7e5ce7112ea8b227c317a9cbfb9d6fed0b00973`, 새 base는
`22947f6a90c998e95fa51238d5b907a0ebd37162`다. 전쪽 시각 측정 producer
`4b149b6cdfc514b9858ed644be2cf8120c731cd4`와 당시 base
`7fdd5441ff1e77c083818e920a48089f294de35e`를 최신 제품의 재출력으로 표시하지 않는다.
충돌 없는 리베이스의 전쪽 재측정 생략 지시에 따라 새 제품의 영향 경로를 별도로 검증한다.

- 개별 리뷰 35개의 입력·쪽·실측 수치와 TSV/PNG를 같은 producer로 갱신했다. 실제 적용
  커밋의 현 브랜치 포함 여부도 확인했다. 완료된 검증과 대기 중인 검증을 분리했다.
- 출력이 바뀐 대표 페이지 9건을 Native/fresh WASM 각각 review와 overlay로 직접 판독했다.
  이번 추가 18건과 앞서 판독한 8건을 합한 26건은 PNG SHA-256·관측·남은 차이를 보존했다.
  기록이 없는 쪽의 직접 판독까지 완료했다고 주장하지 않는다.
- 전체 대표 선택 93개 입력·쪽 조합은 각 backend의 source PNG·입력·PDF·제품 해시가
  대응한다. 파일 대응 검사와 사람의 직접 판독은 다른 근거다.
- 회귀 파일 40개 첫머리에 실제 전쪽 producer·원본별 최저값·주장 쪽 값·정식 경로를
  반영했다. 주석을 제외한 본문 SHA가 직전 커밋과 모두 같고 `rustfmt --check`도 통과했다.
  assertion과 helper 변경이 없으므로 주석만을 이유로 전체 회귀를 반복하지 않는다.
- 최신 리베이스의 Native 집중 대조는 기존 후보의 PR #8498 입력 5쪽과 새 base의
  PR #8585 입력·정상 대조군 PR #8122 입력 각 1쪽, 합계 7쪽이 모두 정확히 같은 raw다.

[준비 상태와 실제 성공 검증 47개](../pr/assets/planet6897_20261103/review-preparation-a7e5ce7112.json),
[직접 판독 기록](../pr/assets/planet6897_20261103/direct-reading-current.json),
[주석 외 본문 불변](../pr/assets/planet6897_20261103/regression-header-body-equivalence.json),
[최신 Native 집중 대조](../pr/assets/planet6897_20261103/rebase_scope_22947f6a90/native-base-comparison.json)를
정식 증거로 보존한다. 실제 전체 Rust 12,561개 통과와 34개 Rust gate, fresh WASM 실행 결과는
이전 단계의 제품 검증 기록을 재사용한다.

## 남은 완료 조건

기존 base WASM의 80개 입력 대조와 새 base WASM의 집중 대조가 실행 중이다. 전자는 base의
쪽수 오류·파싱 거부와 실제 공통 쪽의 수치 하락을 구분해야 하며, wrapper 종료 코드만으로
통과를 선언하지 않는다. Native/fresh WASM 각각 0.1%p 초과 하락 금지와 PR #8389·#8397의
원 TSV 표시값 무하락은 별도 조건이다.

수동 작성한 PR #8477·#8540·#8546·#8547 리뷰의 최신 기록, 남은 직접 판독과 증적 재사용
근거, PR 템플릿 본문을 마무리해야 한다. 공통 검증이 남아 있으므로 아직 최종 승인·PR 준비
완료로 판정하지 않는다. 원격 push·PR 생성·댓글·승인·머지·닫기는 수행하지 않았다.
