# 부드러운 가장자리 도색의 독립 대조군

이 단계는 원본 그림의 효과 반경을 실제 공통 이미지 준비에서 소비하는 보정이다.
모델·저장·조판 전달만 통과한 앞 단계와 구분한다.

`independent-controls.json`은 투명도 8건과 추가 반경 7건의 실제 한컴 Print 성공 기록이다.
입력·PDF는 `pdf/issue8856/`에 보존하며, 2020·2024 버전과 해시·작업 ID를 분리한다.
처음의 0/300/1000/2000 HU 물리 대조군 및 확대/crop 대조군은 앞 단계 자료를 재사용한다.

2024 PDF의 별도 이진 마스크 밀도에서 균일 alpha 128의 중앙은 반경 0/1000 모두
127.5였다. 16×16 문턱의 정수 밀도이므로 원 alpha가 127.5라는 뜻이 아니다.
원래 투명한 구멍은 투명하고, 그 바깥의 불투명 쪽이 감쇠한다.
`alpha-density-observations.json`은 이 직접 관측이며 제품의 시각 일치율이 아니다.
2020 PDF의 RGB 합성도 중앙의 기존 투명도와 내부 구멍 보존을 확인했으나 별도 mask
밀도와 같은 값으로 취급하지 않는다.

`corner-model-comparison.json`은 각 축에서 관측한 alpha 허용 구간으로 모서리를
대조한다. 반경 3종의 약 11.8만 강제 표본에서 곱 모델은 모순 0이지만 최소 모델은
모순이 있다. Gaussian이라는 결론이나 한컴 내부 API 호출 자체를 증명하는 검사는 아니다.

Windows GDI+의 독립 표본은 출처만 `producer.json`에 기록했다. 실제 Mac 최소 글꼴
회귀와 구분한다. 추가 반경의 예측은 `holdout-prediction-results.json`처럼 정수화 경계에서
차이가 남는다. 이를 문서별 상수·기준 PDF 변경·판정 문턱 완화로 숨기지 않는다.
현재 제품의 실제 Native/fresh WASM 완료 근거는 별도로 산출하며, 이 oracle만으로
제품 완료나 최종 PR 통과를 선언하지 않는다.

## 실제 제품 검증

`native-validation.json`·`wasm-validation.json`에 실제 소스·바이너리·글꼴/PDF 공급·명령과
쪽별 결과를 보존한다. 새 검사 2건의 실패 후 최종 새 3건 및 기존 6건이 통과했다.
각 backend의 집중 전 6쪽은 90% 미만·쪽수 불일치 0이며, interview 1쪽은
Native 99.23307% / fresh WASM 99.25469%다. 같은 쪽의 `*-raw.png`, `*-review.png`,
`*-overlay.png`와 전쪽 TSV를 함께 보존한다. 새 Chrome 초기 응답의 해시·실제 캡처 PNG도
검증했으며 로컬 no-opt 빌드다. 전체 회귀·보호 TSV·최종 제출 게이트는 아직 완료하지 않았다.

사진 외곽 효과는 실제 도색에서 복원됐다. 제목의 대체 글리프·간격은 별도 잔여이며 이 단계의
완료에 섞지 않는다. 마스크의 추가 반경·내부 모서리 정수 표본도 정확한 동일성을 주장하지
않는다. `hole-corner-observations.json`·`hole-kernel-comparison.json`은 사각/원형 진단의
한계를 구분하며 판정 문턱이나 기준 PDF를 바꾸는 근거로 사용하지 않았다.

추가 투명 PNG 16조건의 실제 결과는 `alpha-visual-validation.json`과 해당 TSV에 있다.
2024 반경 1000/alpha 128은 양 backend 모두 97.98599%다. 효과 없는 반경 0도
중앙의 RGB가 PDF와 다르므로 raw 100%를 색의 동일성으로 해석하지 않는다.
`alpha-half-r{0,1000}-2024-*-review.png`와 같은 쪽 overlay를 함께 보존한다.
이 색·장치 마스크의 잔여는 같은 브랜치에서 이어서 처리하며 최종 PR 준비는 아직 아니다.
