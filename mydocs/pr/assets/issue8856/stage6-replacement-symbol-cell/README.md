# 명시 대체 기호의 전진 칸과 잉크 원점

`issue1994_behindtext_table_20200830.hwp`의 리터럴 `∙`는 반각으로 측정되지만,
표시용 Haansoft Batang 프로그램은 같은 기호를 전각 칸에 둔다. 음수 자간에서
표시 프로그램의 여백까지 보존해 4쪽 기호가 오른쪽으로 밀리던 문제를 보정했다.
자동 문단 번호·원 PUA 기호·본문 원점·자간은 변경하지 않는다.

기존 결측 기호 보정 `92b52a793d`와 그 반례를 유지한다. 실제 원본 TTF/HFT와
결측·추정 폭에는 새 경로를 적용하지 않는다. 명시 대체, 음수 자간, 확인된 메트릭 hit가
함께 있는 작은 원형 기호만 자간 적용 전의 측정 칸에 놓고 실제 윤곽 크기는 보존한다.
같은 공통 함수가 Native Skia와 Canvas2D의 위치 재생에 사용된다.

## 동일 조건의 전쪽 비교

기존 [Print 설정](../../issue1994/print-frame-20261107/reference-font-environment.json)과
같은 준비기를 사용해 과거 환경 SHA-256 `81705de8…e6e43c9`를 정확하게 재현했다.
이전 공급 기록의 1,210개 파일은 누락·해시 차이가 없다. 독립 Print가 사용한 대체 프로그램에
대응하는 명시 환경이며 원본 산돌 글꼴 복구나 최소 글꼴 환경 검증으로 표현하지 않는다.
[환경 복구 근거](exact-environment-recovery.json)와 [기호 프로그램 관측](bullet-program-origin-oracle.json)을 보존한다.
관측한 잉크 경계의 수평 차이는 256/1024em이다. 두 윤곽의 점 수는 다르므로 완전히 동일한
글꼴 윤곽이라고 주장하지 않는다.

| 쪽 | Native 수정 전 | Native 수정 후 |
| --- | ---: | ---: |
| 1 | 99.96356% | 99.96356% |
| 2 | 99.72650% | 99.72650% |
| 3 | 98.69196% | 98.75608% |
| 4 | 99.80186% | 99.94357% |

같은 입력·PDF·96DPI·compat2022·canonical 2px raw 판정을 사용했다. PDF PNG는 전후 동일하다.
4쪽의 보호값 99.94357% 대비 하락은 0.00000%p로 복원됐다. 전 4쪽의 렌더 트리 SHA도
전후 동일하므로 본문·표·문단 소속이나 배치 칸을 움직여 점수를 맞춘 결과가 아니다.

- 전쪽 [수정 전 TSV](before-native.tsv) · [수정 후 TSV](after-native.tsv)
- 4쪽 수정 전 [raw](before-native-p004-raw.png) · [review](before-native-p004-review.png) · [overlay](before-native-p004-overlay.png)
- 4쪽 수정 후 [raw](after-native-p004-raw.png) · [review](after-native-p004-review.png) · [overlay](after-native-p004-overlay.png)
- 3쪽 수정 전 [review](before-native-p003-review.png) · [overlay](before-native-p003-overlay.png)
- 3쪽 수정 후 [review](after-native-p003-review.png) · [overlay](after-native-p003-overlay.png)
- 독립 PDF raster [3쪽](pdf-p003.png) · [4쪽](pdf-p004.png)

3·4쪽을 직접 판독해 리터럴 기호의 수평 원점 복원과 기존 본문·시·표 배치 보존을 확인했다.
기존의 작은 윤곽·세로 raster 차이까지 완전 동일해졌다는 의미는 아니다.

## 실제 검사와 범위

설치 글꼴에 의존하지 않는 MIT 합성 CFF 관계 검사는 실제 수정 전 1 FAIL에서 수정 후 1 PASS로
바뀌었다. 음수 자간 세 경우의 같은 잉크와 실제 원본 TTF를 보존하는 반례를 검사한다.
최소 글꼴 macOS Native Skia에서 기존 결측 기호 2건, 문서별 선택 7건, HFT 프로그램 19건,
기호 전진 1건도 통과해 총 30 PASS·0 FAIL이다. 선택 밖 검사는 전체 회귀 통과로 세지 않는다.
공유 helper 수집을 등록하기 전의 컴파일 실패와 잘못된 unit-tier 스크립트 호출은 통과 근거가 아니다.
올바른 경로와 고정 base를 지정한 manifest·unit-tier, fmt 검사는 별도로 통과했다.

[검증 기록](validation.json)은 전후 producer·소스/입력/PNG 해시·명령·로그 위치를 구분한다.
검사 후 주석 갱신 시각만 추가했으며 그 외 검사 본문이 같은 것도 검증했다.
이 자료는 집중 Native 보정의 증거다. 새 후보의 fresh WASM 전쪽·최소 글꼴 전체 회귀·필수 게이트와
실제 브라우저 소비는 최종 검증에서 별도로 완료한다. 글꼴 바이너리·로그·임시 스크립트는 Git에 넣지 않는다.
