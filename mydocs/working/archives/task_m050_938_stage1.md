# Task M050 #938 Stage 1 완료보고서 — 워터마크 투명 배경 원인 확정

## 단계 목표

`samples/복학원서.hwp` 1페이지 중앙 워터마크의 옅은 사각 배경 원인을 소스 수정 없이 확정한다.

## 수행 항목

- 현재 브랜치 소스로 `cargo build --release` 실행
- `rhwp export-svg samples/복학원서.hwp`로 현재 SVG 출력 생성
- SVG 내부 `<image>` 노드 추출 및 원본 이미지 픽셀 분석
- `pdf/복학원서-2022.pdf` 1페이지 렌더 및 PDF 내 이미지/XObject 분석
- `DocumentCore::document()` 기반 임시 진단 프로그램으로 BinData와 `raw_picture_extra` 확인
- rhwp-studio overlay 경로(`getPageOverlayImages` + `page-renderer.ts`) 확인

진단 산출물:

- `output/debug/task938/stage1/복학원서.svg`
- `output/debug/task938/stage1/image_current_0.png` — 학교 로고 PCX→PNG
- `output/debug/task938/stage1/image_current_1.jpg` — 중앙 워터마크 JPEG
- `output/debug/task938/stage1/pdf_2022_page1_2x.png`
- `output/debug/task938/stage1/pdf_image_0_xref11.jpeg`
- `output/debug/task938/stage1/*_summary.json`

## 핵심 확인 결과

### 1. 중앙 워터마크는 알파 없는 JPEG

SVG에서 중앙 워터마크는 다음 속성으로 출력된다.

```text
mime=image/jpeg
pixel_size=728×729
alpha_values=[255]
has_alpha_less_than_255=false
border_near_white_245_ratio=0.9571
all_near_white_245_ratio=0.4595
corner_avg_rgba=(255,255,255,255)
```

즉 원본 JPEG 자체가 투명 배경을 갖지 않는다. 이미지 외곽의 95.71%, 전체 픽셀의 45.95%가 거의 흰색이다.

### 2. SVG 경로는 흰 배경까지 필터/opacity 처리

현재 소스 기준 SVG의 중앙 워터마크 노드:

```text
<g filter="url(#rhwp-img-grayscale)">
  <g filter="url(#rhwp-img-bc-b-50c70)">
    <g opacity="0.17">
      <image mime=image/jpeg ...>
```

흰 배경도 그림의 일부이므로, 필터와 `opacity=0.17`이 엠블럼뿐 아니라 사각형 전체에 적용된다. 흰색 코너를 흰 종이 위에 합성해도 평균값이 255가 아니라 약 248이 되어 옅은 사각 영역이 남는다.

### 3. rhwp-studio overlay 경로도 같은 결함 구조

`getPageOverlayImages(0)` 요약:

```json
{
  "imageCount": 2,
  "overlays": [
    { "mime": "image/png", "effect": "realPic", "wrap": "behindText" },
    {
      "mime": "image/jpeg",
      "effect": "grayScale",
      "brightness": -50,
      "contrast": 70,
      "wrap": "behindText",
      "watermark": { "preset": "custom" }
    }
  ]
}
```

`rhwp-studio/src/view/page-renderer.ts`는 워터마크 overlay `<img>`에 다음을 적용한다.

- `filter: grayscale(100%) brightness(0.5) contrast(1.7)`
- `mix-blend-mode: multiply`

이 경로도 원본 JPEG의 흰 배경을 제거하지 않는다. CSS filter가 흰 배경을 먼저 회색 계열로 바꾼 뒤 multiply 합성하기 때문에 사각 영역이 생긴다.

### 4. PDF 기준도 JPEG를 쓰지만 배경은 흰색으로 유지

`pdf/복학원서-2022.pdf`의 1페이지 image XObject:

```text
xref=11
mime=jpeg
size=728×729
smask=0
filter=DCTDecode
alpha 없음
```

PDF에는 soft mask나 ExtGState 투명도 객체가 없다. 대신 PDF에 들어간 JPEG 자체가 이미 한컴 출력용으로 회색조/저대비 처리되어 있으며, 흰 배경은 거의 흰색으로 유지된다.

PDF 추출 이미지 통계:

```text
all_near_white_245_ratio=0.5190
border_near_white_245_ratio=0.9921
gray_spread_le_3=1.0
corner_avg=(255,255,255)
```

한컴은 “알파 마스크를 PDF에 넣는 방식”이 아니라, 워터마크 효과 적용 시 배경을 흰색으로 보존한 결과 이미지를 PDF에 넣는 방식으로 보인다.

### 5. `raw_picture_extra`에는 별도 투명도 정보가 없다

임시 진단 프로그램으로 확인한 중앙 워터마크 IR:

```text
picture s0.2 ci=1 bin=2
effect=GrayScale brightness=-50 contrast=70
raw_extra_len=17
raw_extra=00 BE 00 3A 14 00 00 00 00 48 D5 00 00 84 D5 00 00
decoded:
  border_opacity=0
  instance_id=339345598
  effect_flags=0x00000000
  original=54600×54660
```

확인된 `raw_picture_extra`는 테두리 투명도, instance id, 효과 플래그, 원본 크기다. 중앙 워터마크의 배경 투명 처리에 쓸 수 있는 별도 alpha/색상키 값은 발견되지 않았다.

## 결론

Issue #938의 직접 원인은 **알파 없는 JPEG 워터마크의 근백색 배경을 rhwp가 이미지 본문과 동일하게 필터/합성하는 것**이다.

현재 우선 구현 후보는 다음이다.

1. `ImageAttr::is_watermark()`가 true인 JPEG에 한정해 근백색 배경을 보존 또는 투명화한다.
2. SVG 경로와 rhwp-studio overlay 경로가 같은 결과를 내도록 공통 전처리 또는 공통 판정 함수를 둔다.
3. 일반 JPEG 사진에는 적용하지 않도록 워터마크 메타, 포맷, 픽셀 분포 조건을 모두 만족할 때만 발동한다.

## 구현 계획서에서 확정할 사항

- 근백색 판정 임계값: 현재 데이터상 `RGB >= 245`가 45.95% 전체 배경과 95.71% 외곽을 잡는다.
- 처리 방식:
  - SVG/Canvas에 투명 PNG로 전처리 후 전달
  - 또는 필터 적용 시 근백색 영역만 필터/opacity/multiply 대상에서 제외
- 회귀 테스트:
  - 중앙 워터마크 JPEG가 워터마크 전용 전처리 경로를 타는지 검증
  - PCX 학교 로고는 기존 투명 PNG 변환 유지
  - `issue_514`, `issue_516`, `svg_snapshot` 회귀 유지

## 검증 결과

```text
cargo build --release: 성공
rhwp export-svg samples/복학원서.hwp: 성공
PDF 렌더링: PyMuPDF로 성공
PDF image/XObject 추출: 성공
```

현재 소스 기준 `export-svg`에서 남는 레이아웃 overflow는 2.5px이며, Task #677에서 기록된 tolerance 수준이다. #938의 원인과는 별도다.

## 다음 단계

작업지시자 승인 후 `mydocs/plans/task_m050_938_impl.md` 구현 계획서를 작성한다.
