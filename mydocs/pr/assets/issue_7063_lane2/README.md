# #7063 레인② Visual Sweep 증적

쪽 중간 조각의 상자·clip·정렬을 가른 변경의 수정 전후 대표 증적이다.

## 실행

```bash
VISUAL_SWEEP_CHROME=<chrome> python3 scripts/visual_sweep.py \
  --file-target hwpx_sample2 samples/hwpx_sample2.hwpx pdf/hwpx_sample2-hwpx-2020.pdf \
  --rhwp-bin target/pr-review/release/rhwp --pages 11,12,19,20,21,22,23,24,25,26 --dpi 96 --out <out>
```

정본 `pdf/hwpx_sample2-hwpx-2020.pdf` (Producer `Hancom PDF 1.3.0.550` · 29쪽 = rhwp 29쪽).

## 2px 이웃 관용 내용 실루엣 일치율

```text
        devel   레인②
  p11   98.16 →  97.50
  p12   87.10 →  99.74
  p19   80.96 →  99.40   ← 이슈 본문의 목표 쪽
  p20   79.85 →  86.79
  p21   75.63 →  75.63
  p22   98.93 →  99.66
  p23~26 99.9x →  99.7~99.85
```

## 남은 차이 (이 축 밖)

20·21쪽은 같은 쪽의 **다행·다열** 조각(`pi=185` 5×2 등)이 선언 `outMargin.top` 을 못 받아
남는다. 그 갈래는 계보를 가르지 않고 열어야 하는데(정본 두 벌이 같다) 그러면 전수
overflow 가 2561 → 2614 로 36문서가 어긋나, 조각 두께 축(#6976)이 선행이다.
