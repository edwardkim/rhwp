---
kind: evidence
status: active
canonical: mydocs/manual/pr_review/visual_fixture_evidence.md
last_verified: 2026-10-12
---

# #7702 검증 입력의 한컴 정상 저장 근거

원래 네 합성 입력은 `BodyText/Section0` 한 스트림의 첫 문단과 마지막 문단에
구역 정의를 중복 포함했다. Native는 1쪽, 독립 Print는 2쪽이었다. 이 폴더에
실패 원본 바이트를 보존하며 활성 samples 검증에는 한컴 Office 2020으로 정상 저장한
대조 입력을 사용한다. 원본 입력의 렌더링 문제를 해결했다고 주장하지 않는다.

정상 저장본은 `Section0`/`Section1` 각각의 첫 문단에 구역 정의를 갖는다. 문단 수와
공백 제외 본문을 보존했다. 정상 저장본을 독립 Print(method0)한 전체 8쪽과 기존
`pdf/issue7702/*-2020.pdf`는 96dpi RGB 픽셀이 모두 동일하다. Native/fresh WASM 전체
페이지 최저는 99.56697% 이상이었다. 최종 통합 head에서는 필요한 검증을 다시 고정한다.

| 입력 | 실패 원본 SHA-256 | 정상 저장 입력 SHA-256 | 정상 저장본 Print job |
| --- | --- | --- | --- |
| half_space_ties | `49c6d13a3a23d4daefe2f19b079497c718748904d2af5031004faebcf77f61f1` | `68eb54ca9ed43acb33d39602c7b39b3b622f46f41cbf9a60a271f112a7f600d5` | `3ef89eb8-1efb-429a-980a-e7dd1fa06c87` |
| ratio_steps | `258528534641b413f162005f31e9381a8cb4381027ba35ef5ae19ce7ab1280a7` | `80ff2b2071de772caecad5d04bc8a96738ba621fb496d4f6c348b2293c00a548` | `a01dece8-b4b0-4ed9-8e85-d5d22991d6bc` |
| relative_size_letter_spacing | `cd41e3809c16526044918b37d624dd822816acdd8d8a64567fa48d2003d1de93` | `435f0500fbdd18a68a66d5468dd8ce355c4d0cf90d0f7e02fab0b4048332ecbe` | `1b55e555-3a16-4843-9894-6e77000f41fe` |
| size_ratio_relsize | `c4e8baf878f46a802be6eec31607d1a36e541cae1611eaa827c46be096f1435c` | `19d245828414a15e5cdc516feab055329a4d6c17e66116827e1823f040e19940` | `a8b365f4-895f-4cd3-8f7a-06183980ee0d` |

한컴 저장/Print는 `mydocs/manual/mcp_hwp2024Convert_usage.md`의 client로 engine2020을
명시해 수행했다. endpoint/token은 기록하지 않았다. 개인 글꼴 파일도 저장소에 넣지 않는다.

로컬 상세 증거는 ignored `output/pr-review/planet6897-followup-20261011/`의
`pr7805-source-section-definition-placement.json`, `pr7805-hancom-saved-structure.json`,
`pr7805-normalized-content-preservation.json`, `pr7805-original-normalized-print-raster-equality.json`,
`pr7805-normalized-native-results.json`, `pr7805-normalized-fresh-wasm-results.json`이다.
이 폴더의 원본은 검증 실패/입력 생성 근거이며 활성 회귀의 fixture가 아니다.
