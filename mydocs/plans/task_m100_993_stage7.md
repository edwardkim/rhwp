# Task M100-993 Stage 7 계획서

## 1. 목적

Stage 6에서 `mel-001`의 2페이지 첫 1x1 표 배경 문제는 `BORDER_FILL #7`
gradient payload 저장 계약 문제로 분리했고, 수정 후보가 한컴 에디터에서 배경 정상 판정을 받았다.

남은 문제는 `tb-org-02` 조직도 표의 셀 내부 줄나눔이다.

```text
현상:
  생성 HWP를 한컴 에디터에서 열면 조직도 셀 내부 텍스트가 한글자씩 줄나눔된다.

정답:
  같은 셀에서 텍스트가 2글자씩 줄나눔되어야 한다.

결과:
  한글자씩 줄나눔되면 셀 높이가 증가하고, 조직도 표 전체 높이가 커진다.
```

Stage 7의 목적은 이 현상을 셀 높이 보정으로 우회하지 않고, 정답 HWP와 생성 HWP의
HWP5 record/IR 차이를 비교해 한컴 에디터의 셀 내부 줄나눔 폭 산정이 달라지는 원인을 찾는 것이다.

## 2. 현재 관찰

선행 확인에서 정답 HWP의 조직도 주요 셀은 긴 문자열 하나를 자동 줄바꿈한 형태가 아니라,
2글자 단위 텍스트 조각이 여러 문단으로 저장된 형태가 관찰되었다.

예시:

```text
한국|산업|안전|보건|공단
한국|산업|인력|공단
노사|발전|재단
건설|근로|자공|제회
```

따라서 Stage 7은 다음 두 가능성을 먼저 분리한다.

```text
1. HWPX 원본에도 이미 2글자 단위 문단 분리가 존재하지만 parser/adapter가 합쳐 버린다.
2. HWPX 원본은 의미적으로 하나의 문자열이고, 한컴 저장기는 HWP5 저장 시점에 셀 폭에 맞춰
   2글자 단위 문단/line segment contract를 materialize한다.
```

이 둘을 구분하지 않으면, 셀 높이나 행 높이를 조정하는 잘못된 방향으로 빠질 수 있다.

## 3. 입력

문제 축소 샘플:

```text
samples/hwpx/tb-org-02.hwpx
samples/hwpx/hancom-hwp/tb-org-02.hwp
pdf-large/hwpx/tb-org-02.pdf
```

Stage 6 생성 후보:

```text
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp
```

## 4. 비교 대상 셀

대표 셀을 고정해 정답 HWP와 생성 HWP를 비교한다.

```text
1. 한국산업안전보건공단
2. 한국산업인력공단
3. 노사발전재단
4. 건설근로자공제회
5. 학교법인한국폴리텍
```

비교 항목:

```text
- 셀 row/col, row_span/col_span, width/height
- 셀 padding, vertical_align, text_direction, list_attr
- 셀 내부 paragraph 개수
- paragraph별 text
- paragraph별 char shape ref
- paragraph별 para shape ref
- paragraph별 LINE_SEG count와 vpos, width, height 계열 값
- LIST_HEADER level/size/extra/tail
- PARA_HEADER flag/shape/count 계열 값
```

## 5. 작업 범위

이번 단계에서는 다음을 수행한다.

```text
1. 정답 HWP와 생성 HWP의 조직도 셀 내부 문단 구조를 같은 표 형식으로 덤프한다.
2. HWPX 원본 XML에서 같은 셀의 텍스트 저장 형태를 확인한다.
3. 정답 HWP의 2글자 단위 문단 분리가 HWPX 원본에서 온 것인지, HWP 저장 과정에서 계산된 것인지 판정한다.
4. 차이가 확인된 필드군만 사용해 다음 probe 후보를 설계한다.
```

probe는 바로 생성하지 않는다. Stage 7은 비교와 원인 축 확정 단계다.

## 6. 산출물

작업 기록:

```text
mydocs/working/task_m100_993_stage7.md
```

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage7_tb_org_cell_linebreak_contract/
```

예상 산출물:

```text
tb_org_cell_text_structure_oracle.md
tb_org_cell_text_structure_generated.md
tb_org_cell_text_structure_hwpx.md
tb_org_cell_linebreak_contract_findings.md
```

## 7. 성공 기준

Stage 7의 성공 기준은 구현 결과가 아니라 원인 축 확정이다.

```text
1. 정답 HWP와 생성 HWP에서 같은 셀의 문단 개수/텍스트 조각 차이를 명확히 기록한다.
2. HWPX 원본 XML이 2글자 단위 문단을 갖고 있는지 확인한다.
3. 다음 단계에서 적용할 후보가 "셀 높이 보정"이 아니라
   "셀 내부 문단/줄나눔 HWP5 contract 보존 또는 생성"임을 확정한다.
```

## 8. 비목표

이번 단계에서 다음은 하지 않는다.

```text
- 셀 높이/행 높이 사후 보정
- 한컴 표시 결과를 맞추기 위한 임의 문자열 분할
- mel-001 전체 표 배치 수정
- BorderFill gradient contract 재수정
```

## 9. 승인 요청

이 계획으로 Stage 7을 진행한다.
