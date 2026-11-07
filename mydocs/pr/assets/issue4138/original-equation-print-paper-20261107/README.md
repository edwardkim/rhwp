# 원본 수식의 Print 종이·글줄·SVG 공급 보정

2026-11-07. 부모 `dbf9b54beb419f26d1595b6bfe792f17cf4498b6`, upstream base
`a7feb849e11642b1438b7e9ee460ff4c8a7c946e`. 이 증적과 함께 커밋한 runtime 파일의 SHA는
validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue4138/original-equation-print-paper-20261107/validation.json`)에 고정했다. 브랜치 전체 검증 완료나 PR 제출 가능 판정은 아니다.

## 원인과 공통 소비 경로

- 본문 글리프의 물리 PrintPaper를 구형 원본 HFT 수식만 소비하지 않았다. 공통 paint가 같은
  프레임을 명시적 그룹 행렬로 발행한다. source bbox·저장 크기·글자 크기는 유지한다.
- 단일 글줄의 자연 표시 기준선과 저장 호스트 기준선을 혼용했다. 같은 원본 face·단일 기준선의
  글줄만 기존 논리 높이에서 표시 원점을 발행한다. 채우기 뒤의 반올림 상태와도 구분한다.
- SVG는 변경 전 layout의 표시 조각을 캐시에 남기고 그룹 행렬을 재생하지 않았다. paint layout과
  SVG 조각을 함께 생산하고, SVG도 공통 그룹 행렬을 실제 자손에 전달한다.
- 별도 수식 SVG의 실제 family 대신 문자의 종류로 다른 수학 face를 추정했다. 표시 조각의 선언을
  임베딩 요청까지 전달한다. 명시한 폴더의 파일명이 family와 달라도 내부 이름·스타일·문자 범위로
  같은 파일을 찾으며, 무관한 family를 공급하지 않는다. 폰트 파일을 새로 설치·배포하지 않았다.

## 검증

정식 회귀는 관계형 8개·고정형 5개다. 고정형은 독립 한컴2024 Save 후 Print의 40조건·68글리프
원점을 192DPI ±2px로 대조한다. 4개 입력/PDF와 producer·job·SHA는
[print-oracle.json](../../../../../tests/fixtures/issue4138/original-print-paper/print-oracle.json)에 있다.
동일 4쪽은 Native/fresh WASM의 직접 PNG와 SVG raster에서 각각 모두 100%다.
SVG 8개에 포함된 수식 원본 글꼴의 바이트도 명시한 공급 파일과 같다. 사설 폰트 바이너리·SVG는
ignored output에만 남겼고, 아래에는 PNG와 TSV만 보존했다.

보정 전 실제 실패를 단계별로 확인했다: 물리 프레임 4 PASS·1 FAIL, 오래된 SVG 조각
9 PASS·1 FAIL, SVG family/행렬 10 PASS·2 FAIL, 임의 파일명 공급 12 PASS·1 FAIL.
최종 새 정식 13 PASS, 집중 묶음 130 PASS(기존 output 진단 1개 포함), SVG 소비·경계 묶음
31 PASS다. 13개는 집중 묶음에도 포함되므로 중복 합산하지 않는다.
fmt·workspace all-target Clippy·native-skia Clippy·WASM Clippy·suite manifest·unit tier 통과.
Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며 Docker 결과가 아니다.
#8696 정확한 원본 3종과 독립 글상자 대조 3종도 두 경로에서 100%다.

## 실제 분할 원본의 전197쪽

[native 전](native-before-silhouette.tsv) · [native 후](native-after-silhouette.tsv) ·
[WASM 전](wasm-before-silhouette.tsv) · [WASM 후](wasm-after-silhouette.tsv).
양쪽 모두 쪽수·전쪽 결과를 다시 산출했다. 부모 대비 변경 PNG는 각각 7쪽이며 190쪽은 바이트가 같다.
Native 최저 95.77112%, fresh WASM 최저 95.77492%. 129·130쪽의 복합 그룹·괄호·첨자 간격이
남아 있으며, 해당 미완료 원점을 고정형 검사로 승인하지 않았다. 사용자 요청의 전쪽 98%에
도달하지 않았으므로 추가 보정을 계속한다. 이 단계는 최종 전체 nextest·handbook gate·실제
Studio 소비 경로 검증을 대신하지 않는다.

## 직접 판독 그림

- SVG 원본 공급 전: [review](svg-before-a4-review.png) · [overlay](svg-before-a4-overlay.png).
- SVG 원본 공급 후: [review](svg-after-a4-review.png) · [overlay](svg-after-a4-overlay.png).
- 혼합 문자·숫자·연산자 Native SVG: [review](svg-after-mixed-review.png) · [overlay](svg-after-mixed-overlay.png).
- 같은 fresh WASM SVG: [review](svg-wasm-mixed-review.png) · [overlay](svg-wasm-mixed-overlay.png).
- 원문 129쪽 잔여: [review](native-p129-review.png) · [overlay](native-p129-overlay.png).

대조군은 문자·숫자·연산자·괄호의 누락·크기·기준선을 직접 판독했다. 원문 129쪽의 남은 가로
차이는 별도 논리 전진·그룹 소유 문제로 구분하며, 높은 점수로 완료 처리하지 않는다.
