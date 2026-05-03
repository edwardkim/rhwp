# Task #464 구현 계획서

**이슈**: #464
**브랜치**: `local/task464_v2`
**전제**: 단계 1 완료 (`task_m100_464_stage1.md`)

---

## 1. 결함 위치 (재확인)

### A. typeset.rs:904 — col_break 감지 가드
```rust
let col_breaks = if st.col_count > 1 && st.current_column == 0 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### B. typeset.rs:1975-1981 — col 1 페이지 break 누락
```rust
if bi + 1 < col_breaks.len() {
    st.flush_column();
    if st.current_column + 1 < st.col_count {
        st.current_column += 1;
        st.current_height = 0.0;
    }
    // ELSE 누락: 마지막 단에서 col_break 발생 시 페이지 break 처리 안 됨
}
```

### C. pagination/engine.rs:597-610 — 동기 엔진
같은 패턴.

## 2. 수정 방향

### 2.1 가드 완화 (typeset.rs:904)

```rust
// [Task #464] current_column == 0 가드 제거: col 1 (마지막 단) 에서도
// vpos-reset 인코딩된 col_break 를 감지해 페이지 break 를 트리거.
let col_breaks = if st.col_count > 1 {
    Self::detect_column_breaks_in_paragraph(para)
} else {
    vec![0]
};
```

### 2.2 페이지 break 트리거 (typeset.rs:1975~1981)

```rust
if bi + 1 < col_breaks.len() {
    st.flush_column();
    if st.current_column + 1 < st.col_count {
        st.current_column += 1;
        st.current_height = 0.0;
    } else {
        // [Task #464] 마지막 단에서 col_break: 페이지 break 트리거
        // (HWP 인코딩 vpos-reset 이 단 경계가 아닌 페이지 경계인 경우)
        st.flush_page();  // 또는 적절한 페이지 break 메서드
        st.current_column = 0;
        st.current_height = 0.0;
    }
}
```

`flush_page` 의 정확한 메서드 이름은 단계 3에서 코드 검증 후 확정.

### 2.3 pagination/engine.rs 동기화

같은 가드 + 같은 페이지 break 처리.

## 3. 단계 구성

### 단계 2 (Stage 2) — 베이스라인 + 구현 검증
- 회귀 검증용 SVG 베이스라인 생성:
  - exam_kor.hwp (전체 20 페이지)
  - exam_eng.hwp, exam_math.hwp, exam_science.hwp, exam_social.hwp (전체)
  - 다단 샘플로 식별된 다른 hwp
- typeset.rs / engine.rs 의 정확한 메서드 시그니처 확인 (flush_page 등)
- 산출물: `mydocs/working/task_m100_464_stage2.md`

### 단계 3 (Stage 3) — typeset.rs 수정 + 결함 검증
- typeset.rs:904 가드 완화
- typeset.rs:1975~1981 페이지 break 트리거 추가
- 빌드 + exam_kor.hwp p15 결함 검증:
  - LAYOUT_OVERFLOW 사라짐
  - 페이지 분리 정상 (15p → 16p)
- 산출물: `mydocs/working/task_m100_464_stage3.md`

### 단계 4 (Stage 4) — pagination/engine.rs 동기화 + 통합 검증
- engine.rs 동일 패턴 수정
- 결함 재검증 + 광범위 회귀 검증:
  - 베이스라인 SVG 비교
  - exam_eng/math/science/social 회귀 0
  - exam_kor.hwp 페이지 수 변동 0 (20 페이지 유지)
  - cargo test 통과
  - cargo clippy
- 산출물: `mydocs/working/task_m100_464_stage4.md`

### 단계 5 (Stage 5) — 최종 정리
- 최종 보고서: `mydocs/report/task_m100_464_report.md`
- orders 갱신
- 커밋 + local/devel merge + devel push
- Issue close

## 4. 위험 관리

- **회귀 위험 큼**: 다단 + vpos-reset 처리는 광범위 영향. 베이스라인 SVG 비교로 정밀 검증 필수.
- **#459 회귀 차단**: #459 가 fix 한 col 0 → col 1 케이스가 깨지지 않도록 주의.
- **#418 회귀 차단**: partial-table split 케이스. 단계 4 에서 시뮬레이션.
- 메모리 `feedback_essential_fix_regression_risk.md`: 광범위 샘플 + 한컴 2010/2020 검증 필요.

## 5. 자동승인 진행 정책

자동승인 모드. 단계마다 결과를 명확히 보고하고, 회귀 위험이 식별되면 그 단계에서 진행 중단 후 별도 승인 요청 (#496 학습).

## 6. 단계 2 즉시 진입

본 구현 계획서 작성 직후 단계 2 (베이스라인 + 구현 검증) 에 착수합니다.
