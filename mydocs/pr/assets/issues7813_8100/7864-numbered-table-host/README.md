# #7864 번호 제어가 앞서는 표 호스트

원본 HWPX에서 두 인라인 표 앞의 쪽 번호 제어가 첫 저장 줄의 공간을 차지한다. 한컴
Print에서 쪽 번호 제어 두 종류를 모두 지우거나 표 줄의 저장 시작 위치를 바꾸면 첫 표의
위끝이 약 177.6px 위로 이동한다. 번호 제어를 하나만 남기면 원본과 같은 물리 공간을
유지한다. 각 대조 입력을 한컴에서 별도로 출력해 확인했다.

`stored_marker_prefix_advance`가 원본 제어 슬롯과 저장 줄을 대조해 앞줄의 전진을 구한다.
조판은 각 표의 실제 제어 순서로 줄 높이를 배정하고, 가로 위치를 별도로 구하는 표에도
같은 세로 원점과 전진 끝을 전달한다.

| 비교 | 1쪽 raw | 전쪽 TSV |
| --- | ---: | --- |
| 이전 기준 Native, 기존 2020 Print | 27.84% | [측정표](base-native.tsv) |
| 진단 Native, 기존 2020 Print | 100.00% | [측정표](original-diagnostic-native.tsv) |
| 진단 Native, 동일 원본을 다시 출력한 2024 Print | 100.00% | [측정표](reprint-diagnostic-native.tsv) |

2026-11-28 KST에 `scripts/visual_sweep.py --silhouette-only --dpi 96`으로 색상 경계 보정 전
2px 실루엣을 측정했다. 원본 SHA-256은 `ae4c01001405de8deb2332cab5f469896a3b5ca12b18c1c28a48ff9719cadf85`이다.
2020 Print SHA-256은 `c56fdb74c26611ae6b4c3cf1f1b1ad8e01f1e14b248ddf44c4c6c0bd2d0d3d11`,
2024 재출력은 `4ba1e2ec0515283c302305464373135884e63a56ba3c20771387baf1dfd2b7f3`이다.
이전 기준은 `ca96e9f342`이고 진단 Native SHA-256은 `1c4f3414966fa95defc8bfb9561f09cbad0736ef34799caca98b05428395f73f`이다.

진단 실행 파일에서 두 음성 대조군은 각각 100.00%였고, 번호 제어 하나가 남은 양성
대조군은 각각 0.05%, 0.01%였다. 이 커밋은 해당 양성 대조군의 저장 줄 배정·배치 원점
수정까지 포함한다. 원본과 네 대조 입력의 수정 후 수치는 최신 기준으로 리베이스한 뒤
Native와 fresh WASM에서 다시 확인한다.
