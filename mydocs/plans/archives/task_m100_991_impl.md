# Task #991 구현 계획서 — F2 composer-only marker synthesis

- 이슈: [#991](https://github.com/edwardkim/rhwp/issues/991)
- 선행: [수행 계획서](task_m100_991.md), [Stage 1 진단](../working/task_m100_991_stage1.md)
- 브랜치: `local/task991-fix`

## 1. 결정된 fix 후보: F2-narrow

Stage 1 진단 결과로 후보 평가:

| 후보 | cargo test fail | Editor 영향 | 회귀 |
|------|---------------|------------|------|
| F1 (parser 광범위) | 9 | **있음** | 큼 |
| F1-narrow (ch=11/14) | 5 | 적음 | 중 |
| F2-wide (composer synth, 조건 무) | 5 | 없음 | 중 |
| **F2-narrow** (좁힘 조건) | **0** | 없음 | **없음** |

→ **F2-narrow 채택**.

## 2. 구현 위치 + 시그니처

```rust
// src/renderer/composer.rs

/// HWP5 parser 가 누락한 \u{FFFC} 인라인 마커를 합성하여
/// composer 내부에서만 사용하는 paragraph 반환.
fn synthesize_marker_paragraph(para: &Paragraph) -> Option<Paragraph>;

pub fn compose_paragraph(para: &Paragraph) -> ComposedParagraph {
    let synth = synthesize_marker_paragraph(para);
    let para = synth.as_ref().unwrap_or(para);  // shadow
    // 기존 logic 유지
}
```

## 3. F2 합성 알고리즘

### 좁힘 조건 (모두 만족 시만 합성)
1. `inline_ctrl_count >= 3` — pi=394 패턴 (3 TAC controls)
2. `n_leading >= 2` — leading char_offsets gap 에 2+ extended ctrl
3. `existing_markers < inline_ctrl_count` — HWP3 (markers 있음) 자동 차단

### 합성 로직
1. inline-visible extended ctrl 수 계산 (Header/Footer/Footnote/Endnote/HiddenComment 제외)
2. char_offsets gap (8 wchar 단위) 분석:
   - Leading gap (char_offsets[0] / 8) → leading 마커 push
   - Inter-char gap → 사이 마커 push
   - Trailing → 남은 controls 8 wchar 단위로 push
3. 새 (text, char_offsets) 로 clone paragraph 반환

## 4. 영향 분석

### Editor pipeline
- 영향 없음 — parser 미변경 (para.text 원본 유지)
- insert_text / save / cursor / logical_offset 보존

### Renderer pipeline
- compose_paragraph 진입 시 synth → ComposedParagraph 에 marker 반영
- layout 의 utf16_range_to_text_range 가 마커 포함 text 기준 line 매핑

## 5. 단계 구성 (구현 계획)

| Stage | 내용 | 산출물 |
|-------|------|--------|
| 2 | F2-narrow 구현 | composer.rs +95 lines |
| 3 | cargo test --release --lib | 1297 passed, 0 failed (목표) |
| 4 | 240 sample 페이지 수 회귀 측정 | 변동 0 (목표) |
| 5 | HWP5 sample16 p18 시각 검증 | PDF 정합 (목표) |
| 6 | commit + 보고서 + PR 준비 | PR 생성 |

## 6. 회귀 방어

- 좁힘 조건 (3개 모두) 으로 pi=394 패턴만 catch
- 1-2 TAC paragraph (대부분 sample) 미적용
- HWP3 (markers 있음) 자동 차단
- cargo test 통과 + 240 sample 회귀 0 검증
