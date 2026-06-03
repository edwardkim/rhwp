# Task #546 Stage 1 완료 보고서 — 회귀 origin 정확히 식별

## 결과 요약

**회귀 origin 확정**: `82e41ba` (Task #460 보완5: Square wrap 그림 아래 텍스트 y위치 보정 — layout + typeset)

| 항목 | 값 |
|------|-----|
| commit | `82e41ba` |
| 제목 | Task #460 보완5: Square wrap 그림 아래 텍스트 y위치 보정 (layout + typeset) |
| author | Taesup Jang (tsjang@gmail.com) — 작업지시자 본인 commit |
| 변경 영역 | `src/renderer/layout.rs` (+58) + `src/renderer/typeset.rs` (+36) — 총 +94 LOC |
| date | 2026-05-01 00:55 |

## bisect 진행 (작업지시자 직접 분기 점검)

작업지시자 진단으로 `ab2f4d0` (Task #460 보완4 HWP3 Square wrap) 전후로 시작한 후, 시간 순서대로 좁힘.

| 순서 | commit | 영역 | p2 단 0 items / used | 총 페이지 | 결과 |
|------|--------|------|---------------------|----------|------|
| 1 | `0aa7a5e` (ab2f4d0 직전) | Task #460: Square wrap 레이아웃 수정 + HWPX TextRunReflow 자동 보정 | 37 / 1133.6 px | 4 | ✅ 정상 |
| 2 | `ab2f4d0` | Task #460 보완4: HWP3 Square wrap 위치 계산 paper-relative → column-relative | 37 / 1133.6 px | 4 | ✅ 정상 |
| 3 | `9d9b4ed` | Task #460 보완4: Square wrap 그림 어울림 텍스트 렌더링 수정 | 37 / 1133.6 px | 4 | ✅ 정상 |
| 4 | **`82e41ba`** | **Task #460 보완5: Square wrap 그림 아래 텍스트 y위치 보정 (layout + typeset)** | **2 / 132.7 px** | **6** | ❌ **회귀** |

→ `82e41ba` 단일 commit 으로 회귀 발생 정확히 좁혀짐. 4 단계 점검만으로 식별 완료 (작업지시자 가설 정확).

## 결함 본질 (Stage 2 정밀 분석 대상)

### 변경 영역

`82e41ba` 의 변경:
- `src/renderer/layout.rs` (+58 LOC) — Square wrap 그림 아래 텍스트의 layout y위치 보정
- `src/renderer/typeset.rs` (+36 LOC) — typeset 단계 y위치 처리

### 결함 양상

`samples/exam_science.hwp` 페이지 2:

```
=== 82e41ba 직전 (정상) ===
페이지 2 — 단 0 (items=37, used=1133.6px)
  FullParagraph  pi=32  vpos=0  "7.-다음은 학생 가 수행한 탐구 활동이다."
  Shape          pi=32 ci=0  수식  vpos=0
  Table          pi=33 ci=0  1x1  407.4x269.4px  wrap=TopAndBottom tac=true  vpos=2390
  ... (37 items 정상 배치)

=== 82e41ba (회귀) ===
페이지 2 — 단 0 (items=2, used=132.7px)
  Table          pi=30 ci=0  3x3  407.4x107.1px  wrap=TopAndBottom tac=true  vpos=78952
  Table          pi=31 ci=0  1x10  411.9x19.1px  wrap=TopAndBottom tac=true  vpos=87944
  (본문 35 items 모두 다른 페이지로 분리됨)
```

본문 (paragraph + 수식 + 표) 35 items 가 다른 페이지로 분리되고, p2 에는 표 2개 (`pi=30 ci=0` + `pi=31 ci=0`) 만 남음. 총 페이지 4 → 6 으로 증가.

### 가설 (Stage 2 진단)

- `82e41ba` 의 layout.rs +58 LOC 가 Square wrap 그림 아래 텍스트의 y위치를 보정하면서, 그림이 없는 영역의 y위치 advance 까지 영향
- `typeset.rs` +36 LOC 의 변경이 페이지 경계 (column 끝) 처리에 부작용
- exam_science.hwp p2 에 Square wrap 그림이 있는지, 또는 Square wrap 정정 영역이 다른 본문 영역과 상호작용하는지 추가 진단 필요

## 다음 단계

Stage 1 보고서 승인 후 **Stage 2** (구현 계획서 작성) 진행:

1. `82e41ba` 의 변경 영역 정밀 분석 (layout.rs +58 + typeset.rs +36)
2. exam_science.hwp p2 영역의 본질 식별 (Square wrap 그림 여부, 페이지 경계 영향)
3. 정정 정책 결정 (옵션 A/B/C):
   - **A**. revert 단일 commit (`82e41ba` 의 의도된 정정 손실)
   - **B**. 부분 revert (페이지 경계 부작용 영역만)
   - **C**. 별도 정합 정정 (`82e41ba` 의 정정 보존 + exam_science 영역의 분기 추가)
4. 회귀 테스트 설계 (exam_science p2 페이지네이션 결정적 검증)

## 산출물

- 본 보고서 (`mydocs/working/task_m100_546_stage1.md`)

## 메모리 정합

- ✅ `feedback_v076_regression_origin` — bisect 로 회귀 origin 정확히 식별 (단일 commit)
- ✅ `feedback_search_troubleshootings_first` — Stage 1 직전 troubleshootings 사전 검색 (Square wrap / pagination / exam_science 영역 직접 매핑 없음)
