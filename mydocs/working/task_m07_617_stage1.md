# Task #617 Stage 1·2·3 통합 완료 보고서

## 진행 요약

당초 계획서의 Stage 1 (테스트), Stage 2 (line_segs guard), Stage 3 (임계 + min_pad)
세 단계를 한 작업 사이클에 통합 진행했다. 이유:

- Stage 2 의 line_segs-기반 guard 가 단독으로 보고된 3 개 보기 박스를
  복원하면서 기존 단일 줄 좁은 셀 (table-text, form-002 골든) 회귀를
  발생시키는 사실이 검증 과정에서 즉시 드러났다.
- Stage 3 의 임계 완화·min_pad 30% 하한도 동일 골든을 깨뜨려, 시각적
  관점에서 옳은 동작과 어긋남이 확인됨.
- 결과적으로 가장 좁은 변경 (다중 줄 단락이 있는 셀에서만 shrink skip)
  으로 수렴. 이후 Stage 3 의 임계/하한 변경은 모두 롤백.

## 최종 변경

`src/renderer/layout/table_layout.rs::shrink_cell_padding_for_overflow`
선두에 다음 가드 추가:

```rust
let any_multiline_distributed = paragraphs.iter()
    .any(|p| p.line_segs.len() >= 2);
if any_multiline_distributed {
    return (pad_left, pad_right);
}
```

함수 시그니처에 `paragraphs: &[Paragraph]` 인자를 추가하고, 4개 호출처
(table_cell_content.rs, table_partial.rs, table_layout.rs ×2) 에 `&cell.paragraphs`
전달.

기존 단일 줄 셀에 대한 휴리스틱 (1.15× 임계, 1 px min_pad, 비례 축소) 은
종전 그대로 유지.

## 회귀 테스트

신규 SVG snapshot 추가:

- `tests/svg_snapshot.rs::issue_617_exam_kor_page5` — 6 페이지 16번 보기 박스
- 골든: `tests/golden_svg/issue-617/exam-kor-page5.svg`

기존 골든은 모두 그대로 통과 (변경 0):

- `form_002_page_0` ✓
- `table_text_page_0` ✓
- `issue_157_page_1` ✓
- `issue_267_ktx_toc_page` ✓
- `issue_147_aift_page3` ✓
- `render_is_deterministic_within_process` ✓

전체 `cargo test --release`: 1134 통과 + 다중 모듈 통과.

## 시각 검증

3 개 보고 박스 좌·우 padding 복원 확인:

- 페이지 6 16번 보기 박스 — 본문 "사용한다, 박사 학위..." 좌측 테두리에서 약 3 mm 떨어짐
- 페이지 9 27번 보기 박스 — 본문 "쓰지만 결국..." 좌·우 모두 정상 간격
- 페이지 17 36번 자료 박스 — 본문 "◦ 뎌녁..." 좌측 정상 간격

cropped 이미지 4 종 (`/tmp/v2_p6_crop.png`, `/tmp/final_p9_27.png`,
`/tmp/final_p17_36.png` 등) 으로 확인.

## 영향 범위

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | shrink 함수 가드 + 시그니처, 함수 docs |
| `src/renderer/layout/table_cell_content.rs` | 호출 인자 |
| `src/renderer/layout/table_partial.rs` | 호출 인자 |
| `tests/svg_snapshot.rs` | 신규 회귀 테스트 |
| `tests/golden_svg/issue-617/exam-kor-page5.svg` | 신규 골든 |

## 위험 평가

- **다중 줄 셀에서 line_segs 가 잘못 채워진 비정상 IR**: shrink 우회로
  본문이 셀 밖으로 흘러나올 가능성. 보고된 샘플에 해당 케이스 없음.
  발견 시 Stage 2 가드를 segment_width vs inner_w 비교로 강화 가능.
- **단일 줄 좁은 셀**: 종전 동작 보존 → 회귀 위험 0.

## 잔여 단계

- Stage 4 — 회귀 검증·최종 보고: 진행 후 `task_m07_617_report.md` 작성.
