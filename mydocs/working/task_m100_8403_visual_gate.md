# #8403 패턴 이미지 수정 중 확인한 문자 외곽선 차이

2026-10-31. 패턴 이미지 구현·검증은 승인된 범위대로 진행 중이다.
이 기록은 전체 완료 보고가 아니라 시각 gate의 추가 발견 사항이다.

## 관측

패턴 이미지를 복원한 첫 Native 빌드에서 원본 8쪽을 96dpi·print profile로 출력했다.
원본 PDF의 Poppler 출력과 `visual_sweep.py --silhouette-only --png-pair`로 비교했다.

| 쪽 | Native raw 실루엣 일치율 |
| --- | --- |
| 1 | 88.11% |
| 2 | 98.48% |
| 3 | 99.69% |
| 4 | 98.36% |
| 5 | 99.92% |
| 6 | 95.68% |
| 7 | 96.92% |
| 8 | 98.55% |

그림은 복원됐지만 1쪽 한글 제목이 기준보다 얇다. 실제 콘텐츠에는 `/F2 60 Tf`,
`1.715 w`, `2 Tr`가 있다. ISO 32000-1 §9.3.6의 text rendering mode 2는 글리프의
채움과 외곽선을 모두 그린다. 단순히 굵은 글꼴로 바꾸라는 선언이 아니다.
해당 Font는 `CAAAAA+NanumSquare`, 내장 TTF의 OS/2 weight 400, Regular다.

`--font-path /usr/share/fonts/truetype/nanum`과 Windows 글꼴 경로를 명시해도
1쪽 PNG는 동일했다. 같은 이미지 데이터를 직접 배치한 조사 진단본의 기존 Studio 출력에서도
제목이 얇다. 따라서 패턴 수정이 만든 문자 변화나 글꼴 파일 부재로 분류하지 않는다.
현재 PDF lowering은 글꼴 weight·fill colour를 보존하지만 이 실제 획 두께를 글자 paint로
연결하지 않는다. `issue_7966_pdf_text_paint_style`의 기존 검사는 혼합 글자색 보존이며
이 입력의 `Tr=2` 외곽선 지원을 입증하는 검사가 아니다.

## 증거와 한계

`output/black-image-20261031/` 아래에 다음 자료가 있다.

- `native-prototype/`: 첫 수정 Native 8쪽 PNG.
- `reference96/`: 원본 PDF의 독립 Poppler 96dpi PNG.
- `native-prototype-scores/silhouette.tsv`: 전쪽 원문 TSV.
- `native-fonts-prototype/`: 글꼴 경로 명시 후 1쪽 출력.
- `page-1-content.txt`: 실제 PDF 콘텐츠의 연산자.
- `direct/`: 제품 수정 전 직접 이미지 진단본의 기존 WASM 출력.

이번 수치는 초기 진단 빌드다. 후속 캐시·자원 경계 보정을 포함한 최종 Native/fresh WASM
결과를 대신하지 않는다. 1쪽 미달이 있으므로 전체 시각 검증 완료나 PR 준비 완료를 선언하지
않으며, 새 정식 렌더링 회귀 등록도 아직 하지 않는다.

## 처리 범위 결정

2026-10-31 작업지시자는 처음에는 이미지 수정에 한정한 예외를 승인했으나, 이어서
“결정을 번복합니다. 문자 외곽선도 함께 처리하세요.”라고 지시했다. **이전 예외는 철회됐다.**
패턴 이미지와 실제 문자 획을 함께 수정하며 전쪽 Native/fresh WASM 90% 기준을 적용한다.

글꼴을 강제로 굵게 치환하거나 제목 영역을 비교에서 숨기지 않는다. ISO 32000-1 §9.3.6의
채움/획 모드와 §8.4.3의 획 두께·색·선 연결 속성을 공통 모델로 전달한다. 획은 편집 가능한
글리프를 다시 그릴 때 적용하며 고정 그림을 덧씌우지 않는다. HWP/HWPX의 기본 속성으로
정확히 표현하지 못하는 paint는 기존 제한된 RHWP 확장 메타데이터 계약을 통해 왕복 보존하고,
다른 편집기에서의 표현 한계는 저장 진단에 명시한다.

구현 추적: PDF page/Form 그래픽 상태와 실제 text-show 순서 → 텍스트 paint 관측 →
PDF CharShape(문단·표·글상자에서 공유) → TextRun의 공통 TextStyle → Native Skia/SVG/Canvas2D paint.
폭·줄 높이·기준선 계산은 실제 글꼴 메트릭을 유지한다. 획 설정이 없는 정상 텍스트는 기존 경로를 쓴다.

현재는 구현 중이며 최종 검증 결과가 아니다. 새 회귀 등록은 전쪽 시각 선행 조건을 충족한 뒤 진행한다.

## 문자 획과 family 연결 보정

Native 중간 검증에서 실제 획을 복원한 뒤에도 1쪽은 88.91%였다. 추가로 원본의
`BAAAAA+NanumGothic-Bold`가 PostScript 표기이고 내장 TTF name ID 1은 `NanumGothic`임을
확인했다. CSS/Skia family를 임의 문자열 분해로 추정하지 않고 실제 name 테이블로 연결했다.
F2 `NanumSquare`의 Regular weight와 별도 `1.715 w / 2 Tr`는 그대로 유지한다.

이 보정의 Native 전쪽 결과는 94.70, 98.54, 99.69, 98.36, 99.92, 95.66, 96.91,
98.55%다. `native-family-fixed-scores/silhouette.tsv`가 원문이며 기준은 원본 PDF의
동일 Poppler 96dpi 출력이다. 1쪽을 직접 대조해 검정 이미지가 아닌 로고와 제목 획의 복원을
확인했다. 설치된 family와 원본 내장 프로그램의 버전·글자 위치 차이까지 완전히 같다는 뜻은 아니다.

공개 가능한 별도 대조군은 저장소의 OFL Noto Sans KR로 만든 PDF다. 채움과 획·획만 표시·
ExtGState의 색/투명도·선 연결·균일 CTM·hairline·중첩 Form을 포함한다. 앞선 Native 7쪽
중 6쪽은 100.00%였고 hairline은 54.73%여서, 폭 0을 채움으로 혼동한 Native 소비 경계를
보정하고 재검증 중이다. 이 중간 결과를 최종 통과나 회귀 등록 근거로 사용하지 않는다.

Studio TypeScript와 전체 2,692개 테스트 통과(2개 skip)를 확인했다. 최적화한 fresh WASM,
직접 CanvasKit, 편집·저장 재열기와 전체 Rust/Native 게이트는 아래 후속 결과로 확정한다.

### Native 문자 획 대조군 재검증

hairline을 폭 0의 stroke로 분리한 뒤에도 MuPDF 기준만 낮게 남았다. 직접 판독 결과
MuPDF 96dpi 출력은 해당 글자를 완전히 생략했고, Poppler는 같은 원본에서 얇은 외곽선을
그렸다. ISO 32000-1 §8.4.3.2의 폭 0은 장치 해상도의 최소 획이며 채움이나 생략이 아니다.
참조 PDF를 바꾸거나 기대 이미지를 제품 출력에서 만들지 않고, 원본 실물과 같은 Poppler로
대조군 전쪽을 다시 출력했다. Native 7쪽 및 HWP/HWPX 재열기 각각 전쪽 raw 100.00%다.
실루엣 점수와 별도로 hairline의 농도 차이는 남으며, 문자 외곽선과 일반 채움 대조군의
구분·내용·q/Q 복원은 직접 확인했다. MuPDF의 생략 출력도 진단 증거로 보존한다.

자료: `stroke-reference-poppler/`, `stroke-native-poppler-scores/silhouette.tsv`,
`stroke-native-final-hwp-scores/silhouette.tsv`, `stroke-native-final-hwpx-scores/silhouette.tsv`.
이 기록은 WASM 완료를 뜻하지 않는다.

## PDF Worker 전달 뒤 HWP 자르기 보존

첫 fresh WASM의 이미지 패턴 5쪽은 직접 열기 99.98%, HWP 재열기 98.91%였다.
직접 확대 판독에서 3쪽 Form의 오른쪽 타일과 5쪽 일부 경계가 HWP 저장 뒤 짧아졌다.
90% 통과를 이 차이의 면제 사유로 쓰지 않고 실제 저장 경로를 추적했다.

PDF lowering은 원본 2×2 이미지에 `img_dim=(150,150)`과 부분 선택 `crop.right=75`를
생성한다. Worker의 HWPX 전달본도 두 값을 보존한다. 그러나
`PdfImportSession::reopen_hwpx_export`가 `source_format=Hwpx`로 Core를 만들고,
Studio `exportHwp`가 HWPX 전용 adapter를 실행하면서 SC_PICTURE extra의 기준 크기를
0으로 기록했다. 원본 단위가 사라져 부분 선택이 잘못 해석됐다. 직접 Native PDF 저장은
`source_format=Pdf`이므로 같은 문제가 없었다.

선행 계보는 `47216ca6bd0`(#3930)의 한컴 HWPX 그림 extra 계약 →
`4662c069e5`(#7913)의 PDF 전용 HWP 변환 분리 → `f125c735c7`(#8360)의 요청 예산을
적용한 Worker 재열기다. #3930의 독립 한컴 저장·인쇄 근거와 이미 합쳐진 해당 변환을
그대로 유지한다. 전달용 HWPX는 기존 방식으로 읽되 이 PDF 전용 API가 만든 Core의
원본 형식을 Pdf로 복원해 다음 저장이 PDF 변환을 소비하게 한다. 일반 HWPX 열기와
HWPX 전용 그림 변환은 바꾸지 않는다.

수정 전 증거: `wasm-controls-hwp/page-3.png`, `wasm-controls-hwp/page-5.png`,
`controls.hwpx`의 `imgDim=(150,150)`, `controls.hwp` 재변환본의 `imgDim=(0,0)`.
수정 후에는 같은 Worker 전달·실제 HWP 저장·재열기와 5쪽 시각 비교를 다시 수행한다.

## 기존 #7998 원본의 획 표현 단위

전체 회귀에서 `issue_7998_pdf_table_inference`의 3개 검사가 같은 원본
`pdf/pr7704/6778-square-2020.pdf`를 가져오지 못했다. 원 해결은 #8120의
`80968497c2`이며 기존 보고서의 표·글자 소유 검증과 현재 base의 구현을 대조했다.
기존 assertion을 바꾸는 문제가 아니라 새 획 수용 조건이 기존 PDF를 거절한 회귀다.

원본은 `1.66 w / 2 Tr`와 직교 CTM `(0.119935, -0.119869)`를 사용한다. 두 축의
획 두께는 각각 0.1990921pt와 0.19898254pt이며 모델의 1/1000pt 저장 단위에서 둘 다
199가 된다. 배율이 완전히 같아야 한다는 조건 대신, 두 축을 각각 실제 저장 단위로
양자화했을 때 같은 폭인 직교 변환만 수용한다. 평균 폭·문서 이름·임의 배율 허용치를
사용하지 않는다. 다른 폭을 요구하는 비균일 변환과 퇴화 행렬은 계속 거절한다.
원본의 빈 dash 배열은 PDF의 실선이며 기존 구현에서도 정상 처리되고 있었다.

공개 문자 대조군 4쪽에 같은 폭으로 양자화되는 두 축과 명시적인 빈 dash 배열을 넣었다.
다른 폭을 요구하는 기존 비균일 변환 거절 대조군은 유지한다. 원본 3개 회귀는 기대값을
유지한 채 다시 실행하고, 변경한 대조군은 독립 PDF와 Native/fresh WASM을 다시 비교한다.

### 최종 기존 PDF 대조군의 자원 경계

`55557c3816` Native 직접 출력에서 기존 12쪽 PDF는 전쪽 98.97% 이상이다. 전체 HWP
변환도 완료됐지만 CLI의 전체 HWPX 변환은 기존 standard의 누적 decoded 작업 한도
2 GiB에 도달해 명시적으로 실패했다. live heap이나 파일 크기와 같은 값이 아니며,
기본 프로필보다 큰 override를 허용하지 않는 계약은 유지한다. 이 결과를 전체 HWPX
변환 성공으로 기록하지 않는다. 이 제한의 base 재현은 이번에 수행하지 않았으므로
이번 변경 전부터 같은 입력이 거절됐다는 주장도 하지 않는다.

획 수용 보정의 시각 보존은 동일 원본의 전체 12쪽을 확인한다. HWPX와 WASM Worker 경로는
지원되는 `sourcePages`/`--source-pages`로 1–6쪽·7–12쪽을 각각 가져온 뒤 독립 원본의
해당 쪽과 대조한다. 원본을 편집·래스터화하거나 실패 쪽을 빼지 않는다. 작업 대상 원본
8쪽과 공개 이미지·문자 대조군은 전체 문서 열기·저장·재열기 검증을 그대로 유지한다.

### 1픽셀 crop을 무시하던 소비 경계

최적화 WASM `a5533d35b4a4fc96c0b98d168b665054ee02d6d6139794f44912ba0f679a48c2`의
전쪽 raw 점수는 통과했지만 5쪽 공개 패턴 중 3·5쪽의 색상 확대 판독에서 경계 타일이
압축된 것을 확인했다. 3쪽 오른쪽 타일은 원본 2×2 중 왼쪽 1열만 보여야 한다.
IR의 `imgDim=(150,150)`과 `crop.right=75`는 맞지만 그리는 단계에서 폭 차이 1픽셀을
무-crop으로 취급해 오른쪽 녹색/노란색 열까지 좁은 프레임에 넣었다. 실루엣은 외곽만
비교하므로 100.00%여도 이 색상 선택 오류를 검출하지 못했다. 이전 완료 예정 판정을 철회한다.

계보: #43 `90dd52413f9`의 SVG crop → `0f2d2c0a2c`의 Native 재생 → #2990
`88730063f0`의 imgDim 대응 → #7688/#7697 `21ed64071f`의 등방 crop 단위 보정과
[독립 인쇄 검토](../pr/archives/pr_7697_review.md). 기준 `3c01421e89`에도 0.5/1픽셀
무시 조건이 남아 있다. 원래 crop 단위 계산과 기존 회귀 기대값은 유지한다.

소비 추적: PDF clip → lowering의 원본 좌표 crop → ImageNode/PaintOp →
`compute_image_crop_src` → 각 backend의 `is_cropped`가 결과를 다시 무시했다.
Rust의 공통 전체 원본 창 판정과 Studio의 공통 `imageCropSourceRect`가 실제 선택 창을
소비하도록 수정한다. 큰 이미지의 1픽셀도 원본 내용이므로 문서별·크기별 예외를 만들지 않는다.
전체 이미지 선택은 그대로 전체 이미지이며 타일 bbox·줄 배치·쪽수는 바꾸지 않는다.
최종 Native/fresh WASM 색상 영역과 저장 후 경계를 다시 확인한 뒤 완료한다.

### 최종 검증 스크립트의 텍스트 추출 보정

검증 기록을 재검토하면서 모든 쪽의 텍스트 해시가 같다는 점을 확인했다. 원인은 제품의
텍스트 유실이 아니라 검증 스크립트가 본문 문단용 `getPageText`를 사용한 것이다.
PDF에서 가져온 글자는 위치 지정 글상자에 있으므로 이 API의 빈 본문 결과로 저장 보존을
입증할 수 없다. 이 선행 해시 비교의 증거 효력을 철회한다.

스크립트는 실제 렌더 트리의 `getPageTextLayout().runs`를 읽도록 보정했다. 텍스트가 있는
대조군은 각 쪽의 비어 있지 않은 내용을 먼저 검사한 뒤 저장 전후 순서와 해시를 대조한다.
편집된 문자도 실제 출력의 `ktroke` 14곳과 `Stroke` 0곳을 별도로 검사한다. 제품 코드는
변경하지 않았으며 동일한 최적화 WASM으로 전체 브라우저 절차를 다시 실행한다.

보정한 브라우저 재실행은 통과했다. 원본 8쪽은 모두 비어 있지 않으며 2,987자와 페이지별
내용·순서가 HWP/HWPX에서 보존된다. 14곳 편집·기존 대조군·11종 거절 후 복구도 통과했다.
Canvas2D 117개 PNG는 앞선 최종 시각 측정본과 바이트가 같아 TSV를 그대로 연결한다.

### 기존 SVG 검사의 좌표 읽기 보정

최종 제품 `6d95c45f08`의 전체 회귀는 12,213개 중 12,209 통과, 4 실패, 45 skip이었다.
네 실패는 새 crop viewport 안의 원본 이미지 폭·높이를 페이지 좌표로 읽거나, 없는 `x/y`를
찾다가 다음 텍스트의 좌표를 읽던 테스트 helper에서 발생했다. 실제 그림의 페이지 프레임과
레이아웃 규칙은 바뀌지 않았다. 제품을 과거의 부정확한 crop 무시 조건으로 되돌리지 않는다.

| 검사 | 선행 계보와 독립 근거 | 읽기 오류와 유지할 계약 |
| --- | --- | --- |
| #683 / pr-149 | 원 PR #691 `049dc75318` → `22a4b87989`, `pdf/pr-149-2022.pdf`와 기존 #683 보고서 | 문자열 검색이 뒤 텍스트 y를 읽음. 그림 3개와 기존 cluster 간격 기대값 유지 |
| #5929 | 원 #6077 `ca1fb30d93` → `0c2f1cfe31`, 후속 #6193 `e28f977e0d` → `8d78e829ef`, 기존 HwpViewer 2024 대조 기록 | 원본 image의 x/y 부재를 그림 누락으로 판정. 표가 그림 하단보다 아래라는 관계 유지 |
| #7193 | 원 #7217 `0ce544f913` → `09204fd037`, 기존 Native/WASM·2020 PDF 검토 | margin 없는 대조군의 crop viewport를 놓침. 기존 원본 크기·안쪽 여백·paint/frame 관계 유지 |
| #6776 | `b1127b3520`, 후속 `64f33cfc1c`와 기존 #7445 이관 기록 | source image height를 페이지 높이로 읽음. 큰 TAC 그림 한 번 표시와 종이 내부 포함 유지 |

공통 테스트 reader는 `<image>`의 소유 crop viewport가 있으면 그 표시 사각형을 읽는다.
원래 그림인 경우에는 기존 x/y/width/height를 읽는다. 네 검사의 assertion·기대값·허용 오차를
바꾸거나 비활성화하지 않는다. fixture/golden 추가·갱신도 없으며, 과거 #6776 문서 전체의
미해결 시각 차이를 이번 작업에서 해결했다고 주장하지 않는다. 테스트 관측 경로의 보정이다.

pr-149는 독립 2022 PDF와 같은 제품의 Native/fresh WASM 전 1쪽을 추가 비교했다.
96dpi raw는 각각 99.98%/99.96%이며 세 그림·회색조·흑백·앞뒤 본문을 직접 판독했다.
실제 그림 y는 174.93/426.88/678.83이다. 잘못된 문자열 reader의 마지막 차이 273.28은
그림 간격이 아니라 뒤 텍스트를 읽은 값이었다. 이 확인으로 2개 페이지 비교를 추가했다.
기존 239개와 구분해 TSV·공개 PNG를 정식 validation 경로에 보존한다.

보정 뒤에는 영향받은 전체 검사 모듈과 Native 경로를 재검증한다. 제품·기준 이미지가
바뀌지 않은 범위의 성공한 시각·브라우저 검사를 반복하지 않는다.

### 완료 판정

2026-10-31 제품 `6d95c45f08`의 이미지·문자 획·crop 수정과 실제 사용 검증을 완료했다.
최적화 WASM SHA-256은 `17942ab2a5243aa9be2aea558c7449519b3607ca20368c1c94eee41cb1c52f69`다.
그 뒤 변경은 테스트 reader·증거·문서이며 제품 코드는 변경하지 않았다.

- 기본 전체 nextest: 12,213 실행, 최초 12,209 통과·4 reader 실패·45 skip.
  동일 제품에서 reader 보정 후 해당 모듈 8개 검사가 통과해 네 실패를 모두 해소했다.
  제품·기대값·허용 오차를 바꾸지 않았으므로 성공한 나머지 전체 검사를 반복하지 않았다.
- Native 라이브러리 4,040 통과·12 ignore. Native 패턴 3·자리표시자 2·직접 PDF 22 통과.
- Studio 2,692 통과·2 skip, TypeScript·fmt·Native/WASM/workspace lint와 manifest 통과.
- 실제 브라우저의 원본 8쪽·두 형식 저장·14곳 글자 편집·11종 거절과 복구 통과.
- 기본 239개 페이지 비교 및 pr-149 추가 2개 비교에서 전쪽 raw 90% 이상과 직접 판독 완료.
  원본 전 8쪽 최저 Native 94.70% / WASM 94.60%. 기존 문자 차이 예외는 사용하지 않았다.
- 문서 링크 통과. 메타데이터 기존 16건은 base의 동일 문서와 대조했으며 신규 오류 0건이다.

[완료 보고서](../report/task_m100_8403_report.md)와 정식 validation 경로에 결과를 보존했다.
비공개 원본과 로컬 글꼴 바이트는 포함하지 않았다. 로컬 완료이며 원격 PR·push·머지는 수행하지 않았다.


### 최신 devel 통합과 PR 제출 후보 — 2026-10-31

작업지시자가 PR 생성·self-review·머지를 승인했고 전쪽 raw 90% 이상을 제출 조건으로
확인했다. 기준 `99a176b7cb`에 #8397의 HFT/host font 정책을 유지해 통합했다.
Native `21a4902f14`와 fresh WASM `b13334f327`의 제품 코드는 동일하며 차이는
테스트 helper의 cfg(test) 소속뿐이다. 원본 8쪽은 두 저장 형식까지 Native 최저 94.70%,
WASM 94.77%로 통과했다. 241개 페이지 비교 모두 90% 이상, base 대비 최대 감소 0.02%p다.
HFT 별도 763쪽은 Native와 WASM 각각 base/candidate PNG가 전부 바이트 동일했다.

기본 전체 회귀 12,233건, Native 라이브러리 4,040건, Studio 2,700건이 통과했다.
최종 helper 1건, 첨부 문서 6개를 명시한 보안 검사 20건, 신규 샘플 포함 6종 래칫 81건,
fmt·lint·manifest·unit tier·문서 링크까지 통과했다. 기존 기대값·baseline 완화는 없다.
하네스의 새 글꼴 선택창·host provider/bridge 적용, 큰 글꼴 전달 분리, 공유 캐시 복원과
불필요한 전체 테스트 실행 파일 준비 중단은 [최종 증거](../tech/investigations/issue-8403/validation/rebase/README.md)에
구분했다. 제품 수정을 추가하거나 이전 실패를 지우지 않았다.

실제 Studio 공개 대조군의 Canvas2D/CanvasKit을 확인했다. 원본은 일부 글꼴 미준비로
Canvas2D에 fallback되어 실제 backend를 그대로 기록했다. 비공개 원본·로컬 글꼴 바이트는
게시하지 않는다. 승인된 Open PR → 동일 head CI → self-review 문서 trailing → CI 확인 →
머지 순서로 원격 처리를 진행한다. 상세 결과는 [완료 보고서](../report/task_m100_8403_report.md)를 따른다.


### PR #8410 Render Diff 디스크 부족과 수동 재실행 — 2026-10-31

제출 head `2e37e8f15025832c94dbc157061dd2ad1ddb261d`의
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/41194300572)는 최초 실행과
승인받은 1회 재실행 모두 checkout 중 `No space left on device`로 종료됐다.
캐시 복원·제품 빌드·렌더 검사는 시작되지 않았다. 서로 다른 runner에서 발생했으며
유실된 로그 때문에 공간을 소비한 디렉터리와 초기 여유 용량은 확정하지 못했다.
제출 head의 workflow 변경은 없었고 추적 파일 순증가는 2,909,837 bytes였다.

checkout 전 미사용 도구 정리안을 로컬에서 준비해 관련 계약 80건을 통과시켰지만,
원격에 적용하지 않았다. 사용자가 직접 시작한 동일 head의
[3차 실행](https://github.com/edwardkim/rhwp/actions/runs/41194300572/attempts/3)은
checkout·캐시 복원·WASM/Native 빌드·Canvas 비교·CanvasKit 검사를 모두 통과했다.
디스크 보정의 효과로 성공한 것이 아니며, 최초 두 실패의 근본 원인은 미확정이다.
불필요해진 CI 보정안은 이번 PR에서 제외했다. CI·CodeQL·Adapter·Proptest와
CI Impact Policy까지 성공했고 대기 중인 검사는 없었다. 이 code head 위에 self-review와
오늘할일만 추가하고 trailing head의 실제 CI를 확인한다.
