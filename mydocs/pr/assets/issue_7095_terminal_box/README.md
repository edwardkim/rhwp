# #7095 끝 조각 상자 = 저장 칸 높이의 나머지 — Visual Sweep 대표 증적

- 수정 전: PR #7342 head `cd0fbd0d3` (쪽 고정 조각 valign)
- 수정 후: 이 브랜치의 코드 commit
- 비교 대상
  - `samples/issue7062/tac_object_host_line_height.hwp` ↔ `pdf/tac_object_host_line_height-2020.pdf` 10쪽
  - `samples/hwpx_sample2.hwp` ↔ `pdf/hwpx_sample2-hwp-2024.pdf` 10쪽
  - `samples/task2430/1382000_domestic_violence_survey.hwp` ↔ `pdf/issue2430/1382000_domestic_violence_survey-2020-print.pdf` 30쪽

## 명령

```bash
export RHWP_FONT_PATH=ttfs/hwp:ttfs/windows
export VISUAL_SWEEP_CHROME=<puppeteer chrome 151>
wasm-pack build --release --target web --out-dir <pkg>
venv/bin/python scripts/visual_sweep.py --file-target <key> <hwp> <pdf> \
  --rhwp-bin target/release/rhwp [--wasm-pkg <pkg>] --pages <쪽> --dpi 96 --out <out>
```

Native 는 `--wasm-pkg` 를 뺀 같은 명령이다. 수정 전은 PR #7342 head 로 빌드한 바이너리·WASM 패키지다.

## 파일

| 파일 | 내용 |
| --- | --- |
| `native_t7062_{review,overlay}_010.png` · `wasm_t7062_{review,overlay}_010.png` | 7062 10쪽, 수정 후 |
| `native_hwpx2_{review,overlay}_010.png` · `wasm_hwpx2_{review,overlay}_010.png` | hwpx_sample2 10쪽, 수정 후 |
| `wasm_t7062_overlay_010_before_after.png` | 7062 10쪽 overlay, 왼쪽 수정 전 / 오른쪽 수정 후 |
| `wasm_hwpx_sample2_overlay_010_before_after.png` | hwpx_sample2 10쪽 overlay, 전 / 후 |
| `wasm_1382000_overlay_030_before_after.png` | 1382000 30쪽 overlay, 전 / 후 |

overlay 는 빨강 = 한/글 정본, 파랑 = rhwp 다.

## 사람 판독

- 7062 10쪽: 수정 전 모든 줄이 정본보다 약 10px 위에 겹치고 끝 조각 아래 괘선·뒤 표가 20px 위에 있다.
  수정 후 줄·괘선·뒤 표가 정본과 겹친다.
- hwpx_sample2 10쪽: 수정 전 쪽 전체가 약 30px 위에 겹쳐 두 벌로 보인다. 수정 후 겹친다.
- 1382000 30쪽: 끝 조각 상자 아래가 842 → 924(정본 924)로 옮기고 내용이 가운데로 선다.
