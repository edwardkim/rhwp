# #6761 · #7345 · #7351 Visual Sweep 증적 (1480000-201900042 88·89쪽)

- 입력: `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp`
  (sha256 `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860`)
- 기준: `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf`
  (한컴 2020 경로, 103쪽, sha256 `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79`)
- 명령: `python3 scripts/visual_sweep.py --hwp <입력> --pdf <기준> --dpi 96 --pages 88-89`
  (Native: `--rhwp-bin <release rhwp>`, fresh WASM: `--wasm-pkg pkg`, 인쇄 프로필)
- before: devel `e1ecaa248` release 빌드 · after: 이 PR head 빌드(Native·fresh WASM)

| 쪽 | before Native | after Native | after fresh WASM |
| ---: | ---: | ---: | ---: |
| 88 | 56.98% | 93.05% | 93.01% |
| 89 | 27.19% | 96.09% | 96.07% |

2px 이웃 관용 내용 실루엣 일치율. `pr_review_gate` 는 Native·WASM 모두 `passed`.
전체 103쪽 Native 실루엣 TSV(before→after)는 25쪽 상승, 하락 0쪽, 쪽수 103 = 정본 103.
