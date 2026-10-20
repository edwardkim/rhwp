# Issue #7642: 2022년 11월 시험지 18쪽 미달 증거

- 입력: `samples/3-11월_실전_통합_2022.hwp`.
- 기준: `mydocs/pr/assets/pr_7600_control_print_20261006/input-09-3-11월_실전_통합_2022-2020.pdf`, 한컴 2020 Print.
- 생산 코드 `a9732e242512ad671bbe4a8fe4234b5b57f41ff0`, `--compat 2022`, 인쇄 프로필, 전체 글꼴, 96 DPI.
- 전 21쪽 TSV의 18쪽이 84.80412%, 나머지 쪽은 90% 이상이며 전체 쪽수는 일치한다.
- 이 PNG는 stage479 전쪽 측정의 원본 raster로 canonical 비교·overlay·review 함수를 이용해 새로 합성했다. 새 head의 재출력으로 보고하지 않는다.
- 오른쪽 아래 그래프·주변 내용의 세로 위치 차이를 직접 확인했다. 보정 및 새 Native/fresh WASM 검증 전에는 완료·승인으로 판정하지 않는다.
- 입력/PDF/바이너리 해시·TSV·명령은 ignored stage479 receipt 및 stage500 provenance에 보존한다.

## 현재 수식 간격 수정의 별도 재검증

- 생산 코드 `0e62c1b289`, 동일 원본/Print의 전21쪽 Native 최저 95.56319%, 18쪽 97.67544%, 쪽수 일치.
- `native-p018-current-review.png`는 이 측정의 원본 raster로 새 합성한 그림이다. 오른쪽 아래 그래프의 큰 이동은 해소됐다.
- 이전 `native-p018-review.png`는 a973의 미달 증거로 보존한다. fresh WASM 전21쪽은 stage519에서 진행 중이다.
