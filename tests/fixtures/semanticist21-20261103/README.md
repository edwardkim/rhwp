# 누적 검토의 공개 API 편집 상태

`25607fc28d`의 Native에서 공개 편집 API를 호출해 생성한 HWPX다. 다각형은 9000×4500 HU
열린 3점 다각선에 90도·270도·수평/수직 대칭·30도와 너비 변경을 각각 적용했다. spacing은 세 문단의
가운데 문단에 앞 간격 1500 HU를 적용한 전후다. grow-nested는 한컴 grow3-before의 표를
1×1 부모 셀에 넣은 뒤 둘째 행에 같은 28글자를 삽입한 상태와 무편집 대조군이다.

[검토 계획](../../../mydocs/pr/semanticist21_20261103_review_plan.md)과
[Print 출력 출처](../../../pdf/semanticist21-20261103/print-provenance.json)를 함께 확인한다.
새 독립 Print의 Native/fresh WASM 비교가 끝나기 전에는 이 입력의 시각 검증을 완료로 판정하지 않는다.

forms-before는 본문 라디오 단추 4개이며, forms-edited는 이를 복제한 본문 8개와 셀 8개다. 공개 논리 복사·붙여넣기 API를 사용했다. 글꼴 파일은 내장하지 않았다.
