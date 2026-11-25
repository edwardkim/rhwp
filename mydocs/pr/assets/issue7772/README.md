# HWP 사양서의 제어문자 그림과 쪽 경계

`samples/hwpspec.hwp`의 SHA-256은 `64df877f3c4accda0111ecf39d837da889741bb821b220cf61b6c126dc88364d`다.
한컴 2024 `13.0.0.3901`에서 직접 인쇄한 1-up PDF는 178쪽이다. 변환 job
`e582655b-a02a-433b-9e38-150a302cc0a1`의 인쇄 방식은 `pdf_print_method=0`이고,
PDF SHA-256은 `a4f52594ead9e292f6d124a362f91ed28d84fa29a3e23858232b1a95f7bedd39`다.
[한컴 2024 1-up PDF](../../../../pdf/issue7772/hwpspec-hancom-one-up-2024.pdf)

Print 프로필과 2024 조판, 96 DPI PDFium 기준으로 `--silhouette-only --print-bmp-full-color`를 적용했다.
두 경로에 같은 원본 글꼴 프로그램을 공급하고, 사용된 쪽의 글꼴을 원래 공급 순서로 읽었다.
Native 바이너리 SHA-256은 `cbfb91e489f8d45d6b6dc48a811db2ef8440b2200d7def991324e738dadee53f`,
fresh WASM 모듈 SHA-256은 `b161795a0ea3b1a0df91203e673b70ed1f185c6060d43df9bcd6b859ca0cdd06`다.
[Native 전쪽 TSV](hwpspec-native.tsv), [fresh WASM 전쪽 TSV](hwpspec-wasm.tsv)

| 대조 범위 | Native | Fresh WASM |
| --- | ---: | ---: |
| 전178쪽 최저 | 98.14% | 98.16% |
| 20쪽 | 100.00% | 100.00% |
| 21쪽 | 99.99% | 99.99% |
| 106쪽 | 100.00% | 100.00% |

20쪽의 문자·인라인 컨트롤 그림이 원문 순서대로 한 점씩 놓이고, 21쪽의 확장 컨트롤 그림이
뒤 설명과 겹치지 않는 것을 세 출력을 확대해 확인했다. 원본 그림의 소유 단락과 후속 본문의 순서를
관계형 검사로 보호한다. 쪽수 원장에는 같은 독립 PDF의 178쪽을 복원한다. 2px 공차에서 전쪽의
용지 밖 개체는 0건이었다.
[20·21쪽 확대 비교](hwpspec-control-figures-pages-20-21.png)

106쪽의 호와 두 번째 설명은 같은 셀의 원본 저장 줄을 함께 사용한다. 세 번째
설명은 그 뒤에 놓인다. 곡선 박스 높이와 저장 줄 높이의 관계 및 셀 안 후속
설명의 순서를 검사한다. 최종 조판 소스를 반영한 양쪽 출력에서 20·21·106쪽
결과를 재현했다.
[106쪽 확대 비교](hwpspec-curve-labels-page-106.png)

## 이전에 복원된 원본의 재검증

| 대상 | 전쪽 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| #9083 CBTA | 204 | [98.21%](cbta-native.tsv) | [98.22%](cbta-wasm.tsv) |
| #9277 정책연구 | 215 | [98.97%](policy-native.tsv) | [98.86%](policy-wasm.tsv) |

두 입력은 기존 PR의 독립 Print와 같은 쪽수를 유지했다.
