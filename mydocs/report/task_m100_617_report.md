# Task #617 최종 결과 보고서 — 표 셀 padding shrink 휴리스틱 오작동

## 이슈

GitHub Issue #617: `exam_kor.hwp` 의 16/27/36번 `<보기>`·`<자료>` 글상자에서
본문과 셀 테두리 사이 좌·우 padding 이 거의 0 으로 표시.

## 원인

`src/renderer/layout/table_layout.rs::shrink_cell_padding_for_overflow` 가
양쪽 정렬 한국어 본문의 자연 폭을 과대 추정하여, HWP 가 이미 가용 폭에
자간을 분배해 line_segs 로 줄바꿈을 확정한 정상 입력의 padding 까지
잘못 깎고 있었다.

`estimate_text_width` 가 음수 letter_spacing 만 0 으로 clamp 하고
CharShape `ratio=95% / spacing=-5%` 를 부분 반영해 1.15× 임계 초과로
오인. 850 HU(≈3 mm) padding 이 1 px 까지 축소.

## 수정

**핵심 가드 추가** (`shrink_cell_padding_for_overflow` 진입부):

```rust
let any_multiline_distributed = paragraphs.iter()
    .any(|p| p.line_segs.len() >= 2);
if any_multiline_distributed {
    return (pad_left, pad_right);
}
```

다중 줄(2 줄 이상) 단락이 있는 셀은 HWP 가 가용 폭에 자간을 분배해
줄바꿈을 확정한 상태이므로 padding 을 그대로 보존. 단일 줄 좁은 셀
(수치 셀 등 오버플로우 가능성 있는 케이스)은 종전 휴리스틱 유지.

**시그니처 보강**: `paragraphs: &[Paragraph]` 인자 추가.
호출처 4곳 (`table_cell_content.rs`, `table_partial.rs`, `table_layout.rs` ×2)
동시 수정.

## 시도했으나 채택하지 않은 변경

- 임계 1.15 → 1.30 완화: table-text/form-002 골든 깨짐.
- 최소 padding 30% 하한: 위 골든 시각 회귀.
- segment_width 기반 비교: 골든의 단일 줄 셀에서 HWP segment_width 가
  inner_w 이내였으나 실제 렌더에서 텍스트가 셀 외곽선을 넘어가는
  케이스 발견. 자연 폭 추정 휴리스틱이 그 케이스에서는 의도대로 작동.

→ 결국 **다중 줄 셀에서만 shrink skip** 으로 좁혀 적용.

## 검증

### SVG snapshot 회귀 테스트

신규 추가:
- `tests/svg_snapshot.rs::issue_617_exam_kor_page5` (페이지 6 16번 박스)

기존 골든 모두 통과 (변경 0):
- `form_002_page_0`, `table_text_page_0`, `issue_157_page_1`,
  `issue_267_ktx_toc_page`, `issue_147_aift_page3`,
  `render_is_deterministic_within_process`

### 시각 검증

3개 보고 박스 좌·우 padding 약 3 mm 복원:

| 페이지 | 박스 | 결과 |
|--------|------|------|
| 6 | 16번 보기 | ✓ 본문 "사용한다, 박사 학위..." 좌측 정상 간격 |
| 9 | 27번 보기 | ✓ 본문 "쓰지만 결국..." 좌·우 정상 간격 |
| 17 | 36번 자료 | ✓ 본문 "◦ 뎌녁..." 좌측 정상 간격 |

### 추가 회귀 점검

표가 많은 샘플 빠른 렌더 점검 (오류·panic 없음):
- `biz_plan.hwp`, `2022년 국립국어원 업무계획.hwp`, `exam_math.hwp`

### 전체 테스트

- `cargo test --release`: 1134+ 통과
- `cargo clippy --release --all-targets`: 신규 경고 없음
- `cargo build --release`: green

## 영향 범위

| 파일 | 변경 |
|------|------|
| `src/renderer/layout/table_layout.rs` | shrink 가드 + 시그니처 + docs |
| `src/renderer/layout/table_cell_content.rs` | 호출 인자 |
| `src/renderer/layout/table_partial.rs` | 호출 인자 |
| `tests/svg_snapshot.rs` | 신규 테스트 |
| `tests/golden_svg/issue-617/exam-kor-page5.svg` | 신규 골든 |
| `mydocs/plans/task_m100_617.md` | 수행 계획 |
| `mydocs/plans/task_m100_617_impl.md` | 구현 계획 |
| `mydocs/working/task_m100_617_stage1.md` | 진행 보고 |

## 잔여 위험

- **다중 줄 셀에서 line_segs 가 잘못 채워진 비정상 IR**: 보고된 샘플에 없음.
  발견 시 `segment_width vs inner_w` 비교 가드로 강화 가능.

## 후속 조치

- `local/task617 → local/devel` merge (작업지시자 승인 후).
- 이슈 #617 close (커밋 메시지에서 자동 close 안 함; 수동 close 권장).

## 커밋

- `692457bf Task #617: 표 셀 padding shrink 휴리스틱 다중 줄 가드`
