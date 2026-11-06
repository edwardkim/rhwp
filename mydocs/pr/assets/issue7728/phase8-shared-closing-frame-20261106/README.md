# #7728 — 부모와 자식이 공유하는 빈 종료 공간

부분 개선이다. 사용자 지정 전 18쪽 Native/fresh WASM 각각 98% 목표는 아직 미완료다.

- 원본과 독립 PDF는 phase7과 같으며 교체하지 않았다.
- 수정 전 코드: `7578bccdcd`. 수정 후 Native는 같은 HEAD에 이 단계의 공통 조판 보정을 적용해 빌드했다.
- 자식의 선언된 첫 개체 상자에 대응하는 행 컷만 원본 부모 행의 빈 최소를 소유한다. 내부 줄 컷이 없는 원자 행도 같은 행 커서 질의로 확인한다.
- 부모 여러 열은 칸별 요구 높이의 최댓값을 예약하며, 내용 수용 하단과 빈 띠 종료선을 구분한다.
- 실제 자식 원점 선택을 한 번 확정해 여유 계산과 paint가 공유하고, 부모 아래 안여백과 자식 아래 바깥여백을 별도로 보존한다. 문서명·쪽 번호·보정 px 상수 분기는 없다.
- 6쪽 Native 94.59369→99.97875%, 5쪽 95.41089→99.00598%. 4쪽99.92664%·7쪽96.06208%이며 나머지 쪽은 전 단계와 같다.
- 독립 한컴 대조: 자식 여백/선언 최소 변경 시 부모 종료선은 유지, 부모 안여백 변경 시 자식 종료선만 변하고 부모 바깥여백 변경 시 부모 종료선이 이동했다. 변형 원본·PDF는 임시 진단이며 정본으로 대체하지 않는다.
- 관계 검사 4건 PASS. 6쪽 review와 overlay를 직접 판독해 부모·자식 하단의 심한 역전과 빈 공간 손실이 복구됐음을 확인했다. 가는 획의 미세 위치 차이는 남는다.
- Native 미달: 7쪽96.06208%,9쪽97.70433%,12쪽96.54520%. 새 fresh WASM·정상 대조군·전체 회귀는 미검증이다.

## Native 빌드 출처

- head: `7578bccdcd14ba998092c1e2fdc266cd8d65b47f`.
- source_sha256: `8e7d681b28ff4a2d1bb592c26b47f43efccc4b6c613457fd8df74f158f43a5a0`.
- scan_sha256: `af5a91ebdc48e7836a5f8a348865a4d133e0dc1ce149e030c7b2e1e1e1b80208`.
- paint_sha256: `98280986933e7c1255a18d053f5cbdbb38d0d457fcde993658315aebc468b713`.
- 바이너리 SHA-256: `607621cbdf096b7ef48aed080055f054615bf4ef0dca733e85ad673724b1e7e0`.

## 전쪽 TSV와 대표 PNG

- `before-native-silhouette.tsv`: `d029805d16c9068ff8e4408742f21f856c4c67b112fe15b9a2242d0f4fddf04c`.
- `after-native-silhouette.tsv`: `922174033a46bccddf8f1c0f67711520d0bf498e5685e330359881bb7ad79f67`.
- `after-native-p006-review.png`: `74b1cf9a82726baf20bfb434fd2d328b295e8b640a19cec57dc97a4e190c37b5`.
- `after-native-p006-overlay.png`: `b395554114264c295bcf0c6dc6777988c3c1d5520db8d0eeb8660e2ccee60c8b`.

## 같은 코드9274604528의 fresh WASM 보완

Mac locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다. Docker 최적화 결과가 아니다.
전18쪽 재출력: 4쪽99.93457%,5쪽98.93301%,6쪽99.97413%이며 7쪽95.50736%,9쪽97.31315%,
12쪽96.53672%가98% 미만이다. 6쪽 review/overlay를 직접 판독했고 심한 하단 역전은 없다.
Root pkg와 Studio public의 JS/WASM hash가 frozen package와 같음을 확인했다. 실제 Studio 응답 수용은 아니다.

{"rhwp.js": "370ddff0fd1a1cf6f889edb1b2a42f420fac62f7f57df4b9cff19829e60cdd5f", "rhwp_bg.wasm": "47d10c4d7cb7feaf3c711aa66a4798bfa3b364aadc7ba8f7e4ae83206680d09b"}
