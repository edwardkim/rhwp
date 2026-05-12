# 최종 결과보고서 — Task #849 (M100)

제목: 다단 — 마지막 단 단나누기의 "같은-페이지-새-밴드" 는 "배분(Distribute)" 단에서만 (신문형 단은 새 페이지)
GitHub Issue: edwardkim/rhwp#849 · 브랜치: `local/task849` (← `local/task846`)

## 1. 배경 / 동기

#846 (마지막 단 명시적 단나누기 → 같은 페이지 새 단-밴드, ≈ 닫힌 #768) 의 `start_new_column_band` 가 단 유형(`ColumnType`)을 구분하지 않고 모든 다단 zone 에 적용되어, "일반"(신문형) 단 문서에서 회귀를 일으켰다:
- `exam_math.hwp`: 20페이지(한컴 PDF 2022) → 11페이지 (문제 11·12 가 페이지 4 대신 페이지 3 으로 당겨짐)
- `21_언어_기출_편집가능본.hwp`: 페이지 8/9 콘텐츠 시프트 (`test_539`/`test_548` 실패)
- `cargo test` 3건 실패

## 2. 진단 (Stage 1)

`shortcut.hwp` 의 "보기"/"입력" 2단 zone = 단 유형 **배분(Distribute)** — 콘텐츠를 두 단에 균등 배분 → 밴드가 작음(6줄≈106px). 마지막 단 단나누기 시 한컴은 같은 페이지에 새 (배분) 밴드를 만든다.

`exam_math.hwp` 2단 zone = 단 유형 **일반(Normal/신문형)** — 단 0 을 채우고 단나누기로 단 1 로 넘어가는 방식. 마지막 단의 단나누기는 같은 페이지에 새 밴드를 만들지 **않고** 새 페이지로 간다.

rhwp 는 `current_zone_column_type` (Normal/Distribute/Parallel) 을 이미 추적 중. #846 의 `start_new_column_band` 가 이를 보지 않은 것이 원인.

(수행/구현계획서 v1 은 "다단 밴드 높이 산출 정합"을 가정했으나 진단 결과 그것이 아니라 단 유형 게이트 누락이었음 — 구현계획서 v2 로 정정.)

## 3. 수정 (Stage 2)

`src/renderer/typeset.rs` — `paginate` 의 명시적 `Column` break 경로:

```rust
let is_last_column = st.current_column + 1 >= st.col_count;
if is_last_column
    && st.col_count > 1
    && st.current_zone_column_type == ColumnType::Distribute   // [Task #849]
{
    self.start_new_column_band(&mut st, para_idx, paragraphs);
} else {
    st.advance_column_or_new_page();
}
```

- `Distribute`(배분) zone 에서만 마지막 단 단나누기 → 같은 페이지 새 밴드.
- `Normal`(신문형) zone 은 기존 `advance_column_or_new_page`(마지막 단 → 새 페이지) 유지.
- `Parallel`(평행) zone 도 현 동작 유지 (별도 의미 — 범위 밖).
- 부수: #846 에서 들어온 `start_new_column_band` 내 `find_map(all-Some)` clippy 경고를 `last().map(..)` 으로 정리.

밴드 높이 산출(`max(단별 마지막 문단 vpos_end)`)·`layout.rs` 연동 변경은 불요 — 배분 단의 밴드는 작아 현 산출로 충분.

## 4. 검증 (Stage 3)

| 샘플 | 한컴 PDF | baseline | #846 단독 | **#846+#849** |
|------|---------:|---------:|----------:|--------------:|
| `basic/shortcut.hwp` | 7 (2022) | 8 | 7 | **7** ✅ (pi=94/95 페이지 3) |
| `exam_math.hwp` | 20 | 20 | 11 | **20** ✅ (baseline 복원, PDF 정합) |
| `21_언어_기출_편집가능본.hwp` | 15(2020)/16(2010) | 15 | 15(시프트) | **15** ✅ (시프트 해소) |
| 그 외 다단 샘플 (exam_eng/kor/science/social, k-water-rfp, biz_plan, hwpspec, hwp-3.0-HWPML, aift, treatise, interview, issue-505-equations …) | — | — | — | **무변화** |

- SVG 바이트 비교(`export-svg`): `shortcut.hwp` 페이지 3~7 만 변화 + 페이지 8 제거(8→7), 그 외 다단 샘플 전 페이지 baseline 과 바이트 동일.
- `cargo test --lib`: **1232 passed; 0 failed** (`test_exam_math_page_count`/`test_539`/`test_548` 포함 — #846 단독 시 실패하던 3건 복구).
- `cargo clippy --lib`: typeset.rs 신규 경고 0.

## 5. 커밋 (`local/task849`)

| 커밋 | 내용 |
|------|------|
| `7f637856` | 수행/구현 계획서 (v1) |
| `73103f00` | Stage 1 진단 + 구현계획서 v2 |
| `a5531988` | Stage 2 — `start_new_column_band` 를 배분 단으로 한정 + clippy 정리 |
| `91f25941` | Stage 3 — 광역 회귀 검증 |
| (본 커밋) | Stage 4 — 최종 보고서 |

## 6. #846 과의 관계 / 머지 순서

`local/task849` 는 `local/task846` 위에 분기 — #849 의 효과(특히 `exam_math` 페이지 수)는 #846 의 `start_new_column_band` 가 있어야 드러나고, 두 코드가 같은 파일을 만지기 때문. 머지 순서:

```
local/task846 (closes #846)  ┐
local/task849 (closes #849)  ┘→ local/devel → devel → (릴리즈 시) main PR
```

#846 단독으로는 회귀가 있으므로 **#846 와 #849 는 함께 머지**해야 한다. (즉 `local/task846` → `local/devel`, 이어서 `local/task849` → `local/devel` 를 한 작업 단위로.) `local/devel` push 금지, PR 대상 `devel` (메모리 `feedback_pr_target_devel`).

## 7. 잔존 사항 / 후속

- `process_multicolumn_break` 의 밴드 높이 산출(`vpos_zone_height` = 직전 문단 한 개의 vpos_end)은 단별 max 가 아님 — 현 샘플들에서 문제 없으나, 향후 균등 배분이 아닌 `[다단나누기]` zone 에서 단 0 이 단 1 보다 훨씬 길 때 잠재 과소추정 여지. 발현 샘플 확인 시 별도 이슈.
- `Parallel`(평행) 단의 단나누기/쪽나누기 의미 교차 처리는 본 타스크 범위 밖 — 발현 케이스 시 별도.
