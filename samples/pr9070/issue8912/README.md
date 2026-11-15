# #8912 원 첨부의 파서 진단 자료

원 HWP3의 용지·장평은 정상 조판용 값이 아니다. 원본 바이트는 `tests/fixtures/pr9070/parser-only/hwp3-linechars-symbols.hwp`에 보존했다. `samples/`의 정상 HWP3 장평 검사에는 넣지 않으며 검사 자체의 기대값이나 수집 조건은 바꾸지 않는다.

독립 한컴의 저장 문자 대조는 옆 `hancom-reference/hwp3-linechars-symbols-2020.hwpx`와 `mydocs/pr/assets/pr9070/hancom-decoded-mapping.json`을 따른다. 이 변환본도 용지 0인 원 합성 입력의 문자 해석 근거이며 정상 인쇄 정답지가 아니다.

정상 시각 검증 입력은 `samples/pr9070/hwp3-linechar-specimen.hwp`, 수동 Print는 `pdf/pr9070/hwp3-linechar-specimen-2020.pdf`다.
