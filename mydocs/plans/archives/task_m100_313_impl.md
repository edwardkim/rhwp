# Task #313 구현 계획서

상위: 수행계획서 `task_m100_313.md`, Epic #309
브랜치: `task313`

## 단계 구성

5단계로 분할. 각 단계 독립 커밋 + `_stage{N}.md` 보고서.

### 1단계 — TypesetEngine 호환성 검토 (조사, 코드 변경 없음)

목표: Paginator 출력 vs TypesetEngine 출력 필드 비교 매트릭스 작성. 누락/차이점 식별.

작업:
- `PaginationResult` 최상위 필드 비교: `pages`, `wrap_around_paras`, `hidden_empty_paras`
- `PageContent` 필드 비교: `column_contents`, `active_header`, `active_footer`, `page_number_pos`, `page_hide`, `footnotes`
- `ColumnContent` 필드 비교
- 4샘플로 실제 typeset 결과를 inspect (SVG 임시 생성, 비교)

산출:
- `mydocs/tech/typeset_compatibility_matrix.md`
- `mydocs/working/task_m100_313_stage1.md`

### 2단계 — 누락 필드 보완

목표: 1단계 매트릭스에서 식별된 미채움 필드 보완. (보완할 필드가 없으면 skip)

산출:
- 코드 수정 (typeset.rs)
- `mydocs/working/task_m100_313_stage2.md`

### 3단계 — 진입점 전환 + 4샘플 페이지 수 검증

변경: `src/document_core/queries/rendering.rs::paginate()`
```rust
let result = typesetter.typeset_section(...);
```

검증:
- 21_언어: 15쪽
- exam_math: 20쪽
- exam_kor: 24쪽
- exam_eng: 9쪽
- `cargo test` 통과

산출:
- 코드 + `mydocs/working/task_m100_313_stage3.md`

### 4단계 — 시각 회귀 검증

작업:
- 4샘플 export-svg 생성 (전환 전/후)
- 페이지 수 변경에 따른 자연 변화 외 비정상 (텍스트 깨짐/머리말 누락 등) 식별
- E2E 테스트 (rhwp-studio) 통과 확인 (시간 허용 시)

산출:
- `mydocs/working/task_m100_313_stage4.md`
- (필요 시) 회귀 수정 코드

### 5단계 — 부속물 정리 + 최종 보고서

작업:
- `--respect-vpos-reset` 실험 플래그: 보존 / 제거 결정
- TYPESET_VERIFY 검증 코드 제거 (rendering.rs:837~)
- Paginator 코드: fallback 보존 / 제거
- 최종 보고서 + 오늘할일 갱신 + Epic #309 코멘트

산출:
- `mydocs/working/task_m100_313_stage5.md`
- `mydocs/report/task_m100_313_report.md`

## 회귀 검증 명령

```bash
for f in samples/{21_언어_기출_편집가능본,exam_math,exam_kor,exam_eng}.hwp; do
  pages=$(cargo run --bin rhwp -q -- dump-pages "$f" 2>/dev/null | grep -c "^=== 페이지")
  echo "$(basename $f): $pages 쪽"
done
```

기대값 (전환 후): 15 / 20 / 24 / 9

## 위험 / 롤백

각 단계 회귀 발생 시:
- 페이지 수 차이 → 누락 필드 추가 후 재시도
- SVG 시각 회귀 → 문제 식별 후 typeset 코드 수정
- 다중 회귀 → 본 sub-issue 종료, 잔존 문제로 추가 sub-issue 등록

## 승인 요청

위 분할 승인 시 1단계 시작.
