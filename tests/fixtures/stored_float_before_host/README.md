# 글줄 앞에 저장된 부동 표

`float-with-single-column-marker-a4.hwp`는 공개
[`nested-tail-margins-hancom.hwp`](https://github.com/edwardkim/rhwp/blob/fedc716cbe810985c798254b370962e24aa4ffcf/samples/pr9042/nested-tail-margins-hancom.hwp)의
파일 헤더, 스타일과 레코드 형식을 사용해 만든 A4 합성 HWP다. 단일 구역과 한 셀의 바깥 표,
단일 단 정의, 새 영문 레이블, 작은 부동 표와 네 행의 중첩 표를 새로 구성했다. 합성 파일에는
요약 정보나 미리보기 스트림을 넣지 않았다.

작은 표의 저장 높이는 4,800 HWPUNIT다. 앞 글줄 끝에서 표의 위·아래 바깥 여백 280, 320
HWPUNIT과 표 높이를 더하면 호스트 글줄에 닿는다. 단일 단 정의 뒤에서도 셀의 저장 줄 좌표가
이어진다.

Windows 한컴 2020 Print의 1쪽은 앞 글줄, 작은 표, 호스트 글줄, A–C행 순서다.
2쪽은 D행과 뒤 문단을 포함한다. 기준 인쇄본과 실행 영수증, Native·fresh WASM 전쪽 TSV 및
대표 대조 이미지는 [시각 검증 기록](../../../mydocs/pr/assets/stored-float-before-visible-line/README.md)에 있다.
