# PR #9337 표 높이 편집 검토

**접수 당시 판정은 보완 필요였다.** 단순 표에서 셀을 줄이면 즉시 다시 그려진다. 여러 행에 걸친 셀의 높이를 같은 값으로 바꿔도 호출한 편집 API에 따라 표 뒤 문단이 다른 위치에 남는다.

## 검토 범위

기여자 `semanticist21`, 리뷰어 `edwardkim`, 목표 브랜치 `devel`. 2026-11-24에 원 PR `b69c815caa964dbbba1d55a9aae6654c875a5a05`와 기준 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`을 확인했다. 단독 병합 tree는 `546b652b1c7c650aa25428d17ef7b3508e136635`이다. 원 PR의 Build & Test와 CI Impact Policy는 성공했다. 연결된 자동 종료 이슈는 없다. [원 PR](https://github.com/edwardkim/rhwp/pull/9337)

## 검증과 재현

- 저장소의 공개 글꼴 5개만 노출한 WSL에서 기여자의 셀 높이 검사 3개와 기존 부호 있는 표 오프셋 검사 3개가 통과했다. 기존 표 편집 보정은 `17e34fdca3`의 #6882에서 추적했다.
- 공개 API로 만든 인라인 4행 3열 표에서 마지막 열을 네 행에 걸쳐 합쳤다. 셀 간격은 0이고 뒤 문단에는 ‘뒤’가 있다. HWPX로 저장해 연 동일한 입력에서 병합 셀의 높이를 8,000에서 15,000 HU로 바꿨다. 두 경로 모두 그린 표 높이는 106.7에서 200.0 px로 바뀌었다.
- `setCellProperties`에서는 표 선언 높이가 15,000 HU로 갱신되고 표 아래와 다음 문단 캐럿 사이 간격이 12.4 px였다. `resizeTableCells`에서는 선언 높이가 8,000 HU에 남아 간격이 0.6 px로 줄었다. 각각 HWP와 HWPX로 다시 열어도 차이가 남았다. [측정값](../assets/semanticist21-9337-9354-9377/table-geometry.tsv)
- 두 HWPX를 한컴 2024 13.0.0.3901에서 한 쪽씩 인쇄했다. 다음 문단의 같은 글자 위치는 양쪽 모두 276.79 pt다. 96 DPI에서 전쪽 픽셀의 SHA-256도 `ab56321f2437eb5d05256d784014aa04e7ecf88f490d9092d5cfc703aba6701f`로 같다.
- 국가법령정보센터 계약서 두 번째 표에서 높이 58,083 HU인 세 칸을 확인했다. 수령한 HWP의 SHA-256은 `68842876cdd6f3cadf68e3d00dcd944c6bb66daf0de82715748ddb033ea250aa`다. [별지 제9호 용역표준계약서 원본](https://www.law.go.kr/LSW/flDownload.do?flSeq=160252863)

보완 범위는 크기 조절과 속성 변경이 공통 행 점유 결과로 표 높이와 뒤 문단 위치를 갱신하고, 원 계약서와 병합 셀 사례를 다시 검증하는 것이다.

## 메인테이너 보완 결과

셀 속성과 크기 조절이 같은 행 점유 계산을 사용한다. 내용으로 높이가 정해진 행은 측정된 높이를 쓰고, 여러 행에 걸친 셀과 원래 표의 추가 높이를 편집 전후에 보존한다. 실제 인라인 표 높이가 달라지면 그 표를 소유한 문단의 저장 줄을 무효화한다. 셀 안의 중첩 표는 안쪽 소유 문단에서 이 처리를 시작한다.

- 셀 간격 0·300·600 HU 각각에서 두 편집 경로를 비교했다. 같은 편집 결과의 그려진 높이, 표 선언 높이, 표 아래와 뒤 문단 사이 간격은 HWP·HWPX로 다시 열어도 일치했다. 간격이 0인 입력의 표 높이는 200.0 px, 표 뒤 간격은 12.4 px였다. [속성·크기 조절·재열기](../assets/semanticist21-9337-9354-9377/corrected-table.tsv)
- 별지 제9호 용역표준계약서에서 세 셀을 58,083 HU에서 2,000 HU로 줄였다. 수정 전에는 편집한 표가 2쪽에서 아래로 밀렸다. 보완 뒤 같은 2쪽의 표 윗변은 77.5 px에 있고, 행 높이는 774.4 px에서 26.7 px가 됐다. Windows 한컴 2020에서 원본과 두 형식의 최종 저장본을 각각 직접 인쇄했다. [공개 원본과 인쇄본](../assets/semanticist21-9337-9354-9377/sources-and-print.tsv) · [편집·재열기 좌표](../assets/semanticist21-9337-9354-9377/contract-geometry.jsonl)

한컴에서 직접 인쇄한 7개 입력은 Native와 fresh WASM에서 각각 10쪽이었다. 병합 셀 높이를 두 방식으로 편집한 4종은 전쪽 100.00%였다. 공개 계약서의 원본과 HWP·HWPX 저장본은 각각 2쪽이며 최저는 Native 99.38%, WASM 99.35%였다. 표를 줄인 2쪽은 세 입력 모두 양쪽 경로 100.00%였다. 계약서의 돋움은 원본의 글꼴 이름과 Windows 글꼴 모음 내부의 실제 서체를 대조해 공급했다. [쪽별 실루엣](../assets/semanticist21-9337-9354-9377/focus-pages.tsv) · [계약서 2쪽 확대](../assets/semanticist21-9337-9354-9377/contract-edited-hwpx-comparison.png) · [사용 글꼴·원본 모음의 해시](../assets/semanticist21-9337-9354-9377/contract-font-programs.tsv)

### 인접 셀의 높이를 서로 보상하는 편집

통합 과정에서 공개 원본 `셀보호2.hwp`의 기존 회귀가 표 선언 높이 17,932 HU를 기대했는데 첫 보완본은 18,232 HU를 저장했다. 원래 검사는 #493의 입력과 #1443의 표 셀 보정 커밋 `b7b1dd20a7`에서 유래한다. 한 행의 높이를 키우는 일반 편집과 달리, 이 사례는 이웃 셀에 반대 부호의 높이 변화를 준다. 각 셀에 최소 높이를 적용한 뒤의 실제 변화량을 합산하도록 `7f0106b7e6`에서 수정했다. 기존 단언을 그대로 실행해 통과했다.

셀 너비를 서로 1,200 HU, 높이를 서로 300 HU씩 반대 방향으로 조절해 최종 소스에서 HWP와 HWPX로 각각 저장했다. 두 파일을 한컴 2024로 인쇄하고 rhwp에서 다시 열었다. 한컴과의 내용 실루엣 일치율은 첫 보완본에서 두 형식 모두 75.92%였고, 최종 Native와 Chrome WASM에서는 두 형식 모두 100.00%였다. 옆 셀의 시작점과 마지막 행 아래 괘선의 위치도 대조했다. [실제 편집과 재열기](../assets/semanticist21-9337-9354-9377/compensated-resize.tsv) · [표 아래쪽 확대](../assets/semanticist21-9337-9354-9377/compensation-edited-hwp-comparison.png)

## Merge 후 contributor PR comment 계획

원 기여 커밋을 포함한 통합 PR의 병합 SHA를 확인한 뒤, #9337에 감사 댓글을 게시한다. 계약서 편집본 2쪽을 한컴 인쇄에 대조한 아래 두 출력과 쪽별 TSV를 병합 SHA로 고정해 첨부한다. 높이 변경 직후와 두 형식으로 다시 열었을 때 표 윗변과 뒤 문단 간격을 비교한다. 옆 셀을 반대 방향으로 조절한 사례는 원 표 높이 17,932 HU 보존 여부도 알린다.

- Native Skia의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/native/contract-edited-hwpx.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/contract-edited-hwpx-p002-native-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/contract-edited-hwpx-p002-native-overlay.png)
- 브라우저 Canvas2D의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/wasm/contract-edited-hwpx.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/contract-edited-hwpx-p002-wasm-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/contract-edited-hwpx-p002-wasm-overlay.png)

## 최신 devel 병합 후 원 기여와의 최종 대조

원 PR 최종 커밋 `b69c815caa964dbbba1d55a9aae6654c875a5a05`에서 Native와 브라우저용 WASM을 각각 빌드했다. 원 PR 본문과 변경 파일에 시각 TSV가 없어 같은 원본·한컴 인쇄본·글꼴·96 DPI로 측정했다. 기준 `e961a78771`을 병합한 후보 `ff9194594e`과 쪽별로 대조한 20행에서 0행이 개선됐고 20행은 같은 일치율을 유지했다. 누락과 하락은 0행이었다. [원 커밋·기준·후보의 쪽별 측정과 PNG 해시](../assets/semanticist21-9337-9354-9377/original-candidate.tsv) · [전체 검증 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)

같은 Chrome 156에서 두 WASM 파일을 각각 불러와 실제 편집을 비교했다. 원 기여의 셀 속성 편집과 크기 조절에서 표 뒤 간격은 각각 12.4 px·0.6 px였고, 후보에서는 두 조작과 HWPX 재열기 모두 12.4 px였다. [두 실행 파일의 해시와 편집 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)
