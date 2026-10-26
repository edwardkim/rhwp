# #8153: 편람의 독립 Print 자원

두 JPEG는 기존 `pdf/2025 행정업무운영 편람(최종)-hwpx-kopub-2024.pdf`의
261쪽 xref 2017·2018에서 `PyMuPDF Document.extract_image`로 바이트 그대로 추출했다.
새 PDF나 rhwp의 현재 출력을 정답으로 만들지 않았다. HWP 독립 Print의 같은 쪽 자원도
각각 복호화한 RGB 전픽셀이 같음을 확인했다. 전체 기준 PDF와 두 원본은 기존 경로에 유지한다.

| 파일 | 독립 크기 | 바이트 | SHA-256 |
| --- | --- | ---: | --- |
| handbook-p261-top-print.jpeg | 966×322 | 25090 | `544de0bb8ea9a0a5e86fcb0464bf1a16fb2849e6469e0b691f9e512fca30edd4` |
| handbook-p261-bottom-print.jpeg | 966×768 | 33287 | `ac2309d155238059822a2e4a5790be785ed9258058f71612631771303640ab7b` |

기준 HWPX PDF SHA-256은 `299cb2d994bba8b63f78b681bd74580286e298f25689b4ff4f40c66c54bfa85e`,
HWP PDF는 `3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`다.
8adc3a4010의 Native 전쪽 766조건 최저는 99.46283%, 표준 최적화 fresh WASM
전쪽 766조건 최저는 99.25254%다. 두 원본 모두 383쪽씩이고 측정 누락은 없다.
261쪽의 실제 Native/fresh WASM PNG를 직접 대조하며 새 자원의 단독 RGB 계약을
시각 완료나 전쪽 100%의 증거로 대신하지 않는다.

계획: `mydocs/plans/issue_8153_jpeg_fill_color_20261026.md`.
작업 중 TSV·PNG·해시 감사·실제 호출은 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/`에 보존한다.
소유·순서·원본 보존을 함께 검사하고 색 계약은 독립 자원의 복호화 RGB로 보호한다.
