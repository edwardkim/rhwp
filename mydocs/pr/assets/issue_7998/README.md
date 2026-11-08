# #7998 첫 절편의 전쪽 시각 증거

- 제품 Rust source: `3c6fddb0beebf15aa332a9f9ca3aecebe4135e4d`.
- 기준 devel source: `8766fb47d8ec11547f0d2ae941c53cc3c9780ab2`.
- Print / compat 2022 / 96 DPI, Native Skia 및 실제 fresh Docker WASM Canvas PNG.
- reference는 Poppler 24.02.0 `thinlinemode=none`이다. fill이 무관한 stroke 상태에 따라
  사라지던 향상 모드는 독립 PDF 계약 검사로 보정했다. 이전 shape 증거는 로컬 조사에 보존했다.
- 원래 12쪽 PDF는 기존 추적 파일 `pdf/pr7704/6778-square-2020.pdf`와 바이트가 같다.
  사용자 별칭은 `content_operators-002.pdf`이며 입력 SHA-256은
  `d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656`이다.
- 정상 대조군은 `pdf/issue7998/solid-control.pdf`의 독립 2×2 PDF 격자다.
- 원본 Batang TTC의 SHA-256은 `51d9a8d4205239c865e24b6eb7e6e6dbacfd59c3e2c46f623db0af45e169c9d8`이며,
  실제 Native trace에서 본문이 custom Batang으로 선택됨을 확인했다. 원본 글꼴은 포함하지 않는다.
- 실제 연락처가 있는 견적서와 상세 문자 장부·캡처는 승인 계획 1절대로 로컬에 보존한다.
  입력 SHA-256 `12a573c62884c48a03f67993db5048f485e88bef499f2381b459b994062ecfab`와
  집계 TSV만 포함하며 그 이미지나 원문을 공개 자산에 넣지 않았다.
- 전체 페이지는 Native/WASM 모두 90% 이상이고 같은 backend의 기준 대비 0.01%p 이상 하락이 없다.
  최저 operators는 91.04017% / 90.62487%, quotation은 99.85062% / 99.74444%,
  solid-control은 100% / 99.53052%다. TSV는 원본 바이트 그대로 복사했다.
- 3·5쪽·대조군의 Native/WASM review·standalone overlay를 포함한다. 3쪽 본문은 Batang으로
  확인했고, 표/그림/내용 소속과 정상 네 셀의 실제 경계를 확대 판독했다. 견적서의 가는 선 농도는
  독립 MuPDF/Cairo와 비교해 래스터 차이를 구분했다. 메인테이너 직접 시각 판정은 대기 중이다.
- 의미 표 생성은 정상 대조군 1개이며, 원래 3쪽은 불충분한 병합 근거로 1→0개 보류됐다.
  높은 점수를 작성 당시 셀 구조 복원의 증명으로 사용하지 않는다. 새 지역 가설의 최종 IR 소비는 후속이다.

`evidence.json`에 producer/file SHA, 페이지별 원래 소수값과 delta를 기록했다. 전체 회귀 검증 상태는
`mydocs/working/task_m100_7998_stage2.md`를 따른다. raw source 장부·실행 로그·원본 글꼴은 공개 자산이 아니다.
