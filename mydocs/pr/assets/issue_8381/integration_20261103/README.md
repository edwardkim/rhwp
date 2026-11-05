# PR #8408 PDF editable 재현 입력

세 HWPX는 아래 커밋 원본 PDF를 `rhwp import-pdf <PDF> <HWPX> --layout editable`로
각각 가져온 결정적 출력이다. 원본과 변환본을 함께 보존해 검토자가 동일 입력의 전체 쪽을
다시 측정할 수 있게 한다. 시각 대조의 기준은 **가져오기 전 원본 PDF**다.

| PDF | PDF SHA-256 | HWPX | HWPX SHA-256 |
| --- | --- | --- | --- |
| `samples/21_언어_기출_편집가능본-2010.pdf` | `609d56421009ccab5a50cb2a8f4e78af4d0961de2e7ae85d811905e8434ac251` | `editable-2010.hwpx` | `54df47aa980966679a67f579916972d26d6147a08d327368398abc2552d84412` |
| `samples/21_언어_기출_편집가능본-2020.pdf` | `a1a8b0ffd6ae9b23627f7b1bf58bbdfdc0714a369713cc057aa836c75b143f1e` | `editable-2020.hwpx` | `f1929a65e3901b90087fe9ccb4b0e446ceed32926b1479d4307e22474d23f49e` |
| `samples/21_언어_기출_편집가능본.pdf` | `6bc54661a73af788d437463e000f76652b1325b44607e58443e7391d23e2db83` | `editable-plain.hwpx` | `7a54ea971e88cb9e4d946288beb28a131439640f8015dbdbb44a64189b6f9e7d` |

2026-11-03 검증 당시 제품 CLI는 통합 코드
`0884eafc7b2c34fae20a765109ff314514175e4b`에서 빌드했다. 세 HWPX의 전체
쪽수는 각각 16·15·15쪽이다. 독립적인 한컴 Print 저장본과 비교하는 범위는 원 PR의
별도 PDF·입력 쌍으로 구분한다.
