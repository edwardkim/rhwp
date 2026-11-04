# 누적 검토의 공개 API 편집 상태

`25607fc28d`에서 모델로 만든 작은 대조군에 공개 편집 API를 적용해 생성한 HWPX다. 마지막 wrap은 공개 WASM API에서 생성했다. 생성 방식과 실제 한컴 작성 원본을 구분한다. 다각형은 9000×4500 HU
열린 3점 다각선에 90도·270도·수평/수직 대칭·30도와 너비 변경을 각각 적용했다. spacing은 세 문단의
가운데 문단에 앞 간격 1500 HU를 적용한 전후다. grow-nested는 한컴 grow3-before의 표를
1×1 부모 셀에 넣은 뒤 둘째 행에 같은 28글자를 삽입한 상태와 무편집 대조군이다.

[검토 계획](../../../mydocs/pr/semanticist21_20261103_review_plan.md)과
[Print 출력 출처](../../../pdf/semanticist21-20261103/print-provenance.md)를 함께 확인한다.
최종 검증 head `c6ad3eea59`에서 Native/fresh WASM 전쪽 비교와 직접 판독을 완료했다. source256의 중첩 편집 파일에는 child의 옛 선언 높이 8502 HU가 남아 있어 재현 입력으로 보존한다. 보정 후 실제 저장본은 검토 assets의 saved에 별도로 보존한다.

forms-before는 본문 라디오 단추 4개이며, forms-edited는 이를 복제한 본문 8개와 셀 8개다. 공개 논리 복사·붙여넣기 API를 사용했다. 글꼴 파일은 내장하지 않았다.

no-lineseg-wrap은 공개 WASM API에서 본문 여섯 글자 뒤에 40000×400 HU 그림을 넣어 마지막 그림 단독 줄을 만든 상태다. 원 devel은 끝 캐럿이 12px, 후보는 같은 문단의 13.3px을 유지한다. Native/fresh WASM raw 92.90%/96.56%이며, 독립 한컴 Print와 그림의 160..164px 밴드를 직접 대조했다. 테스트는 절대 좌표 대신 논리 끝 앵커와 글자 모양 관계를 검사한다.
