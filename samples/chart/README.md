# 차트 샘플 코퍼스 (refs #1431)

한컴 2022 편집기로 직접 제작한 차트 27종. 종류별 폴더(가로막대형·세로막대형·라인·원형·분산형·기타[주식형]).

- hwp + hwpx 쌍 동거. 정답지 PDF는 `pdf/chart/{종류}/{이름}-2022.pdf`(한컴 2022 출력).
- 모든 샘플이 OOXMLChartContents(DrawingML) + 레거시 Contents 둘 다 보유.
- 입력한 종류/계열/값(ground-truth)을 알고 있어 파서·렌더·writer 검증 정답지로 사용.
- 용도: #1431 Track C(렌더 정합)·Track A(개체 1급화) 회귀 가드.
