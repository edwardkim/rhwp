# sample10 보호 문서 WASM 전쪽 대조

`hwp3-sample10.hwp`를 macOS에서 새로 빌드한 WASM으로 출력하고, 데스크톱 한컴 2020의 Hancom PDF Print와 비교했다. 96 DPI, 2022 호환, 2px 이웃 raw 실루엣 기준으로 763쪽 모두 완료했다. 최저 99.57%이고 745쪽이 100.00%다. 보호값 하락·상승은 각각 0쪽이며 후보 TSV 전체가 정본 출처 TSV와 바이트 단위로 일치한다. [후보 전쪽 TSV](candidate.tsv) · [입력·PDF·생산물 SHA와 판정](validation.json)

| 쪽 | 후보 / 보호값 | 직접 판독 |
| ---: | ---: | --- |
| 474 | 100.00% / 100.00% | 제목 아래 HNC 이중선이 기준 PDF와 같은 위치에서 끝난다. [review](review-474.png) · [overlay](overlay-474.png) |
| 757 | 99.57% / 99.57% | 상자형 절차도와 본문이 같은 쪽에 있고 보호값을 유지한다. [review](review-757.png) · [overlay](overlay-757.png) |

수용 당시에 나열된 80개 글꼴 중 동일 SHA의 66개를 확보해 사용했다. 나머지 14개는 현재 환경에서 같은 파일을 확보하지 못했으며, 명단을 검증 기록에 명시했다. 이 증적은 해당 공급 환경에서 최종 후보의 표시값을 재현한 기록이다. 같은 공급 조건의 고정 `devel`과 후보도 763쪽 전부 대조했다. 후보는 474쪽 이중선이 한컴 Print의 끝 위치에 맞아 **99.96% → 100.00%**이며 다른 762쪽의 표시값은 같다. 정본의 474쪽 하한은 100.00%다. [고정 기준 TSV](fixed-base.tsv) · [고정 기준 474쪽 review](fixed-base-review-474.png) · [고정 기준 474쪽 overlay](fixed-base-overlay-474.png) · [양쪽 생산물·조건 검증](validation.json)
