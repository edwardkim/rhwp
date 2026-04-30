# Task #495 최종 보고서

**이슈**: [#495](https://github.com/edwardkim/rhwp/issues/495) — exam_science.hwp 2페이지 7번 박스 안 동일 텍스트 중복 렌더링
**관련**: #496 (12번 줄간격 압축, 분리 처리)
**브랜치**: `local/task495`
**마일스톤**: M100 (v1.0.0)
**완료일**: 2026-04-30

---

## 1. 결과

### 결함 해결
- `samples/exam_science.hwp` 페이지 2의 7번 박스 안 paragraph p[1] ("◦ 분자당 구성 원자 수가...모두") 가 SVG에 두 번 그려져 시각적으로 겹쳐 보이던 결함을 수정.
- 같은 페이지의 8번/9번/11번 박스 등 동일 결함 패턴(multi-line paragraph + 인라인 사각형 ls[1]+) 도 함께 정정 (박스 셀 baseline 분포 19개 → 9개, 중복 10건 제거).

### 회귀
- **0건**. 64 SVG 광범위 회귀 검증 통과 (변경 파일 1건은 의도된 결함 수정).

### 테스트
- `cargo test --release`: 1094 + 다수 통합 테스트 모두 통과.
- `cargo clippy`: 본 task 변경부 경고/에러 0. 기존 2건 (`table_ops.rs:1007`, `object_ops.rs:298`) 은 본 task와 무관.

## 2. 결함 본질

### 코드 위치
`src/renderer/layout/table_layout.rs:1573~1635` — 표 셀 paragraph 의 인라인 Shape (`treat_as_char`) 처리 분기.

### 결함 메커니즘
이 분기는 사각형 컨트롤 앞의 텍스트(`text_before`)를 사각형 옆에 별도 TextRun으로 발행한다. 그러나 추출 코드가 `composed.lines.first()` (paragraph 첫 줄) 만 보고 있어:

- **단일 줄 paragraph + 사각형이 첫 줄 안**: 정상 (사각형 앞 텍스트 = 그 줄의 사각형 직전 텍스트)
- **Multi-line paragraph + 사각형이 ls[1]+ 에 있음**: text_before = ls[0] 전체 텍스트 (잘못 추출). paragraph_layout 이 ls[0] 를 정상 발행한 후, 이 코드가 동일 텍스트를 또 발행 → **시각적 중복**

paragraph p[1] (7번 박스 안):
- ls[0]: "◦ 분자당...모두" (35 글자)
- ls[1]: 사각형 ctrl[1] + " 이다."
- ctrl[1] 사각형의 `tac_pos` 가 ls[1] 시작 → 잘못된 text_before = ls[0] 전체

### 비교 — Picture 분기
같은 함수의 `Control::Picture` 분기 (라인 1483~1530) 는 `target_line` 산출 + 줄 변경 시 `inline_x/y` 리셋 로직을 갖는다 — multi-line 정상 처리. **Shape 분기에만 줄 추적 누락**.

## 3. 수정 — 옵션 B (가드 추가)

`tac_pos` 가 paragraph 첫 줄 char 범위 안일 때만 text_before 추출/발행하도록 가드 추가:

```rust
let in_first_line = composed.lines.first()
    .map(|line| {
        let line_chars: usize = line.runs.iter().map(|r| r.text.chars().count()).sum();
        tac_pos >= line.char_start && tac_pos < line.char_start + line_chars
    })
    .unwrap_or(false);
let text_before: String = if in_first_line {
    composed.lines.first().map(|line| { /* 기존 추출 로직 */ }).unwrap_or_default()
} else {
    String::new()
};
```

변경: `+27 -13` (한 파일).

### 가드 정합성

| paragraph 구조 | 가드 동작 | 발행 |
|---|---|---|
| 단일 줄 + 사각형 첫 줄 안 (synam-001 p[16] "년 월 일") | `in_first_line=true` | 기존 동작 ✓ |
| Multi-line + 사각형 ls[0] 안 | `in_first_line=true` | 기존 동작 ✓ |
| Multi-line + 사각형 ls[1]+ (exam_science p[1]) | `in_first_line=false` | 차단 (결함 수정) ✓ |

→ 결함 케이스만 핀포인트 차단, 다른 모든 케이스 기존 동작 보존.

## 4. 진행 경로

### 단계 1 — 원인 정확 진단 (`task_m100_495_stage1.md`)
- 진단용 임시 로그로 paragraph p[1] 의 layout_composed_paragraph 호출이 1회뿐임을 확인.
- 라인 발행은 line_idx=0 (y=219.01), line_idx=1 (y=240.48) 두 줄. 그러나 SVG 에는 y=224.26 추가 라인.
- 다른 TextRun 발행 경로 전수 검색 → `table_layout.rs:1612~1635` 의 Shape 인라인 분기 식별.
- 진단 코드 모두 제거.

### 단계 2 — 베이스라인 + 분기 필요성 검증 (`task_m100_495_stage2.md`)
- 베이스라인 64 SVG 보존 (exam_science p2 + synam-001 35p + k-water-rfp 28p).
- 단계 2 가설: "paragraph_layout 이 paragraph 모든 줄을 발행하므로 text_before TextRun 은 모든 케이스에서 중복" → 후보 (B) text_before 발행 제거 결정.

### 단계 3 v1 — 후보 (B) 적용 후 회귀 발견 (`task_m100_495_stage3.md`)
- text_before TextRun 생성/push 블록 제거 적용.
- exam_science 결함 해결 확인.
- **회귀 발견**: synam-001 p31 의 "2020년 [공백] 월 [공백] 일" 패턴에서 "년", "월" 글자 누락.
- 단계 2 가설이 거짓: paragraph p[16] (8개 사각형 사이의 짧은 텍스트) 의 "년/월" 은 paragraph_layout 이 발행하지 못하고 Shape 분기 text_before 가 유일한 발행 경로였음.
- 코드 즉시 되돌림. 작업지시자 결정 요청.

### 단계 3 v2 — 옵션 B (가드) 재시도 (`task_m100_495_stage3_v2.md`)
- 작업지시자 결정으로 옵션 B (가드) 채택.
- 사전 검증: synam-001 p[16] 이 단일 줄 paragraph (line_seg 1 개) 임을 dump 으로 확인 → 가드 영향 없음.
- 가드 코드 적용 + 빌드 통과.
- 결함 해결 (exam_science y=224.26 사라짐) + 회귀 0건 (64 SVG 비교).

### 단계 4 — 최종 정리 (본 보고서)
- cargo test 통과 + clippy 본 task 무관 검증.
- orders 갱신, 보고서 작성, 커밋 준비.

## 5. 잔존 이슈 (별도 task)

### 사각형 자체 위치 결함
exam_science p[1] 의 사각형 (ctrl[1]) 이 SVG y=206.75 (ls[0] 시작) 에 그려진다. PDF 기준은 ls[1] 위치 (y≈228). 이는 multi-line paragraph 에서 `inline_x` 가 첫 줄 텍스트 폭 누적 후 사각형 위치 계산에 사용되어 잘못된 좌표가 됨. **본 task 범위(텍스트 중복) 와 별도이므로 분리 처리 필요**. 후속 이슈 검토 권장.

### #496 (12번 줄간격 압축)
같은 페이지의 12번 paragraph (pi=60/61) 의 줄간격이 비정상적으로 압축됨. 본 결함과 코드 경로 다름 (paragraph_layout 의 lineseg vpos 처리). 별도 task.

## 6. 변경 통계

| 파일 | 변경 |
|---|---|
| `src/renderer/layout/table_layout.rs` | +27 -13 |

## 7. 산출물

- 수행계획서: `mydocs/plans/task_m100_495.md`
- 구현계획서: `mydocs/plans/task_m100_495_impl.md`
- 단계 보고서: `mydocs/working/task_m100_495_stage{1,2,3,3_v2}.md`
- 최종 보고서: `mydocs/report/task_m100_495_report.md` (본 문서)
- orders 갱신: `mydocs/orders/20260430.md`

## 8. 핵심 교훈

- 단계 2 의 가설 ("paragraph_layout 이 모든 텍스트 발행") 을 단일 케이스 (exam_science) 만 보고 일반화한 것이 단계 3 v1 회귀의 직접 원인.
- 메모리 `feedback_essential_fix_regression_risk.md` 의 "광범위 샘플 검증" 룰을 단계 3 적용 전이 아니라 단계 2 가설 검증 단계에 적용해야 했음. 향후 layout 본질 정정 task 에서 가설 자체를 광범위 샘플로 검증.
- 결함을 만든 코드 자체는 잘못 — 그러나 그 코드가 다른 케이스 (synam-001 p[16]) 의 정상 발행 경로이기도 함. **결함 코드 = 보강 코드** 라는 이중 역할 식별이 옵션 (B) 제거 → 옵션 (B) 가드 전환의 핵심.

## 9. 검증 체크리스트

- [x] exam_science p2 결함 해결 (y=224.26 라인 사라짐)
- [x] 시각 검증 (PDF 정합)
- [x] 회귀 0건 (64 SVG 비교)
- [x] cargo test --release 통과
- [x] cargo clippy 본 task 변경부 경고 0
- [x] orders 갱신
- [x] 진단용 임시 코드 모두 제거 확인
