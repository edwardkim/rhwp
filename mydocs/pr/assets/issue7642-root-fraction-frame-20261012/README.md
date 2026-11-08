# #7642 근호 분수와 위첨자의 공통 프레임

- code source: `9d340e28fc23736ee52a93efa43186f183a79653`
- 원본: `samples/3-09월_교육_통합_2022.hwpx`
- 입력 SHA-256: `b3ed3d5c4f0f95f9a8990b17720da3bef566b5a36174153aa3def28fd46e6029`
- 독립 한컴 2024 Print: `mydocs/pr/assets/pr_7600_control_print_20261006/input-02-3-09월_교육_통합_2022-2024.pdf`
- PDF SHA-256: `052f390b36e53a2374b3883a82c96164802784497259f1e7d5e3fe06b9a163f6`
- Native binary SHA-256: `c6b62b807d114bedcee3d78d97dc69c8b8b17e2d7c66fecf01b50b84333ad333`
- Print profile, compat 2024, 96dpi, 전체 글꼴 바이트다.

새 Native 전체 23쪽은 쪽수 23/23, 최저 91.85645%, 미달 0쪽,
source hash 유지 true, exit 0이다. [원본 11쪽 review](native170-original-p011-review.png)는
99.07698%다. 원본의 큰 괄호 뒤 제곱 첨자를 기존 Native와 독립 Print의
같은 영역에서 직접 대조했으며 새 위치가 기준 출력에 가까워졌다.

[진단 입력의 수정 전·후·Print 확대](native170-diagnostic-before-after-detail.png)는
별도 생성한 대조군이다. 원본의 증거를 대신하지 않는다. 21개 식 한 쪽의
Native는 96.21091%→99.69883%이고, 일반 글자·보통 분수와 근호 분모를
600/900/1500HU에서 대조했다. 같은 그림의 아래·위첨자 병존 식에서
아래첨자 `i`는 아직 높다. 추가 대조군의 근 지수·중첩 근호 뒤 첨자도
보정 대상으로 유지한다. 페이지 점수로 이 결함을 면제하지 않는다.

이 생산 소스의 fresh WASM과 원본 HWP 검증도 종료됐다. HWP/HWPX는
각각 Native/fresh WASM 전체 23쪽 최저 91.83899%/91.85645%, 미달 0쪽이다.
이 기록은 Stage 170 결과이며 후속 소스의 검증과 구분한다. 최종 완료·정식 신규
관계형 회귀·PR 승인은 미완료다. 입력 생성·한컴 job·Print 실행·남은 범위는
`mydocs/working/task_m100_7642_stage170.md`와 Stage 171에 연결했다.
TSV·메트릭·실행 로그는 ignored output의 `stage170-*`에 보존한다.
