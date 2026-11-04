# 표시 문자열을 교체한 방향 결과의 재사용 검증

## 수정 전 재현

Native 034의 공개 DocumentCore로 아랍어 두 글자 문서를 만들고 RenderTree의 같은 런을
Latin 두 글자로 바꿨다. 원문 길이와 기존 전진 배열·전체 폭은 보존했다. SVG에서 A의 원점이
B 뒤에 놓여 관계 검사가 실패했다. 빌드 실패 두 번은 재현 실패로 세지 않으며, 실제 컴파일
성공 후 FAIL 로그는 ignored `output/pr-review/semanticist21-20261019/logs/`의
`bidi-projection-before-141-attempt03.log`다.

## 원인과 수정

방향 결과의 유효성을 scalar 수와 전체 폭만으로 판정했다. 같은 수·폭의 다른 문자열이나
다른 내부 전진 배열도 이전 결과를 재사용할 수 있었다. 생산 시 표시 문자열과 논리 전진의
정확한 원본을 함께 보존하고, 소비 시 둘 다 같을 때만 방향 결과를 재사용한다. 다르면 기존
국소 방향 해석으로 되돌린다. 이 원본 데이터는 JSON에 직렬화하지 않는다.

합성 RenderTree의 논리 소유·내용 순서 계약이며 실물 문서의 시각 검증을 대신하지 않는다.
테스트 초안은 output의 `bidi_style_projection_identity.pending.rs`다. 현재 source로 실제
수정 전후 검사를 완료하고 같은 Print 대조군의 Native/fresh WASM을 확인한 뒤 정식으로 추가한다.

## 실행 결과

Native 035 빌드 exit 0, 212.05초다. 같은 합성 계약은 실제 컴파일 성공 후
수정 전 1 FAIL → 수정 후 1 PASS다. `logs/bidi-projection-after-142.log`와
`bidi-projection-after-142-receipt.json`에 실행 명령·동결 제품을 연결했다.

현재 Native 제품으로 기존/새 대조군 90조건도 완료했다. 86조건은 90% 이상이며,
교차 면적 0의 수동 접경 4조건은 래스터 잔여 선 차이로 미달 상태를 그대로 기록했다.
`common-final-local-142-identity-fixed-native/results.json`을 따른다. 기존 정상 61조건과
양식 7조건의 수치·쪽수는 유지했다. 정식 회귀 추가·최종 PR gate와 fresh WASM은 완료 전이다.
