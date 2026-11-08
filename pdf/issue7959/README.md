# PDF 경로 색상 계약 대조군

사용자 PDF를 포함하지 않는 3쪽 생성 입력이다. `generate.py`는 pikepdf와 Pillow ImageCms로
200×200pt 쪽과 sRGB 프로필을 생성한다. 재생성: `python3 pdf/issue7959/generate.py`.
입력 PDF 자체를 Poppler 96 DPI `-thinlinemode shape`로 출력한 것이 독립 paint 기준이다.

- 1쪽: 이름 기반 RGB·직접 Gray/RGB 전환, q/Q, sc/scn. 순서대로 검정·흰색·50% 회색·빨강·
  25% 회색·초록·75% 회색·빨강을 그린다.
- 2쪽: 부모 RGB와 자식 Form Gray의 같은 이름, Form 복귀·BT/ET 뒤 상태, 빨간 stroke.
- 3쪽: ICCBased N=3 + DeviceRGB Alternate, 안쪽 Gray 변경 뒤 복원, 파란 SCN stroke.

ISO 32000-1:2008 8.6.5.5와 8.6.8(Table 74)의 색 공간·초기값·연산·q/Q 계약이다.
독립 Poppler와 Native Skia/fresh Docker WASM의 전쪽 실루엣 일치율은 각각 100%였으며
세 쪽의 색·흰 영역·Form·stroke를 직접 판독한 뒤 정식 회귀를 추가했다.
좌표·SVG 해시 pin은 사용하지 않는다. ICC CMM 전체 지원을 입증하는 입력이 아니다.

- 입력 SHA-256: `14cc0df58fc30b6a735c4d1a6010d6f848f54ffd1a68260110eb5196fb1e6981`
- runtime source: `8684913e0`
- Docker WASM SHA-256: `c44975b3639fb05c3a37188da80af5bff7ea00c381f877a21bf81143168c38b7`
- 증거: ignored `output/pr-review/issue7957/f6/`의 `scores/{native,wasm}/controls/silhouette.tsv`,
  `native/controls/`, `wasm/controls/`, `before-after-final-colours.json`.
- 수정 전 Native 최종 SVG에는 입력의 5가지 회색/RGB 색이 누락됐고 수정 후 모두 보존된다.
  같은 Rust 회귀 소스를 보존된 수정 전 debug 라이브러리에 연결한 실제 실행은 2 FAIL,
  수정 후 release-test 라이브러리 실행은 2 PASS다. `colour-tests-evidence.json`에 compiler·
  profile·fixture·library/test binary hash를 기록했다. CLI의 동일 색 assertion도 FAIL→PASS다.
