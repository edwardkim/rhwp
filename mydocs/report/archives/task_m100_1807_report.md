# 최종 결과 보고서 — Task M100 #1807

## 이슈

ir-diff: Shape 글상자 내부 문단 재귀 비교 부재 (#1795 유형 소거망 구멍)

## 결론

ir-diff 에 글상자(text_box) 내부 문단 재귀 비교를 추가했다. #1795 유형(글상자 내부
직렬화 결함)이 이제 render-diff 시각 이탈 단계가 아니라 ir-diff 단계에서 즉시 검출된다.

## 구현 내역

| 파일 | 내용 |
|------|------|
| `src/main.rs` | `diff_textbox_paragraph_fields`(문단 쌍 핵심 필드 비교: text/cc/char_offsets/char_shapes/line_segs/field_ranges) + `diff_textbox_paragraph_lists`(목록 비교, 중첩 글상자 재귀) + `diff_shape_textbox`(유무 비교 + 진입점) 추가, Shape 매치 암에서 호출 |

출력 접두어 `ctrl[N] shape tb_p[K]` — 기존 `--summary` 카테고리 집계와 호환.

## 검증 결과

1. **#1795 결함 검출 확인** (pre-#1795 빌드 변환본 seoul_0043.hwp):
   `ctrl[0] shape tb_p[1] char_offsets[35]: A=51 vs B=59`,
   `field_ranges[1]: A=(101..105,c2) vs B=(61..105,c2)` — 결함 시그니처 정확 검출 ✔
2. **수정 후 무소음**: 현재 빌드 변환본 → 글상자 차이 0 (기존 무관 cc 1건만) ✔
3. **코퍼스 스팟체크** (seoul_0765/admrul_0072/seoul_0505/seoul_0978 변환→비교):
   신규 노이즈 0, 크래시 없음 ✔
4. cargo test --release --lib 통과 (도구는 bin 전용 — lib 무영향) ✔

## 비고

- CLAUDE.md 의 ir-diff 비교 항목 문구는 별도 문서 정리 시 갱신 후보
  (그림/도형 항목에 "글상자 내부 문단 재귀" 추가)
