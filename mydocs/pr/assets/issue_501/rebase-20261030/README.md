# #501 실물 표 높이 검사 근거

2026-10-30에 리베이스 직후 렌더 소스 `2ea4a0ae45ac15d2ca000088b13b5f243cb7be39`로
기존 원본과 독립 한컴 Print PDF를 비교했다. 이는 이후 수정의 최종 head 재검증을 대신하지 않는다.

- 원본: `samples/mel-001.hwp`, SHA-256 `58b0542bdfd0d6339241ff12c3705fa61aed8f417290d0c5929fd289111294a3`.
- 독립 기준: `pdf/mel-001-hwp-2020.pdf`, SHA-256 `51bbb0f25e36428f4059db9177fb0b2084bcf9b0b19302fff78fefef487c9745`.
- 96dpi PDFium 래스터와 Native Skia·fresh WASM Canvas를 `visual_sweep.py --silhouette-only`의 양방향 2px 실루엣으로 비교했다. 두 출력은 전 20쪽이며, 기존 PDF를 재출력하지 않았다. 로컬 시스템 글꼴을 사용했다.
- [Native TSV](native.tsv)의 전쪽 최저는 96.43%, [WASM TSV](wasm.tsv)는 96.42%다. 검사 쪽인 2쪽은 각각 99.64%, 99.65%다. [2쪽 Native overlay](p002-overlay.png)를 확대해 첫 행과 표의 소유를 확인했다.
- Native 실행 파일 SHA-256 `6774543ac5c6eaf976543d5d71a43d2b5a6663860fceed48b1f4ae514a95bf8c`, WASM SHA-256 `c494ecce8324cfc4c2bcf4d067ee48829ceda38604c0e2fbc32533da558e28a4`.

원본 표의 0행 셀은 각 1,980 HWPUNIT이며, 96dpi에서 26.4px다. 기존 검사는 이 높이를
검사하기 전에 SVG `x` 속성의 부동소수점 문자열 전체가 같아야 한다고 가정해 실패했다.
수정 검사는 원본의 해당 셀과 같은 행의 실제 상자를 소유 주소로 연결해 높이를 대조한다.
독립 PDF의 그림 및 원본·실제 상자 관계는 별도의 근거다.
