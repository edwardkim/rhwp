# 고정폭 공백의 독립 Print 진단

Issue: #7642, #6597

원본 30307의 글자 속성(휴먼명조 계열 fontRef)으로 1000/1500/2000/3000HU의
`가가`·일반 공백·고정폭 공백을 각각 만들었다. spacing=0, ratio/relSz=100이며
저장 줄은 지워 한컴이 재생성했다. HWPX의 고정폭은 `hp:fwSpace` 요소다.

MCP engine2024, 실제 한컴 13.0.0.3901,
Print method 0, `frame_print_to_pdf_ex_one_up`.
저장 job `6a77d02d-3aac-4f02-9268-35186e4df6ef`, Print job `3c95a55a-7f7e-4652-90b9-6b0ce4cf54e9`.

- figure-space-input.hwpx: `bdeb0b63a37ac67db3a6d8982710d77d95ee7688fc6f5c182c6c9baafb470972`
- figure-space-2024.pdf: `162100fd3eb0c5256ea638ea7ce1ca975f4f6622e813841bb8731194cde32d48`
- figure-space-2024.hwp: `c4f4bcbadbd2a948d4f190e7bd662d01c5b7e738d6eb2591249c7c3c9aba389a`

현재 source `536da3f62906a30e9e269892b9b70e89edfd66ff` Native/fresh WASM 각각
전체 1/1쪽 최저 100%, 미달 없음이다. 실제 두 출력 PNG의 SHA-256도 같다.
확대 판독·독립 전진 측정과 새 정식 검사 결과는
[증적](../../mydocs/pr/assets/issue7642-figure-space-contract-20261012/README.md)를 따른다.
