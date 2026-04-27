# Task #386: exam_eng.hwp 1페이지 단 1 끝부분 밀림 — 구현계획서

> **이슈**: [#386](https://github.com/edwardkim/rhwp/issues/386)
> **브랜치**: `local/task386`
> **작성일**: 2026-04-27
> **마일스톤**: M100 (v1.0.0)

---

## 핵심 가설

`src/renderer/pagination/engine.rs:245-285`의 vpos 기반 `current_height` 보정 로직이 **다단 컬럼 전환 직후에도 단 0 마지막 문단 기준으로 발동**하여 단 1 첫 문단의 시작 누적값을 잘못 상향시킨다.

문제 핵심:
- `prev_pagination_para`는 함수 레벨 변수 (engine.rs:65), 컬럼 전환 시 리셋되지 않음
- 보정 조건에 `prev.column_start == cur.column_start` 가드 부재
- 단 0(블록 표 존재) → 단 1 전환 시 `page_has_block_table=true`이고 `prev_pi`가 단 0 마지막 문단으로 남아있어 보정 발동

## 수정 전략

**보정 로직에 컬럼 경계 가드 추가**.

```rust
// 변경 후 (의사 코드)
if let Some(prev_pi) = prev_pagination_para {
    if para_idx != prev_pi && st.page_has_block_table {
        // 추가: 컬럼 경계 가드
        let same_column = paragraphs.get(prev_pi)
            .and_then(|p| p.line_segs.last())
            .zip(para.line_segs.first())
            .map(|(prev_seg, cur_seg)| prev_seg.column_start == cur_seg.column_start)
            .unwrap_or(true);  // 정보 부재 시 기존 동작 유지
        if !same_column {
            // 컬럼 전환: 보정 스킵
        } else {
            // 기존 보정 로직 그대로
            ...
        }
    }
}
```

대안 (검토 결과에 따라 선택):
- 대안 A: 컬럼 전환 감지 시 보정 스킵 (위)
- 대안 B: `advance_column_or_new_page()` 내에서 `prev_pagination_para`를 None으로 리셋
- 대안 C: 보정 결과 `vpos_h` 자체가 컬럼 전환 시 이전 컬럼 끝 좌표를 가리키므로, page_vpos_base를 컬럼 전환 시 갱신

대안 A가 가장 영향 범위가 작고 명확하므로 1순위. 구현 단계에서 실측 후 결정.

---

## 단계 분할 (4 단계)

### 단계 1: 재현 및 정량 진단

**목표**: 가설을 실측 수치로 확정.

**작업 내용**:
1. `dump-pages -p 0` 출력 보존 (Before 베이스라인)
2. `engine.rs:245-285` 보정 로직에 일시적 디버그 출력 삽입:
   - 진입 시 `prev_pi`, `para_idx`, `prev.column_start`, `cur.column_start`, `current_height(before)`, `vpos_h`, `current_height(after)`
3. exam_eng.hwp p=0 재현하여 단 1 첫 문단(pi=33) 진입 시점의 보정 발동 여부 및 수치 확인
4. 디버그 출력은 단계 종료 시 제거

**완료 기준**:
- 단 0→단 1 전환 시 보정이 발동하는지, 발동 시 current_height 상향 수치가 +217px와 일치하는지 확인
- 가설이 부정되면 대안 원인 재조사 (단계 2로 진행 보류)

**산출물**: `mydocs/working/task_m100_386_stage1.md`

---

### 단계 2: 단위 테스트 추가 (Red)

**목표**: 컬럼 전환 시 보정 미발동 검증 테스트 작성 (수정 전 실패 확인).

**작업 내용**:
1. `src/renderer/pagination/engine.rs` 또는 `tests.rs`에 단위 테스트 추가:
   - 다단 + 블록 표 페이지 모킹
   - 단 0 → 단 1 전환 시 단 1 첫 문단의 current_height가 0으로 시작해야 함
2. 테스트 실행 → 실패 확인 (Red 상태)

**완료 기준**:
- 신규 테스트가 의도한 대로 실패
- 기존 테스트 회귀 0

**산출물**: `mydocs/working/task_m100_386_stage2.md`

---

### 단계 3: 컬럼 경계 가드 구현 (Green)

**목표**: 보정 로직에 컬럼 경계 가드 추가, 단계 2 테스트 통과.

**작업 내용**:
1. `engine.rs:245-285`에 가드 적용 (대안 A 우선):
   ```rust
   let same_column = match (prev_para.line_segs.last(), para.line_segs.first()) {
       (Some(prev_seg), Some(cur_seg)) => prev_seg.column_start == cur_seg.column_start,
       _ => true,  // 보수적: 정보 부재 시 기존 동작
   };
   if same_column {
       // 기존 보정 로직
   }
   ```
2. 단계 2 테스트 통과 확인
3. exam_eng.hwp p=0 재현하여 1페이지에 1~12번 모두 포함되는지 확인
4. `dump-pages -p 0` 단 1 diff 측정 (목표: ±20px 이내)

**완료 기준**:
- 단계 2 테스트 PASS
- exam_eng.hwp 1페이지에 1~12번 항목 모두 표시
- 단 1 used가 hwp_used와 ±20px 이내

**산출물**: `mydocs/working/task_m100_386_stage3.md`

---

### 단계 4: 통합 검증 및 회귀 측정 + 최종 보고

**목표**: 전체 회귀 측정 후 최종 보고서 작성.

**작업 내용**:
1. `cargo test --release` 전체 실행 — 1014+14+25+6+1+1 PASS 확인
2. 골든 SVG 비교 — exam_eng 외 변경 없음 확인
3. 다중 샘플 회귀 측정:
   - `2022년 국립국어원 업무계획.hwp` (Task #356 검증 샘플)
   - `aift.hwp`
   - `exam_math.hwp`
   - `2010-01-06.hwp`
   - `hwpspec.hwp` (블록 표 다수)
   - `kps-ai.hwp` (다단 + 표)
   - LAYOUT_OVERFLOW 카운트 비교
4. Task #62 가드 영향 확인 (글앞으로/글뒤로 Shape 케이스)
5. 최종 보고서 작성: `mydocs/report/task_m100_386_report.md`
6. `mydocs/orders/20260427.md` 상태 갱신

**완료 기준**:
- 전체 테스트 PASS
- 골든 회귀 0건 (exam_eng 변경만 허용)
- 다중 샘플 LAYOUT_OVERFLOW 회귀 없음

**산출물**:
- `mydocs/working/task_m100_386_stage4.md`
- `mydocs/report/task_m100_386_report.md`

---

## 영향 범위 및 위험

| 위험 | 완화 |
|------|------|
| 컬럼 전환 시 정상적으로 보정이 필요한 케이스가 있을 가능성 | 단계 4 회귀 측정 + Task #62 케이스 별도 확인 |
| `prev_seg.column_start` 정보가 없는 leg 케이스 | 정보 부재 시 보수적으로 기존 동작 유지 (`unwrap_or(true)`) |
| 다단 + 표 동시 페이지의 다른 인접 결함 | exam_eng 회귀 0 + 다중 샘플 회귀 측정으로 확인 |

## 커밋 단위

- 단계 1: 진단 보고서만 (소스 변경 없음, 디버그 출력은 단계 종료 시 제거)
- 단계 2: 단위 테스트 추가 + 보고서 (테스트는 실패 상태로 커밋)
- 단계 3: 가드 구현 + 보고서 (테스트 PASS)
- 단계 4: 회귀 측정 결과 + 최종 보고서 + orders 갱신

각 커밋 메시지 형식: `Task #386: <단계 제목>`
