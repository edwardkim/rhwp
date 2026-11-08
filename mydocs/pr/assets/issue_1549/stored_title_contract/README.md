# #1549 합성 원본의 실제 Print와 검사 전제 교정

원 PR #1568의 실문서 증상 설명을 합성 fixture의 독립 정답으로 대신했던 기존 검사 두 곳을
교정한다. `samples/issue1549_multipositive_float_tables.hwpx`의 정확한 바이트를 한컴 2020
11.0.0.9136에서 PrintMethod=0, `hancom2020_pdf_driver_one_up`으로 인쇄한 결과는 2쪽이며,
첫 쪽에서 A·B·C 표 뒤에 제목과 filler 01~30 문단이 순서대로 있다. 마지막 쪽에는
filler 31·32 문단이 이어진다.

- 원본·PDF·원 코드의 SHA와 실제 Print job은 [보존 원장](preservation.json)에 있다.
- 기준 PDF는 [정식 PDF](../../../../../pdf/issue1549/issue1549_multipositive_float_tables-2020.pdf)다.
- [Native 전 2쪽 TSV](native/silhouette.tsv), [fresh WASM 전 2쪽 TSV](wasm/silhouette.tsv)의
  raw 최저값은 97.31566%, 97.44110%다. 1쪽은 양쪽 98% 미만이므로 절대 좌표는 고정하지 않는다.
  중요한 종료 이어받기를 담은 2쪽은 양쪽 100%이며 직접 판독했다. 독립 PDF의 96dpi 글자 원점
  x=113.378550과 기준선 y=143.523153/164.779917px를 ±2px로 보호하고 쪽·문단 소유도 검사한다.
- [Native review](native/review_001.png)·[overlay](native/overlay_001.png)와
  [WASM review](wasm/review_001.png)·[overlay](wasm/overlay_001.png)를 직접 판독해 표·제목·본문의
  순서, 비겹침, 내용 보존을 확인했다. 가는 괘선과 글자 농도·간격의 미세 차이는 남는다.
- Native는 `928ba88d0`, fresh WASM은 `cffebd41b`의 최적화 빌드다. 후속 변경은 빈 host의
  `prepare` 안에 있고 가시 host는 변경 지점 전에 반환된다. Native 전 2쪽 PNG가 바이트까지
  동일함을 재실행으로 확인했다. 최종 통합 head의 fresh WASM 검증 완료를 이 기록으로 대신하지 않는다.

교정 범위는 기존 #1549 함수와 같은 원본을 사용하는 #6929 대조 함수다. 제목을 무조건 표 위로
두는 단언을 원본의 표 순서·제목/본문 비겹침·내용 단일 소유로 바꾸고, 독립 Print 2쪽과
filler 32개 문단의 순서·누락·중복 및 30→31의 이어받기를 확인한다. 별도 빈 host 대조군과 실제 #6929
원본의 저장 오프셋·제목 관계는 유지한다. 생산 코드를 옛 합성 기대값에 맞춰 되돌리지 않는다.

교정 전 두 파일 전체를 같은 폴더의 `.rs.txt` 파일로 보존했다. 이번 변경은 #1549 원 이슈의
모든 실문서가 해결됐다는 선언이나 다른 원본의 제목 순서에 대한 일반 규칙이 아니다.

교정한 #1549 검사는 고정 base `2e3b4f880`의 실제 Native 라이브러리에서도 실행했다.
컴파일은 성공했고, 독립 Print의 2쪽에 대해 실제 1쪽으로 실패했다. 현재 라이브러리에서는
해당 검사와 #6929 대조 검사 모두 통과했다. 같은 집중 실행의 18건 중 16건 통과·2건 실패이며,
남은 두 #6111 검사를 이 증거로 통과 처리하지 않는다. 최종 전체 검증과 PR 제출은 미완료다.
