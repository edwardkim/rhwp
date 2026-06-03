# Task #495 단계 3 v2 — 옵션 B (가드 추가) 재시도

**이슈**: #495
**브랜치**: `local/task495`
**전제**: 단계 3 (`task_m100_495_stage3.md`) 에서 옵션 B 후보 (B) text_before 발행 제거가 synam-001 p31 회귀 발생 → 작업지시자 결정으로 옵션 B (가드 추가) 채택

---

## 1. 사전 검증 — synam-001 p31 paragraph 구조

옵션 B 가드(multi-line + 사각형 ls[1]+ 차단) 가 synam-001 회귀 케이스에 영향 없는지 사전 검증.

`rhwp dump samples/synam-001.hwp -s 0 -p 237` 결과 페이지 31 의 5×3 표 안 셀[5] 의 paragraph p[16]:

```
p[16] ps_id=339 ctrls=8 text_len=11 ls[0] vpos=64292 lh=2092 ls=500
```

- `text_len=11` ("년 월 일" 등 11 글자)
- `ctrls=8` (8개 인라인 사각형)
- **`line_seg 1개 (ls[0])` — 단일 줄 paragraph**

가드 조건은 `composed.lines.first()` 에 사각형 `tac_pos` 가 속하지 않을 때만 발동 → **단일 줄 paragraph 는 항상 통과**. 즉 synam-001 p31 영향 없음 확정.

## 2. 코드 수정

`src/renderer/layout/table_layout.rs:1573~1589` 의 text_before 추출 직전에 가드 추가.

```rust
if let Some(&(tac_pos, _, _)) = composed.tac_controls.iter().find(|&&(_, _, ci)| ci == ctrl_idx) {
    // [Task #495] 가드: 사각형이 paragraph 첫 줄(ls[0]) 범위 안에 있을 때만
    // text_before 추출/발행. multi-line paragraph 에서 사각형이 ls[1]+ 에
    // 있는 경우 composed.lines.first() 만 보던 기존 코드는 첫 줄 전체
    // 텍스트를 잘못 추출해 paragraph_layout 결과와 중복 발행했음.
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
    if !text_before.is_empty() { /* 기존 TextRun 발행 */ }
    prev_tac_text_pos = tac_pos;
}
```

변경 통계: `+27 -13`. 추출/발행 로직 자체는 보존, 가드만 추가.

## 3. 결함 검증

`samples/exam_science.hwp` 페이지 2 SVG 재생성:

```bash
grep -c "translate([0-9.]*,224.26" output/svg/exam_science_p1/exam_science_002.svg
→ 0
```

→ **y=224.26 중복 라인 사라짐** (수정 전 18 글자 발행, 수정 후 0). 결함 해결.

## 4. 회귀 검증 (광범위)

베이스라인(`/tmp/baseline_495`) vs 수정 후(`/tmp/optb_check`) 의 64 SVG 체크섬 비교:

```
변경된 파일:
  exam_science_p1/exam_science_002.svg  (의도된 변경 — 결함 수정)
```

**변경 1건만**. synam-001 (35 페이지) 모두 동일, k-water-rfp (28 페이지) 모두 동일. **회귀 0건**.

## 5. 가드 동작 정합성

| paragraph 구조 | 가드 동작 | 동작 |
|---|---|---|
| 단일 줄 + 사각형이 첫 줄 안 (synam-001 p[16]) | `in_first_line=true` | 기존 동작 (text_before 발행) ✓ |
| Multi-line + 사각형이 ls[0] 안 | `in_first_line=true` | 기존 동작 (정상 케이스) ✓ |
| Multi-line + 사각형이 ls[1]+ (exam_science p[1]) | `in_first_line=false` | text_before 발행 차단 (결함 수정) ✓ |

→ 결함 케이스만 핀포인트 차단, 다른 모든 케이스는 기존 동작 보존.

## 6. 시각 검증

`exam_science.hwp` 페이지 2 PNG 비교 (`/tmp/exam_p2/svg_p2_hi.png` vs `/tmp/exam_p2/svg_after_fix.png`):

- 수정 전: 박스 안 텍스트가 같은 위치에 두 번 그려져 진하게/일그러져 보임
- 수정 후: 박스 안 텍스트 정상 1회만, PDF 정답과 정합

## 7. 단계 4 진입

다음 작업:
- `cargo test --release` 전체 통과 확인 (이미 검증 완료 — 모두 ok)
- `cargo clippy` 검사 (이미 검증 완료 — 본 task 외 기존 이슈 2건 식별)
- 최종 보고서 작성
- orders 갱신
- 커밋

자동승인 정책에 따라 단계 4 즉시 진입.
