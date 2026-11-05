# PR #8622 반영 전 확대 판독

2026-11-05 KST, source `6337fe4d10`. [이미지와 입력/쪽·crop·해시](image-index.tsv), [전쪽 TSV](summary.json)를 함께 확인했다. 최초 oversized 글꼴 팩의 그림을 재사용하지 않고 문서별 한도 안의 글꼴로 새로 출력했다.

| 실제 대조 그림 | 직접 확인한 내용 |
| --- | --- |
| [테두리 전체](../../images/pre8622/borders-review.png)·[원형 점선 확대](../../images/pre8622/borders-circle-detail.png) | 기존 사각 점선/실선이 원형 점선·단일/이중 물결선으로 복원됐다. 본문·표 밖 점유와 기존 3D 비표시 계약을 유지한다. 96 DPI의 채움 가장자리 차이는 남으며 물리 반복 구간은 별도 2020/2024 16굵기 PDF로 대조했다. |
| [글자 획](../../images/pre8622/outline-review.png)·[600 DPI 여섯 종류](../../images/pre8622/outline-native-600dpi-detail.png) | 기존 굵은 채움이 제거되고 투명한 내부·실선·점선·파선·일점쇄선·이점쇄선이 보인다. 96 DPI Native의 장치 획 하한과 AA 차이를 600 DPI의 실제 glyph/pen 대조와 구분한다. |
| [원 각주](../../images/pre8622/footnote-review.png)·[저장 각주](../../images/pre8622/footnote-saved-review.png)·[설정 변경](../../images/pre8622/footnote-configured-review.png) | II 머리와 선두 참조가 겹치지 않으며 실제 본문 줄 아래 각주, 저장한 DASH와 붉은 페이지 아래 구분선이 독립 Print와 대응한다. 마지막 쪽의 유효한 빈 본문 줄도 점유한다. |
| [바탕쪽](../../images/pre8622/master-review.png) | 잘못된 용지 (0,0) 대신 본문 프레임 원점에 표시한다. 독립 PDF 자체에 있는 첫 본문과의 의도된 겹침을 별도 결함으로 없애지 않는다. 머리말과 본문·참조 순서를 보존한다. |
| [셀 높이 적용](../../images/pre8622/equal-height-edited-review.png) | 같은 원본의 적용 전/후를 구분해 병합 밖 셀과 블록의 새 격자, 표 하단·뒤 설명문을 확인했다. independent 패널은 적용 후 저장본의 Print다. |
| [계산식](../../images/pre8622/formula-review.png) | LN/ROUND를 포함한 저장 결과의 숫자·쉼표·소수 자리·셀 내부 포함·주변 본문을 대조했다. base의 저장 결과 표시를 실제 재계산 성공의 증거로 쓰지 않는다. 실제 API 37개 기대값 검사는 별도로 실행했다. |
| [개요 ID](../../images/pre8622/outline-id-review.png) | 가)제목의 머리와 본문 간격을 대조했다. 원자적 오류 처리는 별도 실제 API/회귀 결과로 확인한다. |
| [문단 머리](../../images/pre8622/paragraph-head-review.png) | bullet·number·outline·그림 글머리표와 후속 줄, 일반/해제 문단을 대조해 텍스트 순서와 줄 내용을 확인했다. 저장 bit는 binary/serializer 검사로 확인한다. |
| [첫 글자 장식](../../images/pre8622/drop-cap-review.png) | 기존 大와 뒤 텍스트의 표시를 보존한다. API는 DoubleLine/TripleLine/Margin 메타데이터의 양쪽 저장 보존 범위이며 새 장식 생성 UI·자동 재배치를 구현한 것으로 확대하지 않는다. |

수치 90% 통과와 직접 판독을 별도로 완료했다. 이 기록은 최신 #8622 반영 후보의 출력 검증을 대체하지 않는다.
