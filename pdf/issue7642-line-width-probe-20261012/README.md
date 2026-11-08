# 한컴 Print 선 굵기 진단

원본 sample16 시각 통과를 대체하지 않는 합성 벡터 진단이다. Stage 155의 선 종류 진단
HWPX에서 라벨·도형만 복제하고 선언 굵기를 변경했다. 저장 LineSeg가 없는 입력이므로
문서 위치 차이를 90% 통과나 신규 회귀 fixture의 증거로 사용하지 않는다.

한컴 Office 2024 MCP PrintMethod=0, `frame_print_to_pdf_ex_one_up` 경로로 출력했다.

선 굵기 33개 사례와 0·반올림 경계·원형 점 18개 사례를 포함한다. 측정은 PDF의
선·채움 벡터에서 pt/HWPUNIT으로 수행하며 화면 px나 특정 문서 위치를 기준값으로 쓰지 않는다.

## line-width-probe.hwpx
- 입력 SHA-256: `acf798ccc758a021242af8155d26577c40fe78c841f64ef7dd294839f5a802f2`
- 기준 PDF: `line-width-probe-2024.pdf`
- PDF SHA-256: `b0d92f73731a820e9eca4f46631b10a7965ec5d655a4159170763067de3c1e77`
- MCP job: `9a0d4081-e7b5-4071-802c-d06d253731ae`
- 한컴: `13.0.0.3901`, 3쪽

## line-width-boundary-probe.hwpx
- 입력 SHA-256: `de6279d4d87b89f82720d33c0e1ceb3216f4f1f874ea4078386e7aaa569b0a0f`
- 기준 PDF: `line-width-boundary-probe-2024.pdf`
- PDF SHA-256: `e18f196961edbc46ec5377297d995d1865c4c0d806f8dc02d771188f0c24ff22`
- MCP job: `77fd11b5-0f3f-4e4e-b717-798965ce0780`
- 한컴: `13.0.0.3901`, 2쪽
