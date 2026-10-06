---
kind: analysis
status: completed
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-04
---

# Task #3820·#3821 Stage 7 — p168 표 44 RowBreak 첫 fragment 누락 분석

## 목적과 범위

215쪽 전수 결함 종합 보고서의 D-03은 p168부터 시작하는 연쇄 pagination divergence다. 이 단계는
그 연쇄의 **최초 물리 분기**만 다룬다. p171 이후의 45개 이상 flow 후보를 쪽별로 보정하거나,
rhwp의 기준 PDF 대비 전체 쪽수 `+4`를 전역 page-break 보정으로 상쇄하지 않는다.

정답지는 한컴 2020으로 다시 저장한 다음 PDF이고, 입력은 개인정보를 제거한 같은 HWP 문서다.

- 입력: `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
- 정답지: `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
- 기준 renderer: `target/task-3820-3821-fidelity/release-test/rhwp`
- 전수 증적: `output/task-3820-3821-fidelity/stage7-full-sweep/stage7-full-215/`

## 재현과 최초 분기

다음 0-based page dump와 p168~170 raster 비교로 분기를 확인했다.

```text
target/task-3820-3821-fidelity/release-test/rhwp dump-pages \
  'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  -p 167 --json
target/task-3820-3821-fidelity/release-test/rhwp dump-pages \
  'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  -p 168 --json
target/task-3820-3821-fidelity/release-test/rhwp dump-pages \
  'samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp' \
  -p 169 --json
```

| 사용자 쪽 | 한컴 2020 PDF | 현재 rhwp | 판단 |
| --- | --- | --- | --- |
| p168 | 표 44(`pi=1775`) tail·caption 뒤에 표 `pi=1778`의 첫 fragment가 이어짐 | `pi=1775` tail·caption 뒤 하단이 비고 `pi=1778`이 없음 | **최초 분기** |
| p169 | `pi=1778`의 이어진 fragment 뒤에 그림 65(`pi=1780`)가 함께 놓임 | `pi=1778` 전체가 header부터 반복되고 그림 65는 없음 | p168 누락의 직접 결과 |
| p170 | `(라) 심혈관계 검사`(`pi=1784`) 본문으로 시작 | 그림 65만 놓임 | 이후 logical page owner 연쇄 이동의 시작 |

현재 rhwp p168의 `usedHeight`는 `630.4267px`이고 body height는 `956.1867px`다. 즉 약
`325.76px`의 물리 여유가 남는데도 `pi=1778`의 첫 fragment가 배치되지 않는다. 따라서 이 문제는
전역 page height 부족이 아니라, **남은 공간에서 RowBreak 표의 fragment를 선택·배치하지 못하는
분기**로 조사한다.

## 대상 표의 source 계약

`dump --section 0 --para 1778` 결과, 대상은 빈 host 문단의 다음 control이다.

- 4×3 표, `attr=0x04000006`, `RowBreak`
- `treat_as_char=false`, `wrap=자리차지`, 수직·수평 위치 모두 paragraph
- 크기 `42235×24695 HU` (약 149×87.1 mm)
- page를 넘는 셀 내용이 있어 row 내부 cut가 필요할 수 있음

이 표는 그림 65를 담은 다음 `pi=1780` (2×1, `treat_as_char=true`)와 계약이 다르다. 보정은
`pi=1778` 같은 native HWP5 non-TAC RowBreak 표의 first fragment 선택으로 좁히며, 그림·일반
inline 표·rowspan/footnote가 있는 표에 전파하지 않는다.

## 원인 가설과 구현 전 확인 항목

현재 pagination dump는 p169에 `pi=1778`을 `Table` 전체로 기록한다. 기준 PDF는 p168에서 그 표의
일부를 이미 소비하고 p169에서 continuation을 시작한다. 따라서 `PartialTable` 후보 산출 또는
첫 row/row-internal cut 선택이 p168 잔여 공간에서 실패했을 가능성이 높다.

코드를 바꾸기 전에 다음을 확인한다.

1. native RowBreak 경로가 p168의 실측 잔여 높이와 `MeasuredTable` 행/셀 cut 높이를 어떻게
   비교하는지 진단 로그로 고정한다.
2. fragment를 산출하는 경우 `PartialTable { para_index: 1778, start_row, end_row, start_cut,
   end_cut }`가 p168에 남는지, p169가 header 반복이 아닌 continuation인지 회귀 테스트로 먼저
   적는다.
3. p169에 `pi=1780`을 함께 둘 수 있고 p170 첫 body item이 `pi=1784`인지 확인한다.
4. source의 전체 표 높이, page count 또는 보편적인 table safety margin을 바꾸지 않는다. 다른
   RowBreak fixture의 page owner가 바뀌지 않는 focused regression을 함께 둔다.

## 수용 기준

구현 완료는 다음을 모두 만족해야 한다.

1. p168 render tree/dump에 `pi=1778`의 `PartialTable` fragment가 존재한다.
2. p169의 `pi=1778`은 p168에서 소비한 위치 다음의 continuation이며, PDF에 없는 전체 header
   반복이 생기지 않는다.
3. 그림 65(`pi=1780`)가 p169에 배치되고 p170은 `pi=1784`의 `(라) 심혈관계 검사` 본문으로
   시작한다.
4. p168~170 PDF raster/overlay와 layout ledger를 재생성해 최초 divergence가 해소됐는지 확인한다.
5. 이 수정 뒤에야 전수 215쪽 sweep으로 D-03 연쇄 후보가 실제로 줄었는지 판정한다.

이 분석은 `319ed3dd4`의 source-anchored 회귀와 renderer 보정으로 완료했다. 시각 대조와
잔여 범위는 [Stage 7 visual sweep](task_m100_3820_stage7_visual_sweep.md)에 고정한다.
