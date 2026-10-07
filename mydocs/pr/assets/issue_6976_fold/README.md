# #6976 배치 뒤 행 단위 상자 접기 — Visual Sweep 증적

쪽을 끝내는 조각의 마지막 행 상자를 배치 뒤에 접는 변경의 수정 전후 대표 증적이다.

## 실행

```bash
VISUAL_SWEEP_CHROME=<chrome> python3 scripts/visual_sweep.py \
  --file-target hwpx_sample2 samples/hwpx_sample2.hwpx pdf/hwpx_sample2-hwpx-2020.pdf \
  --rhwp-bin <검증 바이너리> --wasm-pkg pkg --pages 1,11,12,19 --dpi 96 --out <out>

VISUAL_SWEEP_CHROME=<chrome> python3 scripts/visual_sweep.py \
  --file-target issue1949 samples/issue1949_giant_cell_nested_tables_perf.hwpx \
  pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf \
  --rhwp-bin <검증 바이너리> --wasm-pkg pkg --pages 1,2,3 --dpi 96 --out <out>
```

수정 후 실행은 fresh WASM(`pkg`, `rhwp_bg.wasm`
`2d00ba26fc31f93bd388ee21fa9d89c1b8fb7c9eabac9a745a0b209b38469b85`)을 함께 태웠다.

## 대표 문서 — `hwpx_sample2` (gate `passed`)

정본 `pdf/hwpx_sample2-hwpx-2020.pdf` (`Hancom PDF 1.3.0.550` · 29쪽 = rhwp 29쪽).

2px 이웃 관용 내용 실루엣 일치율:

```text
         수정 전   수정 후
  p1      94.008    94.003
  p11     97.478    98.164   <- 접기가 발동한 쪽
  p12     99.698    99.698
  p19     99.369    99.356   <- #7063 레인② 쪽(접지 않는다)
```

p11 은 표 조각 상자가 180.50 → 178.70 으로 접혀 정본(178.39)과의 차가 2.11 → 0.31 로 줄었다.
p19 는 쪽이 상자를 정한 조각이라 접기에서 제외되고, 레인②가 세운 값이 그대로 유지된다.

## 접기가 가장 크게 움직인 문서 — `issue1949` (gate **미통과**, 이 변경과 무관)

정본 `pdf/issue1949_giant_cell_nested_tables_perf-hwpx-2020.pdf` (115쪽 = rhwp 115쪽).

```text
         수정 전     수정 후
  p1     78.55818   78.55818
  p2     78.22815   78.22793
  p3     78.53732   78.53732
```

세 쪽 모두 90% 아래이고, **수정 전후가 소수점 다섯 자리까지 같다.** overlay 를 직접 열어 보면
글꼴 자체는 일치하고 쪽 위쪽에서 시작한 글줄이 아래로 갈수록 누적으로 어긋난다 — 이 문서의
줄간격 누적 차이는 선행 결함이고 이 변경이 건드리지 않는다. 그래서 이 문서는 gate 대표로 쓰지
않고, 표 상자 높이의 정본 대조(아래)로만 인용한다.

```text
  1쪽 표 괘선 상자   정본 79.36 .. 1056.90 (h 977.54)
    수정 전                          h 985.40  (+7.86)
    수정 후                          h 975.80  (−1.74)
```

## 남은 차이

- `samples/hwpx/form-002.hwpx` 1쪽은 정본이 같은 칸을 **쪽까지** 채우는데(585.09..1024.27,
  h 439.18) rhwp 는 이미 9.85px 짧았고, 접기로 10.81px 로 0.96px 더 짧아진다. 원인은 조각
  상자를 쪽으로 고정하는 축(#7095)이 이 형상에서 발동하지 않는 것이고 이 변경의 범위가 아니다.
- `table_giant_cell_overfill` 은 접기 뒤에도 정본보다 3.7px 짧은 쪽이 남는다. 한/글이 마지막
  줄 뒤에 남기는 잔여(문서마다 1.75~3.79px)는 아직 근거가 없어 공식을 세우지 않았다.
