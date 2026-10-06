# #7548 3단계 — 재조판 본문의 어울림 표 띠 회피 대표 증적

- source: `5e8aea70d`(수정) 위 테스트 커밋. 수정 전 = `48ff4bb93`(devel).
- Native `rhwp` sha256 `0179d7fd…cb97f`, fresh WASM `pkg/rhwp_bg.wasm` sha256 `25bf30ea…87dd3`
  (`scripts/wasm-pack-locked.sh --target web --out-dir pkg`, public 동기화 해시 일치).
- 글꼴: `RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`. `scripts/visual_sweep.py --dpi 96`.
- 기준 PDF: 같은 입력을 hwp2024Convert MCP engine 2020(Hancom 11.0.0.9136,
  `hancom2020_pdf_driver_one_up`)으로 출력. Creator Hwp 2020 0.0.0.0, Producer Hancom PDF 1.3.0.550.

| 입력 | 기준 PDF | 1쪽 실루엣(2px 관용) 수정 전 → Native / fresh WASM |
| --- | --- | --- |
| `samples/page_anchored_square/page_anchored_square_reflow.hwpx` | `pdf/page_anchored_square/page_anchored_square-2020.pdf` | 68.00 → 99.46 / 99.46 |
| `..._reflow_narrow.hwpx` | `..._reflow_narrow-2020.pdf` | 62.55 → 99.36 / 99.36 |
| `..._reflow_allow_overlap.hwpx` | `..._reflow_allow_overlap-2020.pdf` | 68.00 → 99.46 / 99.46 |
| `samples/para_square_lane/para_square_lane_reflow.hwpx` | `pdf/para_square_lane/para_square_lane_reflow-2020.pdf` | 65.07 → 100.0 / 100.0 |

생성기 기본 출력(뼈대 header 가 HANGUL fontface 만 선언)의 문단 기준 입력은 수정 전 65.86 →
수정 후 89.04%(Native = fresh WASM)다. 한/글은 선언 없는 LATIN 글자를 Haansoft Batang 으로
그리고 rhwp 는 함초롬바탕으로 재서, 다음 문단 첫 줄 끝 한 글자가 다음 줄로 넘어간다.
줄 수·차선·세로 위치는 정본과 같다. 이 입력은 회귀 fixture 로 쓰지 않는다.

남은 차이: 쪽 기준 표 테두리가 정본보다 약 2px 위(표 상단 = 본문 상단 + vertOffset,
한/글은 바깥 위 여백 140HU 를 더한 위치) — 2단계 배치 경로의 기존 차이이며 이번 변경 밖이다.
