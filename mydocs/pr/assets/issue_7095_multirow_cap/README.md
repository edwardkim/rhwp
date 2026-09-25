# #7095 확장(다행 조각 쪽 상한) Visual Sweep 증적

## 실행

```bash
VISUAL_SWEEP_CHROME=<chrome> python3 scripts/visual_sweep.py \
  --file-target issue7336 samples/issue7336/stored_frame_page_larger_rowbreak.hwpx \
  pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf \
  --rhwp-bin <검증 바이너리> --wasm-pkg pkg --pages 1,2,3,4,5 --dpi 96 --out <out>
```

수정 후 실행은 fresh WASM(`pkg`, `rhwp_bg.wasm`
`5e3e1f71702474a99d90ff0f5af83ea6cbcfb604bec9d8d23b58cd2fe9627dc4`)을 함께 태웠다.
정본 `pdf/issue7336/stored_frame_page_larger_rowbreak-2020.pdf` (7쪽 = rhwp 7쪽). **gate passed.**

## 2px 이웃 관용 내용 실루엣 일치율

```text
         수정 전   수정 후
  p1      91.915    91.874
  p2      95.706    99.572   <- 상한이 발동한 쪽
  p3      99.754    99.754   <- 내용이 먼저 끝나 접지 않는다
  p4      99.951    99.951   <- 같음
  p5      97.218    99.602   <- 상한이 발동한 쪽
```

## 조각 상자 아래끝 — 양쪽 모두 괘선으로 측정

```text
  쪽   정본 아래끝   수정 전     수정 후
  p2    1022.99     1037.30    1024.00
  p3    1020.27     1021.40    1021.40
  p4    1007.63     1008.80    1008.80
  p5    1022.99     1029.50    1024.10
                           본문 94.49 .. 1028.01
```

수정 전 p2 는 본문 아래로 9.3px 나가 있었다.

## 측정 방법 주의

정본 상자를 «칠 영역»으로 집으면 **중첩 셀**을 외곽 틀로 오인한다. 표 괘선은 `m`/`l` 조각
수천 개이므로, x·y 별로 모아 6px 이하 끊김을 이어 붙인 뒤 세로 괘선 두 개가 같은 y 구간을
감싸는 경우만 틀로 센다. rhwp 쪽도 render tree 의 `Table` 노드 `bbox` 가 아니라 export-svg 의
`<line>` 을 같은 방식으로 병합해야 한다(2쪽 노드 bbox 121.70 vs 실제 괘선 167.80).
검증 기준값: `pdf/hwpx_sample2-hwpx-2020.pdf` 19쪽 = `89.91 .. 1081.39`.
