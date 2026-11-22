# PR #9280 버전별 한컴 본문 기호 변환

## 최종 판정

**머지 보류.** 원 기여의 15종 저장 결과는 한컴과 일치한다. 메인테이너가 5.0.1.6 문서 재열기 때의 표시·추출 및 5.0.1.5의 인접 네 기호 누락을 보완하기로 했다. [판독 근거와 보완 계획](pr_9280_review_impl.md)

## 접수와 독립 대조

- Raphael-KR 님의 rhwp 첫 기여다. OPEN 상태에서 reviewer가 없어 `edwardkim`을 지정했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9280)
- 원 head `a0b40b2f937eee79413724928c79c603ce3d2433`의 CI와 CI Impact Policy가 성공했다. 검토 기준 `upstream/devel`은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`이다. 원 커밋 4개를 순서대로 적용한 로컬 후보 `42513825ea046c0a4813d6dd86b25d56d99ce3e6`의 tree는 실제 병합 시뮬레이션 `ba31f73fb59e7b0c72338e7bf038d87f5df3e863`과 같다. [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/44868449596)
- 기존 #5140 본문·겹침 변환은 `f750ef0a1f`·`25c27ec5b1`, #5861 실측 보완은 `d5dada7c14`, #8306 검증 정규화는 `200cf5ecd5`에서 확인했다. 세 이슈는 이미 완료됐다.
- 5.0.1.5와 5.0.1.6 각각 한 쪽의 Windows 한컴 2020 PDF를 따로 인쇄했다. 합성 원본은 rhwp로 만들었으며 버전 차이는 한 바이트다. Windows 2020·2024 및 기여자의 macOS HWPX에서 15문단씩 문자와 굵은 서식이 일치한다. 원본에는 저장 줄이 없어 저장 줄 위치는 한컴의 저장 줄 정보를 사용한 검사와 구분한다. [구형 원본 Print](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.5-2020.pdf) · [현대 원본 Print](../../pdf/issue5140-version-boundary/legacy-symbols-v5.0.1.6-2020.pdf)
- 원 기여 후보로 내보낸 두 HWPX를 Windows 한컴 2020에서 직접 다시 인쇄한 결과는 각 원본 Print와 96 dpi 전체 RGB 픽셀이 같았다. Native에서 후보 HWPX를 다시 열어 얻은 raw는 100.00%·98.33%다. 현대 문자의 기존 표시 차이는 직접 판독했다. [문자·서식 대조](assets/pr9280/symbols.tsv) · [인쇄·저장 작업](assets/pr9280/hancom-jobs.tsv)
