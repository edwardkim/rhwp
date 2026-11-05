# PR #8540 개요 번호 독립 Print 근거

두 HWPX는 원 PR `3de1e510d9039490485505bfb7250182d59e2512`의
`mydocs/pr/assets/scaffold_outline_numbering/`에 포함된 파일이다. 아래 PDF는 해당
HWPX를 수정하지 않고 HWP 2024 MCP의 `start → status → download`로 새로 인쇄했다.
원 PR의 `FileSaveAsPdf` PDF와 구분한다.

| 문서 | HWPX SHA-256 | Print PDF SHA-256 | MCP job |
| --- | --- | --- | --- |
| A | `7dc0e7f41aec968c6510bf2c430dd50cee67590c15bd5f3cc0628a02c883f751` | `20536ee5ac4bdd6dc52cc5f0caf246d07713664e6ca1e46be945358e2c92e12a` | `0ad85e9c-d3dc-4392-9675-2b0aef8e7413` |
| B | `bdd247054116d4fb181f86a843376ccc9318914a2207bc8fd9802392b00d5eba` | `8f4a28a633bcdac9d7dc99edd68cf0705c2e7eadecf9547541763b7f99440325` | `fc0a09dc-196d-4f6c-a756-dd2c7b487ccd` |

MCP 상태에서 두 작업 모두 `succeeded`, `engine: 2024`, 한컴 버전
`13.0.0.3901`, `pdf_print_method: 0`,
`pdf_output_mode: frame_print_to_pdf_ex_one_up`, 전체 1쪽을 확인했다.
비교는 96 DPI에서 Native와 fresh WASM 각각 전체 1쪽을 측정했다.
두 입력의 각 경로 `silhouette_raw_match_percent`는 모두 `100.00%`다.
`a-*`와 `b-*` TSV·review·overlay가 각각 **같은 원본의 1쪽**에 대응한다.

이 결과의 제품 바이너리와 WASM package는 통합 코드
`0884eafc7b2c34fae20a765109ff314514175e4b`에서 빌드했다. 이후
`#8540`의 scaffold 생성 코드를 체리픽했으므로 최종 head에서 관련 검사와 출력을
다시 확인한 뒤 review 판정을 확정한다.
