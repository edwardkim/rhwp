---
kind: reference
status: historical
canonical: mydocs/troubleshootings/README.md
last_verified: 2026-07-16
---

# HWPX LINE_SEG reflow 함정

## 발견일: 2026-03-31

## 현상

`samples/tac-img-02.hwpx` pi=837 문단에서 텍스트+강제줄넘김(\n)+TAC 표 구조의 렌더링이 한컴과 달랐음.
- 텍스트 줄의 높이가 표 높이(187.9px)로 계산되어 과도한 공간 차지
- 텍스트가 잘려서 일부만 렌더링됨

## 원인

### HWPX 원본 LINE_SEG

```xml
<hp:linesegarray>
  <hp:lineseg textpos="0" vertsize="1000" textheight="1000" baseline="850"
              spacing="800" horzpos="0" horzsize="1836" flags="393216"/>
</hp:linesegarray>
```

LINE_SEG **1개**, `vertsize=1000`(텍스트 높이만). 표 높이가 포함되지 않음.

### reflow 후 LINE_SEG

```
ls[0]: ts=0,  lh=14094, th=1300  ← 표 높이(14094)가 포함됨!
ls[1]: ts=55, lh=14376, th=14376
```

`reflow_line_segs`가 원본 1개 LINE_SEG를 2개로 분리하면서 LINE_SEG[0]의 `lh`에 표 높이를 포함시킴.

### HWP 바이너리 (정답)

```
ls[0]: ts=0,  lh=1300, th=1300   ← 텍스트 높이만!
ls[1]: ts=55, lh=14376, th=14376
```

HWP에서는 LINE_SEG[0]의 `lh=1300`으로 텍스트 높이만 사용.

## 함정

1. **HWPX의 LINE_SEG는 최소한의 정보만 포함**됨 (vertsize=1000, 1개만)
2. **reflow_line_segs가 이를 재계산**할 때 TAC 표 높이를 텍스트 줄에 포함시킴
3. **HWP 바이너리는 한컴이 직접 계산한 LINE_SEG**를 가짐 → 텍스트와 표 높이가 분리됨
4. 따라서 **HWPX의 reflow 결과를 신뢰하면 안 되고, HWP를 참조하여 올바른 높이를 사용해야 함**

## 교훈

- HWPX를 해석할 때는 항상 **HWP 바이너리를 정답지로 크로스 체크**해야 함
- HWPX의 LINE_SEG가 부족한 경우 reflow로 재계산하지만, TAC 표가 포함된 문단에서 reflow가 올바른 결과를 주지 않음
- composer에서 LINE_SEG `lh`가 `th`보다 현저히 큰 경우(표 높이 포함) `th`를 사용하는 보정이 필요

## 해결

- composer `compose_lines()`: TAC 표 문단의 LINE_SEG `lh`를 `th`로 보정 (Task #19)
- composer `compose_lines()`: 강제 줄넘김(\n) 전 텍스트를 이전 ComposedLine에 합침 (Task #20)
- pagination: `para_height_for_fit`를 표 실측 높이 + 텍스트 th 기반으로 계산 (Task #19)

## 관련 이슈

- [#19](https://github.com/edwardkim/rhwp/issues/19) TAC 표 혼합 문단 pagination 높이 이중 계산
- [#20](https://github.com/edwardkim/rhwp/issues/20) composer 강제 줄넘김 후 TAC 표 ComposedLine 분리
