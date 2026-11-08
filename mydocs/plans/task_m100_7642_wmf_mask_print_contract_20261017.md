# #7642 WMF 마스크 실패 검사의 독립 Print 대조

Issue: #7642

## 위반 전제와 독립 근거

기존 #6865 검사3개는 SVG 어디에도 `rop_pat`가 없어야 한다고 요구한다.
현재 변환기는 ROP 시퀀스의 중간 1비트 그림을 보이는 칠 대신 역흑백 mask의
정의로 보존하므로 이름 존재와 실제 표시가 다르다. 단순 문자열 조건을 지우거나
현재 SVG를 복사해 기대값을 만들지 않는다. 독립 Print와 비트 연산의 의미를
대조해 실제 보존해야 할 색·범위·마스크 소유를 확인한다.

## 검증 범위

기존 Rust builder와 같은 바이트의 기본 XOR/AND/XOR, 1유닛 overscan 두 종류,
역팔레트와 24비트·단독 마스크 음성 대조를 공개 합성 HWPX에 넣는다.
원래 메타파일 바이트는 수정하지 않는다. 그림/캡션 어울림을 배제한 배치에서
원본 HWPX 자체를 engine2020 Print하고 실제 인쇄 메타데이터를 확인한다.

실물 보도자료15쪽은 현재 code의 Native/fresh WASM raw 최저93.60142/
93.68294%이며 같은 독립 Print와 쪽수가 같다. 이 결과를 합성 입력의 검증으로
대신하지 않는다. 합성 대조군도 전쪽90%·직접 review/overlay 판독을 먼저
수행하고, 실패하면 출력 보정 후에만 정식 검사 변경을 판단한다.

관용 점수 외에도 마스크의 농도·무늬 크기와 실제 표시/정의의 소유를 확인한다.
공유 target의 Cargo 검증은 순차 실행하며, PNG와 TSV는 ignored output에
보존한다. 검증하지 못한 대조군은 정상 fixture로 커밋하지 않는다.

## 실제 대조 결과와 다음 보정 조건

검증 producer는 `cc4255a58d8112c07013e89eda1b1a065c824fea`다.
`stage2335` 입력은 기존 Rust builder의 WMF 바이트를 수정하지 않은 진단용 HWPX이며,
정상 fixture로 채택하지 않았다. `stage2336` Print job
`3cceb344-16df-4586-bccf-100966065c0b`는 engine2020 / 한컴
11.0.0.9136 / `pdf_print_method=0` / `hancom2020_pdf_driver_one_up`로
1쪽을 출력했다. Native raw는 **45.34159%**, fresh WASM raw는 **63.17611%**다.
양쪽 모두 1쪽이지만 무늬 크기·칠 범위·24비트 색과 단독 DPa의 표시가 다르다.

한컴 정상 저장이 입력 배치 속성을 고치는지 분리하기 위해 원 HWPX를 같은 engine으로
HWP 저장한 뒤 다시 Print했다. 저장 job `161c4741-1ee6-4e21-8c96-47b40df5b2f9`,
Print job `3002a9f1-d3af-4f5a-bdc1-68a0101a8093`다. Print의 제품·방법·모드는
위와 같고 결과도 1쪽이다. `stage2344` Native / `stage2345` fresh WASM raw는 각각
**45.34159% / 63.17611%**로 같았다. 같은 저장본으로 실패를 재현한 결과이며,
정상 저장만으로 해결됐다고 판정하지 않는다.

`stage2347` typed IR 대조에서 6개 그림의 common 크기14400×14400 HU,
orgSz/curSz·항등 변환·crop·imgDim·모서리 좌표는 저장 전후 보존됐다.
따라서 그림 삽입 placeholder의 크기가 정상 저장 과정에서 바뀌었다는 가설은
현재 증거로 지지되지 않는다. 원인 계층은 아직 확정하지 않았다. Print PDF에는
1비트8×8 자원의 반복 표시와 24비트 대조군의 단색 회색 칠이 있다.
원시 bitmap brush의 표시를 버리거나 기대 SVG 문자열을 복사하는 보정은 하지 않았다.

입력·저장본·PDF는 `pdf/issue7642-wmf-mask-print-controls-20261017/`에
진단 자료로 보존하며 아직 Git 수용 대상이 아니다. TSV·실행 로그·IR/PDF 자원
분석 JSON은 ignored `output/pr-review/issue7642/failed-inputs-20261008/
stage15-current-validation/`의 stage2335~2348에 있다. 대표 review/overlay는
`stage2340-wmf-mask-review/{native,wasm}/`다. 실제 원인 보정·전쪽 양쪽90%·
확대 판독 전에 기존 마스크 실패 검사3개와 golden을 변경하지 않는다.
