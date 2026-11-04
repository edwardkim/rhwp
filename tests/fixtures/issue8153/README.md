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

# 편람 전쪽 독립 Print 좌표

`all-pages-hwp.json`과 `all-pages-hwpx.json`은 각 원본 및 기존 독립 한컴 2024 Print PDF의
경로·SHA-256과 383쪽의 독립 기준을 기록한다. 기준 PDF를 72pt에서 96dpi CSSpx로 환산했다.
PDF 문자 추출의 원점과 기준선에서 실제 편람 렌더 트리의 같은 텍스트에 대응하는 내용을
쪽마다 하나씩 선택했고, 원본 본문을 우선했다. PDF 안의 원점은 후보 코드의 좌표를
기대값으로 복사하지 않았다. 표지는 독립 PDF 이미지의 네 경계를 사용한다.

각 원본에서 370쪽은 문자, 1쪽은 표지 그림, 12쪽은 PDF에 그려진 내용이 없는 빈 쪽이다.
빈 쪽은 2·10·26·28·182·184·250·252·276·278·308·310쪽이다. 같은 내용의 누락·중복과
쪽 순서도 검사한다. 한 쪽의 한 기준만으로 모든 글자나 그림의 위치를 보호하는 것은 아니므로
기존 주요 표·그림·글자 회귀 검사를 함께 유지한다.

[Native·fresh WASM 전쪽 실루엣 TSV와 205쪽 판독](../../../mydocs/pr/assets/issue_8153/rebase-ad4-validation-20261030/README.md)에서
두 원본의 130쪽만 각 99.96%, 나머지 각 100.00%임을 확인했다. 130쪽은 ±2px, 다른
내용 쪽은 ±1px를 적용한다. 회귀 검사는 글꼴 파일을 공급하거나 시스템 글꼴을 요구하지 않는다.

## 전쪽 검사 수정 전후 실행

2026-10-30에 동일한 검사 파일과 두 JSON을 리베이스 기준
`ad4bf269ffef5adf2142b33004a9981fad7ce804`의 별도 소스에 연결해 `release-test`로 실행했다.
기준 코드에서는 HWP와 HWPX가 각각 좌표 assertion으로 실패했고, 현재 후보에서는 두 검사 모두
통과했다. 예를 들어 HWP 154쪽 문자 기준선은 독립 Print의 337.92px에 대해 기준 코드에서
327.13px였으며 허용 오차는 ±1px다. HWPX에서는 149쪽 기준선이 826.09px로 독립 Print의
829.76px와 달랐다. 이는 각 검사에서 실제로 검출한 차이이며 다른 모든 조판 차이의 검출을
뜻하지는 않는다.

- 검사 파일 SHA-256: `9b2a28f17d0c1567a55c27690d6b9a0e075bac3de39c53858448a604081e879d`.
- HWP JSON SHA-256: `48c6a7d31a2d9e6048b8e651806d051b5a8f5d4c077d5ba0876e08cd0d65c4cc`.
- HWPX JSON SHA-256: `d29a0631740097c0adad448d20682b7a2dcb7bb9ec6c73bc68875be2d5c1bc2d`.
- 기준 실행: 같은 검사 파일을 별도 소스의 integration test로 연결한 뒤
  `cargo test --offline --profile release-test --target-dir /home/tsjang/rhwp/target/pr-review --test issue8153_before -- --nocapture`.
  두 검사 모두 좌표 assertion 실패, 종료 코드 101.
- 후보 실행: 같은 검사와 JSON을 자동 수집하는 `regression_suite_009`에서 nextest로 실행해
  두 검사 통과. 제품 코드는 위 TSV의 후보와 동일하다.
