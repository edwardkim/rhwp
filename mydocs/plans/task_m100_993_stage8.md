# Task M100-993 Stage 8 계획서

## 1. 목적

Stage 7에서 `tb-org-02` 조직도 셀의 1글자 줄나눔 문제는 텍스트 분할 손실이 아니라
HWP5 저장 contract 차이로 좁혀졌다.

반복 차이:

```text
1. 셀 LIST_HEADER
   oracle:    size=47
   generated: size=34

2. 셀 내부 PARA_HEADER
   oracle:    size=24
   generated: size=22
```

Stage 8의 목적은 이 차이를 한컴 에디터 판정으로 분리하는 것이다.

```text
가설:
  한컴 에디터가 셀 내부 문단 폭을 계산할 때,
  셀 LIST_HEADER의 width_ref/extra 또는 PARA_HEADER 확장 필드가 필요하다.
```

## 2. 입력

정답 HWP:

```text
samples/hwpx/hancom-hwp/tb-org-02.hwp
```

생성 기준 파일:

```text
output/poc/hwpx2hwp/task993/stage6_cell_contract_probe/tb-org-02-gradient-fixed.hwp
```

검증 원본:

```text
samples/hwpx/tb-org-02.hwpx
pdf-large/hwpx/tb-org-02.pdf
```

## 3. 대상 셀

Stage 7에서 확인한 대표 조직도 셀을 우선 대상으로 한다.

```text
cell[151] 한국|산업|안전|보건|공단
cell[153] 한국|산업|인력|공단
cell[165] 노사|발전|재단
cell[167] 건설|근로|자공|제회
cell[188] 학교|법인|한국|폴리|텍
```

필요 시 같은 패턴의 조직도 셀 전체로 확장한다.

## 4. Probe 후보

생성 폴더:

```text
output/poc/hwpx2hwp/task993/stage8_tb_org_cell_linebreak_probe/
```

판정 파일:

| variant | 내용 |
|---|---|
| `01_current.hwp` | Stage 6 생성 기준 파일 |
| `02_target_org_cell_list_width_ref_only.hwp` | 대표 셀 LIST_HEADER bytes 6..7만 정답 `0x0001`로 보강 |
| `03_target_org_cell_list_extra_only.hwp` | 대표 셀 LIST_HEADER 34바이트 뒤 extra 13바이트만 보강 |
| `04_target_org_cell_list_width_ref_plus_extra.hwp` | 대표 셀 LIST_HEADER width_ref + extra 보강 |
| `05_target_org_cell_para_header_tail_only.hwp` | 대표 셀 내부 PARA_HEADER 확장 필드만 정답처럼 보강 |
| `06_target_org_cell_list_bundle_plus_para_header_tail.hwp` | 대표 셀 LIST_HEADER bundle + PARA_HEADER 확장 필드 보강 |
| `07_all_org_cells_list_width_ref_plus_extra.hwp` | 조직도 전체 셀 LIST_HEADER width_ref + extra 보강 |
| `08_all_org_cells_list_bundle_plus_para_header_tail.hwp` | 조직도 전체 셀 LIST_HEADER bundle + PARA_HEADER 확장 필드 보강 |

## 5. 구현 방식

이번 단계는 production adapter를 바로 수정하지 않는다.

```text
1. 정답 HWP와 생성 HWP의 BodyText Section0 record를 읽는다.
2. Stage 7에서 식별한 target LIST_HEADER/PARA_HEADER record index를 기준으로 payload 일부를 graft한다.
3. 각 variant를 CFB로 재포장한다.
4. rhwp 재로드 가능 여부와 page count를 기록한다.
5. 한컴 에디터 판정은 작업지시자가 수행한다.
```

중요 원칙:

```text
- 셀 높이를 직접 줄이지 않는다.
- 문자열을 임의로 다시 자르지 않는다.
- 정답 HWP에서 확인된 raw contract 차이만 보강한다.
- 성공 축이 확인된 뒤에만 adapter 일반화 구현을 설계한다.
```

## 6. 판정표

```markdown
| variant | 한컴 판정 유형 | 조직도 셀 줄나눔 | 표 높이 | rhwp-studio 판정 | 비고 |
|---|---|---|---|---|---|
| 01_current.hwp |  |  |  |  | 기준 |
| 02_target_org_cell_list_width_ref_only.hwp |  |  |  |  |  |
| 03_target_org_cell_list_extra_only.hwp |  |  |  |  |  |
| 04_target_org_cell_list_width_ref_plus_extra.hwp |  |  |  |  |  |
| 05_target_org_cell_para_header_tail_only.hwp |  |  |  |  |  |
| 06_target_org_cell_list_bundle_plus_para_header_tail.hwp |  |  |  |  |  |
| 07_all_org_cells_list_width_ref_plus_extra.hwp |  |  |  |  |  |
| 08_all_org_cells_list_bundle_plus_para_header_tail.hwp |  |  |  |  |  |
```

## 7. 성공 기준

1차 성공:

```text
한컴 에디터에서 조직도 셀 텍스트가 2글자 단위로 줄나눔된다.
```

2차 성공:

```text
조직도 표 높이가 정답 PDF/HWP와 같은 수준으로 돌아온다.
```

3차 성공:

```text
rhwp-studio에서 기존 정상 조판이 회귀하지 않는다.
```

## 8. 승인 요청

이 계획으로 Stage 8 probe 생성을 진행한다.
