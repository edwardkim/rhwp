# 최신 리베이스 제품의 Native 영향 경계 대조

제품 source `ba2ebfd34a4ebf402bce8e002254b672a09ef764`, base `f529dc92d4d172bf32390d7749743a45d9e41bfc`.
최소 글꼴 Rust와 구분해 시스템 글꼴 전체·해당 행의 기존 명시 대체 설정으로 측정했다.

**10입력·43쪽**, 전쪽 최저 raw **94.92448%**. 90% 미만·누락·쪽수·해시 오류 0건.
검증 시작 전부터 고정한 c183 또는 세로쓰기 보정 37e의 같은 원본·독립 Print와 비교해
0.1%p 초과 하락은 0쪽이다. canonical TSV의 표시 두 자리와 원시 manifest의 다섯 자리 raw를
서로 대조했으며 0.1%p 판정은 원시 raw를 사용했다. TSV 표시값을 임의로 다시 포맷하지 않았다.

이 문서는 현재 보정 경계의 Native 완료만 입증한다. 새 base 자체와의 대조·fresh WASM·전체 Rust·
필수 편람의 최종 완료나 승인으로 확대하지 않는다. 새 base 자체 대조는 별도 실행 중이다.

| 실제 입력 · 독립 Print | 쪽수 · 최저 raw | 원본 Native TSV |
| --- | --- | --- |
| [vertical_fit.hwpx](<../../../../../samples/issue8470/vertical_fit.hwpx>) · [vertical_fit-2020.pdf](<../../../../../pdf/issue8470/vertical_fit-2020.pdf>) | 5쪽 · 100.0% | [TSV](<candidate/native/pr8475-vertical_fit-8a319181.tsv>) |
| [vertical_oversize.hwpx](<../../../../../samples/issue8470/vertical_oversize.hwpx>) · [vertical_oversize-2020.pdf](<../../../../../pdf/issue8470/vertical_oversize-2020.pdf>) | 5쪽 · 100.0% | [TSV](<candidate/native/pr8475-vertical_oversize-30055ea5.tsv>) |
| [42565_입법예고문.hwp](<../../../../../samples/issue8470/42565_입법예고문.hwp>) · [42565_입법예고문-2020.pdf](<../../../../../pdf/issue8470/42565_입법예고문-2020.pdf>) | 16쪽 · 94.92448% | [TSV](<candidate/native/pr8475-42565_입법예고문-2279178b.tsv>) |
| [scaffold_outline_a.hwpx](<../../scaffold_outline_numbering/scaffold_outline_a.hwpx>) · [scaffold_outline_a-2024.pdf](<../../../../../pdf/issue8540/scaffold_outline_a-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/native/pr8540-scaffold_outline_a-649191af.tsv>) |
| [scaffold_outline_b.hwpx](<../../scaffold_outline_numbering/scaffold_outline_b.hwpx>) · [scaffold_outline_b-2024.pdf](<../../../../../pdf/issue8540/scaffold_outline_b-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/native/pr8540-scaffold_outline_b-82b08f33.tsv>) |
| [3200477_icao_procedure.hwpx](<../../../../../samples/issue6029/3200477_icao_procedure.hwpx>) · [issue6029-3200477_icao_procedure-2024.pdf](<../../../../../pdf/issue6029-3200477_icao_procedure-2024.pdf>) | 1쪽 · 97.97401% | [TSV](<candidate/native/pr8475-3200477_icao_procedure-bc001901.tsv>) |
| [levels.hwpx](<../../issue_8425/probe/levels.hwpx>) · [levels-2024.pdf](<../../../../../pdf/issue-8425/levels-2024.pdf>) | 1쪽 · 100.0% | [TSV](<candidate/native/pr8433-levels-d014d88d.tsv>) |
| [gangseo_exempt_certificates.hwp](<../../../../../samples/issue8532/gangseo_exempt_certificates.hwp>) · [issue8532_gangseo_exempt_certificates-2020.pdf](<../../../../../pdf/issue8532_gangseo_exempt_certificates-2020.pdf>) | 6쪽 · 99.84567% | [TSV](<candidate/native/pr8547-gangseo_exempt_certificates-e5b4dd3b.tsv>) |
| [156500952-k-startup-league.hwpx](<../../../../../samples/issue8510/156500952-k-startup-league.hwpx>) · [156500952-k-startup-league-2022.pdf](<../../../../../pdf/issue8510/156500952-k-startup-league-2022.pdf>) | 5쪽 · 99.25535% | [TSV](<candidate/native/pr8512-156500952-k-startup-league-4c965793.tsv>) |
| [scaffold_table_footnote.hwpx](<../../../../../samples/issue8520/scaffold_table_footnote.hwpx>) · [scaffold_table_footnote-2024.pdf](<../../../../../pdf/issue8520/scaffold_table_footnote-2024.pdf>) | 2쪽 · 99.1045% | [TSV](<candidate/native/pr8535-scaffold_table_footnote-a5d8c38e.tsv>) |

실제 측정 명령과 페이지별 TSV의 전체 명령은 [재실행 가이드](../visual-reproduction.md)를 따른다.
제품 Native SHA-256: `296a0d3f68dcb29f5bd91f3f5411bb78e42c53d04a51f526a3473281ba090cc6`.

[최신 문제 쪽 review·별도 overlay와 직접 판독](native-vertical-fit/README.md)을 연결한다.
PR #8512 외곽선·PR #8535 각주 번호의 기존 차이도 같은 입력으로 포함했다. 이 두 항목은
점수 통과와 별도로 실제 그림에서 원본 계약을 확인한다. 아직 그 차이까지 해결했다고 선언하지 않는다.

로컬 실행 원장: `output/pr-review/planet6897-20261103/logs/rebase8609-native-scope-own-audit.json`.
이 원장은 중간 JSON이므로 제출 자료에 복사하지 않는다.
