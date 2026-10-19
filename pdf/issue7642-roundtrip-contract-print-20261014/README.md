# 왕복 렌더 실패 입력의 독립 한컴 Print 기준

- 원본은 저장소의 기존 `samples/` 입력이다. RHWP 재저장 파일을 원본으로 바꾸지 않았다.
- 공개 저장 제품 정보: #1880 입력은 한컴 2020 `11.0.0.8362`, #1892 HWP3는 미상이다. 두 요청 모두 문서 지침의 `engine: 2020`을 명시했다.
- 실제 출력: 한컴 `11.0.0.9136`, Windows `hwp-managed-direct-dll-host`, `pdf_print_method: 0`, `pdf_output_mode: hancom2020_pdf_driver_one_up`, `input_preprocess: none`.
- `start → status → download`를 실행했으며 session0 font scope는 verified, 등록 실패 0건이다. 기본 원본 용지·방향, 한 쪽씩 Print 출력을 사용했다.
- 클라이언트 절차는 `mydocs/manual/mcp_hwp2024Convert_usage.md`를 따랐다. 인증 정보는 포함하지 않는다.

| 원본 | 원본 SHA-256 | PDF | PDF SHA-256 | 쪽수 | MCP job |
|---|---|---|---|---:|---|
| `samples/issue1880_anchor_stack_sb_convert.hwpx` | `0725f1b5c3424ad094b1241d0b49c7f40ceb04fcfb732da80b12cf67d6b4fdaf` | `issue1880_anchor_stack_sb_convert-2020.pdf` | `7a1e127e65e6f6dc8600d2fcf597b188ad9fc9639e62b5d2bfdc6f1aa1e565c6` | 13 | `7922b61b-dac1-40cf-9c4a-88e412e76776` |
| `samples/issue1892_hwp3_drawing_group_roundtrip.hwp` | `ab6bb4e80c4b0f4228e62bbb3d180229e5cef5f52b20fddee8849bf1c7b4c438` | `issue1892_hwp3_drawing_group_roundtrip-2020.pdf` | `bde778b26c91635f14bf14272eca8b9199da7be7f9d4e0750b7402768b640bc9` | 2 | `e91005c0-d814-436a-bbba-c5ba0b6e706a` |

## 판정 경계

source `73c8bbefa39c939bc5b1a103959e59a02fa40077` Native 전쪽 결과는 #1880 13쪽 최저 18.81919%(6쪽), #1892 2쪽 최저 86.78774%(1쪽)로 미달이다. 이를 회귀 기대값 갱신의 근거로 사용하지 않는다. fresh WASM·base 비교·보정은 아직 남아 있다. 실행 로그·TSV·JSON·대표 실패 PNG는 ignored output의 stage662/667/669에 보존한다.
