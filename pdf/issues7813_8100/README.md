# #7445 잔여 이슈의 한컴 Print 기준 PDF

2026-11-28 KST에 이슈 원본을 한컴 Office PDF 프린터로 쪽당 한 장씩 출력했다. 인쇄 방식은 모두 `0`이다.
#7821의 원본 스크립트는 빈 `OnDocument_New` 함수가 있는 기본 스크립트이며 #7922의 원본 스크립트는 비어 있다.
두 파일은 Print 작업에서만 스크립트 실행을 끄고, rhwp에서는 아래 SHA의 원본을 검사한다.

| 이슈·형식 | 원본과 SHA-256 | Print PDF와 SHA-256 | 쪽수 | 한컴 버전과 전처리 | 작업 ID |
| --- | --- | --- | ---: | --- | --- |
| #7813 HWPX | [issue1949_giant_cell_nested_tables_perf.hwpx](../../samples/issue1949_giant_cell_nested_tables_perf.hwpx) `fc6e5f156de470dfbb14aab392389491720ee7fb1bf6f03fe9a018e93b420c65` | [7813-hwpx-original-2024.pdf](7813-hwpx-original-2024.pdf) `2ed78035ef52a5a130fe730b10170355ca8997643209483b09d464d4629e614b` | 115 | 13.0.0.4225 · `none` | `91258dc4-99e2-47a0-a76f-cab57f2d3237` |
| #7821 HWP | [coanchored_float_tac_order.hwp](../../samples/issue5807/coanchored_float_tac_order.hwp) `15e37d4e8139f8cb494f882f6d7423fe8d7ae991b51e238a23f5388699e8ef8a` | [7821-original-2024.pdf](7821-original-2024.pdf) `3af699859cd40087a0afa504903e0cc1794e081b415e6ea1a3213d730263e684` | 3 | 13.0.0.4225 · `scripts_disabled` | `a2d78497-85c9-4105-9d03-3290183106db` |
| #7822 HWPX | [3232693_employment_support_criteria.hwpx](../../samples/issue6025/3232693_employment_support_criteria.hwpx) `b53e9a83a79903b9c7cf69521519dcdf7b128348ac26ec21e6ac7b8fc6c0e441` | [7822-original-2024.pdf](7822-original-2024.pdf) `1ae214cf4c3b2a52e14086fe4a4099604d900a253bfb5df11b255b438091e44d` | 4 | 13.0.0.4225 · `none` | `3f0e5753-eb04-4a10-8732-df095f6a30a7` |
| #7834 HWPX | [22037757-chuncheon-personnel-rule-annex13.hwpx](../../samples/issue6854/22037757-chuncheon-personnel-rule-annex13.hwpx) `1d2ae7a559189f73d41f380da59b03aba41102511a69fbf67514141362cb962f` | [7834-original-2024.pdf](7834-original-2024.pdf) `551c9c7463b8c85cbab19659336d8a19ed7a08a17310bd4ac375eaac9fdf1709` | 15 | 13.0.0.4225 · `none` | `6041b211-89a9-4126-886d-99dc5cf71180` |
| #7922 HWP | [1192000-202100017-policy-research-report.hwp](../../mydocs/pr/assets/issue7922/trailing-tab-regressions-20261015/1192000-202100017-policy-research-report.hwp) `fd0b95cb4239b08e2ab9130b6b697af56dda379f1c9029dbf5f5a0b97af5ceee` | [7922-original-2020.pdf](7922-original-2020.pdf) `5bc17fa4681c284009d6657446b18cef334a7cfaea26537b64ba618e52cc8564` | 1 | 11.0.0.9172 · `scripts_disabled` | `f60014f9-5940-498f-94df-49289911b3d7` |

#7955는 동일 입력 SHA가 기록된 기존 [한컴 Office 2020 Print 근거](../issue7642-failed-regression-print/README.md)를 사용한다.
