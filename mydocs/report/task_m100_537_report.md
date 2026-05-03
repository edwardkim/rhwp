# Task #537 최종 보고서

**제목**: 21_언어_기출.hwp TAC 표 직후 첫 문단의 trailing line_spacing 누락으로 줄간격 좁아짐
**마일스톤**: M100 (v1.0.0)
**브랜치**: `local/task537`
**이슈**: https://github.com/edwardkim/rhwp/issues/537

---

## 1. 문제 요약

`samples/21_언어_기출_편집가능본.hwp` 의 다수 위치에서 답안 ①과 ② 사이 줄간격(또는 박스 직후 첫 문장과 다음 문장 사이) 이 IR `LINE_SEG.vpos` 보다 정확히 716 HU(= 9.55 px = 1 line_spacing) 좁게 렌더링됨.

작업지시자 보고 위치 11곳:
- P2 q3, P3 q6, P5 q9, P6 q12, P8 q15, P9 q17/18, P12 q23/24, P13 q27, P14 q29
- (모두 TAC `<보기>` 표 직후 첫 답안의 ①→② gap)

## 2. 근본 원인

세 메커니즘의 상호작용으로 716 HU drift 가 column 의 lazy_base 에 영구화:

1. **`prev_tac_seg_applied` 가드** (`layout.rs:1434`): TAC 표 직후 paragraph 의 vpos 보정 건너뜀.
2. **trailing line_spacing 제외** (`paragraph_layout.rs:2645-2654`, Task #479): paragraph 마지막 줄에서 `ls` 가산 생략 → sequential y_offset 이 IR vpos 보다 1 ls 부족.
3. **lazy_base 누적 drift 동결** (`layout.rs:1497-1507`): pi=39 부터 vpos 보정 재개되지만 lazy_base 를 sequential y 에서 역산하므로 drift 716 HU 가 base=716 으로 박힘.

결과: ① 만 IR 정확, ②~⑤ 는 IR_vpos − 716 HU → ①→② gap 만 좁아 보임.

## 3. 수정 내용

### 3.1 코드 변경 (`src/renderer/layout.rs:1494-1521`, A'안)

```rust
// [Task #537] trailing-ls 보정:
// paragraph_layout 의 마지막 줄은 trailing line_spacing 을
// 제외하여 y 를 advance 한다 (Task #479, lh_sum + (n-1)*ls 정책).
// 그 결과 sequential y_offset 은 IR vpos 누적보다
// prev_pi 의 last seg ls 만큼 부족해진다.
// 이 부족분을 y_delta_hu 에 더해야 lazy_base 가
// IR 절대 좌표와 일치한다 (drift 가 base 에 동결되는 것을 방지).
let trailing_ls_hu = paragraphs.get(prev_pi)
    .and_then(|p| p.line_segs.last())
    .map(|s| s.line_spacing.max(0))
    .unwrap_or(0);
let y_delta_hu = ((y_offset - col_area.y) / self.dpi * 7200.0).round() as i32
    + trailing_ls_hu;
let lazy_base = prev_vpos_end - y_delta_hu;
```

총 변경: +14 / -2 LOC.

### 3.2 단위 테스트 (`integration_tests.rs`)

`test_537_first_answer_after_tac_table_line_spacing`:
- 페이지 2 SVG 에서 ① ② ③ baseline y 추출
- gap(①→②) ≈ gap(②→③) ≈ IR vpos delta (5448 HU = 72.64 px) 검증
- TDD: Stage 1 에서 실패 → Stage 2 적용 후 통과

## 4. 검증 결과

### 4.1 자동 테스트
```
test result: ok. 1117 passed; 0 failed; 1 ignored
```
신규 테스트 1건 포함. 기존 테스트 0건 회귀.

### 4.2 작업지시자 명시 11곳 정량 (수정 전/후)

| 페이지 | 문제 | 라인수 | 수정 전 (px) | 수정 후 (px) | IR (px) |
|--------|------|--------|-------------|-------------|---------|
| P2 | 3번 | 3 | 63.09 | **72.64** ✓ | 72.64 |
| P3 | 6번 | 3 | 63.09 | **72.64** ✓ | 72.64 |
| P5 | 9번 | 1+2 | 14.67 | **24.21** ✓ | 24.21 |
| P6 | 12번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P8 | 15번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P9 | 17번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P9 | 18번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P12 | 23번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P12 | 24번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P13 | 27번 | 2 | 38.88 | **48.43** ✓ | 48.43 |
| P14 | 29번 | 2 | 38.88 | **48.43** ✓ | 48.43 |

**11곳 모두 IR vpos delta 와 정확히 일치**.

### 4.3 광범위 회귀 검증 (7개 샘플)

| 샘플 | 변경 페이지 | 검증 |
|------|-----------|------|
| `21_언어_기출_편집가능본.hwp` | (대상) | ✅ 11곳 정상화 + 다른 페이지도 IR 정합 개선 |
| `synam-001.hwp` | 16/35 | ⚠️ 시프트 미세, 한컴 PDF 시각 비교 권고 |
| `복학원서.hwp` | 1/1 | ✅ 좌표 변화 없음 |
| `exam_math.hwp` | 18/20 | ⚠️ 수식 직후 일부 paragraph -14.67 px 시프트, 시각 비교 필요 |
| `exam_kor.hwp` | 18/20 | ✅ +9.17 px 일관 시프트, IR 정합 검증 (pi=7 ① at y=1205.19 = IR 기대) |
| `exam_eng.hwp` | 6/8 | ✅ +7.68 ~ +14.24 시프트 |
| `exam_science.hwp` | 5/6 | ✅ +4.14 ~ +13.07 시프트 |
| `2010-01-06.hwp` | 6/6 | ✅ +19.09 (= 2 ls) 시프트 |

대부분 paragraph 가 **누적 drift 만큼 IR-정확 위치로 하향 보정** = 정합성 개선.

## 5. 메모리 룰 적용

- **본질 정정 회귀 위험**: lazy_base 는 다단·표분할·셀 paragraph 와 상호작용. 광범위 샘플 검증 결과 본 task 직접 대상은 모두 정상화, 일부 수식/Shape 인접 위치 추가 검토 필요.
- **PDF 비교 결과는 절대 기준이 아님**: 한컴 2010/2020 + 한컴독스 PDF 200dpi 시각 비교는 작업지시자 검토에 위임.
- **룰과 휴리스틱 구분**: trailing-ls 보정은 HWP IR vpos 의 정확한 보정이며, 분기 조건/허용오차 없는 단일 룰로 적용.

## 6. 잔존 / 후속 사항

### 6.1 잔존 base=716 케이스
21_언어_기출 의 pi=147 등 일부 위치에 base=716 잔존. 수정 전에도 동일 → 회귀 아님. 별도 메커니즘(prev_seg vs line_segs.last() 차이 등) 추정. **별도 issue 검토 후보**.

### 6.2 exam_math 음의 시프트
수식(Shape) 직후 paragraph 의 -14.67 px 시프트. 시각 비교에서 회귀 확인 시 가드 조건 추가(prev_pi 가 FullParagraph PageItem 인 경우만 보정 적용) → `local/task537_v2` 브랜치로 후속 수정.

### 6.3 Clippy 기존 결함 (본 task 외)
```
src/document_core/commands/table_ops.rs:1007
src/document_core/commands/object_ops.rs:298
```
별도 issue 권장.

## 7. 산출물 일람

| 파일 | 종류 | 변경 |
|------|-----|------|
| `src/renderer/layout.rs` | 코드 | +14 / -2 (lazy_base trailing-ls 보정) |
| `src/renderer/layout/integration_tests.rs` | 테스트 | +76 (TDD 통합 테스트) |
| `mydocs/plans/task_m100_537.md` | 수행계획서 | 신규 |
| `mydocs/plans/task_m100_537_impl.md` | 구현계획서 | 신규 |
| `mydocs/working/task_m100_537_stage1.md` | Stage 1 보고서 | 신규 |
| `mydocs/working/task_m100_537_stage2.md` | Stage 2 보고서 | 신규 |
| `mydocs/working/task_m100_537_stage3.md` | Stage 3 보고서 | 신규 |
| `mydocs/report/task_m100_537_report.md` | 본 최종 보고서 | 신규 |

## 8. 커밋

| 커밋 | Stage | 내용 |
|------|-------|------|
| `226b6446` | Stage 1 | TDD 테스트 + 수행/구현 계획서 |
| `1803bc62` | Stage 2 | layout.rs lazy_base trailing-ls 보정 |
| (Stage 3) | Stage 3 | 광범위 회귀 검증 보고서 + 최종 보고서 |

## 9. 작업지시자 결정 요청

1. **본 task 직접 대상 11곳 — IR 정확 정상화 (검증 완료)**: merge 진행
2. **광범위 시프트** (exam_kor/eng/science 등): IR 정합 개선이 확인되었으나 한컴 2010/2020 PDF 시각 비교를 통한 최종 확인 권고
3. **exam_math 음의 시프트**: 시각 비교 후 회귀 시 `local/task537_v2` 후속 수정
4. **잔존 base=716, Clippy 기존 결함**: 별도 issue 등록 여부

→ 시각 비교는 작업지시자가 직접 진행 후 merge 결정.
