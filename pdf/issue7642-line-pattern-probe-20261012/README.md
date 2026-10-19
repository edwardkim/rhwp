# 선 종류·굵기의 합성 Print 진단

Issue: #7642

`line-pattern-probe.hwpx`는 `samples/hwp3-sample16-hwp5.hwpx`의 TAC 사각형을 복제해
10개의 선 종류·굵기 조합으로 작성한 합성 진단 입력이다. 본문 section을 20개 문단으로
교체하고 텍스트 상자·그라데이션·그림자를 제거했다. 원 좌표·선언 크기·변환 행렬은
동기화하고 저장 LineSeg는 넣지 않았다. 원본 문서 또는 정상 저장 LineSeg의 증거가 아니다.
입력 header의 2024 저장 제품 메타데이터는 template에서 복사한 값이다.

한컴 MCP의 실제 2024 Print로 전체 1쪽을 출력했다. 버전 13.0.0.3901,
PrintMethod=0, 출력 모드 `frame_print_to_pdf_ex_one_up`이며
job은 `8f8cfe97-df63-4328-940c-687c8d8d005d`다. 공식 Print와 독립 선분 분석의
검증 범위는 선 종류·굵기이며 원본 64쪽의 시각 검증을 대신하지 않는다.

- 입력 SHA-256: `9351843971ff25a19e261825a235046952a6f2d1bb3f17087803aea9a49b0314`
- PDF SHA-256: `a8ffdd0b15417e60864ad57eed381b61b973ca857b717c0f85f52853aaabb1aa`
- 조합: SOLID 56HU; DASH 28/56/112/224HU; DOT/DASH_DOT/DASH_DOT_DOT/LONG_DASH/CIRCLE 56HU.

시각 TSV·Overlay·전체 생성 절차와 MCP 실행 영수증은 ignored output의
`stage155-line-pattern-probe`와 `stage155-line-pattern-native-review`에 보존한다.
현재 이 입력은 활성 회귀 fixture/golden으로 등록하지 않는다.
