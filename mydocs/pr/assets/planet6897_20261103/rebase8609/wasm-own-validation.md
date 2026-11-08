# 최신 리베이스 fresh WASM의 영향 경계 대조

제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
실제 빌드 head `dc703ef8180e0c410aa754011c34660e33e68480`; snapshot의 제품 입력 해시와 같다.
JS SHA-256 `bb9be9d806756697eddee5d185e80e9256bd0d97947d78a94777222100f12393`.
WASM SHA-256 `3a9125c0a3fa65e1ebbe45abfd0e2070248fbb96d1f5e083d6e3108baa62ed61`.

**10입력·43쪽**, 전쪽 최저 raw **96.96022%**. 90% 미만·누락·쪽수·입력/PDF·제품 해시 오류 0건.
직전 c183 또는 세로쓰기 보정 37e의 같은 입력·Print와 정밀 raw를 대조해 0.1%p 초과 하락은 0쪽이다.
canonical TSV 두 자리 표시가 실제 manifest 다섯 자리 raw의 표시와 일치하는지도 확인했다.
시각 측정은 시스템 글꼴 전체와 기존 명시 대체 계약을 사용했고, Rust 최소 글꼴 환경과 구분한다.

| 실제 입력 · 독립 Print | 전체 쪽수 · 최저 raw | 실제 fresh WASM TSV |
| --- | --- | --- |
| [vertical_fit.hwpx](<../../../../../samples/issue8470/vertical_fit.hwpx>) · [vertical_fit-2020.pdf](<../../../../../pdf/issue8470/vertical_fit-2020.pdf>) | 5쪽 · 100.0% | [TSV](<candidate/wasm/pr8475-vertical_fit-8a319181.tsv>) |
| [vertical_oversize.hwpx](<../../../../../samples/issue8470/vertical_oversize.hwpx>) · [vertical_oversize-2020.pdf](<../../../../../pdf/issue8470/vertical_oversize-2020.pdf>) | 5쪽 · 100.0% | [TSV](<candidate/wasm/pr8475-vertical_oversize-30055ea5.tsv>) |
| [42565_입법예고문.hwp](<../../../../../samples/issue8470/42565_입법예고문.hwp>) · [42565_입법예고문-2020.pdf](<../../../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | 16쪽 · 96.96022% | [TSV](<candidate/wasm/pr8475-42565_입법예고문-2279178b.tsv>) |
| [scaffold_outline_a.hwpx](<../../scaffold_outline_numbering/scaffold_outline_a.hwpx>) · [scaffold_outline_a-2024.pdf](<../../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/wasm/pr8540-scaffold_outline_a-649191af.tsv>) |
| [scaffold_outline_b.hwpx](<../../scaffold_outline_numbering/scaffold_outline_b.hwpx>) · [scaffold_outline_b-2024.pdf](<../../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/wasm/pr8540-scaffold_outline_b-82b08f33.tsv>) |
| [3200477_icao_procedure.hwpx](<../../../../../samples/issue6029/3200477_icao_procedure.hwpx>) · [issue6029-3200477_icao_procedure-2024.pdf](<../../../../../pdf/issue6029-3200477_icao_procedure-2024.pdf>) | 1쪽 · 98.74661% | [TSV](<candidate/wasm/pr8475-3200477_icao_procedure-bc001901.tsv>) |
| [levels.hwpx](<../../issue_8425/probe/levels.hwpx>) · [levels-2024.pdf](<../../../../../pdf/issue-8425/levels-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/wasm/pr8433-levels-d014d88d.tsv>) |
| [gangseo_exempt_certificates.hwp](<../../../../../samples/issue8532/gangseo_exempt_certificates.hwp>) · [issue8532_gangseo_exempt_certificates-2020.pdf](<../../../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf>) | 6쪽 · 99.87175% | [TSV](<candidate/wasm/pr8547-gangseo_exempt_certificates-e5b4dd3b.tsv>) |
| [156500952-k-startup-league.hwpx](<../../../../../samples/issue8510/156500952-k-startup-league.hwpx>) · [156500952-k-startup-league-2022.pdf](<../../../../../pdf/issue8510/156500952-k-startup-league-2022.pdf>) | 5쪽 · 99.88439% | [TSV](<candidate/wasm/pr8512-156500952-k-startup-league-4c965793.tsv>) |
| [scaffold_table_footnote.hwpx](<../../../../../samples/issue8520/scaffold_table_footnote.hwpx>) · [scaffold_table_footnote-2024.pdf](<../../../../../pdf/issue8520/scaffold_table_footnote-2024.pdf>) | 2쪽 · 99.14798% | [TSV](<candidate/wasm/pr8535-scaffold_table_footnote-a5d8c38e.tsv>) |

두 경로의 두 문제 입력은 각각 전5쪽 100.00%다. Native의 같은 범위는 [별도 대조](native-own-validation.md)에 있다.
대표 쪽은 최신 제품의 review와 별도 overlay를 실제로 열고 별도 기록한다. 이 TSV 성공만으로 직접 판독이나
새 base 자체 대조·전체 Rust·통합 최종 승인까지 끝났다고 선언하지 않는다.

## 재실행

[canonical 명령·입력·Print·compat·글꼴 계약](../visual-reproduction.md)을 따른다.
96 DPI·Print profile·색상 경계 보정 전 2px raw다. 원본·PDF·metric·허용값을 바꾸지 않았으며 글꼴을 재배포하지 않는다.
원시 JSON·로그는 로컬 output에 보존하고 공식 제출 자료에는 복사하지 않는다.
