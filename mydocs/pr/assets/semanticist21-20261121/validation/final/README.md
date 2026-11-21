# 편집 PR 17건의 최종 검증

검증한 제품 소스는 `521f69d19da9176e7947ceb318cf9838bec5ca75`, 기준은 `d7b5286cbf6401678a70eed3d66236a68dd2a72c`이다. 입력·한컴 Print 파일과 공급 글꼴의 해시를 맞춰 96 DPI에서 측정했다. 쪽별 보호 판정에는 색상 경계 보정 전 2px 실루엣의 공식 소수점 둘째 자리 표시값을 사용했다. [실행 파일·검증 조건](validation.json) · [원 PR별 검토](../../../../archives/pr_9292_review.md)

## 정본 보호

23종의 Native 2,023쪽과 fresh WASM 2,023쪽에서 하락과 누락은 각각 0건이다. 기존 하한과 같은 행은 3,971개, 상승은 75개다. 비교에 사용한 공식 표의 SHA-256은 `7f558bd4a1c8b901001492a977476efe1b1df3165dd3fef220f43ca1f87f55e3`이다. [4,046행 전쪽 비교](protected-pages.tsv)

| 처음 미달한 범위 | 보완한 행 | 해당 행 최저 |
| --- | ---: | ---: |
| 형식 사양서 · native | 3 | 100.00% |
| 형식 사양서 · wasm | 11 | 99.92% |
| 화학물질 표시 기준 · wasm | 39 | 99.88% |

같은 영역의 한컴 Print, 보완 전, 보완 후를 확대해 글자 모양·전진, 목차 점선, 캡션 시작 위치, 속이 빈 기호를 확인했다. [두 문서의 devel·후보 전쪽 비교 348행](devel-protected-pairs.tsv) · [53개 영역의 좌표](compared-regions.json) · [화학 문서 49쪽의 여는따옴표](chemical-p49-opening-quote.png)

## 아래아와 현대어의 인쇄 구분

형식 사양서 10쪽 본문과 71쪽 표에서 원문에 저장된 ‘ᄒᆞᆫ글’이 ‘한글’로 바뀌던 것을 보완했다. 11쪽에서는 같은 줄에 있는 옛한글과 현대어를 구분한다. 한컴 2020과 2024에서 인쇄한 제품 설명서에도 두 표기가 함께 쓰인다. [글자별 확대](araea-source-and-rendering.png) · [쪽·좌표·글자와 독립 PDF 해시](araea-source-and-rendering.json) · [해당 글꼴과 원본 경위](../../protected-fonts.md)

| 독립 인쇄 대조군 | 쪽수 | Native 최저 | fresh WASM 최저 |
| --- | ---: | ---: | ---: |
| 원 HWP 3.0 문서 · 한컴 2020 | 24 | 94.86% | 94.80% |
| HWP 저장본 · 한컴 2024 | 24 | 95.40% | 95.34% |
| HWPX 저장본 · 한컴 2024 | 24 | 95.40% | 95.34% |

## 편집·저장과 devel 대조

각주 본문과 참조의 줄 소속, 캡션과 그림 위치, 중첩 표의 분할, 조절한 개체의 속성, 빈 글자 겹침을 확인했다. 원본 열기·즉시 편집·HWP와 HWPX로 내보낸 뒤 다시 열기의 증거가 포함돼 있다. [기능 114행](feature-pages.tsv) · [원본 TSV별 경로·해시·바이트 수](source-tsv/index.tsv) · [최신 기준 세 묶음 2,118행](rebase-pages.tsv) · [차이가 보인 영역과 인쇄 행렬](rebase-inspection/README.md)

## 편집한 중첩 표의 쪽 경계

원본 HWP와 한컴에서 다시 저장한 HWPX에 각각 30줄·70줄을 입력했다. 편집 직후와 HWP·HWPX 재열기의 전 42쪽씩을 한컴 Print와 비교한 최저값은 Native 98.00640%, fresh WASM 97.86924%다. 3쪽은 58번째 줄과 아래 괘선을 포함하며 4쪽은 59번째 줄에서 시작한다. [독립 인쇄본과 84개 쪽별 결과](../../nested-row-split/edited/pages.tsv) · [마지막 줄과 괘선 확대](../../nested-row-split/edited/nested-last-line.png)

## 다음 쪽으로 밀려난 본문과 각주 참조

원 HWP와 각각 HWP·HWPX로 저장해 다시 연 문서를 한컴에서 인쇄했다. 세 상태 모두 2쪽 첫 단에서 세 번째 문단과 참조 번호가 함께 보이고, 이월된 각주 본문이 원래 순서대로 이어졌다. Native·fresh WASM 각 6쪽 모두 raw 100.00000%였다. [원본과 저장본별 전쪽 비교](../../footnotes/right-column-body.tsv) · [본문과 참조 번호 확대](../../footnotes/right-column-body-before-after.png)

## 편람의 옛한글 글꼴

편람 43쪽의 옛한글에서 문서가 지정한 한컴바탕의 글리프와 자간을 적용했다. 한컴의 내장 글꼴과 실제 HWPX Canvas 윤곽 5개·선분과 곡선 84개가 일치했다. HWP와 HWPX 모두 43쪽의 Native·fresh WASM raw는 100.00000%다. [인쇄본과 글리프 공급자](handbook-old-hangul.json) · [수정 전후 확대](handbook-old-hangul.png)

## 필수 검사

최소 글꼴 환경의 Rust 전체 회귀 13,946건, Native 공식 검사 4,071건과 Studio 2,822건을 통과했다. 편람 HWPX Native 383쪽의 최저 raw는 99.97%이고, 382쪽은 100.00%다. [검사별 실행 결과와 검증한 바이너리](checks.json)

상향한 75행과 이전·새 보호표 해시를 기록했다. 새 표를 적용한 편람·공개 문서 검사를 정식 명령으로 재실행했다. [행별 상향과 재검증 결과](../../../visual-protected-baseline/updates/semanticist21-20261121.json)
