# #5790 글자겹침 원문자 검증 자료

- 입력: `samples/issue1880_takeplace_oracle_p13.hwpx`
- 입력 SHA-256: `0725f1b5c3424ad094b1241d0b49c7f40ceb04fcfb732da80b12cf67d6b4fdaf`
- 기준: [한컴 2020 Print PDF](../../../../pdf/issue5790_charoverlap/issue1880_takeplace_oracle_p13-2020.pdf), 13쪽
- PDF SHA-256: `4b965d824b976e5c40ebd93a78772bb62a212f1e038edee75d9ddc2f4605a3d2`
- 실제 제품: 11.0.0.9136, PrintMethod 0, `hancom2020_pdf_driver_one_up`
- 엔진 source SHA: `8c32167d3d87f41beb5c6b045615f4f4e7df112d`, Native142 / 표준 release fresh WASM048
- 96dpi 전쪽 Native 최저 raw 96.10356%, fresh WASM 최저 raw 96.10629%, 전체 쪽수 일치
- [Native TSV](native_silhouette.tsv) / [fresh WASM TSV](wasm_silhouette.tsv)는 원본 그대로다.

5쪽의 실제 글자겹침 ③·④와 주변 본문을 직접 대조했다. 본문의 다른 원문자가 누락을 가리지 않도록
회귀 검사는 CharOverlap 소유 leaf와 원래 source ID·fallback·clip을 유지해 canonical SVG를 재생한다.
SVG의 과거 속성 순서나 절대 좌표로 소유를 추정하지 않는다. 원문자 ③·④ 두 건의 보존과 맨 숫자로의
변환 금지를 검사한다. source table도 남은 leaf와 동기화하며 실제 replay 검증을 우회하지 않는다.

- [Native review](native_p005_review.png) / [Native overlay](native_p005_overlay.png)
- [fresh WASM review](wasm_p005_review.png) / [fresh WASM overlay](wasm_p005_overlay.png)

이 자료는 위 엔진과 입력의 결과다. 이후 엔진 변경의 최종 PR 증적을 대신하지 않는다.
