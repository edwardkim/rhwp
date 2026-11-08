# 수식 줄의 페이지 예산 대조군

두 수식 `x`와 `DMATRIX{ROOT 2 & 0 # 0 & ROOT 3}`을 같은 스타일로 두고 본문 예산만 변경했다. 기본 한컴 정상 저장 HWPX의 구역·글자/문단 스타일을 재사용했고 용지 크기·상단/머리말/꼬리말 여백은 보존했다. 하단 여백으로 본문 예산을 5700 / 6225 / 6750 HWPUNIT(96 DPI에서 76 / 83 / 90 CSS px)로 정했다. 이 단위는 입력의 용지 예산 설명이며 테스트 좌표 선택자가 아니다. 수식의 크기·baseline·LineSeg는 직접 정하지 않고 한컴 정상 저장으로 재발행했다.

독립 출력은 MCP `engine=2020`, 한컴 `11.0.0.9136`, PrintMethod 0, `hancom2020_pdf_driver_one_up`이다. 같은 정상 저장 HWP를 Print 입력으로 사용했다. 엔드포인트와 인증 정보는 기록하지 않는다.

| 파일 | MCP job | SHA-256 |
|---|---|---|
| page-budget-low-hancom-2020.hwp | 2178011b-b3b3-4eee-8b0a-f1e6f239637c | 45e0afbccf8b33dcc4b89ced40370f9fa74dbd6b37c824821b2298d9352af1ee |
| page-budget-low-hancom-2020.pdf | b734de2f-4699-4fdb-903a-f205f9779102 | eb84a8aaf2cee86813b1bf0d4f88a68f10c885c63cefa59683722a6fac1b368d |
| page-budget-middle-hancom-2020.hwp | 823f7453-84f9-44f0-8782-80f252dc0409 | 1236e783a9216bbf4af5dc90b21f283dc69378ca88377e7dc10dab1418043865 |
| page-budget-middle-hancom-2020.pdf | 1614ab3b-cb1e-4aae-984b-92f86d71cda3 | 450d16753af9537f5cb983c89e12e92686818df6dc4b4f6cdeff25642b28ee1f |
| page-budget-high-hancom-2020.hwp | 97d0e007-02cd-4edd-85f2-f5f20e738999 | bbe88a5a38440c38421d9ec624667dc62e462b619fc1c20d49e05e941bc25d4c |
| page-budget-high-hancom-2020.pdf | fb7703f7-74b1-4f01-a47d-3a87f53693b4 | 7dbe7144df2ed044d3a24da360c109e99c4df1ec86d2634f37ad6b2260c3d6dd |

## 관측 결과

코드 `785c2821c7276a506b635e16e6c5b53c3daf53ce`의 실제 Native render tree와 독립 Print에서 낮은 예산은 2쪽, 중간·높은 예산은 각각 1쪽으로 동일하다. 중간 예산에서 마지막 수식의 표시 영역은 들어가지만 전체 마지막 leading까지 수용하지는 못한다. 따라서 카탈로그 18쪽의 차이를 마지막 leading 예약 누락으로 설명하는 가설은 이 대조군에서 입증되지 않았다. 임의의 높이·예산 보정은 하지 않는다.

이는 페이지·문단 소속의 진단 증거이며 Visual Sweep의 90% 통과나 픽셀 일치를 뜻하지 않는다. 정식 회귀 fixture로 추가하지 않았다. 로그·출처 JSON·전체 native tree는 ignored `output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage180-page-budget-print/`에 있다.

입력 작성 도구에서 ZIP metadata를 여러 출력에 재사용한 오류로 첫 middle 변환이 실패했다. 작성 도구를 고쳐 모든 입력 CRC를 확인하고 실패한 middle 및 미실행 high만 정상 입력으로 재산출했다. 성공한 low 작업은 반복하지 않았으며 입력 SHA도 동일했다. 이 실패는 RHWP/한컴 결함 증거로 세지 않는다.
