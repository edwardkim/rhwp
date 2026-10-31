# rhwp JPEG 인코더 보정

crates.io `jpeg-encoder` 0.7.1을 원본 라이선스·저작권과 함께 보존한다.
이 소프트웨어는 Independent JPEG Group의 작업에 일부 기반한다
(this software is based in part on the work of the Independent JPEG Group).
원본 FDCT 코드·문서의 IJG 조건을 유지한다.

## 정수 양자화 경계

`src/quantization.rs`의 역수 빠른 경로는 정확한 반 양자 경계에서 한 단계
작은 값을 내는 경우가 있다. 독립 전체 계수 비교에서 품질 75의 두 표,
64개 계수 위치와 -2048..2048 입력 중 1764개 차이를 확인했다.
빠른 경로의 반 양자 이상 잔차만 정확한 정수 나눗셈으로 재계산한다.
음수에도 부호를 마지막에 적용해 반올림 경계를 대칭으로 보존한다.

`encoder.rs`의 scan 메서드는 `encoder/scans.rs`로 분리하고 필요한 가시성만
`pub(super)`로 바꿨다. 두 파일을 1000줄 이하로 유지하며 알고리즘은 보존한다.
README의 원본 두 줄 끝 공백만 정리했으며
라이선스 문구는 바꾸지 않았다. rhwp 회귀검사는 전쪽 검증 완료 뒤 등록한다.


## #8181 패키징 기능과의 통합

원본 registry package와 보정본을 혼동하지 않도록 내부 이름을 `rhwp-jpeg-encoder`
`0.7.1-rhwp.1`로 구분한다. root의 코드 import alias `jpeg_encoder`는 유지한다.
`publish = ["crates-io"]`, 필요한 source·README·원본/패치 기록·세 라이선스 파일만 포함하는
package include와 source 사전 검사 후보를 등록한다. 실제 게시·registry 업로드는 수행하지 않는다.

## 세로 평균 표본화의 하단 패딩

마지막 유효 행 그룹을 평균한 뒤 iMCU의 남은 출력 행에 반복한다. 원본 마지막 행을
먼저 반복하던 경로를 공개 libjpeg의 전처리 계약에 맞춘다. 가로 경계·bias·원본 RGB는
유지하며 두 scan 경로가 같은 함수를 소비한다. 일반 이미지 decoder는 교체하지 않는다.
