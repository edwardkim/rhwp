# #7751 그림 색조의 저장·출력 계약 — 6단계

2026-10-16, 시작 HEAD `e0b69960e`. 기존 독립383쪽 Print PDF
`3cdf329004cffad0ebacd122586e5e1702aff36b3015cbcb401d038857019593`를 유지한다.
전체 원본의 반복 MCP 출력은 하지 않는다.

## 독립 근거와 반례

한컴2024가 직접 저장한 합성 WMF HWP는 의미 밝기/명암
`(0,0),(-5,0),(8,0),(-20,0),(20,0),(0,-5),(0,8)`을 이진 레코드에
`(0,0),(0,-5),(0,8),(0,-20),(0,20),(-5,0),(8,0)`으로 기록한다.
job `5bf9b059-e3a6-4331-877a-439cbfbfe54a`, 저장본 SHA
`0413cd5f75c80abac870181cea1b865a1590ff00d9f561e554bd6db7a4fe9e9c`다.
따라서 현재 parser/serializer의 명암→밝기 순서를 단순히 되돌리지 않는다.

반면 HWPX→HWP adapter의 `normalize_picture_geometry_for_hwp`에는
serializer가 모델 순서대로 기록하던 시절의 `mem::swap`이 남아 있다.
실제로 rhwp 변환본은 첫 비영 조건을 `(-5,0)`으로 저장해 한컴에서 명암-5로
출력한다. 한컴 저장본과 현재 serializer의 계약을 중복 적용하는 오류다.
메타데이터 보존 경계에서 의미 값을 유지하고 파일 바이트 순서는 serializer가
한 번만 결정하도록 보정한다. 이미지 형식·묶음 여부로 이 교환을 분기하지 않는다.

130쪽의 별도 문제는 아직 미확정이다. 원본 그림376의 이진 `(0,-5)`는
의미 밝기-5이며 원본 TIFF354의 `(0,8)`는 밝기8이다. 실제 한컴 HWPX 저장도
이 해석과 같다. 그런데 기존 편람 Print의 WMF 흰색은 RGB249, 현재 Native는
242다. 합성 그림의 Print 및 기존 PDF에서 실제 원본 그림의 내부 픽셀·색공간·
변환/필터 경로를 대조한다. 합성 입력 통과를 원본 편람의 일치 증거로 대신하지 않는다.

## 실제 소비 경로와 검증 순서

1. HWPX semantic `ImageAttr` → HWP adapter → `serialize_picture_data`의
   두 i8 → HWP parser → 실제 한컴 Print를 대조한다. 정상 비영 값·부호·0을
   독립 저장본 상수와 비교하며 재저장 시 재교환하지 않는지 확인한다.
2. 원본 그림 → `resolve_image_payload`의 WMF→SVG → Native의 SVG raster와
   `adjusted_picture_bytes`/필터, fresh WASM의 같은 그림 소비 위치를 추적한다.
   변환 전후 형식이 바뀌어 보정의 색공간·순서가 달라지는지 확인한다.
3. 원본 그대로의130쪽과375쪽, 원점 변경으로 소폭 하락한224·226쪽을 먼저
   확인한다. 원인이 설명되지 않은 차이는 미검증으로 남기며 허용치를 바꾸지 않는다.
4. 새 Native/fresh WASM에서 전383쪽 raw TSV와 baseline 비교를 갱신한다.
   90%/98% 조건 및 직접 확대 판독이 충족된 뒤 보류 중 정식 회귀를 추가하고
   lint·필수 전체 검증을 진행한다. 최종 code head와 증적의 관계를 기록한다.

원시 진단 입력·job 상태·관측표는 ignored
`output/pr-review/pr7494-20261016/`에 보존한다. 변환기의 이중 교환 오류와
원본130쪽의 raw 하락은 구분하며, 하나의 수정으로 모두 해결됐다고 주장하지 않는다.

## 리베이스 전 체크포인트

adapter의 이중 교환을 제거한 Native CLI 빌드는 공용 `target/pr-review`,
jobs8, `--locked --profile release-test --features native-skia --bin rhwp`로
4분59초에 exit0이었다. 합성7조건을 HWPX→HWP 변환한 실제 i8 바이트는
한컴 직접 저장본의7조건과 모두 일치했다. 수정 전 결과는 앞 절의 반대 순서로
남아 있다. 이 검사는 저장 계약의 증거이며 편람130쪽의 시각 통과 증거가 아니다.

원본 그림376의 속성·WMF를 보존하고 본문을 한 그림으로 줄인 별도 진단 HWP는
한컴 Print1쪽을 만들었다(job `46c52893-c7ea-4108-a0c5-195fbcb8a360`).
그 PDF 내부 그림과 기존 편람130쪽 내부 그림의 JPEG 바이트가 동일했다.
공통 SHA는 `55be64ec566d4b858359e265d1176f5177a319bac8600a75192100cc5ced56ae`다.
기존 PDF의 잘못된 색을 가정하거나 새로운 PDF로 기준을 교체할 근거가 없음을
확인했다. 진단 첫 시도는 DocInfo 구역 수14와 실제1구역의 불일치로 preflight에서
실패했고, 진단본의 구역 수만 수정한 뒤 실행했다. 이 환경 실패를 결함 재현으로 세지 않는다.

`picture-contract-adapter-evidence.json`, `isolated-picture-canonical-equality.json`,
`logs/picture-contract-native-build.log`에 관측을 보존했다. 사용자의 동기화 지시에
따라 새 기준 `552514f423ec91f1b7171f6e6b0882a4efe034f2`로 리베이스한다.
상위 표 조판 변경을 포함하므로 이전 producer를 보존하고, 리베이스 후 최종
시각·회귀·lint 검증을 갱신한다. 현재 PR 제출 및130쪽 보정 완료는 보류다.

## 2026-10-17 WMF 색 처리 경계 진단

리베이스 후 코드 HEAD `424646820`의 Native/fresh WASM 빌드는 모두 성공했다.
3·130·224·226·375쪽을 다시 출력했고, 130쪽 raw는 각각79.94718%·78.04306%다.
따라서 현재 상태는 보류다. Native의 다섯 쪽 PNG는 리베이스 전과 바이트가
동일했다. 이 좁은 확인을 새 기준 `552514f42`의 전쪽 검증으로 대신하지 않는다.

색조 원인을 분리하기 위해 같은 유효한 WMF DIB와 그림 속성을 사용하는
한 쪽 합성 입력을 한컴2024 Print로 대조했다. 아래 값은 JPEG 내부의
균일한 흰색 영역이다. JPEG 압축 오차와 실제 색조 분기를 구분한다.

| 대조 | 독립 Print 관측 | 판정 |
| --- | --- | --- |
| 11색 WMF, 밝기−5 | RGB243 | 기존 인코딩 RGB 계산과 일치 |
| 같은 WMF에 작은 다색 영역 추가, 밝기−5 | RGB249 | 원본 편람 그림의 색조 차이 재현 |
| 같은 다색 픽셀의 PNG | 일반 RGB 계산 | WMF와 일반 raster를 구별해야 함 |
| 같은33색의 순서만 세 가지로 변경 | 모두 RGB243 | 인접 색의 5비트 보간 gamut256 경계 가설 기각 |
| 같은33색을 한 번/두 번 반복 배치 | RGB243 / RGB249 | 원본 색 수만으로 분기할 수 없음 |
| 같은11색을 여덟 번 반복 배치 | RGB249 | 색 경계와 래스터 확대의 영향을 확인 |

순서 변경 job은 `63996bf8-34fd-40c5-ae33-c3621f035d03`, 반복 배치 job은
`6b9ecee0-9eb6-4a68-b53b-f334babf046b`다. 둘 다 HOffice13.0.0.3901,
PrintMethod0, `frame_print_to_pdf_ex_one_up`이며 한 쪽이다. 원본 편람 PDF를
재출력하거나 교체하지 않았다. `wmf-palette-order-observations.json`과
`wmf-resampling-color-observations.json`에 색별 관측을 남겼다.

보간 후 RGB 색 수가256을 넘을 때의 팔레트 변환이 유력하지만 아직 구현
계약으로 확정하지 않았다. 앞선 회색256색+파랑1픽셀 대조군의 파랑은 맨 아래
행에 있었고, 실제 Print의 JPEG 전 픽셀에서 B−R 최댓값이0이었다. 따라서
해당 대조군을 ‘출력된257색’의 반례로 사용할 수 없다. 색 수만 세기 전에
실제로 그려진 범위와 해상도를 확인한다. WMF placeable 헤더의 inch와 보간
해상도 대조를 이어가며, 문서 ID 분기·임의 보정계수·parser 역교환은 추가하지 않는다.


### 색조만 바꾼 원본 진단본

원본 HWP를 ignored 경로에 복사하고 그림376의 WMF DIB RGB만
`clamp(floor(v−12.75)) ×32/31`(정수 floor·255 포화)로 변환했다.
해당 Picture의 밝기만−5에서0으로 바꿔 중복 적용을 막았다. 다른 모든 원본
컨트롤과 WMF 기하 레코드·입력 파일·기준 PDF는 보존했다. 이 작업은 원인
분리를 위한 변형 입력이며 제품 코드 수정이나 정식 원본 검증이 아니다.

| 130쪽 raw | 변경 전 원본 | 색조만 바꾼 진단본 |
| --- | ---: | ---: |
| Native | 79.94718% | 98.19094% |
| fresh WASM | 78.04306% | 93.34779% |

`p130-tone-only-diagnostic.json`에 원본 SHA
`40d6d05eac4d55bdc4b0c62c42d93af104d5123b447581246f36fd15de7bd46f`와
변형본 SHA `e4eeee15f066f884c93f68236954c04d3587f2c141dcf866b2db72a2704d0ec5`,
변경 범위를 기록했다. 각 `p130-tone-only-diagnostic-{native,wasm}/handbook/`
아래 실제 PNG·overlay·review·측정값과 로그를 보존했다. Native overlay는
직접 열어 확인했다. 점수 차이는 색 처리 원인에 대한 증거이며 남은 선·글자
차이나 공통 WMF 처리 계약의 완료를 뜻하지 않는다. 특히 WASM93.34779%는
98% px 회귀 추가 조건을 충족하지 않는다.

placeable inch만96→600으로 바꾼 대조에서는 같은64색 WMF도 밝기−5의
흰색이249→243으로 달라졌다(job `c88f5898-b2d7-4692-a22a-f67397da145f`).
따라서 원본 색 수만 사용하는 조건이나 모든 WMF에 동일한 gain을 적용하는
조건은 채택하지 않는다. Windows10(10.0.19045.6466)에서 직접 GDI+와
`PlayMetaFile` 래스터 대조를 수행 중이며, 한컴의 색조 적용 순서와 실제
보간된 색 집합을 구별한다. 진단 코드는 ignored 자료이며 정식 회귀로
등록하지 않았다. 원본 입력의 Native/fresh WASM gate는 여전히 보류다.

### 256단계 회색의 독립 대조

새 한 쪽 합성 WMF는32×32픽셀 타일256개에 회색0…255를 넣고, 아래32행에
다색 band를 추가했다. 순수 회색 대조군과 회색조 효과 대조군도 같은 문서에
넣었다. job `42f85cbf-1130-4036-960a-65550cd105d8`은25초에 성공했고,
HOffice13.0.0.3901 PrintMethod0의 한 쪽 PDF를 만들었다. 그림별 중앙 타일
11×11영역의 중앙값을 `wmf-gray-grid-observations.json`에 보존했다.

한컴 PDF의 실제 JPEG 양자화 표·4:2:0 설정으로 예측값을 인코딩해 비교했다.
단순 `floor(v×32/31)`은1,280개 중1,255개만 일치했다. 반면 명암 중심을128로
계산하고 RGB555의8단계 ordered dither와5비트→8비트 bit replication을
적용한 모델은 실사 효과5조건×256타일, **1,280개 관측값이 모두 정확히 일치**했다.
검증 조건은 B/C `(0,0),(-5,0),(-20,0),(8,0),(0,-5)`다.
중심127.5 모델은 마지막 명암 조건의3개 타일에서 차이가 있었다.
`wmf-gray-transfer-pivot128-models.json`과 비교 모델 결과에 근거를 남겼다.
이것은 JPEG 안의 균일한 회색값 재현 증거이며, 모든 RGB 채널·dither 위상·
한컴 내부 구현 자체를 확인했다는 주장은 아니다.

순수 회색과 회색조 효과의 밝기−5는 양쪽 모두 기존 일반 RGB 결과였다.
따라서 모든 WMF 또는 모든 raster에 RGB555를 일괄 적용하지 않는다.
현재 남은 검증은 팔레트 경로와 RGB555 경로의 선택 조건이다. 동일 픽셀·
동일 배치에서 DIB24/8비트만 바꾼 대조군을 추가했다. 이 경계를 확인한 후
공통 그림 producer에서 변환하고, Native/Canvas/SVG가 같은 결과를 소비하게 한다.

별도 Windows GDI/GDI+ 대조는 Windows10 `10.0.19045.6466`, PowerShell,
System.Drawing과 `PlayMetaFile`로 실행했다. 이는 rhwp의 Windows 전체 검증이
아니다. 결과와 진단 소스를 로컬에 보존한 뒤, 원격의 직접 만든
`C:\Users\admin\rhwp\output\pr-review\pr7494-wmf-gdi-diagnostic-20261017`
파일19개와 디렉터리를 제거했다. 관련 진단 프로세스0개와 삭제 후 경로 부재를
확인했고 `logs/wmf-gdi-windows-cleanup.log`에 기록했다.

### 공통 producer 구현 중간 상태

DIB8/24비트의 동일 픽셀 대조 job
`004a9083-6695-4d64-883a-c0693fb95380`은 각 쌍에 동일한 PDF 그림 객체를
재사용했다. 입력 비트 깊이로 색 처리 분기를 고르는 가정도 제외한다.
`wmf-dib-bit-depth-observations.json`은 XObject 개수가 아니라 실제 Do 순서로
일곱 그림을 대응한다.

팔레트 선택 후보를 구분하는 독립 대조 job
`73ae24ed-6a4a-4086-9cfb-07ebe0dffd18`에서는 후보 계산의255/256/257색 입력이
실제로는 RGB555/RGB555/일반 RGB로 출력됐다. 기존 대조7개에 맞았던 단순한
최종 너비의 align-corners 색 수 모델은 이 새 입력에서 기각됐다.
`wmf-palette-boundary-cross-validation.json`에 남은 후보와 반례를 기록했다.
아직 선택 조건 전체를 확정했다고 보고하지 않는다.

동시에 일반 WMF의 회색256단계에서도 명암−5의 중심128이 전 타일과 일치했다.
명암+8은 gain1.08이 전 타일과 맞았고, 이전1/0.92 식은34타일이 달랐다.
job `8b84b485-17b5-420b-94ad-3305b797c72f`의 추가 대조는 양의 명암에서
`floor(10000/(100−C))`를 백분율 정수 gain으로 먼저 사용하는 모델을 확인했다.
명암8/20/37/−37, B20·C37, 다색 C37, 다색 B−5·C8의1,792타일 중1,791개가
실제 JPEG 중앙값과 정확히 맞았고, 남은1개(B20·C37, 원본 회색80)의 차이는
2바이트 값이었다. 이 차이를 임의 bias로 맞추지 않는다.
`wmf-integer-gain-observations.json`과 `wmf-integer-gain-models.json`에 보존했다.

현재 구현은 단일 raster WMF의 밝기·명암·회색조를 공통 PNG 바이트에 적용한다.
단일 image 밖의 벡터·ROP 합성은 이 변환으로 대신하지 않는다. 소스 조정색이
256색을 넘는 경로에는 RGB555 dither 평균을 반영한다. 저색수의 확대 후
팔레트 선택과 축소 후 색 수는 **아직 미검증인 중간 구현**이다. 이 소스 색 수
조건만으로 모든 WMF의 선택 문제가 해결됐다고 주장하거나 최종 수용하지 않는다.

소비 경로는 `wmf_picture.rs` → `resolve_image_payload` → paint builder의
resolved payload → Native/SVG/Canvas다. 이미 적용한 효과는 suppress_effects로
중복 적용을 막는다. JSON·직접 PaintOp에서도 같은 producer fallback을 사용한다.
지연 로딩은 `wmftone_B_C_회색조` variant를 키에 포함하고
`emitted_source_image_bytes`에서 같은 바이트를 재현한다. 같은 BinData의 다른
밝기·명암·효과가 동일한 캐시 키를 쓰지 않는다.

초기 Native 빌드는4분49초에 성공했지만 첫 원본130쪽은79.94718%로 그대로였다.
생성 SVG의 빈 image가 `<image></image>`인데 선택 검사는 `<image/>`만 받는
원인을 확인했다. 해당 표기를 정규화하도록 수정하고 재빌드·재검증한다.
이 초기 실행을 색 보정 통과나 의도한 결함의 red/green 증거로 세지 않는다.
원본 SHA·기준 PDF는 유지하며 새 렌더링 회귀 등록·PR 제출은 계속 보류한다.

### 원본 입력의 새 producer 검증

빈 image 표기를 수정한 v2 Native 빌드는7분03초, fresh WASM 빌드는5분49초에
성공했다. 원본 HWP와 기존 독립383쪽 PDF를 그대로 비교한130쪽 raw는
**Native98.19094% / fresh WASM93.34522%**다. 이 값은 앞의 변형 입력 진단과
구별되는 실제 제품 코드의 원본 검증이다. Native overlay를 직접 열어 색 경계와
잔여 얇은 선의 차이를 확인했다. 130쪽은 양쪽98%를 충족하지 않아 px 핀 대상이 아니다.

동일 코드의 Native 전383쪽 TSV도 완료했다. 누락0쪽, 90% 미만0쪽이며 최저는
90쪽92.53354%다. fresh WASM 전쪽 검증과 새 devel 기준의 전쪽 하락 검사는
아직 남아 있으므로 전체 검증·PR 준비 완료로 보고하지 않는다.

공개 DocumentCore API와 실제 paint LayerBuilder를 통과한 진단3건은 모두
통과했다. 원본 그림376에서 inline payload와 source key 조회 바이트가 같고,
실사/회색조 효과·밝기·명암 키를 구분하며, 이전 epoch·잘못된 variant를 거부했다.
회색조 출력과 효과 중복 억제도 실제 payload에서 검사했다.
`logs/wmf-payload-contract-v2.log`에 실제3PASS를 보존했다. 아직 정식 렌더링
회귀로 등록하지 않았으며 전쪽 선행 조건 뒤 `tests/cases/`에 필요한 검사를 넣는다.

소스는 `wmf-rgb555-v2-source-provenance.json`의 파일별 SHA로 고정했다.
Native 바이너리 SHA는
`473b6f23ff8cc44c2ad28436bc4a86f0f9cd3ff721e1c9f3bfdd5c3897c316dc`다.
`wmf-rgb555-v2-{native-review,wasm-review,native-all}/handbook/`에 실제 결과가 있다.
후속 저색수 진단에서 앞서 남은5개 팔레트 선택 후보도 모두 반례가 확인됐다.
해당 경계 전체의 일치를 주장하지 않으며, 원본 편람의 전쪽 검증과 devel 대비
무회귀 확인을 먼저 완료한다. 새로운 임의 sampler·계수로 해당 후보를 고정하지 않는다.
