# #8975 데스크톱 Print 대조에서 확인한 기존 간격 차이

2026-11-13. Native와 fresh 최적화 WASM Canvas를 같은 Desktop Print와 직접 대조했다.

원 PR의 웹 인쇄본과 별도로 동일 입력을 Windows Hancom 2020 `11.0.0.9136`의
MCP Print(`pdf_print_method=0`, `hancom2020_pdf_driver_one_up`)로 출력했다.
CellBreak·RowBreak 모두 2쪽이며 행별 쪽 소속, 글줄 순서, 누락·중복 없음은 주장과 같다.

| 입력 | Native p1 / p2 raw | WASM p1 / p2 raw | 2쪽 표 뒤 문단의 검은 픽셀 윗변: PDF / Native / WASM(96dpi) |
| --- | --- | --- | --- |
| e-cellbreak | 100.00 / 94.62% | 100.00 / 94.90% | 410 / 418 / 417px |
| f-rowbreak | 100.00 / 93.46% | 100.00 / 93.79% | 303 / 311 / 310px |

같은 crop의 x 범위·글자 높이는 같고 표 아래 본문이 Native에서 8px, WASM에서 7px 아래에 있다.
표 괘선·내용은 보존됐으며 겹침·누락은 없다. 글꼴 모양만의 차이로 처리하지 않는다.
제품 Rust source는 최신 devel `180a4ec972`와 후보 `ea627d6017` 사이에 같으므로
이 PR이 만든 새 회귀는 아니다. 기존 웹 인쇄본의 100%를 데스크톱 Print의 100%로 재인용하지 않는다.

원본 SHA-256:

- e `e566e14f5a0b4a4b8dbebefc6ab3d233710030ab7f53ee1d7307442ddf54f141`.
- f `904520801739be5795f0dd64dc907367a15f131ab4b4a985439724fed0936e4b`.

새 Desktop Print SHA-256:

- e `a144223cc49c6be43027a5b95e969867dd6ff455115030351271d091183a28c1`.
- f `a6f8ffe6c730ac30812dc3a52f8aa6a1585d92e0f118231705c2574624e397b9`.

최종 TSV·대표 PNG는 [검증 원장](assets/sacru2red-20261113/README.md)에, Desktop Print는
`pdf/issue7620/{e-cellbreak-row-moves-next-page,f-rowbreak-row-splits-at-line}-2020.pdf`에 보존한다.
현행 직접 판독 규칙에 따라 수치가 90% 이상이라는 이유만으로 이 기존 차이를 면제하지 않는다.
#8951의 footer baseline 제거 판정과는 구분한다.
