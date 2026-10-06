# #7095 쪽 고정 조각 상자 안 칸 valign — Visual Sweep 대표 증적

- source: 수정 `195a3390e` (base `1966af77f`, devel)
- 비교 대상
  - `samples/issue7062/tac_object_host_line_height.hwp` ↔ `pdf/tac_object_host_line_height-2020.pdf` (한/글 2020)
  - `samples/task2430/1382000_domestic_violence_survey.hwp` ↔ `pdf/issue2430/1382000_domestic_violence_survey-2020-print.pdf`
- 수정 전은 base 를 별도 worktree 에서 빌드한 release 바이너리와 WASM 패키지로 같은 명령을 돌렸다.

## 명령

```bash
export RHWP_FONT_PATH=ttfs/hwp:ttfs/windows
export VISUAL_SWEEP_CHROME=<puppeteer chrome 151>
wasm-pack build --release --target web --out-dir <pkg>

# fresh WASM
venv/bin/python scripts/visual_sweep.py \
  --file-target t7062 samples/issue7062/tac_object_host_line_height.hwp pdf/tac_object_host_line_height-2020.pdf \
  --rhwp-bin target/release/rhwp --wasm-pkg <pkg> --pages 2,3,4,7,10 --dpi 96 --out <out>
venv/bin/python scripts/visual_sweep.py \
  --file-target s1382000 samples/task2430/1382000_domestic_violence_survey.hwp \
  pdf/issue2430/1382000_domestic_violence_survey-2020-print.pdf \
  --rhwp-bin target/release/rhwp --wasm-pkg <pkg> --pages 17,27 --dpi 96 --out <out>

# Native — 같은 명령에서 --wasm-pkg 만 뺀다 (7062 3·4쪽, 1382000 17·27쪽)
```

## 파일

| 파일 | 내용 |
| --- | --- |
| `native_t7062_review_004.png` · `native_t7062_overlay_004.png` | 7062 4쪽, 수정 후 Native |
| `wasm_t7062_review_004.png` · `wasm_t7062_overlay_004.png` | 7062 4쪽, 수정 후 fresh WASM |
| `wasm_t7062_overlay_003_before_after.png` | 7062 3쪽 overlay, 왼쪽 수정 전 / 오른쪽 수정 후 (WASM) |
| `wasm_t7062_overlay_004_before_after.png` | 7062 4쪽 overlay, 왼쪽 수정 전 / 오른쪽 수정 후 (WASM) |
| `native_1382000_overlay_027.png` · `wasm_1382000_overlay_027.png` | 1382000 27쪽, 수정 후 |
| `wasm_1382000_overlay_017_027_before_after.png` | 1382000 17쪽 전/후 · 27쪽 전/후 (반례 17쪽은 글자 불변) |

overlay 는 빨강 = 한/글 정본, 파랑 = rhwp 다.

## 사람 판독

- 7062 3·4쪽: 수정 전에는 모든 줄이 정본보다 약 14~17px 위에 겹쳐 두 줄로 보인다. 수정 후 줄·표
  외곽·제목 상자가 정본과 겹친다. 남는 차이는 가로 글자폭(글꼴)이다.
- 1382000 27쪽: 수정 전 약 11px 위 → 수정 후 정본과 겹친다.
- 1382000 17쪽(저장 칸 높이 282HU 반례): 글줄 위치 불변. 칸 상자 아래 괘선은 1006 → 993
  (정본 980) 으로 옮겼고 이 PR 밖의 차이가 남는다.
- 7062 10쪽(끝 조각): 전후 동일. 정본 상자가 19px 길고 첫 줄이 9.8px 아래인 차이는 남는다.
