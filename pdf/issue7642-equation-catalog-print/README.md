# #7642 수식 카탈로그의 독립 Print 진단 자료

`src/renderer/equation/fixtures/catalog.tsv`의 259개 이름·스크립트와 일치하는
기존 Stage 117 입력을 한컴 2020으로 재저장한 뒤 별도 Print PDF로 출력했다.
baseUnit은 1500HU이며 원본 스크립트별 대응은 ignored Stage 117의
`input-provenance.json`에 보존했다. 이 파일은 수식 엔진 golden의 자동 갱신 근거가 아니다.

- HWP SHA-256: `5821913b3311590a0a724f93a8f0c5d84966c06bbf42cdd3d8e30fe5664ad3b5`
- PDF SHA-256: `376ccde19b057c1dddd8b1548d7e5ac048e703c28d94c250467e8844d32680dc`
- HWP 재저장 job: `d99fafd2-670f-49da-b7cb-f49749eaa257`
- Print job: `ab402477-dc50-418d-931a-4b8811b56084`
- Hancom: `11.0.0.9136`, engine `2020`, PrintMethod `0`,
  `hancom2020_pdf_driver_one_up`, 18쪽

Stage 159 Native 전체 18쪽 중 11쪽이 90% 미만이며 최저는 17쪽 33.20773%다.
14쪽은 뒤 수식의 세로 위치 차이가 있고, 17쪽은 일부 외부 구문(`BEGIN`, `QUAD`,
`OPERATORNAME`, `TEXT`, `STACKREL`)의 해석 차이도 있다. 점수와 구문 대응을 각각
확인하며 이 자료를 통과한 회귀 fixture로 보고하지 않는다. 한컴 구문과 엔진 확장
구문의 차이를 무시해 기존 기능을 제거하거나 golden을 일괄 갱신하지 않는다.

현재 catalog.tsv SHA-256은
`a67deff726e27a625f97d69e01d9258cf4473433f5c2fe4f78752b664fffde28`이다.
실행 로그·전체 TSV는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage161-catalog-whole-native/`,
대표 14·17쪽 비교 PNG는 같은 위치의 `stage162-catalog-detail-review/`에 있다.
Native/fresh WASM 전체 90%와 확대 판독을 충족하기 전에는 신규 렌더링 회귀나
기존 기대값 변경의 시각 통과 근거로 사용하지 않는다.
