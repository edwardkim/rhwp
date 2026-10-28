# #7998 표준 실선의 행·열 상속

`generate.py`는 공개 stage20 `../compound-borders/compound-solid.pdf`를 기반으로 부모/자식
pen을 표준 0.15mm의 point 값(`0.4251968503937008`)으로 작성한다. 한 stroke의 여러 독립
축은 그대로다. 삽입을 검증할 공간을 확보하도록 용지를 600×720pt로 바꾸고 원래 그림을
위치 보존 평행이동한 뒤, 뒤 본문은 660pt 위치에 둔다. `native-pen-blue`는 색을 PDF
`0.2 0.4 0.6 RG`(#336699)로, 글꼴을 Arial로 명시한 별도 대조군이다. 검정 입력의 기존
Helvetica 대체 글꼴과 구분한다.

재생성은 저장소 루트에서 `python3 pdf/issue7998/structure-borders/generate.py`를 실행한다.
입력 SHA와 14개 독립 한컴 2020 Print의 source/hash/engine/method/job은 `provenance.json`에 있다.
Print는 네 방향 삽입, 원래 두 번째 행·열 삭제와 유색 행 삽입을 HWP/HWPX 각각 저장한 뒤
인쇄했다. Native와 Studio의 14개 저장본 바이트가 같음을 확인했다.

표준 실선 동등성의 전 구간/crop/양쪽 반쪽을 검증한 뒤 BorderFill로 옮겨야 새 셀의 표준
서식 상속과 삭제가 의미를 보존한다. 임의 굵기·phase/alpha·채움 띠·병합/spacing/zone의
일반 paint 상속은 미완료이며 tagged 경계는 구조 변경 전에 거절한다. 검증 결과와 후속
문단 R2 경계는 [stage21](../../../mydocs/working/task_m100_7998_stage21.md)에 기록한다.
