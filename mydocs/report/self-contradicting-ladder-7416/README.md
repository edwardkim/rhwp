# #7416 제 선언 폭과 모순되는 저장 줄 사다리 — 시각 증적

- 입력: `samples/issue6639/rhwp-table-cell-minimal-repro.hwp` (1쪽)
- 기준 PDF: `pdf/issue6639/issue6639-original-160-2020.pdf` (한/글 2020, 1쪽)
- 명령: `python3 scripts/visual_sweep.py --hwp <입력> --pdf <기준> --key issue6639 --rhwp-bin <bin> --dpi 96`
  (fresh WASM 은 같은 명령에 `--wasm-pkg pkg`)
- 2px 이웃 관용 내용 실루엣 일치율

| 출력 | 일치율 | 쪽 수 (rhwp / PDF) |
| --- | ---: | ---: |
| 수정 전 Native | 38.17% | 1 / 1 |
| 수정 후 Native | **95.69%** | 1 / 1 |
| 수정 후 fresh WASM | **95.69%** | 1 / 1 |
| (참고) 사다리를 지운 대조 입력 `issue6639-reference-input-160.hwpx` | 95.69% | 1 / 1 |

수정 후 원본 출력은 사다리를 지운 대조 입력의 출력과 같은 점수다 — 한/글이 원본 사다리를
쓰지 않는다는 기존 관측(두 한/글 PDF 의 96dpi 래스터가 화소 단위로 같음)과 일치한다.

| 파일 | 내용 |
| --- | --- |
| `issue6639_before_after_oracle.png` | 칸 31 확대 — 수정 전 / 수정 후 / 한/글 PDF |
| `issue6639_p1_before_review.png` | 수정 전 review |
| `issue6639_p1_after_review.png` | 수정 후 review |
| `issue6639_p1_after_overlay.png` | 수정 후 overlay |

남은 차이: 위 표 머리(행 2~4)의 세로 괘선 일부 위치는 이 변경 전후가 같고 이 이슈의 범위가 아니다.
