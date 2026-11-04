# 기존 sample16 2018 HWP 검사의 측정 이력

`issue_8160_saved_page_boundaries.rs`에는 2022·2024 저장본 외에 2018 저장본의
기존 소유 검사도 함께 있다. 2022 저장본의 좌표 허용 오차를 바꿀 때 다른 원본의
수치를 혼동하지 않도록, 리베이스 직후 렌더 소스 `2ea4a0ae45ac15d2ca000088b13b5f243cb7be39`에서
기존 PDF를 재사용한 측정 결과를 기록한다. 이 입력은 이번에 새 회귀로 추가한 대상이 아니다.

- 원본 `samples/hwp3-sample16-hwp5-2018.hwp`, SHA-256 `9c3e00a1948a76f524603b3f7d5aba90a524b6e21583d199e40c2539845ae116`.
- 한컴 2020 Print `pdf/hwp3-sample16-hwp5-2018-hwp-2020.pdf`, SHA-256 `3ce5263f0dafc1aececab7447e22bd3676bbf0d983f001931ba1504ef9faa2c1`.
- 2026-10-30, 96dpi PDFium과 Native Skia·fresh WASM Canvas를 `visual_sweep.py --silhouette-only`의 2px raw 실루엣으로 비교했다. 전 64쪽이며 시스템 글꼴을 사용했다.
- [Native TSV](native.tsv)·[WASM TSV](wasm.tsv)의 전쪽 최저는 각각 27.39%, 27.38%다. 18쪽은 39.97% / 40.04%, 35쪽은 27.39% / 27.38%로 90% 미만이다.
- 기존 소유 검사에 쓰인 4·6·22·23쪽은 두 경로 모두 100.00%, 5쪽은 99.98% / 99.99%다. 이 수치는 아래 두 미달 쪽을 해결했다는 의미가 아니며, 2018 원본에 새 회귀나 기대값을 추가하는 근거로 사용하지 않는다.
- Native 실행 파일 SHA-256 `6774543ac5c6eaf976543d5d71a43d2b5a6663860fceed48b1f4ae514a95bf8c`, WASM SHA-256 `c494ecce8324cfc4c2bcf4d067ee48829ceda38604c0e2fbc32533da558e28a4`.
