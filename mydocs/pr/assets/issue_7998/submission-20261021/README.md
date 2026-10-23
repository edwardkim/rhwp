# #7998 제출 직전 실제 제품·저장 검증

제품 source는 `d1d3091c5`, 최종 테스트 head는 `168e96b86`이다.
제품 source에서 새 Native와 Docker WASM을 만들었고,
입력·출력의 바이트 수/SHA-256은 [files.tsv](files.tsv), producer SHA는
[기존 고정 집합의 producers.tsv](../final-p5/producers.tsv)에 기록했다.
빌드 후 기록·허가 비트 정리는 제품 코드 변경이 아니다. 실행 로그·서버 응답 원문은 공개하지 않는다.

## 실제 Studio 편집·저장과 한컴 Print

원본은 기존 `samples/issue7957/resource-limit.pdf`이며 사용자 제공 `reousrce_limit.pdf`와
바이트가 같다. 새 WASM을 실제 Studio에 로드해 첫 셀에 X를 넣고 직접 삭제한 뒤 HWP/HWPX를
저장·재열기했다. 양 형식의162쪽·첫 쪽274셀·문자 복원을 확인했다.
무편집 Native 저장본과 실제 편집 복원본은 section0.xml이 달라 같은 입력으로 취급하지 않았다.
여기의 `resource-restored.*`와 Print는 **실제 편집 복원본**이다.

[Print 출처](hancom-print.tsv)는 한컴2024 `13.0.0.3901`, 전처리none, Print method0,
원본 용지·100%·한 쪽씩 인쇄이며 양 형식 각각162쪽이다. 한컴2020의 새 전체 인쇄는 worker가
종료돼 실패했고 그 결과로2024 성공을 대신하지 않았다. 아래 Print PDF와 실제 입력을 직접 대조한다.

| 실제 저장 원본 | 독립 Print | Native 전체 TSV | fresh WASM 전체 TSV | 전쪽 최저 Native / WASM |
|---|---|---|---|---|
| [HWP](resource-restored.hwp) | [한컴2024 PDF](resource-restored-hwp-hancom2024.pdf) | [162쪽](resource-restored-hwp-native.tsv) | [162쪽](resource-restored-hwp-wasm.tsv) | 97.08677 / 97.12104% |
| [HWPX](resource-restored.hwpx) | [한컴2024 PDF](resource-restored-hwpx-hancom2024.pdf) | [162쪽](resource-restored-hwpx-native.tsv) | [162쪽](resource-restored-hwpx-wasm.tsv) | 97.08677 / 97.12104% |

90% 미만·누락·전체 쪽수 차이는 없다. 43쪽의 전각 공백 장평197%로 인한 줄 넘침은
보정 후 한 줄을 유지하며, 실제 복원 Print 대비99.56189/99.57354%다.
최저인162쪽과43쪽을 같은 배율에서 직접 판독했다. 원본 PDF에 대한 rhwp 외형과 한컴
재인쇄의 머리행 위 선·일부 제목 줄 표현 및 가는 선 농도 차이는 구분한다.
이 기존 호환 차이는 [백로그 B26](../../../../plans/task_m100_7998_backlog.md#b26-162쪽-표-머리행의-한컴-재인쇄-호환)에 명시했으며,
이번 보정으로 완전한 한컴 재인쇄 일치를 주장하지 않는다. 작업지시자의 범위 종료·잔여 백로그 지시를 유지한다.

## 회귀의 독립 입력

`p43-source-p43.pdf`는 동일 원본의43쪽만 PyMuPDF `insert_pdf(from_page=42,to_page=42)`로
분리한 것으로 글꼴·내용을 다시 그리지 않았다. `p43-selected.hwpx`는 그 원본43쪽을 선택해
현재 PDF importer로 저장했다. `p43-selected-2020.pdf`는 이 저장본의 새 한컴2020 Print다.
해당 한 쪽의 Native/fresh WASM은99.55895/99.57078%이며, 직접 판독 뒤 공백 관계와
실제 Worker의 양 저장 계약 검사를 추가했다. 같은 Worker 검사는 수정 전 WASM에서
정확히197% 저장 단언에 실패하고 새 제품에서는138검사·실제7 Worker가 통과한다.

## 추가 실사용 입력과 승인된 범위

`import-*-native.tsv`·`import-*-wasm.tsv`는 각 원본 전체138쪽/backend의 새 실행이다.
각 PNG는 바로 전 수용 제품과 동일하다. 각 문서의 전체 TSV를 그대로 보존했다.
RGB 세 채널로 확인한 [기존 수용본 비교](rgb-vs-accepted.json)와
[추가 입력 비교](rgb-preservation.json)를 함께 보존한다.
개인 이력서 원문·식별 가능한 이미지·저장 파일은 로컬에만 보존하고 `font-metric`의
비식별 TSV만 포함한다. 나머지 공개 가능한 원본은 `pdf/issue7998/import-cases/`에 있다.

| 범위 | Native 최저 | fresh WASM 최저 | 판정 |
|---|---:|---:|---|
| font metric4쪽 | 99.88105 | 99.80979 |  열기·글꼴 자원 보정; 원문 비공개  |
| ownership34쪽 | 92.89699 | 92.38421 |  열기·소유 보정  |
| codec025쪽 | 90.25971 | 90.46384 |  마스크·열기 보정  |
| XObject18쪽 | 88.80969 | 88.39467 |  열기 개선 수용; 기존2쪽 글꼴/무늬 차이는 백로그  |
| extraction1쪽 | 99.45899 | 99.46414 |  열기 보정  |
| collection69쪽 | 49.46762 | 94.55309 |  열기 개선 수용; 기존 Native 한글 미표시 등 출력은 백로그  |
| 회전 글자7쪽 | 80.62888 | 80.69685 |  열기·회전 보정 수용; 기존7쪽 출력은 백로그  |

위 수치는 전체 원문 TSV가 우선한다. 승인된 범위 예외를 일반90% 통과로 바꾸지 않는다.
기존 고정169쪽의 Native 글꼴 예외6건과 비공개 견적서의 devel 대비 하락도
[기존 원문 증거](../final-p5/README.md)·[#8109](https://github.com/edwardkim/rhwp/issues/8109)·
[#8110](https://github.com/edwardkim/rhwp/issues/8110)에 그대로 유지한다.

## CI 글꼴 경계

새 공백 회귀는 저장소의 메트릭·원본 공백·CHARSHAPE·문단 단일 소유를 검사하며 로컬
상용 글꼴이나 개인 PC 절대 경로를 CI에 요구하지 않는다. CI와 같은 Studio/Chrome/Firefox/
VS Code 배포 빌드 뒤 공개 글꼴/라이선스8검사, 실제 Canvas font binding8검사와 확장 배포3검사가
통과했다. 첫 배포 글꼴 검사 실패는 미생성 dist 경로 때문이었고, 배포 산출물을 만든 뒤 같은
검사를 통과했다. 글꼴/기준값/허용치를 바꾸지 않았다.

## 추가 승인된 기존 세 쪽 결함

`tac-img-02.hwp`의26쪽 쪽번호와45·53쪽 표/제목 사이 간격을 보정했다.
원본과 독립 한컴2020 Print66쪽을 새 Native/fresh Docker WASM에서 모두 대조했다.
전쪽 최저는96.39090/95.51053%, 누락·쪽수 차이는0이다.
메인테이너는 세 쪽에 **시각 판정 통과**를 주었다.
[전체 TSV·대표 비교/overlay·글꼴 공급 근거](tac-baseline/README.md)를 따른다.
