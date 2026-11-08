# sample16 원본의 새 Print 대조 — 미충족 증거

원본: `samples/hwp3-sample16-hwp5.hwpx`, SHA-256
`49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`.
기준: `pdf/issue7642-sample16-print-refresh-20261012/hwp3-sample16-hwp5-hwpx-2024.pdf`.
source: `fe5ad4db922e03f38665a6c1fb7ba34627e13596`.

전64쪽 Native: 최저 27.23722%, 90% 미만 17쪽.
미달 쪽: 1, 6, 8, 19, 21, 22, 23, 24, 35, 43, 45, 47, 48, 49, 52, 53, 57. 원본64 / Print64로 쪽수는 같지만 내용의 쪽 소속과
여러 문단 위치가 다르므로 시각 미충족이다. 6쪽 32.25411%, 23쪽 27.23722%의 비교 PNG는
전체 검증에서 생성한 최신 PNG를 같은 Visual Sweep 함수로 합성한 진단이다.
새 source를 추가 export한 결과와 구분한다. 전체 TSV/log/JSON은 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage147-sample16-whole-native/`에 있다.

6쪽의 `(4) 사업자 선정방식`은 Native5쪽의 본문 경계 아래에 남고 Print6쪽 상단에
배치된다. 23쪽은 첫 문단 앞부분의 쪽 소속과 뒤 흐름이 다르다. 글꼴 차이로 면제하지 않는다.
18쪽 제목은 같은 줄의 여섯 TextRun으로 한 번 보존되지만 그 부분의 높은 점수로 전체
미달을 상쇄하지 않는다. 테스트 수정·복원은 전체 시각 선행 조건을 충족한 뒤 판정한다.
