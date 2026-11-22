# PR #9240 리뷰 — fix(render): 쪽을 넘긴 표 조각에 cellzone 배경을 칠하고, Native 글꼴 공급이 렌더러의 표시 대체를 공개 Noto 로 덮지 않게 한다 (planet6897/rhwp-issue-docs#886)

## #9316 리베이스 후 주장 검증

고정 기준은 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`, 검증한 제품 소스는 `954226e426517dc02ef0a15cc739899ab54b7fae`다. 같은 원본과 독립 한컴 Print로 주장 문서를 전쪽 대조했다.
아래의 이전 검토 자료는 각각 명시한 실행 소스의 기록이다.

| 입력 | 경로 | 쪽수 | 최저값 | 대표 쪽 | 전쪽 TSV 및 이미지 |
| --- | --- | ---: | ---: | ---: | --- |
| 547 | native | 22 | 99.68% | 22 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/native/review/review_022.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/native/overlay/overlay_022.png) |
| 547 | wasm | 22 | 99.68% | 22 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/wasm/review/review_022.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/547/wasm/overlay/overlay_022.png) |
| 886 | native | 64 | 98.12% | 8 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/native/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/native/review/review_008.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/native/overlay/overlay_008.png) |
| 886 | wasm | 64 | 98.15% | 8 | [TSV](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/wasm/silhouette.tsv) · [review](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/wasm/review/review_008.png) · [overlay](../assets/planet6897_9240_9282_20261121/after-9316/claims/886/wasm/overlay/overlay_008.png) |

합성 `fallback.ttf`와 `RHWPHftEmptyPoint.otf` 및 전용 검사는 PR 커밋 범위에서 제거했다. [글꼴 공급 및 출력 증거](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 최종 판정

**통합 검증 완료.** 원 PR의 변경 경계와 전체 Rust 회귀를 확인했다. 동일 공급 조건의 정본 보호 문서 46조건·4,046쪽에서 기존 보호값 및 고정 기준 대비 하락은 0쪽이다. [통합 전쪽 대조](../assets/planet6897_9240_9282_20261121/protected-candidate-v5/fixed-base-comparison.json)

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

`cef9ff9acc08f92431e89450945462514fffa772` 제품 코드의 Visual Sweep 96 DPI·2px raw 실루엣을 확인했다. Native 전쪽 최저 **98.12%**, fresh WASM 전쪽 최저 **98.15%**다. Native와 fresh WASM 각각 고정 base와 후보의 전쪽에서 2%p 초과 하락은 0쪽이다. [Native 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/native-base-comparison/results.json) · [fresh WASM 기준·후보 TSV](../assets/planet6897_9240_9282_20261121/wasm-base-comparison/results.json) · [Native 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/native/silhouette.tsv) · [fresh WASM 전쪽 TSV](../assets/planet6897_9240_9282_20261121/coordinate-evidence/886/wasm/silhouette.tsv)

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

최신 제품 코드 `19646e69fc9`의 Studio에서 2·8쪽 화면·인쇄 12건을 다시 확인했다. Canvas2D 실제 출력은 12건이며 그중 8건은 CanvasKit 요청이 글꼴 준비 단계에서 대체된 결과다. 전체 60건의 초기화 응답이 새 WASM 해시와 일치했다. [Studio 최신 빌드 검증 영수증](../assets/planet6897_9240_9282_20261121/validation/actual-studio-v4.json)

최신 제품 코드 `19646e69fc9`에서 원 HFT 3종을 제외한 Native·fresh WASM 64쪽을 각각 다시 검증했다. 1쪽은 두 경로 모두 94.93%이며 fresh WASM에 등록된 표시 대체는 HY견고딕이다. [최신 검증 기록](../assets/planet6897_9240_9282_20261121/886-no-original/v4/evidence.json) · [1쪽 review](../assets/planet6897_9240_9282_20261121/886-no-original/v4/wasm-review-001.png) · [overlay](../assets/planet6897_9240_9282_20261121/886-no-original/v4/wasm-overlay-001.png)

## 최신 Native·fresh WASM 전쪽 확인

제품 소스 `65b6fe383c3bd8cbcf83de7909c7a6c24eb32c35`의 Native와 fresh WASM을 각각 사용해 전쪽 64쪽을 다시 출력했다. 공식 두 자리 raw 실루엣의 최저값은 Native **98.12%**, fresh WASM **98.15%**다. 이전 검토에서 비교한 모든 쪽의 RHWP 및 Print PNG 픽셀, TSV가 이번 결과와 일치했다. [최신 전쪽 비교 및 생산물 SHA](../assets/planet6897_9240_9282_20261121/validation/claims-v5.json)

## #547 및 원 HFT 부재 조건의 최신 대조

#547의 22쪽을 Native와 fresh WASM에서 각각 확인했다. 두 경로의 최저값은 99.68%다. 2쪽 목차의 테두리와 쪽번호, 21·22쪽 서약 내용을 보존했다. 원 기호·날짜·쪽번호에는 미세한 차이가 남는다. 원 HFT 3종을 제외한 #886의 1쪽은 두 경로에서 각각 94.93%다. fresh WASM이 표시 대체로 등록한 HY견고딕의 글자 윤곽과 간격 차이를 확인했다. [전쪽 TSV·직접 판독·글꼴 영수증](../assets/planet6897_9240_9282_20261121/9240-supplement-v5/validation.json)

## Merge 후 contributor PR comment 계획

- [#886](https://github.com/planet6897/rhwp-issue-docs/issues/886)의 1쪽 제목에서 원 HFT가 있을 때와 없을 때의 표시를 각각 설명한다. 8쪽의 노란 배경 띠 두 곳은 원 PR이 복구한 영역이다. 기존 [#9277](https://github.com/edwardkim/rhwp/pull/9277)의 표 조각 그리드에서 배경을 한 번만 칠하도록 통합한 이유를 함께 알린다. 2쪽 목차 우측 탭은 메인터너가 보정한 경계로 명시한다.
- #886은 Windows 데스크톱 한컴 2020 Hancom PDF Print의 64쪽을 사용한다. [#547](https://github.com/planet6897/rhwp-issue-docs/issues/547)은 원 PR이 개선 또는 보존을 주장한 별도 입력이므로, 독립 한컴 2024 Hancom PDF Print 22쪽과 2·21·22쪽을 따로 대조하고 해당 PNG를 표시한다.
- 첫 코멘트에 #886의 1·8쪽, 메인터너 보정인 2쪽, #547의 2·21·22쪽에 해당하는 review와 overlay를 넣는다. 원 HFT 미설치 조건은 전쪽 결과와 1쪽의 실제 대체 글꼴을 따로 설명한다.

[Visual Sweep 공식 코멘트 절차](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)를 연결한다. 통합 merge SHA에 실제 포함된 원본 출력·한컴 PDF 래스터·review·overlay만 사용한다. 최신 CI와 원본 및 Print 영수증, 실제 확인한 쪽수와 자동 비교 수치, 직접 판독 결론을 함께 기록한다. 게시 전에 PNG 원격 응답과 검증 자산의 SHA를 대조한다.
