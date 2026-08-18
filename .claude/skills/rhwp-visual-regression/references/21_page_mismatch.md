# 21 — PAGE_MISMATCH

`pageCountA != pageCountB`. 시각 회귀의 가장 명백한 신호라
우선순위 1위다. 변위나 구조를 보기 전에 쪽 수가 갈라진다.

텍스트 출력에 `⚠ 페이지 수 불일치 — 시각 회귀 강신호` 가 붙는다.

의도한 경우: 긴 값을 넣어 한 쪽이 늘어난 메일머지, 페이지 나누기
편집. 그때는 F05 를 "정상, 기록만"으로 닫는다.

의도하지 않은 경우: 같은 길이 치환인데 쪽이 늘거나 줄었다.
`dump-pages --json` 으로 어느 쪽에서 갈라지는지 좁힌다.

JSON: `pageCountMismatch: true`, `status: PAGE_MISMATCH`,
`regression: true`.

배치 TSV 에서는 `pages_a` 와 `pages_b` 가 다르고 `status` 가
`PAGE_MISMATCH` 다. `max_disp` 는 0 일 수 있다 — 쪽 수가 갈라지면
변위를 재기 전에 끝난다. 같은 폴더의 다른 행은 계속 측정된다.
