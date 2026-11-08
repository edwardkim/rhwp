# 공간 기저 첨자의 독립 한컴 Print 진단

- 목적: 명시적 공간과 기저 없는 첨자를 구분한다. 시험지 원본·기준 Print를 대체하지 않는다.
- template: `pdf/issue7642-term-context-diagnostic/term-context-input.hwpx`. 수식 캐시·저장 줄을 제거하고 900/1800HU의 18개 식을 작성했다.
- MCP engine 2024, Hancom 13.0.0.3901, backend `hwp-managed-direct-dll-host`.
- 저장 job: `1c6ca7e5-7190-40a5-a19b-607b29a537dc`. Print job: `fd7edb33-c817-4f6f-8651-7cb8537e2027`.
- PDF 전1쪽, `pdf_print_method=0`, `pdf_output_mode=frame_print_to_pdf_ex_one_up`를 실제 결과에서 확인했다.
- 입력 HWPX·한컴 저장 HWP·독립 Print PDF를 함께 보존한다. 상세 해시·18개 script·실행 명령은 ignored stage579의 input manifest·print provenance에 있다.
- Print 판독: backtick/tilde 공간과 빈 그룹은 왼쪽 첨자 3과 뒤 C/P·오른쪽 첨자를 보존한다. 공간 없는 `_3{rmC}_1=`은 내용을 출력하지 않는다. 보통 숫자의 `3_1`과 명령형 `LSUB`도 별도 대조했다.
- 현재 RHWP는 공간 뒤 `_`에서 입력을 중단해 조합식 본체를 누락했다. 공간을 실제 기저로 연결하는 parser 수정은 새 source의 Native/fresh WASM 전체 90% 이상·확대 판독과 시험지 원본 검증을 거쳐 판단한다.
- 정식 회귀 대상 등록이나 골든 갱신을 선행하지 않는다. 입력·독립 기준은 진단 자료로 보존한다.
