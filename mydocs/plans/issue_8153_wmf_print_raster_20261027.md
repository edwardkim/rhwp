# #8153: WMF의 Print 자원과 최종 장치 표본 구분

Issue: #8153. 두 편람의 전쪽 Native·표준 최적화 fresh WASM에서 raw 100%와
미대응 0픽셀을 달성한 뒤 PR 준비까지만 완료한다. upstream 동기화와 원격 쓰기는 하지 않는다.

## 현재 제품과 조사 범위

현재 문서 head는 `64f95959d4be6e6474c3f8563fdb9a75859273db`다.
이번 단계에서는 제품 코드를 바꾸지 않았다. 검증용 Native producer는
`b2b33fda6027c03cbcfaa051ee1f5968fb7930d`이며, 이전 전쪽 결과는 766조건 중
733조건의 raw 100%·미대응 0픽셀이다. 33조건과 최종 표준 WASM 전쪽 검증은 남아 있다.
이번 진단을 제품의 전쪽 개선 수에 합산하지 않는다.

작업 증적은 다음 ignored 경로에 있다.

```text
output/pr-review/issue8153-handbook-raw100-20261022/
  remaining-raw100-diagnostic/wmf-halftone-contract/
```

HWP 5.0 설명본의 표 32는 그림의 밝기·명암·효과를, 표 107은 그림 사각형·자르기·안쪽 여백을
구분한다. 원본 그림의 `brightness=-5`와 자르기 `[0,391,57840,50640]`를 유지한다.
현재 이미지의 실제 paint 영역은 `[154.9733333333,463.3293333333,480,354.96]`이다.
자르기 비율로 복원한 전체 원본 영역은
`[154.9733333333,460.5673010541,480,357.7220322792]`다.
PDF 자원의 unclipped 원점과 가시 clip 원점을 혼동해 조판 좌표를 이동하지 않는다.

## 원본 WMF와 실제 소비 경로

`BinData/image376.wmf`의 SHA-256은
`8bcb672373323e99561c8fbc29cb228e6d0b022bffcefcac16afd1d327cb4a13`이다.
비 placeable WMF이며, 창은 771×675, `META_DIBSTRETCHBLT(0x0b41)`의
원본·목적 사각형은 770×674, DIB는 24bpp BI_RGB다. 원본 stretch mode는 `HALFTONE(4)`다.

Microsoft의 [WMF 구조](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-wmf/ba5458c6-e885-41e6-b5d7-d54ef9e1065f)는
원래 WMF와 별도의 배치·축척 헤더를 갖는 placeable WMF를 구분한다.
[SetStretchBltMode](https://learn.microsoft.com/en-us/windows/win32/api/wingdi/nf-wingdi-setstretchbltmode)는
HALFTONE과 그 뒤의 brush origin 설정을 규정한다. 현재 SVG 변환기의
`set_stretch_blt_mode`는 미구현이다. 이것만으로 Print 불일치 전체의 원인을 확정하지 않는다.

소비 경로를 확인했다.

1. `src/paint/metafile.rs::image_payload`가 Print 장치 크기를 전달한다.
2. `metafile_surface.rs::payload`는 밝기·명암이 있으면 거절한다. 따라서 해당 그림은
   `image_resolver.rs`의 WMF 색 보정 SVG 경로로 fallback한다.
3. `wmf_picture.rs::adjusted_wmf_picture_svg`가 색 보정을 굽고 `suppress_effects=true`를 반환한다.
4. `skia/renderer.rs`의 실제 `draw_image` 호출은 그 플래그에 따라 밝기·명암을 0으로 전달한다.
   밝기가 두 번 적용된다는 의심은 이 실제 소비 지점으로 기각했다.

`actual-paint-frame.json`과 `replay_planes.rs`가 실제 payload와 paint 사각형을 확인한다.
진단의 기준 PNG는 최신 완료 Native의 해당 페이지와 양쪽 원본 모두 RGB 전체가 같다.

## 공개 Windows API의 직접 재생

`win10-ted`에서 원본 바이트를 메모리로만 보내 다음 경로를 실행했다. GUI와 원격 파일은 사용하지
않았다. caller가 넘긴 3000×2625는 이 단계에서 관측된 PDF 크기이며, 아직 생산 규칙의 증거로
사용하지 않았다.

| 재생 | 관측 | 근거 |
| --- | --- | --- |
| GDI `SetMetaFileBitsEx`·`PlayMetaFile` | 원본 HALFTONE과 임시 mode 3의 출력이 다름 | `gdi-replay-result.json` |
| 흰색·검은색 배경 차이로 얻은 GDI coverage | `[0,0,2996,2621]`; Print mask와 16,860픽셀 차이 | `first-gdi-comparison.json` |
| GDI+ `Metafile.FromStream`·`DrawImage` | 불투명 전체 사각형; Print mask와 5,624픽셀 차이 | `gdiplus-result.json` |
| 명시적인 window/DIB 크기의 placeable header를 전달한 GDI+ | 두 출력이 기본 GDI+와 바이트까지 동일 | `explicit-frame-result.json` |

기본 `FromStream`은 메타파일 경계를 1023×767로 보고했다. 원본 창 좌표로 간주하지 않는다.
원본 mode와 공개 API를 실행했다는 사실도 한컴의 실제 호출 경로를 입증하지 않는다.
현재 JPEG codec·확인된 -5 밝기 LUT·RGB555를 적용해도 직접 재생 자원은 Print RGB와 다르다.
`codec-comparison.json`에 실패 결과를 보존했다. 단순 HALFTONE 지원만으로 해결됐다고 쓰지 않는다.

## 같은 배치에서 색·마스크 표본화의 책임 분리

원래 crop을 물리 clip으로 남기고, 전체 원본 영역의 색·마스크를 공통 16.16 장치 표본기로
각각 축소했다. 실제 paint의 정수 장치 격자는 `[154,460,481,359]`다.
현재 producer로 다시 만든 기준 출력의 RGB 동일성을 먼저 확인했다.

| 두 문서의 130쪽 각각 | 미대응 픽셀 | raw |
| --- | ---: | ---: |
| 현재 제품 | 377 | 99.46283% |
| 독립 Print 자원과 올바른 mask를 분리 표본화한 진단 | 0 | 100% |
| 원본의 GDI 재생·RGB555·현재 JPEG를 분리 표본화한 진단 | 35 | 99.94656% |
| 원본의 GDI+ 재생·RGB555·현재 JPEG를 분리 표본화한 진단 | 9 | 99.98562% |

근거는 `planes-audit-both.json`이다. `p130-boundary-diagnostic-review.png`를 열어 같은 물리 영역을
같은 2배 배율로 판독했다. 독립 자원 재생과 Print의 표 외곽·내용·간트 막대를 대조했다.
독립 자원 대입은 원인 분리이며 제품 보정·회귀·전쪽 완료 증거가 아니다. 원본에서 같은 자원을
만드는 공통 경로와 Native·fresh WASM 소비 지점을 구현·검증해야 한다.

## 마스크 추출 오류의 정정

이전 `p130-oracle-isolation/prepare.py`는 `Pixmap(doc, mask_xref)`가 반환한 값을 원시 mask
비트로 보고 다시 반전했다. 해당 API는 이미 불투명도를 반환한다. 이 실험의 과거 결과를
색·마스크 결합 방식이나 원래 배치를 기각하는 근거로 쓰지 않는다.

이번에는 압축을 해제한 실제 1bit stream을 행 stride와 함께 직접 읽었다. 기본 Decode에서
0비트가 칠해지고 1비트가 비워진다. 원시 비트를 한 번 반전한 coverage는 기존 올바른
`reference-mask.png` 및 PyMuPDF Pixmap의 결과와 전 픽셀이 같다.
원본과 새 폭 대조군 모두 opaque 7,869,376픽셀, transparent 5,624픽셀,
opaque 영역 `[0,0,2999,2624]`다. `mask-polarity-proof.json`이 근거다.
과거 스크립트를 고치고 과거 결과에 무효 사유를 연결했다. 이번의 377→0 진단은 올바른 극성으로
양쪽 원본에서 재실행한 결과다. 제품 알파 회귀의 결과를 이 잘못된 과거 실험과 혼동하지 않는다.

## 폭을 바꾼 독립 Print 대조군

원본 전체 PDF는 재출력하지 않았다. 기존 3쪽 소형 대조군의 WMF 바이트와 세로·자르기 계약을
유지하고, 그림 표시 폭·대응 scale/frame 폭만 바꿨다. 밝기는 모두 -5다. 출력 전 예상치를
`width-control-predictions.json`에 기록했다.

한컴 `13.0.0.3901`, `PrintMethod=0`, `frame_print_to_pdf_ex_one_up`, preprocess 없음으로
job `963fdf9c-2317-4355-9b0c-14947e25efad`가 3쪽을 출력했다.
PDF SHA-256과 크기는 `width-control-observed.json` 및 `width-control-download.json`에 있다.

| curSz 폭(HWPUNIT) | 폭만 고려한 600dpi 예상 | 실제 자원 |
| --- | --- | --- |
| 36,000 | 3000×2625 | 3000×2625, 1bit mask 있음 |
| 18,000 | 1500×1312 | 2554×2236, mask 없음 |
| 27,000 | 2250×1969 | 2554×2236, 이전 쪽과 같은 자원 재사용 |

폭만 사용하는 규칙은 기각했다. 기존 `metafile_surface.rs::device_size`에도 두 축 중 큰
배율을 사용한 뒤 원본 비율로 자원을 만들고 최종 사각형으로 늘리는 계약이 있다. 해당 원본의
밝기 경로도 그 공통 결과를 소비해야 한다. 다만 원본 창·실제 DIB 경계의 선택과 정수화 순서는
현재 구현과 새 대조군을 대조해 더 확인해야 한다. 위 숫자에 맞추는 문서별 크기나 픽셀 오프셋,
mask 테두리 고정값을 추가하지 않는다. 이 소형 대조군은 자원 계약 조사용이며 시각 통과 fixture로
등록하지 않았다.

## 다음 실행과 남은 완료 조건

1. source-only WMF의 색·mask 생산과 원본 창→장치 크기 정수화를 위 대조군에서 확인한다.
   임의로 선택한 Windows API의 출력이나 PDF 자원을 제품의 원본 입력으로 대신하지 않는다.
2. 확인한 결과를 공통 Print 자원과 표본화 경로에 연결하고, Native·Canvas2D·CanvasKit의 실제
   소비를 확인한다. source crop을 물리 clip으로 보존한다. 다른 WMF·밝기 0·벡터/ROP는 반례다.
3. 작은 실제 페이지 전후 비교 뒤 전쪽 Native·표준 최적화 fresh WASM TSV와 핵심 경계 회귀를
   실행한다. 기존 33조건과 전쪽 핵심 ±2px·관계 검사, 최종 전체 게이트·PR 문서는 계속 필수다.

이번 단계에서는 130쪽의 조판 좌표나 제품·점수·원본·기준 PDF·글꼴을 변경하지 않았다.
