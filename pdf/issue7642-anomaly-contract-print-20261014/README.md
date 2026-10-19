# 회귀 실패 입력의 독립 한컴 Print PDF

원본은 저장소에 보존된 HWP 파일이며 RHWP 재저장본으로 대체하지 않았다. 저장 제품 정보에 따라 engine 2020 또는 2024를 명시하고, 문서의 원래 용지·방향으로 한 쪽씩 인쇄했다.

- 절차: `mydocs/manual/mcp_hwp2024Convert_usage.md`의 MCP start → status → download.
- 실제 backend: `hwp-managed-direct-dll-host`, `pdf_print_method: 0`, `input_preprocess: none`.
- 2020 출력 경로: `hancom2020_pdf_driver_one_up`; 2024: `frame_print_to_pdf_ex_one_up`.
- 모든 성공 요청에서 session0 글꼴 공급은 verified이고 등록 실패는 0건이다. endpoint와 인증 정보는 포함하지 않는다.

| 원본 | 원본 SHA-256 | PDF | PDF SHA-256 | 쪽수 | 실제 한컴 버전 | MCP job |
|---|---|---|---|---:|---|---|
| `samples/issue7235/156086935_none_image_fill.hwp` | `b12aa445776bef38a003e8679865a9a0ed2f729ecef18099625c976efe64d6ae` | `156086935_none_image_fill-2020.pdf` | `0944517d51c831be14e6d5de2827718409f47d912a3ffaa8178b20eb6851ef72` | 14 | 11.0.0.9136 | `35b05b2d-8a42-4176-b5c4-2bce8f834911` |
| `samples/hwp3-sample5-hwp5-v2018.hwp` | `2a958c6559d704726aa2a1d8b4b97755ce0e6882cb683fc234a8d02dff183a9a` | `hwp3-sample5-hwp5-v2018-2020.pdf` | `f45e6028a65857977a892f0eb1b333b4c14c71dbb99dd936852f248b450254a8` | 64 | 11.0.0.9136 | `81765134-c46b-4f69-b2c2-834e05f91178` |
| `samples/hwp3-sample5-hwp5.hwp` | `2a958c6559d704726aa2a1d8b4b97755ce0e6882cb683fc234a8d02dff183a9a` | `hwp3-sample5-hwp5-2020.pdf` | `7f83b636a81a2abd6cf24c34631d0298d04d69f73fe0480a7b2c44fb2e5ba458` | 64 | 11.0.0.9136 | `7e1ee79c-3ba6-482b-bf92-e555dec81274` |
| `samples/hwp3-sample5-hwp5-v2024.hwp` | `510b5ee95a29c6e2c9e3d760c382c300c34035b401e42661adb7996afdd072e3` | `hwp3-sample5-hwp5-v2024-2024.pdf` | `85239cd60e24d1a19b7ea18dd1d7f66e6508c3e06b612c9bb915b3576c0f2462` | 64 | 13.0.0.3901 | `ee8f35a4-600b-4ab6-9423-98bc2a2afb6c` |
| `samples/issue6929/148776468_search_ad_terms_press_release.hwp` | `a05d313f34852e07305d8bf16223303d9cc00c3d9bdc0b5258f96dfcb347a43d` | `148776468_search_ad_terms_press_release-2020.pdf` | `bb760a6d0060927d48c00b4e9842c3c0c5c2d52a0a91c7d4e4c6f8b59df0c936` | 17 | 11.0.0.9136 | `9707bc56-ef6e-4125-9a8b-71c81ab941a2` |

## 미확보 입력과 판정 경계

`samples/hwp-3.0-HWPML.hwp`의 2020 요청(`1f8ec402-930e-4ec6-8d4c-40c9ecc99c0b`)은 `HwpEditorFrame.SaveAsDocument returned false for HWP`로 실패했다. PDF를 확보한 것으로 보고하지 않는다.

이 자료는 독립 기준 확보 증거이며 시각 검증 통과 증거가 아니다. 동일 원본의 전쪽 Native/fresh WASM TSV, 쪽수·내용 보존 및 대표 영역의 직접 판독이 남아 있다. 원본 v2018과 기본 hwp5 파일은 바이트가 같지만 자동 수집 경로를 구분해 보존한다. 로그·TSV·중간 JSON은 ignored output에만 둔다.
