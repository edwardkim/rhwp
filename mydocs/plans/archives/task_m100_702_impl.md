# Task #702: 구현 계획서

## Stage 1 진단 결과 요약

### 본질 1 정정 (수행계획서 본질 1 재진단)

**최초 가설 (수행계획서)**: `SectionColumnDef` 후속 갱신 누락 — 첫 번째 단정의는 동작, 두 번째 이후 무시.

**실제 원인 (Stage 1 진단)**: 단정의 갱신은 정상 동작. 결함은 **inter-paragraph vpos-reset 검출 임계값**이 보수적이라 짧은 컬럼에서 column-advance 가 발동하지 않는 것.

`src/renderer/typeset.rs:430-434`:
```rust
let trigger = if st.col_count > 1 {
    cv < pv && pv > 5000   // ← 다단 케이스 임계값
} else {
    cv == 0 && pv > 5000   // ← 단단 케이스 임계값
};
```

**문제 케이스 (지우기 섹션, 6항목 2단 배분)**:
- pi=32 ("한 단어 지우기") last vpos = 3000 (< 5000 임계값)
- pi=33 ("앞 단어 지우기") first vpos = 0
- → 임계값 미달로 trigger=false → column-advance 미발동 → pi=33~35 가 col 0 에 적층

PDF 정답: 3+3 (col 0: pi=30~32, col 1: pi=33~35)
rhwp 출력: 6+0 (col 0 에 모두 적층)

**왜 5000 이 도입됐는가**:
- 단단(`col_count==1`): partial-table split 의 LAYOUT 잔재로 vpos가 high→low 변동 발생 (Issue #418, hwpspec pi=78→pi=79). 이걸 column-break 로 오인 방지.
- 다단(`col_count>1`): Task #470 에서 `cv != 0` 도 column-break 인정하되 동일 임계값 유지 (안전 보수).

### 본질 2 진단 — 시각 영향 미미

`src/renderer/typeset.rs:1023`:
```rust
st.current_height += if st.col_count > 1 { fmt.height_for_fit } else { fmt.total_height };
```

다단 누적은 `height_for_fit` (trailing_ls 제외) 사용. Task #359 + exam_eng 8p 정합으로 도입.

`hwp_used > used` 차이 (예: 단 2 used=186.7 vs hwp_used=273.3)는 trailing_ls 누적 분만큼 발생. 그러나 **본질 1 정정 후에도 이 측정 차이가 페이지 분기에 추가 영향을 주지 않을 가능성**이 높음 (col_count>1 분기는 fit 판정도 height_for_fit 사용해 balance 유지). 따라서 **본질 2는 본 사이클 범위 외**로 분리.

## 정정 방안 (본질 1)

### 옵션 비교

| 옵션 | 변경 범위 | 회귀 위험 | 본질 정합 |
|------|----------|----------|----------|
| A. `pv > 5000` 임계값 완화 (`pv > 0`) | 1줄 | 🔴 높음 (Task #321/#418/#470 회귀 가능) | 🟡 헤유리스틱 |
| B. `MultiColumn` 직후 zone 만 임계값 완화 | 가드 추가 | 🟡 중간 | 🟡 휴리스틱 |
| C. `ColumnType::Distribute` 한정 임계값 완화 | 가드 추가 + ColumnDef 전달 | 🟢 낮음 | 🟢 본질 (스펙 정합) |

**채택: 옵션 C**

이유:
- HWPX 명세에 `BalancedNewspaper` (=ColumnType::Distribute) 는 컨텐츠 균등 분배가 본질. 짧은 컬럼이라도 vpos-reset 가 column-break 신호.
- `Normal` (`NEWSPAPER`) 단은 기존 임계값 유지 → 회귀 차단.
- 메모리 룰 "essential_fix_regression_risk" 정합 — 범위 한정.

### 구현 상세

#### 변경 1: `current_zone_layout` 에 ColumnType 보존

`src/renderer/page_layout.rs` 의 `PageLayoutInfo` 또는 별도 zone state 에 `column_type: ColumnType` 추가.

`process_multicolumn_break` (typeset.rs) 에서 새 ColumnDef 매칭 시 column_type 전파.

#### 변경 2: vpos-reset 임계값 완화 가드

`src/renderer/typeset.rs:430-434` 수정:
```rust
let column_type = st.current_zone_column_type();  // 신규 메서드
let is_distribute = matches!(column_type, ColumnType::Distribute);

let trigger = if st.col_count > 1 {
    if is_distribute {
        cv < pv && pv > 0   // 배분: 임계값 완화 (짧은 컬럼 허용)
    } else {
        cv < pv && pv > 5000   // 일반 다단: 기존 유지
    }
} else {
    cv == 0 && pv > 5000   // 단단: 기존 유지
};
```

#### 변경 3 (회귀 가드 테스트)

`src/renderer/pagination/tests.rs` 또는 별도 `tests/shortcut_distribute.rs`:
- 짧은 Distribute 컬럼 (3+3 분배) 정합 테스트
- Normal 다단 (긴 컬럼) 회귀 차단 테스트

## 검증 절차

### Stage 2 검증 (본질 1 정정)

1. `cargo build --release` 빌드 통과
2. `rhwp dump-pages samples/basic/shortcut.hwp` 출력에서:
   - 페이지 1 단 5 가 `(items=3) + 단 6 (items=3)` 로 분할
   - 페이지 수가 7쪽에 근접 (현재 10쪽 → 7쪽 목표)
3. `rhwp export-svg samples/basic/shortcut.hwp` 후 SVG 시각 정합 확인 (`qlmanage` 비교)

### Stage 3 회귀 검증

1. **Distribute 다단 정합**:
   - shortcut.hwp 7쪽 ≈ PDF 7쪽
   - 페이지별 컬럼 분배 정합
2. **Normal 다단 회귀 차단**:
   - exam_eng 류 다단 샘플 8p 정합 유지 (Task #470)
   - hwpspec 다단 컬럼 회귀 차단 (Issue #418)
3. **단단 회귀 차단**:
   - kps-ai pi=317 단독 페이지 차단 유지 (Task #359)
   - hwp-multi-001 force_page_break 회귀 차단
4. **광범위 샘플**:
   - KTX, aift, hwp-multi-001, kps-ai, hwpx/aift export-svg 무회귀
   - `cargo test` 전체 통과

### Stage 4 본질 2 영향 평가 (선택)

- 본질 1 정정 후에도 hwp_used vs used diff 가 시각 결함을 만드는지 평가
- 만들면 별도 task 로 분리, 안 만들면 보고서에만 기록

## 단계 구성

### Stage 2 — 본질 1 정정 (옵션 C)

작업 항목:
1. `PageLayoutInfo` 또는 `TypesetState` 에 `current_zone_column_type` 필드/메서드 추가
2. `process_multicolumn_break` 에서 ColumnDef 매칭 시 column_type 전파
3. `typeset.rs:430-434` vpos-reset 임계값 분기 추가 (Distribute 한정 완화)
4. 회귀 가드 테스트 작성
5. shortcut.hwp dump-pages + export-svg 시각 검증

산출: `mydocs/working/task_m100_702_stage1.md` (Stage 2 단계별 보고서 — 명명 규약상 stage1 지만 실제 구현 단계 1)

### Stage 3 — 광범위 회귀 검증 + 시각 판정

작업 항목:
1. `cargo test --release` 전체 통과
2. 광범위 샘플 export-svg 회귀 (KTX, aift, hwp-multi-001, kps-ai, hwpx/aift, exam_eng 류)
3. shortcut.hwp 7쪽 vs PDF 7쪽 페이지별 시각 정합 (qlmanage 이미지 비교)
4. 한컴 2010/2020 정답지 비교 (가능한 경우)

산출: `mydocs/working/task_m100_702_stage2.md`

### Stage 4 — 최종 보고 + 부수 결함 분리

작업 항목:
1. 본질 2 평가 + 부수 결함 (탭 leader, PUA 글자, 바탕쪽 자동번호) 별도 이슈 등록
2. 최종 결과 보고서 작성
3. 작업지시자 승인 후 commit

산출: 
- `mydocs/working/task_m100_702_stage3.md` (회귀 + 시각 판정 보고)
- `mydocs/report/task_m100_702_report.md` (최종 보고서)

## 회귀 위험 평가

🔴 **High**: 옵션 C 는 ColumnType::Distribute 한정으로 범위가 좁지만, 다단 정의 type 정보 전파 경로 추가는 zone 상태 관리에 영향. PageLayoutInfo / TypesetState 변경은 광범위 영향 가능.

🟡 **Medium**: Distribute 임계값 완화로 wrap_around 표/그림 + Distribute 다단 조합에서 잠재 회귀 가능.

🟢 **Low (회귀 가드)**: Normal 다단 + 단단 분기는 기존 임계값 유지 → 핵심 회귀 차단.

## 승인 요청

본 구현계획대로 Stage 2 (본질 1 정정) 진입 가능 여부 승인 부탁드립니다.
