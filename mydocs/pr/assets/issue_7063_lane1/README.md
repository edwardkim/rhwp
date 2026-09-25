# #7063 레인① Visual Sweep 증적

`#7095` 조각 상자 계약의 계보 게이트를 여는 변경의 수정 전후 대표 증적이다.

## 실행

```bash
VISUAL_SWEEP_CHROME=<chrome> python3 scripts/visual_sweep.py \
  --file-target issue2004 samples/issue2004_cell_image_stack.hwpx \
  pdf/issue2004_cell_image_stack-hwpx-2020.pdf \
  --rhwp-bin target/pr-review/release/rhwp --pages 4-8 --dpi 96 --out <out>
```

정본 `pdf/issue2004_cell_image_stack-hwpx-2020.pdf` (Producer `Hancom PDF 1.3.0.550` ·
8쪽 = rhwp 8쪽). 이 문서의 **HWP 쌍둥이**는 `#7095` 로 이미 정본과 맞고 HWPX 만 표 위
바깥여백 283HU(3.77px)만큼 위에 있었다.

## 2px 이웃 관용 내용 실루엣 일치율

```text
        수정 전    수정 후
  p4    81.88  →   99.94
  p5    77.06  →  100.00
  p6    76.51  →  100.00
  p7    79.05  →  100.00
  p8    77.34  →  100.00
  gate  re_review_required → passed
```

## 남은 차이 (이 축 밖)

`hwpx_sample2.hwpx` 20쪽은 79.85% → 86.79% 로 좋아지지만 90% 에 못 미친다. 줄별 대조상
그 격차는 이 수정이 고친 조각(줄 0~10 일치) 밖이다 — `pi=184` 어울림 TAC +1.91px,
`pi=185`(5×2, `#7095` 형상 밖) +3.79px, 별도 블록 +15.68px, 그리고 rhwp 에만 있는 줄 하나
(48줄 vs 정본 47줄).
