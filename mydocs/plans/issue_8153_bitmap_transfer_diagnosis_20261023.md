# #8153 본문 그림 전송과 273쪽 하락 진단

기존 [구현 계획](issue_8153_handbook_raw100_20261022.md)의 후속 조사다.
이 기록은 완료된 원본 검증과 조사용 입력을 구분하며, PR 제출 완료를 뜻하지 않는다.

## 현재 원본 검증

코드 후보는 `ed630b398e87035a7bff35db7484ca864f2ad401`, 직전 후보는
`a8119d4885edfc36a142fffd979820757062ca7f`다. 두 후보의 원본 편람과 독립 Print PDF는
동일하다. Native 96dpi, threshold232, radius2로 전 766쪽의 TSV와 정확한 미대응 마스크를
확인했다. `target/pr-review`에서 jobs8로 빌드한 동결 producer를 사용했다.

| 형식 | 쪽수 | 실제 미대응 0인 쪽 | 최저 raw % | 전체 미대응 픽셀 |
| --- | ---: | ---: | ---: | ---: |
| HWPX | 383 | 174 | 93.26012 | 68276 |
| HWP | 383 | 143 | 93.25979 | 67380 |

직전 후보의 미대응 픽셀은 HWPX68258/HWP67362였다. 미대응 0인 쪽은 HWPX에서 2쪽 늘었지만,
총 미대응은 두 형식 각각 18픽셀 늘었다. 평균 개선이나 완료 쪽 증가로 하락을 면제하지 않는다.
전 766쪽의 semantic render tree는 동일하며, 실제 관측된 대체 글꼴 정보도 동일하다.

두 형식 모두 273쪽이 직전 대비 0.11899%p 하락했다. 현재 base 대비 하락은 HWPX0.11922/HWP0.11915%p다.
HWPX의 미대응은 2216→2437픽셀이며, 위 그림은 305→311, 아래 그림은 1911→2126이다.
새 미대응 419개와 해소 198개가 공존한다. 문단·쪽수·텍스트 소속 변경으로 보고하지 않는다.
이 하락과 전쪽 100% 미완료 때문에 PR 제출은 계속 보류한다. 현재 코드의 fresh WASM 전쪽 검증도 남아 있다.

## 그림 데이터와 배치를 구분하는 조사

273쪽의 원본은 `image432.bmp` 877×510과 `image433.bmp` 777×522이며 묶음 개체의 자식이다.
원본 바이트·묶음 행렬과 그림 소속을 추적했다. 이미 좌표에 적용한 묶음 변환과 paint 단계의
잔여 transform을 구분한다. 단순히 묶음이라는 이유로 검증된 본문 장치 변환을 끄지 않는다.

별도 진단 사본에서 두 리소스만 원본 Print PDF의 JPEG로 바꿨다. Native 쪽수와 semantic
tree는 그대로이며 미대응은 2437→1670, raw는 99.08875%다. 이미지 전송은 차이의 일부를
설명하지만 전부는 아니다. **정답 PDF의 픽셀을 복사한 이 결과는 인과 조사에만 사용한다.**
원본 검증 점수·정식 fixture·구현 기대값·완료 증거로 사용하지 않는다.

## 독립 Print로 확인한 계약과 제외한 추정

진단 입력은 원본 리소스와 기존 유효한 한컴 구조에서 만들었다. Hancom13.0.0.3901,
PrintMethod0, `frame_print_to_pdf_ex_one_up`의 실제 job, 출력 해시, PDF 이미지 객체를 확인했다.
기존 편람 Print PDF를 다시 만들거나 교체하지 않았다.

- 동일 BMP의 DPI 메타데이터만 0/150/300으로 바꾼 세 조건은 같은 JPEG 객체를 재사용했다.
  실제 디코딩한 원본 RGB는 동일하다. 이 범위에서 DPI 메타데이터가 압축 차이를 만든다는 가설은 지지되지 않는다.
- 서로 다른 원본 BMP 여섯 개를 같은 표시 크기에 배치해 JPEG 양자화표를 비교했다.
  v1은 뒤 세 리소스의 manifest href가 기존 TIF/JPEG를 가리켜 해당 조건을 제외했다.
  v2는 manifest를 수정하고 여섯 실제 원본 크기가 모두 맞는 것을 확인했다.
  양자화표는 표준 JPEG quality35/53/35/52/35/35와 각각 같지만 이를 구현 상수로 채택하지 않는다.
- 겹치지 않는 앞 두 그림의 XML 및 z 순서만 바꿔도 각 JPEG 바이트는 그대로다.
  이번 범위에서 처리 순서가 원인이라는 가설을 제외한다.
- 같은 BMP의 표시 크기를 달리한 4쪽 PDF는 이미지 객체를 한 번만 생성했다.
  재사용된 객체의 양자화표가 같다는 사실을 크기별 독립 압축의 증거로 오인하지 않는다.
  중복 재사용을 배제한 별도 문서 4개도 실제 Print를 완료했다. 원본 463×538 BMP의
  가로 크기 7200/14400/21600/28800HU는 각각 표준 quality35/35/46/62와 같은 양자화표다.
  실제 JPEG 크기는 모두 463×538이며 바이트 수는 15950/15950/18979/23309다.
  이 결과는 고정 품질 가정을 반박하지만, 네 점에 맞춘 식을 사양이나 구현 근거로 승격하지 않는다.
  기존 묶음 개체의 양자화표까지 같은 좌표·크기 소비 경로로 설명해야 한다.

[한컴 공식 PDF 저장 설명](https://help.hancom.com/hoffice130/ko-KR/Hwp/file/to_pdf.htm)은
자동 그림 품질이 이미지 크기에 따라 압축 비율을 정한다고 설명한다. 이 문서만으로 현재
Print 경로의 양자화·축소 수식을 확정하지 않는다. HWP 사양의 좌표·크기와 PDF 출력기의
압축 정책을 구분하며, 이 관측을 공식 HWP 사양 오류라고 주장하지 않는다.

## 조사 자료와 다음 검증

모든 임시 자료는 `output/pr-review/issue8153-handbook-raw100-20261022/` 아래다.

- `body-raster-half-ed630b398e/whole-native-exact-audit.json`: 전쪽 PNG 해시와 정확한 미대응 수
- `body-raster-half-ed630b398e/whole-native-semantic-font-audit.json`: 전쪽 트리와 관측 글꼴 비교
- `body-raster-half-ed630b398e/diagnostics-273/`: 원본 비교 PNG, 추가/해소 마스크와 진단 사본 receipt
- `picture-pen-probes/{bitmap-dpi,bitmap-content-v2,bitmap-order,bitmap-scale}-print-job.json`: 독립 실행·출력
- `picture-pen-probes/*-inputs.json`, `*-output-audit.json`: 입력 해시와 실제 PDF 객체
- `picture-pen-probes/bitmap-separate-size-{inputs,output-audit}.json`: 별도 문서별 크기 실험
- `logs/bitmap-separate-size-independent-print.log`: 위 실험의 실제 실행 로그

다음은 별도 문서별 압축·픽셀 전송의 독립 대조와, 최종 프레임에서 미세선이 보존되는 재구성
계약 확인이다. source ID별 분기·품질 상수·픽셀을 지우는 임계값 변경으로 점수를 맞추지 않는다.
근거를 확보한 공통 경로를 보정한 뒤 원본 영향 쪽과 정상 대조군을 실제 재출력한다.
최종 승인에 쓰는 축소 입력·기준 PDF는 조사용 output 자료와 구분해 정식 경로에 보존해야 한다.

## 그림 실선의 장치 pen 소비 보정 후보

최적화 fresh WASM `ed630b398e`의 핵심 52쪽은 exit 0으로 완료했다. 두 형식 모두 383쪽이며
최저 raw는 HWPX 93.24686%, HWP 93.24774%다. 46쪽은 양쪽 WASM에서 100%지만 Native는
95.74702/95.74674%다. Canvas의 범용 최소 선폭과 Native의 원본 2HU 직접 소비가 서로 다르다.
이 수치를 양쪽 구현의 일치나 완료로 보고하지 않는다. HWP 205쪽의 backend 차이도 남는다.

실제 원본 46쪽과 기존 폭 15종×ROUND/FLAT 대조군에서 선언 폭 0/1/2HU의 채운 띠는
장치 한 픽셀이고, 28HU는 두 픽셀, 34/36HU는 세 픽셀이다. 별도 외곽선은 모두 장치 한 픽셀이다.
0HU를 화면용 기본 0.1mm로 해석한 이전 후보는 이 계약과 다르다. 선언 폭은 HWP/HWPML의
HU 원본을 보존하고, 인쇄 시 실제 전달된 장치 픽셀에서 띠와 외곽 pen을 합성한다.
[GDI pen 단위](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-extlogpen)와
[경로 채움·외곽선](https://learn.microsoft.com/en-us/windows/win32/gdi/path-functions)의 계약은
장치 단위와 논리 단위, 채움과 현재 pen을 구분한다. 2024 Print의 실제 띠 폭·외곽선 관측은
공식 HWP 사양 자체가 최소 폭이나 인쇄 해상도를 명시했다는 주장과 구분한다.

값 추적은 `Picture.border_width → RectangleNode의 원본 HU 메타데이터 → LayerBuilder의
Print 장치 pen → Rectangle PaintOp → Native/Canvas`다. 논리 bbox·style과 문단/쪽 소유는
유지한다. 화면·2020 호환·일반 도형·점선·회전/affine은 기존 경로를 소비한다. 그림의 실선과
양수 장치 단위가 실제 공급된 Print 경로에서만 적용한다. 점수용 임의 최소 화면 픽셀을 두지 않는다.
확정한 실선은 빈 명시적 반복 구간으로 전달해 Canvas의 범용 최소 0.5px 처리가 장치 선폭을
다시 덮어쓰지 않게 한다. Native와 Canvas의 기존 명시적 구간 재생을 함께 소비한다.

현재 후보는 아직 검증 전이다. 원본 영향 쪽·독립 폭 대조군과 정상 대조군을 먼저 실행해
실제 PNG, semantic tree, 신규 하락을 확인한다. 확대된 실선이 내용을 가리거나 없는 선을
추가하면 후보를 보정한다. 정식 회귀 추가와 최종 제출은 Native/fresh WASM 선행 조건 뒤 진행한다.

### 장치 pen 후보의 Native 선행 검증 결과

코드 `220c70a677`은 공용 `target/pr-review`, jobs 8의 Native 빌드를 6분 21초, exit 0으로
완료했다. 두 원본의 핵심 52쪽, 독립 선폭 대조군 2쪽과 정상 대조군을 실제 실행했다.

| Native 범위 | 직전 `ed630b398e` | 현재 후보 | 판정 범위 |
| --- | ---: | ---: | --- |
| 원본 HWPX 46쪽 raw | 95.74702% | 100% | 정확한 미대응 0 확인 |
| 원본 HWP 46쪽 raw | 95.74674% | 100% | 정확한 미대응 0 확인 |
| 핵심 52쪽의 총 미대응 픽셀 | 67,977 | 61,557 | 6,420개 감소 |
| 선 유무·선폭 대조군 raw | 94.20765% | 98.26749% | Native 진단 결과 |
| 선폭·ROUND/FLAT 30조건 raw | 87.66244% | 97.37492% | Native 진단 결과 |

핵심 52쪽의 현재/직전 PNG 해시와 독립 PDF raster 해시를 확인했다. 직전 코드 대비 신규
0.1%p 이상 하락은 없고 semantic tree도 모두 동일하다. 실제 관측 글꼴 정보도 두 형식에서
각각 동일하다. `picture-pen-device-220c70a677/focused-vs-previous-exact.json` 참조.
이 비교를 최신 devel의 최종 검증이나 전쪽 통과로 대신하지 않는다. 기존 273쪽 하락은 남는다.

2022 호환 정상 대조군 12쪽은 RGB가 동일하다. 2024 호환 대조군 12쪽도 기존 비교 기준 대비
0.1%p 이상 하락은 없으며 최대 하락은 0.05441%p다. 이 대조군의 PDF는 기존 2020 Print를
보존한 것으로, 원본 2024 출력의 충실도 증거로 이름을 바꾸지 않는다.

46쪽과 선폭 대조군의 실제 review PNG를 열어 확인했다. 46쪽 raw 100%가 그림 테두리의
농도·외형까지 픽셀 단위로 같다는 뜻은 아니다. 인쇄의 채운 띠·외곽선 경로와 현재 단일 선폭
재생의 차이는 계속 검토한다. 새 후보의 Native 전 766쪽과 최적화 fresh WASM 빌드·핵심 52쪽
검증을 시작했으며 완료 결과는 아직 없다. 원본 273쪽, 그림자 처리와 전쪽 raw 100%는 미완료다.

## 2026-10-25: 채운 띠·외곽 pen과 실제 출력 격자의 구분

`4bb17e9f9c`의 원점 보정 후 94·163쪽은 두 형식·Native/fresh WASM 모두 raw 100%다.
그러나 같은 96dpi PNG의 외곽선을 3배 확대하면 Print는 검은 선, 후보는 옅은 선이다.
이 발견으로 기존 외곽선 검사 4건의 기대값 교체를 보류했다. 같은 고정 library에서 기존
검사는 4 FAIL, 소유·중심·잉크 포함 관계로 바꾼 진단 초안은 4 PASS지만 정식 변경은 아니다.

원 구현 `1a948bb522`와 위 독립 15폭×ROUND/FLAT 원장을 먼저 대조했다. 제품은 인쇄의
`채운 띠 + 외곽 pen`을 단일 물리 선폭 `(band + 1) × 600dpi 단위`로 합쳤다. 실제 독립
PDF의 대응 객체는 검은 고리의 fill과 0.120pt stroke를 함께 갖는다. 공개 PDFium의
[`RasterizeStroke`](https://pdfium.googlesource.com/pdfium/+/refs/heads/chromium/7999/core/fxge/agg/cfx_agg_devicedriver.cpp)는
실제 출력 변환에서 stroke 최소 폭을 별도로 적용한다. 기존 `raster_stroke`의 공통
최소 폭 계산도 같은 목적이지만 현재 소비자는 Line이며 그림 외곽 Rectangle에는 연결되지 않았다.
HWP의 원본 선폭·Print 장치 띠 폭·최종 PNG의 최소 stroke는 서로 다른 단위다.

원본에서 생산한 band는 유지하고 외곽 pen에만 최소 raster 폭을 적용하는 진단은
18조건의 raw·미대응 수를 전부 유지했다. 163쪽 상단 확대 영역의 RGB 평균 절대 오차는
10.9521→3.1315, 94쪽은 7.7543→3.5422로 줄었다. 이는 제품 검증이 아니다.

중심선 frame도 별도로 확인했다. 선언 폭 0은 화면용 기본 0.1mm의 안쪽 겹침을 인쇄에
전달하지 않는다. 나머지는 원래 테두리의 논리 경계를 보존하고 각 끝점을 장치 정수로
변환한다. 독립 PDF의 94·163쪽 path/image 원점을 대조한 추가 진단은 앞선 회색 선을
검은 선으로 복원했고, 해당 확대 영역의 RGB 평균 절대 오차는 각각 1.6901 / 3.0838이다.
두 형식 18조건에서 raw 하락·미대응 증가는 없다. 곡선·안티앨리어싱의 RGB 잔여를 숨기지 않는다.

다음 제품 연결은 다음과 같다.

1. `Picture.border_width → PagePaintContext`가 띠와 외곽 pen의 물리 폭을 따로 게시한다.
   원본·문단·그림 소유는 보존하며 자동 폭 0과 명시 폭을 구분한다.
2. 같은 소유 그림의 source frame과 테두리의 source frame에서 인쇄 중심선 경계를
   확정하고, `ImagePaintFrame`에서 각 backend에 공통 결과를 공급한다.
3. Native·Canvas2D·CanvasKit이 실제 출력 변환으로 계산한 공통 최소 폭과 동일한 Rust의
   `band + max(outline, raster minimum)` 결과를 소비한다. JSON과 WASM adapter에도 두
   원래 폭을 전달하며 TypeScript에 같은 계산을 복제하지 않는다.
4. Screen·벡터 출력·다른 도형·점선·회전/별도 좌표계의 선택을 이 계약으로 추정하지 않는다.
   영향을 받는 원본과 기존 폭/모서리 대조군, 96/192dpi를 실제 재검증한다.

제품 연결 후 새 source에서 Native/fresh WASM·확대 판독을 완료한 다음 기존 검사의
중심선 bbox 동일성 가정을 소유·중심·잉크 포함 관계로 보정한다. 독립 163쪽 ±2px 보호와
그림 내용·순서 검사는 유지한다. 전체 필수 검증·전쪽 raw100 목표를 진단 통과로 대신하지 않는다.
원 자료는 ignored `handbook-inline-anchor-candidate/border-direct-review/`,
`handbook-picture-pen-{raster,frame}-diagnostic-4bb17/`에 보존한다.
