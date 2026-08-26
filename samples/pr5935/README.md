# PR #5935 저장 버전 판별 증적

한컴오피스 2010, 2018, 2022, 2024에서 각각 새로 저장한 HWP5 원본이다. 이 파일들은
`info_hancom_save_version_contract`에서 `lastSavedWith`의 제품명과 전체 `revisionNumber`를
검증하는 재현 fixture다.

| 파일 | `revisionNumber` | `lastSavedWith.product` | SHA-256 |
| --- | --- | --- | --- |
| `test-2010.hwp` | `8.0.0.466` | `hancom-office-2010` | `a448aea8bb18299ab2e391ede290dd46ebe65174b543a5b8551ed7447db5a635` |
| `test-2018.hwp` | `10.0.0.5060` | `hancom-office-2018` | `7abedd9239954e40bbf46e834e2d17caa81adf35acd176a49965eed87981d465` |
| `test-2022.hwp` | `12.0.0.4605` | `hancom-office-2022` | `5df570e17e3ba0a716da2ad2f149c963a778cfa9e10b905f00218b9cf2b28a70` |
| `test-2024.hwp` | `13.0.0.3379` | `hancom-office-2024` | `91a4f0cec582df4d21c2eced423482e6307aafd998639ac32c57f55ae56e740e` |
