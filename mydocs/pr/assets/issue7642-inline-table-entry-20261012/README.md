# #7642 저장 글줄의 표 진입 간격 보정

- code source: `4c1bf7f59787afe3998097099761631290c1c38b`
- 원본: `samples/3-09월_교육_통합_2022.hwp`
- 입력 SHA-256: `d451ffbc72e8c84433f380c0cbcc8472f23286d53dce4b793a4bbb82f8715108`
- 독립 한컴 2020 Print: `mydocs/pr/assets/pr_7600_control_print_20261006/input-01-3-09월_교육_통합_2022-2020.pdf`
- PDF SHA-256: `523477f1bd0572f95665c3f3651d1deb4fb94701e96301abe8835edaee69dd04`
- Native binary SHA-256: `71190fa6ad42b55e787bbc2f4685309f66eaadb5d28e4c09a68be9ab71466c25`
- print profile, compat 2022, 96dpi, 전체 글꼴 바이트다.

4쪽이 **78.47295%→97.74341%**로 개선됐다.
[새 Native 4쪽 review](native169-p004-review.png)에서 첫 표의 괘선과 이후
문항의 시작을 독립 Print와 대조한다. 원본 글줄의 후행 간격을 복원했으며
문서별 px 좌표를 더하지 않았다. 이전 PNG는
`../issue7642-owned-endnote-cut-20261012/native167-p004-review.png`에 있다.

전체 23쪽 Native는 쪽수 23/23, 최저 91.84074%, 미달 0쪽, source hash 유지
true다. fresh WASM169도 쪽수 23/23, 최저 91.84074%, 미달 0쪽,
source hash 유지 true, exit 0으로 완료됐다. 실제 WASM 페이지로 만든
[새 WASM 4쪽 review](wasm169-p004-review.png)도 97.74341%다.
이 문서의 양쪽 전체 페이지 점수 통과를 새 회귀/PR 승인으로 대신하지 않는다.
별도로 확인한 11쪽의 큰 괄호 첨자 위치
문제도 보정 대상으로 유지한다.

전쪽 TSV·실행 로그·빌드 영수증·입력/글꼴 해시는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/`의
`stage169-*`에 보존한다. 새 관계형 회귀는 같은 원본의 전체 Native/fresh WASM
90% 이상과 변경 영역 확대 판독 후 새 이름으로 추가한다.
