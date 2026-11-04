# #8533 단독 문맥 문자의 Print 경계 보완

Print TTF 대체 윤곽은 한 cluster가 한 글자라는 이유만으로 문맥 shaping을 대체하면 안 된다.
기존 소유 fixture의 cmap에 U+0628만 추가한 합성 face로 단독 문자 반례를 만들었고,
직전 WASM에서 실제 `fillText` 경로가 사라지는 실패를 확인했다. 공개 글꼴이나 실제
아랍어 자형을 복제한 파일이 아니다. 제품 글꼴 경로에 배치·설치하지 않는다.

WASM 전용 painter에서 공통 `contextual_text::is_contextual`을 확인하여 이 경우 기존
painter를 유지한다. 공급되지 않은 프로그램, 원 face가 공급한 `H`, Screen, 2022 Print,
HighQuality의 비적용 경계도 같은 브라우저 검사에서 확인한다. 조판·문자 소속은 유지한다.

- 추가 반례: 수정 전 실제 1 FAIL. 경계 보완 후 브라우저 묶음 13 PASS, 0 FAIL.
- 새 locked wrapper `--target web --out-dir pkg --no-opt` 빌드: exit 0, 4분 48초.
  WASM SHA-256 `b366ca4bee534ef618ad82a38ab71801cf84585fb6b000438d0b6ce09d54d03b`.
  root pkg와 Studio public이 일치한다. Docker 최적화 빌드가 아니다.
- 실제 Studio의 편람 3쪽 클릭은 외곽 group을 선택했고 console/command 오류가 없었다.
  초기 fetch 응답의 WASM hash도 새 빌드와 일치했다.
- 이 변경 파일은 Native compiler의 의존 목록에 없다. 앞선 새 Native 실행 파일
  `8ae681680593809015ebb5c0a19ddaa5269ba11aa56d467882d6be81ab3d8728`의
  전 383쪽 결과는 #8389 표시 하한을 유지했다. 130쪽 raw 99.96465%, 다른 쪽의 표시값은
  100.00%다. 모든 쪽의 원시값이 정확히 100%라는 뜻은 아니다.
- 경계 보완 전 WASM `3be326…`도 383쪽 하락 0건이었다. 새 `b366ca…`의 전쪽 결과로
  대신 인용하지 않고 최소 글꼴/일반 환경을 새 빌드로 다시 확인한다.

전체 회귀·필수 게이트와 최신 WASM 전쪽 검증은 계속한다. 중간 산출물·로그는 ignored
`output/pr-review/issue8533-wasm-advance-20261104/`에 보존한다.
