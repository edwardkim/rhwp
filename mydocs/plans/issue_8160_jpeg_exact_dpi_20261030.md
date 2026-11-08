# #8160 JPEG Print 회귀 기준의 96 DPI 정책

## CI 실패와 독립 기준

PR #8389의 Frontend package gates에서 `issue_8160_print_jpeg_frame.browser.mjs`는
로고 영역의 2px 양방향 미대응을 0/0으로 기대했으나 11/0으로 실패했다. 같은 검사는
리베이스 전 WASM `50cdf6701f`에서도 11/0으로 재현됐다. 전체 쪽의 실루엣 표시값
100.00%는 이 작은 영역의 차이를 배제하지 않는다.

입력 `samples/hwp3-sample16-hwp5-2022.hwp` SHA-256은
`fec3671c9a83cb8388cbdaab38754cb17a8da54d0314ca7e3f56860e871506fe`, 독립 한컴
Print PDF `pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf`의 SHA-256은
`c4e05635726869d737f1640db4e4f29c868df9e1c31e350b0b06e115fd210b33`이다.
기준 PDF를 재출력하지 않았다. 동일 PDF의 1쪽을 PDFium 153.0.7999.0 / pypdfium2 5.13.0으로 다시 래스터해
`[52,64,185,102]` 영역과 실제 브라우저 Canvas를 비교했다.

```bash
venv/bin/python scripts/visual_sweep_pdf.py \
  --pdf pdf/hwp3-sample16-hwp5-2022-hwp-2020.pdf --dpi 96 \
  --prefix output/pr-review/issue8153-handbook-raw100-20261022/ci8389-jpeg-diagnostic/pdf-exact \
  --pages 1
```

| 기준 | 래스터 정책 | 전쪽 PNG SHA-256 | 실제 Canvas 미대응 |
| --- | --- | --- | --- |
| 기존 테스트 PNG | `pdfium_smooth_rgb_v1`, 정수 버퍼 크기에 페이지 맞춤 | `6052593e338fb506d8a33a03fe01f3790515a87ef1238b4ea98053bc822ba596` | 11/0 |
| 새 테스트 PNG | `pdfium_smooth_rgb_exact_dpi_v2`, 두 축 `96/72` 물리 배율 | `dadd39d960f9e74022953e80b7a2c53dbcda94b2937013bc05421994a6e4d5e8` | 0/0 |

기존 방식으로 다시 만든 PNG의 전쪽 SHA와 crop 픽셀은 기존 증거와 일치한다. 11개
차이는 모두 crop의 x=128 열에 있다. 이전 기준을 새 기준과 직접 비교하면 반대 방향에
11픽셀이 생긴다. 이는 페이지 물리 크기를 반올림한 출력 버퍼에 맞추면서 표시 배율까지
늘린 차이다. 현재 정책의 독립 근거와 24개 용지·회전·DPI 대조는
[#8153 PDF 물리 DPI 계획](issue_8153_pdf_raster_dpi_20261024.md)을 따른다.

## 수정 범위와 검증

원본 PDF·2px 대응 반경·임계값·0/0 계약·문서 좌표는 바꾸지 않는다. 같은 PDF의 같은
영역을 현재 정식 Visual Sweep 정책으로 만든 새 PNG와 SHA로 연결한다. 기존 v1 PNG는
이전 검증 이력으로 보존한다. 제품 그림 처리나 글꼴은 변경하지 않는다.

리베이스 전 WASM에서 새 기준과의 직접 비교는 0/0이며, CI와 같은 브라우저 묶음은
9건 모두 통과했다. `venv/bin/python -m unittest discover -s scripts/tests -p
'test_visual_sweep*.py'`는 201건 통과했다. 현재 HEAD `471e1ec9179b5fd532055e0674c714bbc3f3256e`의 fresh 최적화 WASM
SHA-256 `965458db1272a0666b201ac0f1f4eef70b132d355ab94f99b3ef4f7a01ad8a5e`으로
같은 브라우저 묶음 9건도 모두 통과했다.
현재 PR HEAD `471e1ec9179b5fd532055e0674c714bbc3f3256e`의 최적화 전 WASM
`d82647cbc5f9e6d76c32a83afb2fc5ab769132deb98fcee504c085081932131e`로도
해당 브라우저 검사가 1건 통과했다.
