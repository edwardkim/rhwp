# sample10 보호 문서 Native 전쪽 대조

`hwp3-sample10.hwp`를 macOS Native로 출력하고 데스크톱 한컴 2020의 Hancom PDF Print와 비교했다. 96 DPI, 2022 호환, 2px 이웃 raw 실루엣 기준으로 763쪽을 모두 확인했다. 최저 99.57%이고 741쪽이 100.00%다. 후보 TSV 전체가 정본 출처 TSV와 바이트 단위로 일치하며 보호값 하락·상승은 각각 0쪽이다. [후보 전쪽 TSV](candidate.tsv) · [입력·PDF·생산물 SHA와 판정](validation.json)

| 쪽 | 후보 / 보호값 | 직접 판독 |
| ---: | ---: | --- |
| 474 | 100.00% / 100.00% | 제목 아래 HNC 이중선의 끝이 기준 PDF와 같은 위치에 있다. [review](review-474.png) · [overlay](overlay-474.png) |
| 757 | 99.57% / 99.57% | 상자형 절차도와 본문은 같은 쪽에 있다. 절차도의 점선과 문자가 겹치는 부분에 미세한 차이가 보인다. [review](review-757.png) · [overlay](overlay-757.png) |

수용 기록의 글꼴 파일 수는 66개이고 이번 Mac 글꼴 인벤토리에는 1,219개가 있다. 수용 당시 파일의 개별 SHA 목록을 확인할 수 없어 동일한 글꼴 세트를 사용했는지는 미확인이다. 같은 글꼴 공급 조건으로 고정 `devel`을 별도 실행했다. 그 결과도 763쪽 전부의 표시값이 후보와 같고 [전쪽 TSV](candidate.tsv)의 바이트와 SHA-256까지 일치한다. [생산물과 조건 대조](validation.json) [fresh WASM 전쪽 결과](../protected-sample10-wasm/README.md)
