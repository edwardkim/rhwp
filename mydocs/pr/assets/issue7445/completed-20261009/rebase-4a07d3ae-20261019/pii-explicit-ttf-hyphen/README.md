# 명시 TTF 하이픈 프로그램 계약 재검토 — C445

Issue: #7445. 기존 검사는 `9cbfd78ccc`에서 저장 구두점의 언어 슬롯·원문 소속·독립 위치를
보호하려고 추가했다. C442 `df88f3d5699c26dfa8e9edad04e26ff6d0dbfe5e`는 원본의 명시 TTF
대체 프로그램을 전체 소스의 HFT 부분 메트릭과 구분했다. 기존 검사의 HFT 메트릭 owner
기대는 이 원본의 실제 프로그램 선언과 충돌하므로 같은 검사에서 교정한다.

원본 `samples/task2097/75544_pii_bunseok.hwpx` 문단95, charShape45의 Latin8은
HCI Poppy/type1이며 명시 substitute Palatino Linotype/type1이다. 독립 Print
`pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf` 10쪽 실제 하이픈은 Type0/CIDFontType2
PalatinoLinotype-Roman, CID8 `/W333`, 15pt다. 20CSSpx 크기의 자연 폭은6.66CSSpx이며,
현재6.66667은 원본 HU 변환 정밀도1HU 이내다. 원점85.44·baseline141.76CSSpx는 기존
±2px를 유지한다. 원문 텍스트·순서·누락/중복·다른 본문 글꼴 검사도 유지한다.

- frozen247 Native / fresh250 WASM, declared TTF 환경/compat2022/Print/192DPI.
- 현재 PDFium canonical 전66쪽씩 새로 산출: 최소94.58132/94.60117%, 미달0.
- 10쪽 양쪽100%, review/overlay 실제 판독. 하이픈 및 후속 원문 정상,
  작은 AA/기준선 색 테두리는 남고 raw100은 RGB 동일성을 의미하지 않는다.
- 동일 backend 직전219의 전66 PNG와 각각 바이트 동일,0.1%p 하락0/132쪽.
- corrected test: frozen202 실제1FAIL → frozen247 실제1PASS.
- 공식 `run-rust-test.mjs`: suite025 해당1PASS/249제외,
  기존 source-program 대조 suite024 21PASS/233제외. manifest prepare/check exit0.
- 관련 근거 원장은 ignored `repair260-pii-hyphen-program-diagnostic/`,
  `repair261-pii-declared-current/`, `repair263-pii-hyphen-regression/`다.
  원본 HFT glyph 표본/PDF glyph/폰트 바이너리를 제품이나 이 증적에 복사하지 않았다.

이 변경은 검사를 제거하거나 허용치를 완화하지 않고 원본 프로그램과 독립 전진폭을 추가한다.
다른 문서/전체 회귀의 잔여와 실제 소비 경로 전체 수용은 별도다.
