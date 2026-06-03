# Task #314 2단계 완료 보고서: HWPX normalize 적용 + 잔존 차이 식별

상위: 구현 계획서 `task_m100_314_impl.md`

## 변경

`src/document_core/commands/document.rs::from_bytes`:
- HWPX 분기에서 신규 함수 `normalize_hwpx_paragraphs` 호출
- HWPX 파서가 채우지 않는 paragraph 필드를 HWP 라운드트립 결과와 일치시킴

`normalize_hwpx_paragraphs` 처리:
1. 빈 char_shapes 에 default `[(0, 0)]` 추가 (HWP 스펙: 최소 1개 PARA_CHAR_SHAPE)
2. control_mask 를 controls + field_ranges + text 기반 재계산 (HWP 직렬화기와 동일)
3. 셀 내부 paragraph 도 재귀 처리

## 검증

### normalize 효과 (paragraph 필드 차이)

| 필드 | 1단계 (직전) | 2단계 (이후) |
|------|-------------|-------------|
| char_shapes_len | 59 | **0** ✓ |
| control_mask | 27 | **0** ✓ |
| char_count | 3 | 3 (잔존) |
| char_count_msb | 2 | 2 (잔존) |
| raw_break_type | 4 | 4 (잔존) |
| raw_header_extra | 130 | 130 (잔존) |

### 페이지 수 차이 (목표 달성 X)

```
[#178 Stage 4] hwpx-h-02: orig=9, after_adapter=10
```

normalize 후에도 +1쪽 잔존. 즉 char_shapes / control_mask 차이는 **paginate에 직접 영향 없음**. 진짜 origin은 잔존 차이 또는 더 깊은 곳.

### 회귀 검증
- `cargo test`: 전체 992+ 통과, 회귀 0
- 4샘플 (21_언어/exam_math/exam_kor/exam_eng) 페이지 수 무변화: 15/20/24/9

## 잔존 차이 분석

남은 차이 중 paginate 영향 가능성:

| 필드 | typeset 영향 가능성 |
|------|----------------------|
| `raw_break_type` 4건 | 낮음 (typeset은 column_type 사용, 동일) |
| `raw_header_extra` 130건 | 매우 낮음 (header 끝 padding) |
| `char_count` 3건 | 중간 (composer가 char_offsets/text 길이 사용) |
| `char_count_msb` 2건 | 매우 낮음 (마지막 paragraph 표시) |

**가장 의심**: char_count 3건 (paragraph 0.0, 0.34, 1.0). 0.34 는 페이지 3 단 0 내부 paragraph로 +1쪽 발생 페이지와 일치.

추가 가능성: paragraph 외 데이터 (style, doc_info, table cell paragraphs 의 다른 필드) 차이.

## 결론

본 sub-issue 완료 조건 (격리 테스트 통과) 미달성. 부분 진전:
- ✅ HWPX normalize 코드 추가 (IR 정합성 개선, 다른 잠재 회귀 방지)
- ✅ paragraph 필드 차이 86건 → 139건 정도로 감소 (char_shapes 59 + control_mask 27 = 86건 해소)
- ❌ 페이지 수 +1쪽 차이 미해결

잔존 origin 조사는 더 깊은 분석 (typeset 내부 처리 차이) 또는 직렬화/파싱 라운드트립 손실 추적 필요. 본 sub-issue 범위를 넘는 작업.

## 다음 단계 권장

3단계 작업을 다음으로 변경:
- 본 sub-issue 종료 (부분 진전 + 잔존 사안 별도 sub-issue)
- normalize 코드는 보존 (정합성 가치)
- 격리 테스트 (`#[ignore]`)는 유지
- 잔존 origin 조사 위한 새 sub-issue 등록 권장
