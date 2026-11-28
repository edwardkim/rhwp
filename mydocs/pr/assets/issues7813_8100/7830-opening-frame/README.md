# #7830 저장 빈 줄과 다음 쪽에 시작하는 중첩 표

8쪽의 빈 문단은 앞쪽 본문과 마지막 제목 사이에 줄 상자를 차지한다. 저장 좌표는 빈 문단
59,568HU에서 제목 61,628HU로 2,060HU 전진한다. 이 값은 빈 줄 높이 1,100HU,
줄간격 660HU, 다음 문단 앞 간격 300HU의 합이다. 뒤따르는 중첩 표는 같은 호스트 문단의
별도 줄을 소유하며 저장 세로 위치가 0으로 돌아가는 다음 쪽에서 시작한다.

`stored_empty_successor_geometry`가 빈 줄 전진을 확인하고, 그 결과로 셀 컷의 유효 줄 범위를
보존한다. `CellCutFrame`은 제목 줄 하단 62,728HU에서 닫히는 원본의 점유 범위를 예약과
가운데 정렬에 전달한다. 후보에서 8쪽 제목은 한컴 Print 위치로 이동했고, 앞 본문의 위치와 전체
29쪽 구성을 유지했다.

| 비교 | 8쪽 raw | 전29쪽 최저 | 전쪽 TSV |
| --- | ---: | ---: | --- |
| 기준 Native | 96.12% | 96.12% | [측정표](base-native.tsv) |
| 후보 Native | 100.00% | 96.67% | [측정표](candidate-native.tsv) |

측정은 2026-11-28 KST, 96dpi, 2022 프로필, 색상 경계 보정 전 2px 실루엣 기준이다.
기준 커밋은 `ca96e9f342a541da8aee40f96495180b6b2ee863`, 후보 Native 실행 파일 SHA-256은
`1c4f3414966fa95defc8bfb9561f09cbad0736ef34799caca98b05428395f73f`이다.
`venv/bin/python scripts/visual_sweep.py --silhouette-only --hwp samples/hwpx_sample2.hwp --pdf pdf/hwpx_sample2-2020.pdf --rhwp-bin <고정 후보 실행 파일> --compat 2022 --dpi 96`을 실행했다.

- [HWP 원본](../../../../../samples/hwpx_sample2.hwp) SHA-256 `ca66d521d070ac8442e09fd3f283e17611eb9efb45eb4bf6c3a8f65e604f6b5f`.
- [한컴 Print PDF](../../../../../pdf/hwpx_sample2-2020.pdf) SHA-256 `3d792e4116183a9cbf9f31b6abe561d040565f2e16af30ab57eb449732f99c78`.

1쪽의 96.67%는 기준과 같다. 1쪽 차이 판독, 후보 fresh WASM 전쪽과 기존 회귀 재검사는
후속 단계에 남아 있다.
