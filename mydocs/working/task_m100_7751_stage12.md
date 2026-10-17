# #7751 바탕쪽 raster 그림의 Print 참조 프레임 — 12단계

시작 HEAD `9d3caea79`.11단계의 전쪽 결과와 재검토 근거를 커밋한 뒤 시작한다.
PR 생성·승인·통합은 devel 대비 하락 재검토가 끝날 때까지 보류한다.

## 독립 관측과 적용 범위

현재 편람 원본의 한컴2024 Print와 Native 실제 paint op를 대조했다.
바탕쪽 JPEG 전체 폭은 PDF740px, Native740.7867px다. 같은 JPEG의 잘라낸
제목 부분은 PDF `(158.708,74.552,511.4467,74.0733)`px, Native
`(158.84,74.6267,512.0267,74.0533)`px다. 원본 master control의 위치 기준은
Paper이며 raster 그림은 독립적인 그림 배치 경로를 사용한다.

반면 바탕쪽의 동적 제목 텍스트는 Native 원점89.04px, PDF89.12px로 별도 GDI
글자 원점이다. 텍스트·벡터·본문을 통째로 축소하지 않는다. source 모델의 좌표와
측정·문단 흐름은 유지하고 Print의 바탕쪽 raster picture가 사용하는 목적 프레임을
물리 종이 좌표계에 맞춘다. 논리/물리 종이 비율은 독립 MediaBox에서 정한 값이며
문서 ID·그림 번호·페이지·점수에 따른 상수나 분기는 추가하지 않는다.

바탕쪽 출처는 기존 `RenderLayerInfo.master_page`가 명시한다. MIME은 기존
image resolver의 raster 식별을 사용한다. 본문 raster, 바탕쪽 글자·벡터/WMF,
Screen/preview의 그림은 이 변경을 적용하지 않는다. vector picture는 자체
metafile 좌표계를 소비하므로 raster 참조 프레임의 근거로 섞지 않는다.

## 생산 결과와 실제 소비

공통 page paint context에 논리/물리 종이 크기와 Print 여부를 한 번 정한다.
`print_bbox`가 있으면 기존 선택을 먼저 마친 뒤, 바탕쪽 raster image의
`PaintOp::Image.bbox`와 leaf bounds를 같은 목적 프레임으로 만든다.
그 뒤 `image.paint_bbox`의 원본 비율 inset과 crop source 단위는 그대로 사용한다.
Native Skia·WASM Canvas·SVG·direct PDF·JSON이 동일한 image op를 소비한다.
별도의 backend 위치 보정·clamp·비교 mask 변경은 하지 않는다.

## 선행 진단

동결한11단계 library의 공식 Native와 같은 font path·Print로15쪽을 출력했다.
비투영 대조군은 공식 PNG와 픽셀이 모두 같았다. 그림 op의 목적 프레임만
투영한 진단에서9개 하락 후보의 devel 대비 차이는0.003%p 미만이었고,
3/130/224/226/297/375쪽의 기존 중요한 경계도 유지했다. 이 진단의 글자·벡터는
투영하지 않았음을 로그에 남겼다. 진단 결과를 제품의 통과로 대신하지 않는다.

## 검증

1. 기존 편람 경계 focused5건으로 Native producer를 준비하고 대표15쪽을 직접 비교한다.
2. 실제 Native/fresh WASM 편람383쪽과 고정 devel의 raw를 재비교한다.
   하락0쪽·90% 미만0쪽·누락0과 쪽수/내용 보존을 확인한다.
3. 입력5개의 실제 Canvas/RGB·Native 계약과 Native Skia·전체 회귀·lint/Studio를 검증한다.
4. 완료된130쪽 등의 독립 그림·종이 좌표를±2px로 보호하고 관계 검사도 유지한다.
   적용 제외인 본문 raster/WMF·바탕쪽 글자·Screen 경로도 같은 source에서 확인한다.

## Native 선행 실행 결과

- 실제 제품 Native 경계5건이5PASS/0FAIL로 완료됐다.
- 별도 공개 API 계약은 바탕쪽 원본 raster의 종이 폭 관계와 Print 종이 내 포함,
  image op와 leaf의 공통 bounds, 본문 WMF·동적2025글자의 원본 프레임 보존을 검사한다.
  동결한11단계 library에서 compile0/test101, 현재 library에서 compile0/test0이었다.
  수정 전 실제 원인은740px 종이 밖으로 나간740.7867px의 바탕쪽 테두리다.
- 실제 제품으로 다시 출력한 대표15쪽은 사전 진단과 raw가 같았다.
  57/71/131/133/135/139/173/177/205쪽의 devel 대비 하락은 모두0.003%p 미만이다.
  3/130/224/226/297/375쪽도 유지됐다. 전체383쪽 판정은 아직 하지 않았다.
- upstream/devel을 다시 fetch한 결과fe5725dc3로 같아 rebase는 하지 않았다.
- SourcePrint `print_bbox` 선택 후 image의 목적 사각형을 정하며, 다른 op의 첫
  bbox는 원래 node bbox를 받는 생성자를 사용한다. 글자·도형의 leaf 프레임을 바꾸지 않았다.


## 전쪽 재검증과 정식 회귀 등록

실제 제품 소스 `0206db46656aa12e28a162b3324af07e3c4e3d32`에서 Native와
fresh WASM을 각각383쪽 새 출력했다. 독립 PDF는 같은 기존383쪽 파일을 재사용했다.
전쪽 raw 최저는 Native93.07066%,fresh WASM92.98362%이며,90% 미만·누락·
devel 대비0.01%p 이상 하락은 양쪽 모두0쪽이었다. Decimal로 고정 기준과 비교했다.
기준 산출물은 실제552514f42 생산본이며, 최신fe5725dc3 사이 해당 엔진 소스가
동일함을 확인했다. 실행하지 않은 최신SHA를 기준의 생산SHA로 바꾸지 않았다.
위 Native 선행 실행 당시의 전쪽 미판정 상태는 이 실행으로 해소됐다.

130쪽 raw는 Native98.69152%,WASM98.45828%였다. 두 경로의98% 조건을 확인한 뒤
`tests/cases/issue_7751_print_paper_frames.rs`에2개 정식 검사를 등록해2PASS였다.
독립 MediaBox555×754pt와754×555pt를 양쪽 방향에서 확인하며 논리 조판 좌표는
보존한다. 바탕쪽 전체 그림의 원본 종이 폭과 Print 종이 안의 포함 관계, 실제op와
leaf의 같은 프레임을 검사한다. 독립 PDF130쪽/Im8의96dpi 사각형은
`(0,0,740,1014.301333)`이며±2px 검사도 함께 둔다. 본문 WMF와 동적 제목 글자의
원본 프레임 보존으로 적용 제외 경로를 보호한다. 실패 원인의 before/after는 앞 절과 같다.

기존 편람 경계5건·일반 회색조/흑백 Native RGB2건도 각각5PASS·2PASS였다.
최종 fresh WASM의 실제 브라우저 묶음은8PASS였다. 여기에는 실제 저장 색조 입력5개,
HFT 윤곽 글꼴 선택, 실제 WMF crop의drawImage 소비, PNG의 실제FontFace 연결이 포함된다.
Studio는2,023PASS/0FAIL/0SKIP, TypeScript 검사는exit0이었다.
manifest는1,510sources/6,555static attrs/48targets, unit-tier는4,103tests/296modules로
고정base 대비 검사도 통과했다. 생성manifest/harness는 커밋하지 않는다.

대표 review·overlay는 [안정 경로](../pr/assets/pr7494_issue7751/README.md)에 보존했다.
최저90쪽의 작은 화면 그림·본문,130쪽의WMF source crop과 테두리,241쪽의이어받기
끝/제목/다음표,290쪽의Q&A종료 여백,297쪽의주의표·연도·법령명,375쪽의두단서식을
독립PDF와 같은 크기로 직접 대조했다. 내용 누락·중복·겹침은 관측되지 않았다.
일부 글자 힌팅·미세 전진폭과 그림 래스터화 색 차이는 남으며100%완전 일치로
표현하지 않는다. 원본의 선언 속성을 보존하고,224/226쪽 점수를 맞추려고 기각한
글꼴 축소 실험은 제품 코드에 포함하지 않았다. 남은 필수 전체Rust·Skia·lint와
정확한최종head CI는 이 focused/Studio 통과로 대신하지 않는다.


## 전체 Native Skia에서 검출한 기존 신원 검사

최초 전체lib는3,860PASS/1FAIL/12IGNORE였다. 실패는
`paint::resources::tests::source_image_key_changes_only_when_jpeg_base64_is_baked`로,
일반 GrayScale의밝기1/대비0 또는밝기0/대비1을워터마크wmpng로기대했다.
모델의두축워터마크계약과10단계독립Print의일반회색조RGB에따라 이원본신원은src다.
기존함수에서두단일축의src,두축의JPEG워터마크wmpng,RealPic제외,비JPEG제외와
zeroID거부를확인하도록보완했다. 제품코드·색조술어·허용치를바꾸지않았으며 새
source단위검사도추가하지않았다. 보완후focused와전체lib는별도재실행으로판정한다.

최종5개색조입력의전쪽Native/fresh WASM재산출도각각5쪽모두raw100%였다.
편람3쪽Direct PDF를현재제품으로다시변환·래스터화해독립Print와raw100%였다.
63쪽연결선과130쪽SVG payload는기존Direct PDF손실방지계약으로거부됐다.
130쪽Direct PDF도성공할것이라는첫진단assertion은잘못된가정이어서성공으로세지않았다.
전체편람Direct PDF지원이나해당guard해제를이PR의완료범위로확장하지않는다.
Visual Sweep Python계약156건도156PASS였다.
