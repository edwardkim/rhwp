# Task M100-993 Stage 5 계획서

## 1. 목적

`tb-org-02.hwpx`의 조직도 셀 내부 텍스트가 rhwp-studio에서 한컴 에디터보다 크게 렌더링되는
문제를 수정한다.

`tb-org-02`는 `mel-001`에서 문제가 되는 조직도 높이 증가 표만 별도 문서로 저장한 축소
repro 샘플이다. Stage 5에서는 이 샘플을 primary repro로 사용하고, `mel-001`은 실제 문서
회귀 guard로 사용한다.

작업지시자 확인 결과:

```text
- 한컴 에디터: 조직도 셀 내부 글자 크기 8.0 pt
- rhwp-studio: 조직도 셀 내부 글자 크기 10.0 pt
```

따라서 Stage 5의 목적은 HWP 저장 계약이 아니라 HWPX 렌더링 기준선에서 다음 두 축을 찾고
고치는 것이다.

```text
1. CharShape/Style 해석이 8pt를 10pt로 fallback하는 경로
2. 셀 세로 방향 정렬 top이 셀 내부 layout에 적용되지 않는 경로
```

## 2. 배경

Stage 3에서 `mel-001.hwp` 파일손상은 해소되었다.

```text
원인: caption table CTRL_HEADER common_attr의 0x20000000 bit 누락
결과: 한컴 에디터에서 파일 정상 열림
```

Stage 4에서 HWP5 cell `LIST_HEADER` tail과 cell `PARA_HEADER` tail/flag를 정답지에서 graft한
probe를 만들었지만 조직도 표 높이와 셀 내부 텍스트 배치는 개선되지 않았다.

이후 확인된 핵심 전환점:

```text
mel-001.hwpx 자체를 rhwp-studio에서 열어도 2페이지 조직도 셀 내부 텍스트 배치가 틀린다.
즉, 잘못된 IR/렌더링 상태를 HWP로 저장하고 다시 불러오고 있었다.
```

## 3. 입력

Primary repro:

```text
source HWPX: samples/hwpx/tb-org-02.hwpx
oracle HWP:  samples/hwpx/hancom-hwp/tb-org-02.hwp
oracle PDF:  pdf-large/hwpx/tb-org-02.pdf
```

Regression guard:

```text
source HWPX: samples/hwpx/mel-001.hwpx
oracle HWP:  samples/hwpx/hancom-hwp/mel-001.hwp
oracle PDF:  pdf-large/hwpx/mel-001.pdf
```

## 4. 작업 범위

이번 단계에서는 HWP 저장기를 수정하지 않는다.

우선 대상:

```text
1. tb-org-02 HWPX 파서가 조직도 셀 텍스트의 CharShape/Style 참조를 올바르게 IR로 옮기는지 확인
2. 렌더러가 셀 내부 텍스트의 CharShape font size를 실제 glyph/layout 계산에 적용하는지 확인
3. 8.0pt가 10.0pt로 fallback하는 위치를 특정
4. 조직도 셀의 vertical align 값이 top으로 해석되는지 확인
5. 셀 내부 paragraph layout에서 top align이 y 좌표 계산에 반영되는지 확인
6. 특정한 위치만 수정하고 tb-org-02.hwpx 렌더링을 재확인
7. mel-001.hwpx에서 동일 문제가 회귀되지 않는지 확인
```

탐색 후보:

```text
src/parser/hwpx/
src/model/
src/renderer/
src/renderer/layout.rs
```

## 5. 진단 산출물

산출물 위치:

```text
output/poc/hwpx2hwp/task993/stage5_org_cell_font_size_probe/
```

생성할 진단 자료:

```text
1. tb-org-02.hwpx SVG export
2. tb-org-02 정답 HWP SVG export
3. 조직도 표의 대표 셀 텍스트 layout dump
4. 대표 셀 텍스트의 CharShape id, Style id, resolved font size dump
5. 대표 셀의 vertical align 원본값, IR 값, layout 적용값 dump
6. 수정 후 tb-org-02.hwpx SVG export
7. 수정 후 mel-001.hwpx 2페이지 SVG export
```

대표 셀 후보:

```text
- 장관
- 정책보좌관
- 운영지원과
- 노동시장정책관
- 조직도 하단의 2줄 이상 텍스트 셀
```

## 6. 성공 기준

1차 성공 기준:

```text
tb-org-02.hwpx를 rhwp-studio에서 열었을 때 조직도 셀 내부 글자 크기가 한컴 기준 8.0pt와
일치한다.
```

2차 성공 기준:

```text
조직도 셀 vertical align top이 적용되어, 셀 내부 텍스트 배치가 정답 HWP/PDF와 시각적으로
일치한다.
```

3차 성공 기준:

```text
수정 후 mel-001.hwpx에서도 Stage 3의 조직도 표 높이 문제가 재현되지 않는다.
```

## 7. 비목표

이번 단계에서 다음 문제를 함께 해결하지 않는다.

```text
- 2페이지 첫 1x1 표 배경 문제
- 1페이지 첫 부분 정체 불명 선
- 추가 HWP5 tail/payload 보강
```

위 항목은 8pt 렌더링 기준선이 정상화된 뒤 별도 단계에서 다시 판정한다.

## 8. 승인 요청

이 계획으로 Stage 5를 진행한다.
