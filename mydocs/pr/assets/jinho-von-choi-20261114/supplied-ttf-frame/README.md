# 실제 공급 TrueType의 Print 프레임 검증

제품 source는 `8ad1c5c08a`다. 두 producer의 전체 SHA와 바이트 해시를 함께 기록했다.
입력은 `samples/basic/issue2007_nested_cell_pagination_42065.hwp`, 독립 Print는
`pdf/review-integration-20261026/issue2007_nested_cell_pagination_42065-2020.pdf`다.
PDFium·96dpi·색상 경계 보정 전 2px raw이며 원본·PDF·측정 마스크를 바꾸지 않았다.
문서 전체 글꼴 환경은 [공급 JSON](../font-supply/nested-symbol-slot-environment.json) 하나다.

[Native](native-nested.tsv)·[최적화 fresh WASM](wasm-nested.tsv)은 각각 전 17쪽이다.
전쪽 최저는 98.63%·98.65%이며 새 보호 하한의 하락은 양쪽 모두 0쪽이다.
11쪽은 99.67%·99.63%, 14쪽은 99.42%·99.35%다. 이전 0.01%p 미재현 두 쪽도
예외 없이 하한 이상으로 복원했다. 이 결과를 전체 최종 검증 완료로 확대하지 않는다.

[4쪽](page-04-review.png)·[11쪽](page-11-review.png)·[14쪽](page-14-review.png)은
같은 crop과 배율의 독립 Print, Native 235, Native 8ad, WASM 235, WASM 8ad 순서다.
확대 판독에서 흰 화살표의 내부·외곽, 본문 글자·밑줄·줄 간격·내용 보존과 셀 포함을 확인했다.
대조 crop은 표시용이며 전체 쪽 측정에는 적용하지 않는다. TTF 프로그램 자체를 바꾸거나
특정 글꼴·문자·문서의 좌표 상수를 추가하지 않았다.

공개 글꼴의 실제 WASM 최소 대조는 2022·2024 × 미공급/정적 TTF/CFF/가변 ×
원 메트릭/자연 잉크 × Print/Screen/HQ의 48개 상태다. 보완 전에는 실제 정적 TTF의
Print 양성 2개가 실패했고 음성 대조 46개는 통과했다. 보완 후 48개 모두 통과했다.
실제 공급자의 SFNT 형식 증거를 메트릭 축약 전에 보존하고 공통 paint가 소비한다.
원문 유형·전진과 Source bbox를 변경하지 않는다. 후속 정식 회귀는 기존 suite/브라우저
묶음에서 자동 수집하며 workflow에 개별 호출을 추가하지 않는다.
