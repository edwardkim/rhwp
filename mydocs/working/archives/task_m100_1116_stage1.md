# Task #1116 Stage 1 보고서 — 3mm 격자 기준선 재측정

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 수행계획서: `mydocs/plans/task_m100_1116.md`
- 구현계획서: `mydocs/plans/task_m100_1116_impl.md`
- 브랜치: `local/task1116`
- 작성일: 2026-05-25
- 상태: 소스 수정 전 기준선 측정 완료

## 1. 작업지시자 기준 자료

작업지시자가 3mm 격자 표시 상태의 한컴오피스 캡처 2장을 제공했다.

1. 3쪽 본문: `I. 사업개요`부터 `3. 주요 추진내용`까지 표시.
2. 2쪽 목차: `목차` 전체와 하단 별첨 항목까지 표시.

기존 `mydocs/feedback/hwp3-sample16-hwp5-analysis.md`는 격자를 1mm로 가정한 비교가 포함되어 있었다. 이번 Stage 1부터는 해당 가정을 폐기하고 3mm 격자로 해석한다.

## 2. 실행 명령

```bash
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 1
target/debug/rhwp dump-pages samples/hwp3-sample16-hwp5.hwp -p 2
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 1
target/debug/rhwp dump-pages samples/hwp3-sample16.hwp -p 2
```

문단 상세:

```bash
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 69
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 70
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 71
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 72
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 73
target/debug/rhwp dump samples/hwp3-sample16-hwp5.hwp -s 0 -p 74
```

SVG 기준선:

```bash
target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/debug/task1116/hwp5-p2-grid \
  -p 1 \
  --show-grid \
  --debug-overlay \
  --show-control-codes

target/debug/rhwp export-svg samples/hwp3-sample16-hwp5.hwp \
  -o output/debug/task1116/hwp5-p3-grid \
  -p 2 \
  --show-grid \
  --debug-overlay \
  --show-control-codes
```

생성 파일:

```text
output/debug/task1116/hwp5-p2-grid/hwp3-sample16-hwp5_002.svg
output/debug/task1116/hwp5-p3-grid/hwp3-sample16-hwp5_003.svg
output/poc/render-spacing/hwp3-sample16-hwp5-page3-grid/hwp3-sample16-hwp5_003.svg
```

## 3. 환산 기준

```text
1mm = 283.465 HWPUNIT
3mm = 850.395 HWPUNIT
96dpi SVG 기준 1mm = 3.7795px
96dpi SVG 기준 3mm = 11.3386px
```

## 4. p3 본문 기준선

`hwp3-sample16-hwp5.hwp` p3 `dump-pages` 결과:

```text
body_area: x=56.7 y=75.6 w=680.3 h=971.3
used=874.5px, hwp_used≈803.5px, diff=+71.0px
```

동일 구간의 HWP3 원본:

```text
used=901.5px, hwp_used≈828.7px, diff=+72.9px
```

즉 p3 본문은 변환본만의 단독 현상이라기보다 HWP3 sample16 계열에서 렌더러 진행량이 파일의 `LINE_SEG.vpos` 흐름보다 약 71~73px 길어지는 패턴이다.

### 4.1 주요 문단 좌표

| 항목 | pi | vpos(HU) | 렌더 top(px) | 비고 |
|------|---:|---:|---:|------|
| `I. 사업개요` | 69 | 0 | 75.6 | 본문 top |
| `1. 추진목적` | 70 | 3200 | 124.0 | overlay 기준 |
| 목적 박스 호스트 | 71 | 5760 | 156.2 / 282.6 | 실제 shape top과 host overlay가 분리되어 보임 |
| 빈 문단 | 72 | 16304 | 293.0 | `spacing_before=568`, `lh=300` |
| `2. 추진방향` | 73 | 16768 | 299.2 | `spacing_before=852` |
| 첫 본문 항목 | 74 | 19328..23360 | 333.3 | 3줄 문단 |
| `3. 주요 추진내용` | 79 | 43984 | 662.1 | 누적 하강 관찰 지점 |

### 4.2 `pi=71 -> pi=73` 진행량

파일상 상세:

```text
pi=71 ls[0]: vpos=5760,  lh=9764, ls=780
pi=72 ls[0]: vpos=16304, lh=300,  ls=164, spacing_before=568
pi=73 ls[0]: vpos=16768, lh=1600, ls=960, spacing_before=852
```

목적 박스 하단에서 `2. 추진방향` line segment 시작까지:

```text
16768 - (5760 + 9764) = 1244 HU
1244 HU = 약 4.39mm = 약 1.46칸(3mm 격자)
```

렌더된 SVG에서 목적 박스 실제 하단으로 보이는 좌표와 `2. 추진방향` 문단 top을 비교하면 약 32.1px 차이다.

```text
32.1px = 약 8.49mm = 약 2.83칸(3mm 격자)
```

파일상 vpos 간격보다 렌더 상단 간격이 더 크게 보이는 이유는 `spacing_before`, 빈 문단 높이, overlay 기준점이 함께 섞여 있기 때문이다. Stage 3에서는 이 구간을 실제 shape bbox 기준과 paragraph top 기준으로 분리해 봐야 한다.

### 4.3 3mm 격자 캡처와의 육안 비교

채팅에 첨부된 캡처는 원본 이미지 파일 좌표를 직접 추출한 것이 아니라 화면상 확인이므로 아래 판단은 육안 기준이다.

| 구간 | rhwp 기준 | 3mm 격자 환산 | 한컴 캡처 육안 |
|------|----------:|--------------:|----------------|
| 목적 박스 높이 | 9764 HU / 34.45mm | 약 11.48칸 | 약 11~12칸 |
| 목적 박스 하단 -> `2. 추진방향` | 렌더 기준 약 8.49mm | 약 2.83칸 | 약 2~3칸 |
| `2. 추진방향` -> 첫 본문 항목 | 약 34.1px / 9.03mm | 약 3.01칸 | 약 2~3칸 |
| `2. 추진방향` 본문 4개 항목 누적 | 각 항목 71.1px | 각 약 6.27칸 | 큰 차이 후보 |

상단의 목적 박스 자체 높이와 바로 아래 간격은 3mm 격자 기준으로 아주 큰 차이로 보이지 않는다. 다만 p3 전체 `used`가 `hwp_used`보다 71px 길기 때문에, 줄 문단이 여러 개 누적되는 `2. 추진방향` 본문과 `3. 주요 추진내용` 이후에서 차이가 커질 가능성이 높다.

## 5. p2 목차 기준선

`hwp3-sample16-hwp5.hwp` p2 `dump-pages` 결과:

```text
body_area: x=56.7 y=75.6 w=680.3 h=971.3
used=969.0px, hwp_used≈963.4px, diff=+5.6px
```

HWP3 원본:

```text
used=968.1px, hwp_used≈962.5px, diff=+5.6px
```

p2 목차는 전체 높이 누적 차이가 작다. 따라서 p2 문제는 페이지 진행량보다 다음 항목이 우선 후보다.

1. right-tab 위치와 page number 우측 정렬.
2. leader 점선의 시작/종료 x.
3. `TopAndBottom + treat_as_char=true` 사각형 조판부호가 탭 계산에 끼어드는지 여부.

`--show-grid` 포함 SVG 기준선에서 첫 목차 항목은 다음처럼 렌더된다.

```text
pi=27 y=152.9
텍스트: "    1. 추진목적  1"
같은 문단에 Shape ci=0 wrap=TopAndBottom tac=true
```

작업지시자 캡처의 p2 목차는 페이지 번호가 본문 우측 근처에서 정렬되고 leader 점선이 페이지 번호 직전까지 이어진다. rhwp 기준선은 사각형 조판부호 텍스트가 표시되는 디버그 SVG라 직접 시각 비교가 어렵지만, right edge 산포와 leader x 좌표를 수치 테스트로 잡을 수 있다.

## 6. Stage 1 결론

1. 3mm 격자 전제로 보면 p3 상단 목적 박스 높이와 바로 아래 간격만으로는 큰 오차라고 단정하기 어렵다.
2. p3 전체에서는 `used - hwp_used`가 약 71px로 크다. `2. 추진방향` 이후 3줄 본문 문단들의 line height/line spacing 누적을 우선 조사해야 한다.
3. p2 목차는 높이 누적 차이가 작다. right-tab, leader, 페이지 번호 정렬을 별도 수치 테스트로 잡는 방향이 맞다.
4. 구현은 아직 하지 않았다. 다음 단계는 p2 목차 수치 테스트와 p3 줄 문단 누적 높이 차이 분석이다.

## 7. 다음 단계

Stage 2:

- p2 목차 SVG에서 페이지 번호 text의 right edge와 leader 종료 x를 추출한다.
- 기존 `tests/issue_874_ktx_toc_page_number_right_align.rs`의 방식을 `hwp3-sample16-hwp5.hwp` p2에 적용할 수 있는지 확인한다.

Stage 3:

- `pi=74~77`의 3줄 문단에서 `LINE_SEG.vpos` 기반 높이와 렌더러 문단 높이의 차이를 분해한다.
- `spacing_before=568`, `line=155%`, `ls=716`이 렌더러에서 중복 또는 과대 반영되는지 확인한다.
