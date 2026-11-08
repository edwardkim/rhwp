# #8028 / #7994 직렬화 변경의 독립 인쇄 재검토

원 source `2d0fc147b5bdc42a2b316c1975ddc6fa8535d110`은 통합 commit
`e9afaf7ef4c2c2e388ab280c954e031fe3be3f92`으로 출처를 보존했다.
아래는 수정 구현의 기대값을 재인용한 검사가 아닌 실제 독립 한컴 인쇄다.

| 같은 한컴 2020 소비자 | 쪽수 | 원본 |
| --- | ---: | --- |
| 원본 HWP | 24 | `samples/HWP5-nopassword-123456.hwp` |
| devel runtime의 HWPX 직렬화 | 24 | [기준 직렬화](base-spacing.hwpx) |
| 보정 전 후보 `65ef09ad8`의 HWPX 직렬화 | 27 | [보정 전 직렬화](current-spacing.hwpx) |
| 보정 후 `9fa90f2e5`의 HWPX 직렬화 | 24 | [보정 후 직렬화](restored-spacing.hwpx) |

원본 저장 제품에 맞춘 한컴 2024 Print도 24쪽이다. 후보 HWPX는 writer가
한컴 2020 저장 메타데이터를 쓰므로 한컴 2020 Print를 기준으로 확인했다.
동일 소비자에서도 24→27쪽이므로 제품 버전 차이만으로 설명할 수 없다.
마커를 이해하는 rhwp의 재열기만 24쪽으로 일치하는 결과는 한컴 소비자 보존의 증거가 아니다.

PDF는 `pdf/issue7994/`에 두고 입력·PDF 해시, 실제 제품/빌드·Print mode·전쪽 TSV를
파일별 검증 표에 연결한다. 로그·MCP job receipt와 중간 JSON은 `output`에 둔다.
이 원 PR의 완료·이슈 종료를 주장하지 않으며 메인터너는 이 새 인쇄 회귀를 제거하는
보정을 별도 commit으로 기록한다. source patch·반례와 독립 Print는 이관 이슈에서 보존한다.

## 최신 devel에서 달라진 전제

[#8064](https://github.com/edwardkim/rhwp/pull/8064)에 포함된
[`b9bb438d99df6137c2ec6e8fb09d8d98eb2cb681`](https://github.com/edwardkim/rhwp/commit/b9bb438d99df6137c2ec6e8fb09d8d98eb2cb681)은
HWP3 계보를 가진 HWP5에서도 원본 문단 간격을 IR에 보존하도록 바꿨다.
이 변경은 기준 runtime `efc52afb7`에도 이미 들어 있다. 계보 표시와 문단 간격의
저장 단위는 별도 계약이며, style resolver의 해소와 원본 IR을 혼동하지 않는다.

#8028은 파서가 IR의 위·아래 간격을 반감한다는 전제에서 두 배 복원을 추가한다.
현재 기준에서는 그 전제가 성립하지 않아 `hwp3_variant_hwpx_export_writes_stored_paragraph_spacing`도
원본 raw와 IR 사이의 반감 증인을 찾지 못했다. 이 검사 실패와 같은 소비자의 24→27쪽
독립 인쇄를 함께 근거로 삼는다. 최신 devel의 동작을 보존하는 보정이 필요하며,
이 결과만으로 #7994의 다른 문서까지 해결됐다고 판정하지 않는다.

## 최신 기준 보존과 정상 속성 검사

두 배 복원과 marker `2`의 재반감 경로를 공통 저장·열기 경로에서 제거했다.
원 source commit을 유지하고 별도 메인터너 보정으로 기록한다. 이전 검사 원문은
이 디렉터리에 바이트 그대로 보존한다.

활성 검사는 기존의 원본 PARA_SHAPE raw→외부 소비자 IR 간격 비교를 유지했다.
현재 기준에서 성립하지 않는 반감 전제와 Native끼리의 쪽수·컷 대칭 검사는 분리했다.
원본의 실제 비영(非零) 간격으로 두 배 변경을 검출하고, 기존 계보 표지 `1`과 재열기도
같은 저장 속성을 보존하는지 검사한다. 이는 문단 속성의 저장 계약이며 실물 배치의
시각 통과나 고정 쪽수로 보고하지 않는다.

보정 전 `5d81dc62d`에서는 실제 외부 소비자 IR이 `(0, 2272)`인데 원본 raw는
`(0, 1136)`인 오류로 검사가 실패했다. 0건 실행은 증거에서 제외했고, 파생 harness를
재생성한 뒤 1건이 실제 실행돼 이 조건에서 실패한 로그를 보존했다.
보정 후 `9fa90f2e5be7f3a794ef56998c99e07fa90cbe03`에서 같은 활성 저장 계약이
실제 1건 실행·1건 통과했다(Nextest `59f079fe-63b4-4d86-a514-585f7ae15684`).
보정 후 CLI로 내보낸 위 HWPX의 SHA-256은 `c2808f8811e24b512b3a818ae9a980732d8030fc0b97f7fabd376cb60fbc054b`다.

독립 한컴 2020 `11.0.0.9136`, PrintMethod=0, 1-up에서 이 저장본은 24쪽으로 복원됐다.
PDF는 `pdf/issue7994/HWP5-nopassword-123456-restored-spacing-2020.pdf`,
SHA-256 `be5c026948386b9cdb3c2c2e0ba20814d689b2cef7b6db5e57888d06faba464e`다.
기존 원본 PDF는 다시 만들지 않았다. 기존 base HWPX와 보정 후 파일의 간격·나머지
ZIP entry는 같고, `section0.xml`의 `hyphen`→`hypen` 태그 한 곳만 별도 통합 변경으로
달라 새 저장본의 인쇄만 수행했다. 이전 27쪽 파일과 실패 검사도 그대로 보존한다.

이는 최신 기준의 저장 계약과 같은 제품에서의 쪽수 보존 검증이다. 전쪽 Native/fresh
WASM 시각 통과나 #7994의 다른 원본 해결을 뜻하지 않는다.
