# 첨자가 붙은 연산자 경계의 독립 Print 대조군

- 목적: 곱셈 기호의 부착 첨자와 뒤 본체 사이 소유·간격을 확인한다. 시험지 원본이나 기존 기준 PDF를 대체하지 않는다.
- 입력: 기존 공간 기저 HWPX의 같은 section/header와 paragraph를 사용하고 수식 캐시·저장 줄을 비운 24식이다. 900/1800HU에서 공백·braced 인자·명시적 빈 그룹·곱셈·일반 변수 첨자를 대조한다.
- MCP engine 2024, Hancom 13.0.0.3901, backend `hwp-managed-direct-dll-host`.
- 저장 job `402c3a07-971b-4e16-ae9b-1130aeebe0be`, Print job `9aee4acf-0956-4de5-aed0-7c4b8c047a8e`.
- 실제 Print 실행: method 0, `frame_print_to_pdf_ex_one_up`, PDF 전1쪽. 입력 HWPX·정상 저장 HWP·Print PDF를 함께 보존한다.
- source `0083c6a01dfc7eb2d63ff688c9048c3b64402fee`의 stage617 Native 전1쪽은 97.91049%다. 확대 대조에서는 곱셈 기호의 부착 첨자 앞 간격 누락이 남아 있다. Native의 90% 통과를 직접 판독 완료로 보고하지 않는다.
- PDF 텍스트 추출기가 넣은 가상 공백을 실제 첨자 원점과 구분했다. `rawdict`의 `synthetic=false` 실제 글자 좌표로 확인하며 기준선을 임의로 이동하지 않는다.
- 상세 24식·해시·실행·Print provenance와 실제 글자 원점은 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage615-operator-script-anchor-probe/`에 있다. 신규 정식 회귀는 보정 후 Native/fresh WASM 전쪽 90% 이상과 확대 판독 뒤 추가한다.
