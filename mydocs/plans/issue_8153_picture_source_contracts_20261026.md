# #8153: 편람 그림의 원점과 crop 자원 계약

## 실제 원본과 원인 분리

이 단계의 시작 기준 source `ddfacbc9fa305175a6e0237a7f6e14800a3cede4`의 두 편람은 전쪽
Native·표준 최적화 fresh WASM 1,532조건 중 1,380조건이 raw100·정수 미대응
0이다. 이 작업은 잔여 152개 조건의 원인 보정이며 원본·기준 PDF·점수 조건을 바꾸지 않는다.

HWPX 139쪽 원본 BMP의 현재 Print JPEG는 독립 PDF의 실제 PDFium decode와
RGB 전 픽셀이 같다. 현재 장치 경계는 `[139,658,586,853]`, 독립 PDF의
경계는 `[139,658,585,853]`이다. 원본·색 자원·본문을 유지하고 독립 그림
행렬만 재생한 진단은 미대응 **52→0픽셀**이다. 자원만 바꾸면 52픽셀이 유지된다.

141쪽은 반대다. 독립 행렬을 적용해도 정수 출력 영역과 34픽셀이 같다.
실제 원본과 배치를 유지하고 독립 그림 JPEG만 재생한 진단은 **34→0픽셀**이다.
두 진단은 `picture-frame-isolation/comparison.json`에 있으며 제품 개선 수로 세지 않는다.

139쪽 원본 앞에는 같은 글자 모양의 공백 6개, 앞 그림에는 7개가 있다.
원본은 11pt·장평 98%·자간 −12%다. 현재 측정은 공백마다 6.293333px다.
141쪽은 원본의 언어별 장평이 다르며, 공백의 상속 장평을 `font_space_em=0.49`로
환산하는 경로가 있다. copied input의 영문 장평만 바꾸면 측정과 Image layout x는
변하지만 최종 PNG는 RGB 전 픽셀이 같다. 이 결과만으로 공백 보정을 제품에 넣지 않는다.

실제 기준 PNG의 backend는 **pypdfium2/PDFium**, 정확한 96dpi 행렬이다.
PyMuPDF는 자원·벡터 추출 도구로 사용했으며 기준 PNG의 renderer로 혼동하지 않는다.
PDFium의 정수 장치 영역이 같으면 작은 행렬 변화로 표본이 달라지지 않을 수 있다.
`print_image_sampling::rasterize`의 정수 영역 자체를 결함으로 간주하지 않는다.

## 독립 계약 확인

1. 원본의 같은 Picture XML·BMP·글자 모양과 표 셀 소유를 복제한 작은 진단 문서를
   만든다. 원본의 0·1·6·7개 공백을 각각 실제 한컴 2024에서 저장·Print하여,
   원점·공백 전진·마지막 간격·그림 장치 경계를 분리한다. 새로 만든 작은
   대조군만 인쇄하며 이미 보존한 편람 383쪽 PDF를 다시 출력하지 않는다.
2. 141쪽 원본 crop과 전체 원본을 같은 wrapper에서 대조한다. 원본 bitmap,
   원래/현재 크기, 변환 행렬, clip 좌표와 저장 후의 실제 자원을 기록한다.
   현재 RGB555·압축 전달과 실제 재표본화의 선후 관계를 확인한다. 단순 crop
   원점 내림은 기존 RGB 오차를 늘렸고 Windows GDI+ 12조합도 독립 자원과 완전히
   같지 않았으므로 그 가설을 완료한 구현으로 채택하지 않는다.
3. 수동으로 편집한 저장 메타데이터의 가정과 한컴이 다시 저장한 정상 메타데이터를
   구분한다. 변경한 원본 속성과 기대 결과를 실행 전에 기록하며 실제 Print의
   행렬·자원·글꼴과 대조한다. 변환 engine·제품·Print method와 입력/PDF SHA를
   기록하고 인증 정보는 출력하지 않는다. 임시 입력·PDF·분석은 ignored output에 둔다.

## 공통 생산·소비와 제품 보정 기준

원점은 `원본 글자 모양/저장 줄 → 실제 공백 전진과 소유 셀/줄 원점 →
inline Picture layout bbox → PagePaintContext::image_frame →
PrintPaper 단위 → Native/Canvas의 공통 그림 장치 표본`을 추적한다.
측정 뒤 앵커 선택·반올림·덮어쓰기가 있는 실제 소비까지 확인한다.
crop은 `원본 imgClip·imgDim·bitmap → SourceWindow/색 전달 →
Print 압축 → 실제 decode·장치 표본`을 별도로 추적한다.

같은 사양을 쓰는 경로의 공통 생산 결과를 고치며, 문서·쪽·그림 ID 분기,
PDF 좌표 주입, −0.12pt 이동 같은 수치 보정으로 대신하지 않는다. 진단에서
사용한 독립 PDF의 행렬·압축 바이트는 제품 입력이 아니다. 소수 원점·비등방
축소·기존 본문 단 원점 대조군의 원래 계약을 먼저 확인해 보존한다.

보정 뒤 실제 두 원본의 영향 쪽 Native/fresh WASM과 확대 PNG를 먼저 확인한다.
방향이 맞을 때 전쪽 1,532개 조건과 소유·배치 보존을 확인한다. 적합한 독립 기대값의
정식 `tests/cases/issue_8153_...` 검사를 수정 전 FAIL/수정 후 PASS로 보호한다.
기존 자동 수집 경로를 쓰며 CI에 중복 호출을 추가하지 않는다.
목표 전쪽 raw 100%·정수 미대응 0과 최종 필수 검증 전에는 PR 준비 완료로 판정하지
않는다. upstream 동기화·원격 GitHub 쓰기·PR 생성은 하지 않는다.

## 단 원점 소유의 반례와 공통 보정

한컴이 실제 저장한 작은 대조군의 0·1·6·7개 공백에서 장치 x는
`[732,772,968,1008]`이다. 표의 가로 offset을 +6HU/−6HU로 바꾸면 각각
`[733,772,969,1008]`/`[732,771,968,1007]`이다. 단의 왼쪽 여백 +6HU는
기존 값을 유지하고, 표 바깥 여백 +6HU는 offset +6HU와 같다. `hasMargin=0`인
셀의 자체 여백은 비활성 속성이며 이를 바꾼 결과를 유효한 padding 검증으로 해석하지 않는다.

셀 원점 floor·상대 ceil 가설과 성분별 반올림 가설은 위 반례에서 실패했다.
기존 본문 그림의 `round(소유 단 원점) + round(단에 대한 상대 위치)`는
위의 실제 독립 장치 x를 모두 설명한다. 같은 계산의 실제 원본 139쪽 진단은
PDF 좌표·색 자원을 주입하지 않고 **52→0픽셀**이다. 진단 원본·명령·출력은
`picture-frame-isolation/source-column-*`에 있다. 제품의 전쪽 결과로 세지 않는다.

원본 계약은 HWP5 표 70의 글자처럼 취급 bit 0과 가로/세로 참조 축이며,
HWPX의 `treatAsChar`, `horzRelTo=PARA`, `vertRelTo=PARA`와 대조했다.
사양에 없는 Print 장치 양자화는 정상 저장본의 독립 인쇄 관찰로 구분한다.
사양 오류를 확인한 것은 아니다. 단일 문서 좌표로 사양을 정정하지 않는다.

현재 `publish_table_paint_layer`는 일반/분할 표에서 원본 참조 축을 발행하고,
`TableFillOrigin`은 실제 소유 단을 보존한다. 이 공통 소유 결과를 셀 내부 인라인
그림까지 전달하되, 인라인 여부는 원본 Picture의 `treat_as_char`로 명시한다.
중첩 표·글상자·머리말/꼬리말·각주·affine group은 독립 좌표계로 유지한다.
기존 그림의 crop 앞 경계 floor와 회전·채움·inset 경로는 현재 계약을 유지한다.
새 좌표 상수나 별도 backend 계산을 추가하지 않고 기존 장치 계산을 소비한다.
논리 bbox·내용 순서·소유는 바꾸지 않는다.

ubuntu-ted2의 `5df67d3299befb467a49dc4eaafd12d6e1877e2d`에서 WMF Print
자원의 source guard와 상위 clip 보존을 읽었다. 이 경로는 밝기 0과 전체 crop을
요구하므로, 밝기 −5와 crop이 있는 편람 130쪽에 그대로 적용하지 않는다.
읽기 전용 증거는 `ubuntu-ted2-picture-reference/read-only-metafile-print-20261026*`다.
원격 작업 트리나 현재 base는 변경하지 않았다.

139쪽의 독립 PDF 그림 영역을 쓰는 정식 회귀는 시스템 글꼴 없이 두 원본에서
실행한다. 다른 paint를 분리한 PNG의 투명 영역은 PDF와 같은 흰색 종이에 먼저
합성한다. alpha를 버려 검정으로 비교한 초기 진단은 제외했다. 기준 PNG를 유지한
동일 검사에서 이전 `ddfacbc9fa` 라이브러리는 **2 FAIL**, 보정 `49fa71149a`
라이브러리는 **2 PASS·0 FAIL·0 ignored**다. 이 focused 결과로 전쪽 검증을
대신하지 않았으며, 이후 같은 제품 source에서 아래 전체 결과를 확인했다.

## 제품 source 8b960562c8의 완료한 검증

Native와 표준 최적화 fresh WASM의 같은 source
`8b960562c8830709b05571a71c34b9349bff5909`에서 두 원본의 전쪽을 검증했다.

| 원본 | backend | 전체 쪽 | raw 100%·미대응 0 | 최저 raw | 미대응 합계 |
| --- | --- | ---: | ---: | ---: | ---: |
| HWPX | Native | 383 | 349 | 99.46283% | 1,232 |
| HWP | Native | 383 | 346 | 99.46283% | 1,248 |
| HWPX | 표준 최적화 fresh WASM | 383 | 348 | 99.25254% | 1,335 |
| HWP | 표준 최적화 fresh WASM | 383 | 345 | 99.25254% | 1,350 |

53쪽 4→0픽셀, 139쪽 52→0픽셀이 두 원본·양쪽 backend에서 모두 확인됐다.
전체 1,532개 조건 중 raw 100%·정수 미대응 0은 **1,380→1,388**, 잔여는
**144개 조건**이다. 미대응 픽셀 증가·쪽수 불일치·누락은 없다. Native의 논리
렌더 트리 JSON 766개도 이전 source와 바이트 단위로 같다.

145쪽은 미대응 픽셀이 Native 9개·WASM 6개로 그대로다. 비교 분모가 달라져
raw가 Native 99.99414→99.99406%, WASM 99.99606→99.99601%로 소폭 낮아졌다.
Native의 기존 미대응 mask는 전후 완전히 같고, 엄격한 이진 픽셀 차이는
18,561→14,619로 줄었다. 이 설명으로 목표 raw 100%를 면제하지 않는다.

정식 자동 수집 경로 `regression_suite_007`에서 새 그림 회귀는 **2 PASS·0 FAIL·
0 ignored**다. 실제 Canvas Print 기하 모드·Screen 복귀 브라우저 검사도 **1 PASS**다.
fresh WASM은 release·기본 최적화를 모두 포함하며 jobs 8·codegen units 16·
Binaryen 8로 빌드했다. 빌드만 약 38분 2초가 걸렸다. 임의 no-opt 결과로 대신하지 않았다.

증거는 `picture-frame-isolation/column-owner-product/`의
`same-source-whole-summary.json`, 문서별 Native/WASM `whole-score/silhouette.tsv`,
`native-render-tree-preservation.json`, `actual-ci-selector-receipt.json`, `browser.log`,
`standard-release-build/build.json`, Native/fresh WASM 139쪽 review·overlay다.
139쪽 그림과 표 경계를 직접 대조했다. raw 100%는 RGB 전체 동일의 뜻이 아니며,
기존 본문 획 농도 차이를 이 보정의 해결 범위로 올리지 않는다.

목표인 전쪽 100%와 최종 전체 필수 검증은 아직 완료하지 않았다. PR 생성·push·
upstream 동기화는 하지 않았으며, 남은 보정 뒤 PR 준비까지만 진행한다.

## 3개 공식 참조본에 따른 원본·소비 경로 분석

작업지시자의 지시에 따라 이후 보정은 인쇄 API 조합을 탐색하기 전에 다음 참조본의
원본 레코드 종류·단위·속성을 대조한다. 사양의 명시 규칙과 독립 인쇄에서 관찰한
미기술 동작을 구분하며, 화면 점수에 맞춘 수치나 문서별 분기를 추가하지 않는다.

- [HWP5 revision 1.3](../tech/한글문서파일형식_5.0_revision1.3.md):
  표 70의 글자처럼 취급·참조 축, 표 83~85의 원래/현재 크기와 저장 행렬,
  표 107의 그림 사각형·crop·안쪽 여백, 표 116의 기준 이미지 크기를 확인한다.
- [수식 revision 1.3](../tech/한글문서파일형식_수식_revision1.3.md):
  영문 기본 이탤릭·`rm`/`it`/`bold`, 첫 글자 크기 기준 `SCALE`, 항 구분과
  표시 간격을 구분한다. HWP5 표 105의 수식 글자 크기 단위는 HWPUNIT다.
- [차트 revision 1.2](../tech/한글문서파일형식_차트_revision1.2.md):
  ChartObj 구조와 저장 속성을 먼저 확인한다. 표 1의 차트 크기는 twip,
  표 3의 글꼴과 표 42의 pen 너비는 point이며, 일반 그림의 좌표와 혼용하지 않는다.
  HWP5 4.3.9.6의 OLE `Contents`와 `OOXMLChartContents` 구분도 따른다.

### 원본에서 확정한 구분

HWP 원본을 레코드 수준으로 열거하고 HWPX의 구역·바탕쪽 등 전체 Contents story를
대조했다. **두 원본 모두 그림 113개, 수식·OLE·차트 데이터 0개**다. HWPX 구역만
읽은 101개에 바탕쪽 그림 12개를 포함한 수다. 따라서 현재 그림 잔여를 수식·차트
엔진 문제로 분류하지 않으며, 그림에 포함된 기호에도 수식 기본 글꼴 규칙을 적용하지 않는다.
그림 밖 본문·글상자 문자는 HWP5 표 33의 언어별 글꼴·장평·자간·상대 크기·위치와
표 62의 저장 글줄 원점·높이·베이스라인을 대조한다. 같은 모양의 기호라도 실제
수식 레코드와 일반 문자 run의 적용 규칙을 혼용하지 않는다.

표 116은 기준 이미지 크기를 **HWPUNIT 4×2바이트**로 정의한다. 실제 HWP 그림
113개는 모두 91바이트·효과 flags 0이고, 기준 크기·crop·BinData의 조합이 HWPX
113개와 일치한다. `src/parser/control/shape.rs`도 기준 크기를 `Picture.img_dim`에
읽는다. 일부 주석의 “픽셀 크기·HWPX만 적재” 설명은 실제 타입·코드와 다르지만,
그 주석만으로 기능 손실이나 파서 결함을 판정하지 않는다.

그림 꼭짓점의 실제 INT32 8개는 `(x0,y0,x1,y1,x2,y2,x3,y3)` 순서로 읽을 때
HWPX의 점 4개와 **113개 모두** 일치한다. 현재 IR의 `border_x`/`border_y`는 각각
앞뒤 INT32 4개를 보존하고 HWPX 파서·직렬화기도 같은 scalar 배치를 쓴다. 이름만 보고
X 배열·Y 배열로 재배열하면 왕복 계약을 깨므로 변경하지 않는다. 표 107의 원문
표기와 실제 저장 배치를 구분해 설명해야 한다. 독립 hwplib의 고정 SHA에서 reader와
writer도 같은 좌표 쌍 순서를 확인했다. 원문 PDF를 보존하면서 프로젝트 설명본 표 107과
[정오표 37절](../tech/hwp_spec_errata.md#picture-corners-storage)을
정정했다. 표 116의 `X2`도 HWPUNIT 너비·높이 두 값으로 풀어 썼다. 이 필드는 현재
renderer에서 직접 소비하지 않으므로 이번 raw 잔여의 원인이라고 주장하지 않는다.

### 다음으로 대조할 실제 변환 경계

141쪽 그림은 HWP/HWPX 모두 기준 크기 `(80280,30900)`과
crop `(1091,0,79016,30900)`을 저장하고, 실제 bitmap은 `1070×412`다.
기준 좌표를 raster로 대응하면 x 시작은 `1091×1070/80280`이며 약 14.54123px다.
이 원본 범위는 `Picture.crop_reference_size` → `ImageNode.original_size_hu` →
`SourceWindow::for_image`로 전달된다. 그 뒤 `print_tone::print_bitmap_payload`가
정수 내포 창 `[15,0,1053,412]`을 잘라 `OutputDeviceWindow`를 발행한다.
`image_resolver`의 window 소비와 최종 paint frame까지 이 대응이 보존되는지 확인한다.
이 정수화가 HWP5의 명시 규칙이라고 간주하거나 독립 근거 없이 필터를 바꾸지 않는다.

원본 레코드·XML·대응 비교와 참조본 SHA는 ignored
`output/pr-review/issue8153-handbook-raw100-20261022/remaining-raw100-diagnostic/spec-contract-audit/`
아래에 있다. 원본·PDF·글꼴·현재 base는 변경하지 않았다.

원본 crop을 되살리고 같은 bitmap의 Native full-source JPEG와 기존 sampler로
재생한 141쪽 진단은 **34→30픽셀**이다. PDF 자원·좌표나 새 필터는 주입하지 않았다.
그러나 전부 해결되지 않았고 미대응 위치도 달라져 제품 보정으로 채택하지 않았다.
이 결과는 `spec-contract-audit/source-crop-replay-comparison.json`에 별도로 두며
위 제품의 1,388개 완료 조건에 합산하지 않는다.

### 일반 문자와 저장 줄 계약의 실제 대조

145쪽의 `행정안전부장관`은 수식·차트·그림이 아닌 실제 일반 문자 run이다.
HWP5 표 33·34·62와 원본 HWPX의 `charPr 386`, 언어별 `fontface`, 저장
`lineseg`를 현재 동결 라이브러리로 연 두 원본의 RenderTree·최종 paint까지 대조했다.

| 계약 | 원본 저장값 | 두 원본의 실제 소비값 |
| --- | --- | --- |
| 글자 크기 | 1,100HU·상대 크기 100% | 11pt = 14.6666667px |
| 한글 장평 | 98% | ratio 0.98 |
| 한글 자간 | −6% | letter spacing −0.88px |
| 글자 위치 | −25% | 줄 기준선에 −3.6666667px 한 번 적용 |
| 저장 줄 높이 | 1,844HU | 24.5866667px |
| 저장 베이스라인 거리 | 1,567HU | 20.8933333px |
| 굵기·기울임 | 해당 속성 없음 | bold false·italic false |

한글 fontRef 7의 원래 face는 `정부상징 부처명_16040911`이다. 언어별 fontface
목록의 같은 숫자를 혼용하지 않는다. 최종 표시 사다리는 HWPX
`정부상징 부처명_16040911,한컴바탕`, HWP
`정부상징 부처명_16040911,Haansoft Batang`이다. 파일에 요청된 face와 실제 선택된
대체 프로그램도 구분한다. 별도 읽기 전용 비교에서 독립 PDF의 Haansoft Batang
subset과 설치본의 해당 7개 글리프 윤곽·전진·힌트가 같은 것은 확인했으나,
그 사실만으로 두 raster backend의 출력이 동일하다고 판정하지 않는다.

두 원본의 run bbox와 위 해소 값은 같다. 논리 줄 기준선은 412.0533333px,
글자 위치 적용 뒤는 408.3866667px다. 최종 Skia와 Canvas가 공통
`script_draw_metrics`를 소비한다. 이 경로에서 원본 값의 누락·이중 위치 적용을
확인하지 못했으므로 값 보정이나 임의 offset을 추가하지 않는다.

`spec-contract-audit/source-text-contract-audit.json`에 독립 원본과 소비값의
계산 대조를 기록했다. `handbook-hwp`·`handbook-hwpx` 하위 JSON은 그림·글꼴
바이트를 제외한 진단이며, 선택한 run만 남긴 paint JSON의 source table 경고는
이 진단 축소에서 생긴 것으로 제품 검증 결과로 사용하지 않는다. 기존 전체 TSV를
재산출하지 않았고, 이번 사양 설명·진단 기록으로 144개 잔여가 개선됐다고 세지 않는다.
