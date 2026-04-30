# Task #495 단계 2 보고서 — 회귀 베이스라인 + 분기 필요성 검증

**이슈**: #495
**브랜치**: `local/task495`
**전제**: 단계 1 완료 (`task_m100_495_stage1.md`)
**자동승인 모드**: 본 보고서 작성 후 단계 3 진입.

---

## 1. 회귀 베이스라인 수집

수정 전 SVG 출력을 `/tmp/baseline_495/` 에 보존. 페이지별 SHA-256 체크섬을 `checksums.txt` 에 기록 (총 64 SVG).

| 샘플 | 페이지 수 | 비고 |
|---|---|---|
| `samples/exam_science.hwp` (페이지 2 만) | 1 | 결함 재현 페이지 |
| `samples/synam-001.hwp` | 35 | 셀 안 컨트롤 다수 |
| `samples/k-water-rfp.hwp` | 28 | 분할 표/그림 |

총 64 SVG. 단계 4 회귀 검증 시 동일 명령으로 재생성 후 `diff` 또는 체크섬 비교.

## 2. git 이력 조사

`git blame` 결과 `src/renderer/layout/table_layout.rs:1568~1647` Shape 인라인 분기는 **initial commit (`f0f7f1a`, 2026-03-27 v0.5.0)** 에 도입. 별도 task 번호나 도입 사유 주석 없음.

→ 이력으로부터 분기 도입 의도 확인 불가. 코드 동작과 회귀 위험 분석으로 결정.

## 3. layout_composed_paragraph vs Shape 분기 발행 영역 비교

### 3.1 paragraph p[1] 의 paragraph_layout 발행

단계 1 진단 결과:
- ls[0] (첫 줄): 18 글자 "◦ 분자당...모두" → SVG y=219.01 (x=97.07~398.83, ~17px 간격)
- ls[1] (두번째 줄): 사각형(자리 비움) + " 이다." → SVG y=240.48 ("이다." 글자만)

→ **paragraph_layout 은 paragraph 의 모든 줄 텍스트를 발행하며, 사각형 자리는 비운다.**

### 3.2 Shape 분기의 text_before 발행

같은 paragraph p[1] 처리 시:
- ctrl[1] 사각형 (tac_pos = ls[1] 시작 위치)
- `composed.lines.first()` = ls[0]
- prev_tac_text_pos = 12 (ctrl[0] 수식 자리), tac_pos = ~19
- 추출되는 text_before = ls[0] 의 글자 [12, ~)
- 이 텍스트가 사각형 옆에 별도 TextRun 으로 발행 → SVG y=224.26 중복

→ **Shape 분기는 paragraph_layout 이 이미 발행한 ls[0] 텍스트를 다시 발행. 모든 케이스에서 중복.**

## 4. 사각형 자체 위치 — 별도 결함 발견

베이스라인 SVG 의 사각형(rect):
```
<rect x="465.07" y="206.75" width="62.99" height="22.88" fill="#fff" stroke="#000"/>
```

- 사각형은 ls[1] (두번째 줄, "[사각형] 이다.") 위치에 그려져야 함 (PDF 기준 y≈228)
- 그러나 baseline SVG 는 y=206.75 (= ls[0] 첫 줄 시작 y) 에 그려짐
- x=465.07 = inline_x 누적값 = 첫 줄 텍스트 폭 누적 후 위치

→ Shape 분기 자체가 **multi-line paragraph 처리를 못함**. 사각형 위치도 ls[0] 끝 자리(가짜 줄)로 잘못 계산.

이는 **별도 결함**. 본 task #495 범위(텍스트 중복) 와 분리 — 후속 처리 (별도 이슈 검토 후).

## 5. 수정 방향 (A)/(B)/(C) 결정

### (A) Picture 정합 — target_line 산출 + 줄별 처리

**채택 근거**:
- Picture 분기 (라인 1483~1530) 와 동일 패턴
- multi-line paragraph 의 인라인 컨트롤 처리 룰 정합
- 사각형 자체 위치 결함도 함께 해결 (target_line 변경 시 inline_x/y 리셋)

**위험**:
- 변경 범위 큼. 단일 줄 paragraph 케이스 회귀 가능
- Picture 분기 코드를 그대로 가져오면 Shape 의 text_before TextRun 발행 부분 처리 미정 — text_before 발행 자체가 단계 3.1, 3.2 분석 결과 항상 중복이므로 제거 가능

### (B) text_before TextRun 발행 제거 — inline_x 누적만 유지

**채택 근거**:
- 단계 3.1 결과 paragraph_layout 이 모든 줄 텍스트를 항상 발행. text_before TextRun 은 모든 케이스에서 중복.
- 코드 단순화

**위험**:
- inline_x 누적 자체에 의존하는 사각형 위치 계산이 잘못된 경우 (단계 4 결함) 위치 결함이 그대로 남음

### (C) 가드 추가 — tac_pos 가 첫 줄일 때만 발행

**채택 근거**:
- 변경 최소 (한 if 추가)
- 결함 케이스 (ls[1]+ 사각형) 만 차단

**단점**:
- 단일 줄 + 사각형이 첫 줄에 있는 케이스에서도 중복 발행 유지 (단지 같은 위치라 시각적으로 안 보일 수 있음 — 폭 계산이 다르면 보임)
- 사각형 자체 위치 결함 미해결

---

### 최종 결정: **후보 (B) text_before TextRun 발행 제거**

근거:
1. 단계 3.1 검증으로 paragraph_layout 이 paragraph 모든 줄을 발행한다는 것이 확인됨 → text_before TextRun 은 항상 중복.
2. (C) 가드는 단일 줄 케이스에서도 중복 발행을 유지하여 같은 결함이 다른 형태(같은 위치 두 번 그림)로 잠복할 수 있음.
3. (A) Picture 정합은 변경 범위 큼. 사각형 위치 결함까지 함께 다루려면 task 범위 확대. **본 task 는 텍스트 중복 한정**으로 명시.
4. inline_x 누적은 유지 (사각형 위치 계산에 사용). 단지 TextRun 발행만 차단. 사각형 위치 결함은 별도 task 로 분리.

### 수정 코드 형태 (단계 3 에서 적용)

```rust
// src/renderer/layout/table_layout.rs:1573~1635 부근

if let Some(&(tac_pos, _, _)) = composed.tac_controls.iter().find(|&&(_, _, ci)| ci == ctrl_idx) {
    // 이 Shape 앞에 아직 inline_x에 반영되지 않은 텍스트 폭 계산 (inline_x 누적용)
    let text_before: String = composed.lines.first()
        .map(|line| {
            let mut chars_so_far = 0usize;
            let mut result = String::new();
            for run in &line.runs {
                for ch in run.text.chars() {
                    if chars_so_far >= prev_tac_text_pos && chars_so_far < tac_pos {
                        result.push(ch);
                    }
                    chars_so_far += 1;
                }
            }
            result
        })
        .unwrap_or_default();
    if !text_before.is_empty() {
        let char_style_id = composed.lines.first()
            .and_then(|l| l.runs.first())
            .map(|r| r.char_style_id).unwrap_or(0);
        let lang_index = composed.lines.first()
            .and_then(|l| l.runs.first())
            .map(|r| r.lang_index).unwrap_or(0);
        let ts = resolved_to_text_style(styles, char_style_id, lang_index);
        let text_w = estimate_text_width(&text_before, &ts);
        // [Task #495] paragraph_layout 이 이미 paragraph 의 모든 줄 텍스트를 발행하므로
        // 여기서 text_before 를 별도 TextRun 으로 발행하면 중복. inline_x 누적만 수행.
        // (사각형 자체 위치는 multi-line paragraph 에서 inline_x 가 첫 줄 기준이라 별도 결함이며 본 task 범위 밖)
        inline_x += text_w;
    }
    prev_tac_text_pos = tac_pos;
}
```

(원래 1612~1635 의 TextRun 생성/push 블록 + `let text_baseline`, `let font_line_h`, `let adjacent_shape_h` 계산 블록을 제거)

## 6. 단계 3 작업 목록

1. table_layout.rs:1612~1635 의 TextRun 생성/push 블록 제거 + 라인 1599~1611 의 baseline/adjacent_shape_h 계산 블록 제거 (TextRun 에서만 쓰임)
2. inline_x 누적 (`inline_x += text_w`) 와 prev_tac_text_pos 갱신은 유지
3. `cargo build --release` 통과
4. exam_science.hwp 페이지 2 SVG 재생성 후 검증:
   - cell-clip-21 영역에서 y=224.26 라인 사라짐
   - paragraph p[1] 의 layout_composed_paragraph 발행 (y=219.01, y=240.48) 만 남음

## 7. 단계 3 즉시 진입

자동승인 정책에 따라 단계 3 (코드 수정 + 결함 검증) 으로 진입합니다.
