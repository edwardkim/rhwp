# #8856 참조 번호의 원본 언어 슬롯과 공통 paint 전진

stage13 `01d2197973` 뒤, 생성 번호의 문자군에 따라 글꼴 슬롯을 선택하고 언어별 상대 크기·장평과
원본 메트릭 문맥을 측정 및 모든 paint 소비자에게 함께 전달했다. 본문 자간·줄 채움의 비간섭,
control 소유와 다음 본문의 비겹침을 유지했다. 일반 참조·leading 참조·표 호스트는 같은 결과를 쓴다.

독립 Print의 잘못 표기된 font 이름 대신 실제 descriptor·내장 프로그램을 확인했다. Gulim 숫자의
표시 폭과 TJ 보정을 분리했으며 실제 전진을 제품 상수로 복사하지 않았다. 원본 글꼴·최소 공개 글꼴·
Print 공급 대조를 나누어 기록하고 설치 글꼴 바이너리를 제출하지 않는다.

새 검사 3건은 실제1 PASS/2 FAIL→3 PASS였다. 기존 paint 칸 검사 1건의 비양자화 자연 폭 가정은
독립 출력 선행 확인 후 전진 배열·칸·논리 소유·뒤 본문 비겹침으로 교체했다. 본문 자간 비간섭 검사와
함께 2 PASS다. 신규 source 검사는 모두 `tests/cases/`에 수집한다. 기존 inline fixture의 변경은
추가 필드의 기본값뿐이며 inline 신규 검사를 만들지 않았다.

[동일95·96쪽의 Native/fresh WASM 수치·대표 review/overlay·생산 신원](../pr/assets/issue8856/stage14-note-reference-source-style/README.md)

Native와 fresh WASM 빌드, Studio TypeScript 및 2,784건 통과를 확인했다. Mac의 fresh WASM은
locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 최적화로 보고하지 않는다.
root pkg와 Studio public의 JS/WASM hash가 각각 일치한다. CanvasKit의 기존 guard는 우회하지 않았다.
전체 회귀·전쪽/보호·실제 소비 검증은 최종 source에서 이어가며 이 기록만으로 PR 준비 완료를 선언하지 않는다.
