# Task #928 Stage 4: 시각 회귀 검사

## 1. 한컴 2022 PDF 정합 비교 (Primary 검증)

`pdf/exam_kor-2022.pdf` 5쪽 13번 문제 `<보 기>` 박스 안 다이어그램 (시각 권위 자료):

```
(가) ⇨ [A단계] ⇨ (나)
```

→ 3 요소 (괄호+한글 / 화살표 / 사각형+텍스트 / 화살표 / 괄호+한글) 만 출력.

**rhwp Fix 후 결과**:
- y=421.73 단일 baseline: `(가)` x=246, `⇨` x=279, gap, `⇨` x=386, `(나)` x=412
- y=427.30 (사각형 내부): `A 단계` x=326-341 (gap [294, 379] 안에 정상 배치)

→ **한컴 정답지와 시각 정합 일치**.

## 2. svg_snapshot 회귀 검사

```
cargo test --release --test svg_snapshot
test result: ok. 8 passed; 0 failed
```

| 테스트 | 대상 | 결과 |
|--------|------|------|
| form_002_page_0 | HWPX simple form | ✅ |
| table_text_page_0 | HWPX table | ✅ |
| issue_157_page_1 | 위아래 wrap 비-TAC 표 | ✅ |
| issue_267_ktx_toc_page | KTX.hwp 목차 right tab | ✅ |
| issue_147_aift_page3 | aift.hwp MEMO 컨트롤 | ✅ |
| issue_617_exam_kor_page5 | exam_kor.hwp 6페이지 셀 padding | ✅ |
| issue_677_bokhakwonseo_page1 | 복학원서.hwp PartialParagraph + PUA | ✅ |
| render_is_deterministic_within_process | 동일 입력 → 동일 출력 | ✅ |

→ 기존 회귀 차단 패턴 영향 없음.

## 3. 동일 패턴 샘플 추가 검사

`samples/exam_science.hwp` 의 `pi=33` (1×1 표, tac=true wrap=TopAndBottom):
- 셀[0] r=0,c=0 의 `p[1]` ctrls=2 (사각형 tac=true wrap=TopAndBottom)
- `ls[0] vpos=1610, ls[1] vpos=3220` — multi-line paragraph + ls[1] inline 사각형
- Task #500 의 참조 케이스 ("exam_science p2 7번 글상자 ㉠")

**검증 결과**: `cargo build --release` + svg_snapshot 통과. exam_science page 2 SVG 생성 정상 (1178 텍스트 노드, 123 unique y 좌표 — 회귀 시 발생할 동일 paragraph 다중 baseline 패턴 미관측).

본 fix 의 `will_render_inline` 가드는 Picture 분기 (Task #500 이후 안정) 와 동일 패턴이므로 Task #500 의 ls[1]+ 좌표 보정 로직과 충돌 없음:
- `will_render_inline=true` (paragraph_layout 이 처리 가능): 새 경로로 inline_pos 좌표 사용
- `will_render_inline=false` (Task #500 보정 필요 케이스): 기존 target_line + tac_img_y 경로 유지

## 4. 다중 shape 혼재 케이스 분석

| 케이스 | 분석 | 영향 |
|--------|------|------|
| 단일 shape, will_render_inline=true | 회귀 해소 (본 fix 대상) | 정상화 |
| 단일 shape, will_render_inline=false | 기존 경로 유지 | 변화 없음 |
| 다중 shape, 모두 will_render_inline=true | 텍스트 발행 / prev_tac_text_pos 갱신 전부 스킵 → 트레일링 블록도 미진입 | 정상화 |
| 다중 shape, 모두 will_render_inline=false | 기존 경로 그대로 | 변화 없음 |
| 다중 shape, 혼재 (일부 true, 일부 false) | false 케이스에서만 prev_tac_text_pos 갱신 → 트레일링 블록이 false 케이스 이후 텍스트만 발행 (paragraph_layout 이 발행한 true 케이스 영역과 잠재 중복 가능성) | **이론적 잔존 회귀** (관측 사례 없음) |

→ 혼재 케이스는 본 fix 영역에서 가설적이며 회귀 관측 사례 없음. 발견 시 별도 이슈로 분리 (트레일링 블록도 가드 추가 필요).

## 5. 다른 컨트롤 분기 검토

같은 함수 (`layout_table_cells`) 의 다른 컨트롤 분기:

| 분기 | 가드 상태 | 비고 |
|------|----------|------|
| Picture | ✅ `will_render_inline` 가드 (1698) | Task #877 등에서 정착 |
| **Shape** | ✅ **본 fix 로 가드 추가 (1818)** | Task #928 |
| Equation | ✅ `already_rendered_inline` 가드 (1974) | Task #287 (#301) |
| Table | (별도 path) | layout_composed_paragraph 가 처리 |

→ 패턴 일관성 확보.

## 6. Stage 4 결정 사항

- ✅ 한컴 2022 PDF 시각 정합 확인 (5쪽 다이어그램 정답 3 요소)
- ✅ 자동 회귀 0건 (`cargo test --release` 전체 통과)
- ✅ 동일 패턴 샘플 (exam_science) 회귀 미관측
- ✅ 다중 shape 시나리오 분석 완료 (혼재 케이스만 가설적 잔존, 관측 없음)
- ⏳ Stage 5: 최종 정리 + 보고서 작성 + 오늘할일 갱신
