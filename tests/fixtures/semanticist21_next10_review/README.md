# semanticist21 두 번째 10건의 실제 편집·저장 입력

2026-11-05 Native 제품 API로 공개된 기여 fixture를 편집하고 저장한 문서다. 생성 source는 `0838c170b709cde5288eb53cff097506398acdfd`이며 그 뒤 문서 전용 commit은 제품 동작을 바꾸지 않는다. 원 기여 fixture는 원래 경로에 보존한다.

셀 균등화는 원 한컴 저장본의 격자와 대조했고, 미주 번호는 한컴의 continue/restart 저장 번호와 대조했다. 계산식은 F1=7 및 A1=100 편집을 실행했다. LN(0)과 ROUND(1/3,7)*1000000의 결과도 한컴 `hancom.hwp`의 C7·D8과 비교해 `잘못된 계산식`·`333,333`을 확인했다. 문단 머리는 본문 편집과 머리 끄기를 실행했고, 각주 모양은 빈 속성 객체를 적용해 기존 모양을 유지하며 번호 코드 동기화를 실행했다. `outline-id2`는 원 #8490 test의 실제 fixture 생성·구역 ID 2 적용 경로다.

이 파일들을 같은 bytes로 한컴 Print에 전달해 독립 PDF를 얻고 Native/fresh WASM 전쪽·저장 재열기를 검증한다. 생성 자체는 시각 통과가 아니다. 최종 결과는 `mydocs/pr/assets/semanticist21-20261105-next10/`와 개별 PR 검토 기록에 연결한다.


`footnote-shape-corrected.hwpx`는 공통 LineType2 수정 후 실제 Native CLI가 저장한 DASH 보존 대조군이다. `footnote-shape-configured.hwpx`는 source `6337fe4d10`의 최적화 fresh WASM에서 실제 각주 모양 API로 digit·일반 번호·페이지 아래·빨간 구분선을 설정하고 조회한 뒤 저장했다. 독립 Print의 출처와 양쪽 전쪽 측정은 [검증 기록](../../../mydocs/pr/assets/semanticist21-20261105-next10/validation.md)에 있다. 초기 22개 저장본은 이전 source `0838c170b7`의 실제 API 생성본이라는 provenance를 유지하며 최종 source 생성본으로 바꾸어 부르지 않는다.
