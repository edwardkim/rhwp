# 양쪽 바탕쪽 맞쪽 프레임 대조군

공개 한컴 저장 `../issue9046/hancom/02-both.hwpx`의 구역 좌우 여백만 변경했다. `symmetric.hwpx`는 양쪽 11338 HWPUNIT, `reverse.hwpx`는 왼쪽 5669·오른쪽 11338이다. 용지·바탕쪽·도형 속성·본문은 바꾸지 않았다. 원본의 왼쪽 11338·오른쪽 5669와 함께 비교한다.

정상 한컴 2020 11.0.0.9136에서 각각 인쇄한 전 3쪽 PDF는 `pdf/pr9071/master-frame-controls/`에 보존한다. 별도 `hancom-hwp-both-2020.pdf`는 공개 한컴 HWP의 독립 재인쇄이며 HWPX와 같은 짝수 쪽 막대 절단을 확인한다. Windows 호스트 COM은 실행하지 않았다.


`rectangle-normalized.hwpx`는 원 Both의 그림을 실제 사각형으로 바꾼 대조군을 정상 한컴에서 HWP → HWPX로 저장한 결과다. `table-normalized.hwpx`는 같은 바탕쪽에 실제 표를 배치한 뒤 같은 순서로 정상 저장한 결과다. 그림을 사각형으로 잘못 간주하지 않고 객체별 실제 배치 소비를 확인한다. 두 파일은 각 3쪽이며 내장 글꼴 프로그램이 없다. 원문 본문 `page 1`·`page 2`·`page 3`을 유지한다.

독립 Print는 `pdf/pr9071/master-frame-controls/rectangle-2020.pdf`·`table-2020.pdf`이며 원본/출력 해시와 한컴 job 영수증은 `mydocs/pr/assets/pr9071/hancom-object-controls.json`에 보존한다. 이 테스트는 바탕쪽 논리 용지 프레임의 검증으로, 기존 #7751 물리 Print raster 배율·clip 계약을 변경하지 않는다.
