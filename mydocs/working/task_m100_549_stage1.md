# Task #549 Stage 1 완료 보고서

**제목**: TDD RED 테스트 + 광범위 사전 평가 + 옵션 결정
**브랜치**: `local/task549`
**이슈**: https://github.com/edwardkim/rhwp/issues/549

---

## 1. TDD 통합 테스트 추가 (RED 확인)

`integration_tests.rs` 에 `test_549_cell_inline_brackets_centered_p14` 추가.

페이지 14 [28~30] 박스 [A] inline TAC 표 (pi=299 cell[2]) 의 두 줄 정렬 검증:
- 검증 1: line 0 "[" cell-rel x ≥ 3.0 px (좌측 붙음 방지)
- 검증 2: 두 줄 bracket 위치 차이 ≈ PDF (5.58 px ±1.2)

```
test test_549_cell_inline_brackets_centered_p14 ... FAILED
[A] cell line 0 "[" cell-rel x=2.33 px 가 너무 좌측 (3.0 px 이상 필요).
버그(수정 전): bracket_rel=2.32 (aim=false hack 으로 pad=510 HU 강제).
PDF 한컴 2010/2020 기대값: 3.28 px (cell_center 기준 Center 정렬).
```

`#[ignore]` attribute 적용. 1122 단위 테스트 모두 통과.

## 2. 광범위 사전 평가

`examples/scan_aim_cells.rs` 로 6 샘플의 aim=false hack 발동 케이스 분석.

### 2.1 Hack 발동 분포

| 샘플 | total cells | aim=false | hack 발동 (cell>table) |
|------|------------|-----------|----------------------|
| 21_언어_기출 | 70 | 70 | 32 (45.7%) |
| exam_kor | 209 | 209 | 81 (38.8%) |
| exam_math | 25 | 25 | 25 (100%) |
| exam_eng | 278 | 278 | 275 (98.9%) |
| exam_science | 375 | 375 | 245 (65.3%) |
| synam-001 | 750 | 750 | 332 (44.3%) |
| **합계** | **1707** | **1707** | **990 (58.0%)** |

> 모든 샘플 cells 의 aim=false → 한컴이 cell.apply_inner_margin=false 로 저장.
> Task #347 hack 이 1707 cells 중 990 cells 에서 발동 (cell.padding > table.padding).

### 2.2 옵션 분리 분석

| 샘플 | 옵션 B 적용 (table=0) | 옵션 B 보존 (table>0) |
|------|---------------------|---------------------|
| 21_언어_기출 | 24 | 8 |
| exam_kor | 75 | 6 |
| exam_math | 4 | 21 |
| exam_eng | 235 | 40 |
| exam_science | 154 | 91 |
| synam-001 | **12** | **320** |
| **합계** | **504** | **486** |

**중요 관측**: synam-001 의 332 hack 케이스 중 320 cells (96.4%) 는 table.padding > 0
케이스. 옵션 A 시 320 cells 회귀 가능. 옵션 B 시 보존.

### 2.3 회귀 위험 정량화

#### 옵션 A (hack 전면 제거)

- 영향: **990 cells** (모든 hack 케이스)
- 회귀 위험: 매우 큼
  - synam-001 320 cells (KTX 목차 등 비대칭 의도 케이스)
  - exam_eng 275 cells (98.9% 셀)
  - Task #347 KTX (cell.right=1417 비대칭) 회귀 거의 확실

#### 옵션 B (table.padding=0 케이스만 hack 제거)

- 영향: **504 cells** (table.padding=0 인 케이스만)
- 보존: **486 cells** (table.padding>0 인 KTX-like 케이스 — Task #347 의도 보존)
- 회귀 위험: 중간
  - 504 cells 모두 "table=0 + cell>0" 본질 버그 케이스 (작성자가 cell.padding 만
    설정하고 table.padding 미설정한 상황 — 한컴은 spec 대로 table.padding 사용)
  - 실제 시각 회귀는 Stage 2 광범위 검증 시 fix 적용 후 확인

## 3. 옵션 결정

**옵션 B 선택**.

### 3.1 근거

1. **본질 정정**: 504 cells 모두 "table.padding=0 인데 cell.padding>0" 인 케이스로,
   HWP 스펙 (aim=false → table.padding) 따르면 **0 padding** 적용해야 함. 한컴 PDF
   가 그렇게 동작 (페이지 14 [A] 검증).
2. **KTX 보존**: Task #347 의 비대칭 cell.padding > table.padding > 0 케이스는 보존.
   486 cells (synam-001 320 cells 포함) 변경 없음.
3. **명확한 분리**: "table=0 + cell>0" 은 작성자의 의도라기보다 IR 인코딩 노이즈
   (cell.padding 디폴트가 510 HU 같은 값으로 저장됨). table>0 인 경우만 의도된
   비대칭으로 본다.

### 3.2 적용 코드 (Stage 2 예정)

```rust
let prefer_cell_axis = |c: i16, t: i16| -> bool {
    if cell.apply_inner_margin {
        c != 0
    } else {
        // [Task #549] aim=false: cell이 table보다 명백히 크고 table.padding > 0 일 때만
        // cell 우선 (Task #347 KTX 의도). table.padding=0 + cell>0 은 IR 인코딩
        // 노이즈로 보고 spec 대로 table 사용 (=0).
        (c as i32) > (t as i32) && (t as i32) > 0
    }
};
```

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/integration_tests.rs` | RED 테스트 1건 (+115 LOC) |
| `examples/scan_aim_cells.rs` | 광범위 평가 도구 (참고용) |
| `mydocs/working/task_m100_549_stage1.md` | 본 보고서 |

## 5. 다음 단계 (Stage 2)

1. `resolve_cell_padding` aim=false 분기 옵션 B 적용
2. RED → GREEN 확인
3. 1122 단위 테스트 무회귀 (Task #534v2/#537/#539/#540/#544/#547/#548 보존)
4. Stage 2 보고서 + 커밋

## 6. 승인 요청

Stage 1 완료. 옵션 B (table.padding=0 케이스만 hack 제거) 진행 OK?

승인 후 Stage 2 (fix 적용) 진행합니다.
