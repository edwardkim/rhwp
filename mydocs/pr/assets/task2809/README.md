# Task #2809 검증 증적

이 디렉터리는 나눔정렬 마지막 줄과 Canvas 2D 음수 자간 glyph 폭 정정을 검증한
자료를 보관한다. 기준 PDF는 HWP 2020의 정상 출력이며 변경 대상이 아니다.

## 입력과 기준

| 파일 | 역할 | SHA-256 |
|---|---|---|
| `samples/issues/2809/jubo_20260104.hwp` | rhwp 입력 HWP 6쪽 | `3fb2e25a8bd57ec8f2c8b2754613e3e69ace831d441a63f601f6dbea8e79d3ac` |
| `pdf/issue-2809-jubo_20260104-2020.pdf` | HWP 2020 정상 기준 PDF 6쪽 | `a73d50620bf8fe96beaff72ba0e40cd34f396ec75de9798ac1fd0402e28f8e2b` |

## 직접 확인 자료

| 파일 | 내용 | SHA-256 |
|---|---|---|
| `jubo_p2_144dpi_after_review.png` | 실제 rhwp Studio Canvas/PDF/overlay를 한 장에 모은 최종 144dpi 검토판 | `6a3feaa4843b0730584ae87a5e1ea275d68a59c7f6753d65a1f8b405850807d3` |
| `issue-2809-split-alignment-report.html` | 좌표·glyph 폭·우측 무잘림을 포함한 E2E assertion 7건 | `2821c8e40302a9682385bdae7624f4e2baa0e32bd2cdec79ae3179a567c623a5` |

중복 증적을 피하기 위해 저장소에 직접 두는 PNG는 최종 review 한 장으로 제한한다.
WASM Canvas와 실제 편집기 캡처는 E2E 실행 시 `output/e2e/task2809/`에 생성되며
커밋하지 않는다.

최종 Canvas 픽셀 검사 결과는 다음과 같다.

- 문자 위치 span: 위 `77.973px`, 아래 `76.693px`.
- 첫 `다` glyph 잉크 폭: 위 `28px`, 아래 `28px`.
- 위쪽 마지막 `이` glyph 잉크 폭: `22px`로 셀 우측에서 온전히 표시된다.
- 즉 음수 자간은 문자 시작 위치에 반영되지만 glyph 자체를 누르거나 자르지 않는다.
- 최종 review의 왼쪽 패널은 `export-svg` native 미리보기가 아니라 E2E가 실제
  `canvas2d` 백엔드에서 캡처한 `jubo_p2_wasm_canvas_2x.png`다. 이를 144dpi 기준
  PDF canvas에 비율 유지 정규화한 뒤 compare/overlay/review 패널을 생성했다.
- 최종 review의 내용 픽셀 중심 자동 일치율 보조값은 `13.08202%`다.

중복·대용량 산출물 압축본은 커밋하지 않는다. 재현 명령의 전체 결과는 로컬
`output/`에서 확인하고, PR에는 최종 review PNG와 판정 보고서만 포함한다.

## 재현 명령

```bash
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && npm run e2e:issue-2809
python3 scripts/task1274_visual_sweep.py \
  --key task2809-jubo \
  --hwp samples/issues/2809/jubo_20260104.hwp \
  --pdf pdf/issue-2809-jubo_20260104-2020.pdf \
  --page 2 --dpi 144 \
  --out output/task2809-visual-stage3-final \
  --rhwp-bin target/debug/rhwp
```

마지막 명령은 native SVG 보조 sweep이다. 실제 rhwp 화면의 최종 판정과 저장용 review
PNG는 바로 앞 E2E 명령이 생성한 `output/e2e/task2809/jubo_p2_wasm_canvas_2x.png`를
rhwp 패널로 사용한다.
