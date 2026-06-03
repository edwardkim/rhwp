# Task #312 구현 계획서

상위: 수행계획서 `task_m100_312.md`, Epic #309
브랜치: `task312`

## 단계 구성

3단계로 분할. 각 단계 독립 커밋 + `_stage{N}.md` 보고서.

---

## 1단계 — 단별 used_height 측정 도구

### 목표
`dump-pages` 출력에 각 단의 누적 사용 높이(`current_height` 종료 시점)와 HWP가 의도한 단 사용 높이를 함께 노출. **동작 변경 없음 (회귀 0)**.

### 변경 파일
- `src/renderer/pagination.rs::ColumnContent` — `used_height: f64` 필드 추가
- `src/renderer/pagination/state.rs::flush_column` / `flush_column_always` — `current_height` 저장
- `src/document_core/queries/rendering.rs::dump_page_items` — 단 라인에 `used=...` 출력 + HWP intended 추출 (마지막 항목의 마지막 LINE_SEG vpos + line_height)

### 출력 예
```
페이지 7 (global_idx=6, section=0, page_num=7)
  body_area: x=117.2 y=209.8 w=888.2 h=1226.4
  단 0 (items=13, used=1208.7px, hwp_used≈1196.0px, diff=+12.7px)
    ...
```

`hwp_used` 계산: 단의 마지막 PageItem이 가리키는 paragraph의 last LINE_SEG vpos + line_height (HWPUNIT → px). vpos-reset에 걸린 PartialParagraph는 reset 직전 줄 기준.

### 검증
- `cargo test`: 992 passed
- 4샘플 페이지 수 무변화

### 산출
- 코드 + `mydocs/working/task_m100_312_stage1.md`

---

## 2단계 — 차이 origin 식별 (조사 단계)

### 목표
1단계 도구로 21_언어 페이지 7 단 0의 `diff` 발생 위치(어떤 paragraph에서 누적)를 식별하고 코드 후보를 좁힌다. **코드 변경 없음** (조사 only).

### 작업
1. 페이지 7 단 0 각 paragraph 처리 후 our `current_height` 추적 (디버그 println 또는 도구 확장)
2. HWP의 LINE_SEG vpos 진행과 비교 — 누적 차이가 어떤 시점에서 +로 벌어지는지 식별
3. 후보 (engine.rs 라인 참조):
   - **trailing_ls 보정** (engine.rs:602~620): trailing line_spacing 제외 비율
   - **spacing-after** 마지막 처리
   - **MeasuredParagraph.line_heights vs LINE_SEG.line_height**: 폰트 메트릭 재계산 차이
   - **0.5px 부동소수점 마진** (engine.rs:620)
   - **inline control 후 처리** (표/Shape 동반 paragraphs)
4. 정량 평가 표 작성: 후보별 차이 기여 px

### 산출
- `mydocs/tech/column_fit_origin_analysis.md`
- `mydocs/working/task_m100_312_stage2.md`
- 코드 변경 없음

---

## 3단계 — 보정 + 4샘플 검증

### 목표
2단계 식별된 origin 중 큰 기여자를 보정. 21_언어 + `--respect-vpos-reset` 결합으로 PDF 일치 (15쪽) 시도.

### 변경 후보 (2단계 결과에 따라 결정)
- engine.rs trailing_ls 보정 로직 수정
- MeasuredParagraph 의 line_height 정의 검토
- spacing-after 처리

### 검증 매트릭스
| 모드 | 21_언어 | exam_math | exam_kor | exam_eng |
|------|--------|-----------|----------|----------|
| OFF (기본) | ?쪽 | 20쪽 ✓ | 25쪽 | 11쪽 ✓ |
| ON (`--respect-vpos-reset`) | **15쪽** 목표 | 20쪽 ✓ | 25쪽 | 11쪽 ✓ |

다른 3샘플 회귀 0이 필수. 21_언어 OFF 모드도 가능하면 감소.

### 종료 조건
- A) 21_언어 + ON 결합으로 15쪽 → Epic #309 클로즈 평가
- B) 부분 개선 + 잔존 차이 정량화 → 추가 sub-issue 등록

### 산출
- 코드 + `mydocs/working/task_m100_312_stage3.md`
- 최종 보고서 `mydocs/report/task_m100_312_report.md`
- Epic #309 코멘트

## 회귀 검증 명령

```bash
for f in samples/{21_언어_기출_편집가능본,exam_math,exam_kor,exam_eng}.hwp; do
  off=$(cargo run --bin rhwp -q -- dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  on=$(cargo run --bin rhwp -q -- dump-pages "$f" --respect-vpos-reset 2>/dev/null | grep -c "^=== 페이지")
  echo "$(basename $f): OFF=$off ON=$on"
done
```

## 위험

- **2단계에서 origin 단일 식별이 안 됨**: 여러 미세 요인의 누적 → 부분 보정으로 끝내고 추가 sub-issue 등록
- **3단계 보정이 다른 샘플 회귀 유발**: 매 변경마다 4샘플 검증, 회귀 발생 시 즉시 롤백 후 재검토
- **PDF 일치 미달성**: 완료 조건 B로 처리, Epic 후속 작업 등록

## 승인 요청

위 분할로 진행. 승인 시 1단계 시작.
