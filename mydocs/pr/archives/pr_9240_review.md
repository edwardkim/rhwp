# PR #9240 리뷰 — fix(render): 쪽을 넘긴 표 조각에 cellzone 배경을 칠하고, Native 글꼴 공급이 렌더러의 표시 대체를 공개 Noto 로 덮지 않게 한다 (planet6897/rhwp-issue-docs#886)

## 최종 판정

**머지 보류 — 검증 진행 중.** 원 PR의 시각 검증과 변경 경계의 집중 회귀를 마쳤다. 전체 Rust 검증은 통과했고 정본 보호 문서의 기존 점수 재현을 확인 중이다.

## 접수 정보

- [원 PR #9240](https://github.com/edwardkim/rhwp/pull/9240) · planet6897 · devel
- 원 head `8a8de71f53b6091e8c968450596a79b61b809438` · 지정 리뷰어 `jangster77`
- 통합 비교 기준 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`
- 원 커밋 `5aa898396126cd1e6c61e1c375539977f2f41943` → 통합 커밋 `8f42203f55c15c5b180eee0ba2389317c9a937ce`
- 원 커밋 `8a8de71f53b6091e8c968450596a79b61b809438` → 통합 커밋 `68c3a7b047f85dc48abfcfad38e5c77cc90011fc`

## 변경과 검토 범위

1쪽 HFT 원본 글꼴 유무에 따른 표시 대체, 8쪽 이어받은 표 조각의 cellzone 배경을 확인한다.

### 메인터너 충돌 해소

#9277이 이어받은 표 조각의 cellzone 배경을 조각 그리드 좌표로 이미 칠한다. 이 경로를 채택하고, 같은 영역을 다시 칠할 수 있는 사각형 합집합 경로는 적용에서 제외했다. 정본 Print 8쪽에는 동일 색의 띠 두 곳이 있다. 표 조각별 배경 소속과 도색 순서는 렌더 트리 및 겹침 이미지에서 대조한다. #9208의 조상 커밋은 이미 devel에 들어 있어 중복 적용을 생략했다.

## 검증 입력과 결과

- 원본 `samples/issue886/1660000-201400009-kobe-earthquake.hwp` · SHA-256 `a2a6d53dad049bf27c4855e05227e467aba7b71184f43a0dbd1136d972bea13f`
- 기준 `pdf/issue886/1660000-201400009-kobe-earthquake-hancom-2020.pdf` · 64쪽 · SHA-256 `d0fcc123cf3c73aa68ecd6e3ecd7676c887b0c21b6eb13833fce00a3278d6f59`
- Windows 데스크톱 한컴 2020 `11.0.0.9136` Hancom PDF Print · MCP 작업 `4c7556ed-21b3-405f-97a4-70ddc70939e8`. [인쇄 영수증](../assets/planet6897_9240_9282_20261121/mcp-print-verification.json)
- 웹한글기안기 PDF는 공식 비교 자료로 사용하지 않는다.

첫 한컴 2024 인쇄본에서 HCI 글자 프로그램이 빠진 페이지를 확인했다. 정상 출력한 같은 입력의 한컴 2020 Hancom PDF Print를 비교 기준으로 고정했다.

`cef9ff9acc08f92431e89450945462514fffa772` 제품 코드의 Visual Sweep 96 DPI·2px raw 실루엣을 확인했다. Native 전쪽 최저 **98.12%**, fresh WASM 전쪽 최저 **98.15%**다. Native 고정 base와 후보의 전쪽에서 2%p 초과 하락은 0쪽이다. [Native 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/native-base-comparison/results.json) · [Native 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/silhouette.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/silhouette.tsv)

## 시각 증적과 남은 차이

| 쪽 | Native raw | fresh WASM raw | 시각 증거 |
| ---: | ---: | ---: | --- |
| 1 | 100.00% | 100.00% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/review_001.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/overlay_001.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/review_001.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/overlay_001.png) |
| 2 | 99.87% | 99.87% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/overlay_002.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/review_002.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/overlay_002.png) |
| 8 | 99.99% | 99.99% | [Native review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/review_008.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/overlay_008.png) · [WASM review](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/review_008.png) · [overlay](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/overlay_008.png) |

원본 HFT 공급 상태에서 비교한 전쪽 결과다. 원 HFT 3종만 제외한 경우 1쪽은 두 경로 모두 94.93%였고 표시 대체 HY견고딕을 공급했다. 이 페이지는 글꼴 선택 관계를 검사한다. 2쪽의 목차 쪽번호와 8쪽 표 이어받기 배경은 독립 Print와 대조해 ±2px로 고정하고 쪽·개체 소속도 검사한다.

최소 글꼴 환경 집중 회귀 10건을 통과했다. [회귀 실행 기록](../assets/planet6897_9240_9282_20261121/coordinate-regression-validation.json) · [독립 기대 좌표](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/evidence.json) · [원인과 메인터너 보정](../assets/planet6897_9240_9282_20261121/maintainer-repair.md)

통합 회귀 13,874건과 Native Skia 라이브러리 4,034건을 통과했다. [Rust 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/rust.json)

실제 Studio에서 주장 경계 9쪽의 화면·인쇄 출력을 확인했다. Canvas2D 실제 선택 50건 중 32건은 CanvasKit 글꼴 준비 실패에 따른 정상 대체이고, CanvasKit compat 2건과 default 2건은 각 backend가 직접 렌더했다. 브라우저 초기화 응답 54건의 WASM 해시를 빌드와 대조했다. [Studio 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio.json)

원 HFT 부재 조건의 [양쪽 전쪽 TSV 기록](../assets/planet6897_9240_9282_20261121/886-no-original/evidence.json) · [1쪽 review](../assets/planet6897_9240_9282_20261121/886-no-original/wasm/review_001.png) · [overlay](../assets/planet6897_9240_9282_20261121/886-no-original/wasm/overlay_001.png)

## Merge 후 원 PR 코멘트

통합 merge SHA의 원본 출력·한컴 PDF raster·review·overlay를 해당 주장 페이지별로 고정하여 본문에 표시한다. 실제 CI와 사용한 한컴 Print 제품 및 MCP 영수증을 함께 남긴다.
