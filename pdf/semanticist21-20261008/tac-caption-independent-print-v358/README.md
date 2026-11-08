# TAC 캡션 축소본의 독립 한컴 Print

기존 회귀 `issue_6575_tac_picture_sits_on_line_top_when_line_is_taller`의 기대값과 실제
원본 출력을 대조하기 위한 자료다. 새로운 회귀·golden·정상 baseline으로 등록하지 않았다.

- 입력: `samples/issue6575/tac_picture_line_top.hwpx`.
- 입력 SHA-256: `ae4c01001405de8deb2332cab5f469896a3b5ca12b18c1c28a48ff9719cadf85`.
- 입력은 이미 커밋된 축소본이며 이번 변환에서 편집하지 않았다.
- 실행: client0.9.0의 `start → status → download`, target pdf, engine별 timeout600초.
  인증 값은 private env-file로 전달했고 기록에 포함하지 않았다.
- 두 결과 모두 PrintMethod0의 1-up 인쇄이며 실제 PDF는 각각1쪽이다.
- 조판 세대에 맞춘 최종 Native/fresh WASM 비교와 확대 판독이 필요하다.

| engine | job ID | version | Print 경로 | PDF SHA-256 |
| --- | --- | --- | --- | --- |
| 2020 | `0de49cd5-934a-4cf3-8025-0a448acd1b59` | `11.0.0.9136` | `hancom2020_pdf_driver_one_up` | [tac-picture-line-top-ae4c01001405-2020.pdf](tac-picture-line-top-ae4c01001405-2020.pdf) `c56fdb74c26611ae6b4c3cf1f1b1ad8e01f1e14b248ddf44c4c6c0bd2d0d3d11` |
| 2024 | `e57e27cb-fb10-40d3-a24a-705516675e6a` | `13.0.0.3901` | `frame_print_to_pdf_ex_one_up` | [tac-picture-line-top-ae4c01001405-2024.pdf](tac-picture-line-top-ae4c01001405-2024.pdf) `c4202808b13e0d08c434222dee7d524ac144178443ad8afd2f016319add2ab55` |

두 PDF의 세 그림 상자는 동일하다. 마지막 그림의 top은476.479pt다. 기존 검사의
고정 y444.68px 및 현재 Native y458.32px를 이 Print의 기대 위치로 바꾸어 쓰지 않는다.
소유 TextLine과 그림이 서로 맞는다는 관측만으로 앞 문단의 배치까지 정상이라고 판단할
수 없으므로 같은 입력의 base/현재 Native 및 fresh WASM을 v360에서 비교하고 있다.

상태·download·검증 hash·그림 좌표·Print PNG 및 명령은 ignored
`output/pr-review/semanticist21-20261008/tac-caption-independent-print-v358/`에 있다.
실제 입력과 PDF를 보존하며 필수 비교가 끝나기 전 기존 테스트를 제거하거나 완화하지 않는다.
