# 평가결과서 원본과 독립 한컴 Print

- 대상: [PR #8638](https://github.com/edwardkim/rhwp/pull/8638),
  [이슈 #8619](https://github.com/edwardkim/rhwp/issues/8619)의 「감독․검사」 공급 글꼴과 줄바꿈.
- 원본: [evaluation-original.hwp](evaluation-original.hwp), 15,360바이트,
  SHA-256 `a89b8d63ec93e34281c8dbe22f6ef89804a97130855432bda5416a2a3e1ad5ec`.
  로컬 `korea_downloads/prism_downloads/금융위원회/`의
  `1160100-200900004_D0150005-1-000_평가결과서.hwp`이며 원 이슈의 원본 SHA와 일치한다.
- 저장 메타데이터: `lastSavedWith.product=null`, `version=6.7.7.1027`.
  제품 연도를 추정하지 않는다. 원 PR이 같은 원본의 한컴 2022 Print를 근거로 한
  구형 문서임을 확인해, 현행 MCP 절차의 2022 이하용 `--engine 2020`을 지정했다.
- 기준: [evaluation-original-2020.pdf](../../../../../pdf/planet6897-20261106/pr8638/evaluation-original-2020.pdf),
  SHA-256 `10866c14fdf960ad41407ee8c12629c2f60e73e2817e1891e1081f3da71cfc3e`, 전체 2쪽.
- 출력: 2026-11-06 19:23 KST, 실제 Windows 한컴 2020 `11.0.0.9136`,
  MCP `start → status → download`, job `22b3d700-9850-407b-9089-85082b917a40`.
  `request_target=pdf`, `pdf_print_method=0`,
  `pdf_output_mode=hancom2020_pdf_driver_one_up`을 실제 응답에서 확인했다.
  용지·방향·내용을 바꾸지 않은 전체 문서 1-up Print다.
- PDF 글꼴: `Dotum`, `Haansoft Dotum` 외에 원본 한양 계열과 Type3 프로그램도
  내장돼 있다. 제목은 실제 텍스트 연산자에서 `/T1` Type3를 사용한다. 글꼴 목록을
  문서 전체의 단일 공급자로 해석하지 않으며 U+2024는 해당 문장과 확대 출력으로 판정한다.
- 원 PR의 기존 2022 PDF와 이번 2020 PDF는 서로 다른 기준이다. 과거 TSV를 이번 PDF의
  측정값으로 재사용하지 않는다. 전쪽 Native/fresh WASM과 직접 판독 완료 후 최종 리뷰에 연결한다.

클라이언트의 명령은 저장소의 `tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz`를
사용하며 `--target pdf --engine 2020 --output-filename evaluation-original-2020.pdf`를
명시했다. 비공개 접속 정보는 기록하지 않는다.

## R15 직접 판독 뒤 추가 분석

첫 제목의 원본 문단 앞 간격은 1,000 HWPUNIT다. 한컴 Print의 제목 기준선은
161.42 CSSpx, 별도 글꼴 준비 없는 portable 원점 진단은 148.13 CSSpx다. 이 진단을
시스템 글꼴 시각 측정으로 승격하지 않는다. 약 13.3px 차이는 앞 간격 1,000 HU의
96dpi 환산과 일치한다. 제목의 SectionDef는 별도 배치 상자가 없지만 재조판 paint의
컨트롤 일괄 제외에 걸린다. 측정·배치의 공통 간격 소비로 보정하고 실제 두 경로를 재검증한다.

이 기록은 이미 완료한 R15 측정과 보정 전 차이의 증거다. 새 보정 후보의 통과를
선언하거나 90% 통과만으로 제목 위치 차이를 면제하지 않는다.
