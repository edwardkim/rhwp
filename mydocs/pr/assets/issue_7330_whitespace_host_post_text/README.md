# #7330 Visual Sweep 증적

`samples/issue4599/36374873_night_guard_log.hwpx` 1쪽 (문서 전체가 1쪽).

## 왼쪽 = 수정 전 / 가운데 = 수정 후 / 오른쪽 = 한/글 2022 정본 PDF

| 파일 | 영역 | 판독 |
| --- | --- | --- |
| `compare_bottom_before_after_oracle.png` | 본문 하단 (engine y 700~1122px) | **수정 전에는 `붙임: … 끝.` 줄이 쪽에서 사라진다**(용지 1122.5px 밖 1168.1px). 수정 후 1073.3px 로 들어오고 정본과 같은 자리에 선다. 13×8 표 외곽선은 셋 다 동일. |
| `compare_top_before_after_oracle.png` | 머리 (engine y 60~330px) | 제목·결재란 3×4 표·문서번호·표 머리행이 전후 동일하고 정본과 일치한다 — 이 수정이 위쪽을 건드리지 않음을 보인다. |
| `compare_full_before_after_oracle.png` | 쪽 전체 | 위 둘의 원본. |

## 생성 절차

- source SHA: `7dd7973f7` (base `1966af77f`)
- 수정 전 바이너리는 같은 HEAD 에서 `git revert --no-commit HEAD` 후 재빌드해 얻었다.
- SVG: `rhwp export-svg <문서> -p 0 -o <out>`
- raster: `node scripts/rasterize-svg-webfonts.mjs --input <svg> --output <png> --zoom 1.5`
  (`VISUAL_SWEEP_CHROME` 지정, 실제 Chrome 실행)
- 정본: 한/글 2022 출력 PDF 를 `pypdfium2` scale 2.0 으로 raster (595×841pt → 1190×1682px,
  rhwp 1191×1685px 와 같은 축). 원본 PDF 는 비공개 코퍼스에 있어 이 폴더에 복사하지 않는다.

## 남은 차이

수정 후 `pi=6` 은 1052.0px, 정본은 1022.0px (저장 사다리 1023.3px) — **+30.0px 이 남는다.**
이 잔여는 `#4599` 의 다른 축이며 이 PR 이 해결하지 않는다. 이 PR 이 잠그는 것은 **용지 밖 이탈**이다.
