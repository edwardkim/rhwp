# #7642 PrEP 각주 예약 보정의 Native 진단 증적

- 생산 소스: `89285d012dba022d50989816267830be0101bd2d`
- Native binary SHA-256: `02162ae0ab251625edbfbb9259a877a7e7839ebf64ec1ae4b64904e49b2eb262`
- 원본: `samples/issue2006/1790387_prep_final_report.hwpx`
- 원본 SHA-256: `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`
- 독립 한컴 2024 Print: `pdf/issue2006/1790387_prep_final_report-2024.pdf`
- PDF SHA-256: `04b95a6e41420fb45934ce2ee5abd8cf6dac4ce12fd47977dacbe7fca28018a8`

Native 전체 140쪽의 최저 일치율은 91.75234%, 미달 0쪽이며 입력과 PDF 모두
140쪽이다. 각주 예약이 실제 배치의 공통 메트릭을 사용해 68쪽 본문 꼬리와
69~70쪽 표 조각 소유가 개선됐다. 아래는 새 코드의 직접 판독을 위한 대표 PNG다.
전체 TSV·실행 로그·페이지별 입력 해시는 ignored `output`에 보존하며 커밋하지 않는다.

| 쪽 | Native 관용 실루엣 일치율 | 대표 이미지 |
| --- | ---: | --- |
| 68 | 99.70727% | [review](native163-p068-review.png) |
| 69 | 99.94526% | [review](native163-p069-review.png) |
| 70 | 99.95329% | [review](native163-p070-review.png) |

69쪽은 앞 문단 꼬리·표 제목과 행 순서, 마지막 행의 해당 쪽 소유를 Print와
대조했다. 래스터의 선·글자 농도와 미세 형태는 자동 점수와 구분해 판독한다.
현재 fresh WASM은 미검증이며 이 자료만으로 신규 회귀·PR 준비·승인을 판정하지 않는다.
코드가 바뀌면 영향 범위를 재출력하고 제출용 증적을 갱신한다.

세부 생산·소비 추적과 현재 전체 Rust 검증 상태는
[Stage 163](../../../working/task_m100_7642_stage163.md)에 있다.
