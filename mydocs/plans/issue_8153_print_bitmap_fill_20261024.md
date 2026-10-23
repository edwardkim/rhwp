# #8153: Print bitmap 채움의 원본 창과 물리 프레임

## 변경 전 계약과 근거

HWP의 영역 맞춤과 HWPX의 `TOTAL`은 같은 배경 채움이다. 의미 조판의 셀·문단 상자와
원본 그림의 크기는 유지하고, 인쇄에서 재생할 그림 프레임과 원래 채움 clip을 구분한다.
현재는 일반 Picture의 가로 축 투영을 배경에도 사용해 세로 장치 좌표·원본 창·clip을 잃는다.
편람 두 형식의 290쪽은 Native raw 98.59333% / 98.59338%, fresh WASM raw
98.60005% / 98.60022%다. 같은 독립 2024 Print에서 가로 배경 경계가 남는다.

공개 `Graphics::DrawImage`의 명시적 원본 픽셀 창과 전체 Metafile 자동 선택을 구분한다.
독립 출력의 확대는 원래 채움 사각형을 장치 1픽셀씩 확장한 목적 영역과 원본 bitmap
픽셀 수의 source rect를 조합한 공개 API 실행으로 재현된다. 8×9와 실제 1677×591,
세 높이씩의 변환이 일치한다. 32×17의 DPI 96/300 대조군은 같은 영역 맞춤 결과이며,
513×257에서도 재현된다. 오른쪽 위 정렬은 다른 계약이므로 영역 맞춤으로 바꾸지 않는다.

공식 단위·source/destination 계약:

- [ENHMETAHEADER](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-enhmetaheader):
  `rclFrame`은 0.01mm, `szlDevice`는 pixel, `szlMillimeters`는 mm이며 오른쪽·아래 경계를 포함한다.
- [Graphics::DrawImage](https://learn.microsoft.com/en-us/windows/win32/api/gdiplusgraphics/nf-gdiplusgraphics-graphics-drawimage(image_constrect__int_int_int_int_unit_constimageattributes_drawimageabort_void)):
  source rect의 단위를 별도로 지정하고 선택한 원본을 destination rect에 대응시킨다.
- [MS-EMF BitFIX28_4](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-emf/075e694f-9018-48ad-a22c-81dd07c67c69):
  장치 좌표의 소수부는 1/16 단위다. 이 자료는 좌표 표현의 근거이며, 아래의 실제 API
  정수화 순서 전체가 사양에 명시되어 있다는 뜻은 아니다.

공개 API 실행에서 확보한 15개 독립 변환(원본 8×9·1677×591·32×17·513×257,
32×17의 DPI 96/300, 각각 세 목적 높이)을 별도로 대조했다. 513×257의 첫 높이는
투영 길이를 바로 내림하면 장치 1픽셀 차이가 난다. 28.4 끝점으로 먼저 반올림한 뒤
올림하고 포함 경계의 마지막 픽셀을 제외 끝점으로 옮기는 순서로 보정했다.
이후 15개 변환과의 최대 차이는 장치 0.00048828125픽셀(f32 변환 정밀도)이다.
이는 독립 변환의 진단 결과이며 최종 Native/fresh WASM 시각 검증을 대신하지 않는다.

2024 Print bitmap carrier의 참조 장치(1024×768pixel / 271×203mm)와 정수 HIMETRIC
프레임으로 source pixel 창과 물리 프레임을 구분한다. 이는 실행 호스트의 화면 DPI를
읽는 설정이 아니다. 독립 출력과 공개 API에서 확인한 Print 장치 계약으로 취급하며,
문서 ID·페이지·폭에 따라 수치를 선택하지 않는다. HWP/HWPX의 원본 조판 수치는 바꾸지 않는다.

## 생산과 소비

`border_fill_background_nodes`의 의미 bbox·fill mode → 공통 paint builder의
`PagePaintContext` → 그림 프레임과 clip의 한 결과 → Native/Canvas2D/PDF의 동일한
`PaintOp::Image`·`LayerNode::ClipRect` 소비로 연결한다. 일반 Picture의 테두리 프레임
캐시에 배경의 확장 프레임을 넣지 않는다. backend별 좌표 예외를 만들지 않는다.

- 적용: 2024 Print, 축 정렬, BMP/PNG/JPEG의 영역 맞춤(`FitToSize`/`Total`), 원본 창 전체.
- 비적용: Screen·2022, 자연 크기·정렬·반복·Zoom, affine 그룹·회전·별도 crop/inset.
  이 경로는 서로 다른 source/destination 계약을 사용하므로 기존 전송을 유지한다.
- 원본 HU에서 장치 원점·크기를 각각 정수화하고 물리 종이의 두 축으로 투영한다.
- source pixel 수와 HIMETRIC 프레임을 사용해 인쇄 재생 너비·높이를 계산한다.
  확장 그림은 원래 채움 clip 안에서만 보인다. 조판의 높이·다음 내용은 이동하지 않는다.

## 검증과 완료 조건

1. 독립 Print의 실제 배경 3개·Q 그림 2개의 프레임·clip을 양쪽 형식에서 대조한다.
   변경 전 소비 결과와 원본 bbox를 고정한다. 진단용 예측의 최대 차이는 0.00370pt이며
   이 값으로 기대값을 바꾸거나 완료를 선언하지 않는다.
2. 먼저 영향 쪽 282·284·286·290·291·292·294·295·299·306·307과 정상 대조군을 Native로
   직접 확인한다. 감소·누락·그림 경계 차이가 있으면 전체 실행 전에 원인을 보정한다.
3. 같은 source SHA에서 fresh WASM을 빌드하고 같은 범위의 PNG·TSV를 대조한다.
   관련 전쪽 90%·직접 판독을 충족한 뒤 독립 PDF 기준의 관계·±2px 회귀를 정식 등록한다.
   임시 consumer 검사는 `output/`에서 수정 전 FAIL / 수정 후 PASS를 먼저 확보한다.
4. 전쪽 Native/fresh WASM 및 PR8148 corpus를 같은 base와 비교한다. 기존 정상 문서의
   후퇴가 있으면 공통 로직을 보정하며, 필수 최종 검증을 완료한 뒤 PR을 제출한다.

이 문서는 구현 전 계획이다. 실제 보정·시각 결과·정식 회귀와 최종 검증은 아직 완료되지 않았다.
비공개 분석과 임시 산출물은 `output/`에만 보존한다.

## Native 실행 결과

- 후보 `b919780c3b29cc5f4fa9f46738ab0858e378d4ec`, jobs 8, 공용 `target/pr-review`의
  `release-test` + `native-skia` 실제 빌드: 396.24초, 종료 코드 0.
- 두 형식 각 13쪽, 26개 PNG의 SHA·실루엣을 확인했다. 기존 `2cb8223029`와 비교하면
  미대응 픽셀 수의 합은 동일하고, raw 최저 변화는 -0.00050%p다. 프레임 보정만으로
  전쪽 raw 100%에 가까워졌다고 보고하지 않는다.
- 독립 Print p290의 그림 5개, 원본 소유·순서·의미 bbox, 확장 그림과 원래 clip 관계를
  검사한 진단은 이전 producer에서 실패하고 후보에서 두 형식 모두 통과했다.
- p290의 raw는 HWPX 98.59283%, HWP 98.59288%이며 각각 미대응 픽셀 1,534개가 남았다.
  실제 PNG와 독립 PDF의 그림·clip·흰 채움 연산을 대조했다. 다음 단계에서 래스터 단계의
  정수 출력 크기와 실제 DPI 대응을 독립 길이 계약으로 확인한다. fresh WASM·정식 회귀와
  전쪽 검증은 이 기록으로 완료 처리하지 않는다.
