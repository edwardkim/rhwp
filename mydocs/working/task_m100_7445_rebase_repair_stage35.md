# #7445 보정35 — 문서 대체 필드의 이름 선택과 제목 소유

Issue: #7445, 기능 보존 #8064/#7642.

한컴2020/2024 독립 Print10조건으로 주 글꼴과 대체 필드의 영문 HCR 선택 차이를 확인하고 공통 조회·공급·측정·paint 경로를 보정했다. 원본/직렬화·내장·명시 사용자 환경·주 글꼴 HCR은 유지한다. 지침/remote 변경은 없다.

[원인·기존 해결 계보·독립 Print·전쪽 TSV·review/overlay·중요 ±2px 회귀·실제 producer·잔여](../pr/assets/issue7445/completed-20261009/stage35-20261021/README.md).

온새미로 두 원본 각46쪽의 Native 최저91.43430%,fresh94.45835%,90% 미만0이다. HWPX Native의 기존 하한 대비 하락13쪽·base 대비11쪽이 해소됐고 두 형식 모두 Native/fresh 고정base 대비0.1pp 하락0이다.46쪽 양쪽100%·직접 판독 후 중요한 제목의 독립 시작/끝 ±2CSSpx와 내용 소유를 추가했다. 같은 최종 새3검사는 실제3FAIL→3PASS, 정식 자동 suite5실행5PASS다. 기존16PASS·registry/projection42PASS와 fmt/manifest/unit tiers도 확인했다.

추가 교육30조건과 가용 primary4조건의 실제 Native/fresh 영향 검증은 진행 중이며 교육18쪽 -0.10354pp 및 WMF 등 기존 잔여를 계속 보정한다. 전체 회귀·필수 gate·Actual36 미완료이며 PR 준비 완료가 아니다.
