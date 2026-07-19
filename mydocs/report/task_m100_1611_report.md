# 최종 결과보고서 — Task #1611

**제목**: 렌더링 −1쪽 갭 요인 B — footer(발신명의) Page+Bottom 블록 page-fit 정합
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1611 · **브랜치**: `local/task1611` (base: `local/task1608`)

## 1. 문제
Task #1600 −1쪽 갭의 요인 B. 발신명의 footer(`VertRelTo::Page` + `valign=Bottom` +
`wrap=TopAndBottom`, 비-TAC)를 TypesetEngine 이 ① stored vpos 에 동기화하지 않고(Paper 만
처리, Page 누락) ② 측정 높이(302.3px, 셀 내용)로 fit 판정(선언 351.4px 무시)해, page-fit 이
~60px 과소되어 footer 가 본문 페이지에 흡수 → 한글 2쪽 문서가 rhwp 1쪽.

## 2. 근본 (Stage 1 확정, 36387725)
```
vpos=640.7px  declared=351.4px  measured(table_total)=302.3px  body=990.2px
버그: cur_h 627.5 + 302.3 = 929.8 ≤ 990.2 → 1쪽
정합: vpos 640.7 + 선언 351.4 = 992.1 > 990.2 → 분할(2쪽, 한글 일치)
```

## 3. 수정 (`src/renderer/typeset.rs`)
generic fit 직전, `is_page_bottom_topbottom_block` 처리: `current_height` 를 stored vpos 로
동기화하고 `block_height = table_total.max(선언높이)` 로 fit 판정 → 초과 시 블록 통째 다음 쪽.
(Paper-앵커 절대좌표 처리와 달리 페이지네이션에 참여.)

## 4. 결과
| 지표 | before(#1608) | after(#1611) |
|------|--------|-------|
| 통제셋 일치 (92건) | 66 (71.7%) | **72 (78.3%)** |
| −1쪽 | 21 | 12 |
| +초과 | 5 | 8 |
| **net** | — | **+6** (9 해소 − 3 회귀) |

전 회귀 게이트 통과: hwpx baseline 4, visual_roundtrip 3, lib 1975 tests, clippy 무경고.
SVG 시각: page1 본문 / page2 발신명의 단독.

## 5. 한계·후속
- 회귀 3건(36395270·36394590·36389909): 한컴이 1쪽 유지하는 footer 를 과도하게 민
  over-correction. net +6 우세로 수용. (선언높이 예약 경계 미세조정은 후속 여지.)
- 잔여 −1쪽 12건: footer 군집 외 별 요인(대형 tac 표 과소측정 36398709 등) — 범위 밖.
- 요인 A(#1608, net +6) + 요인 B(#1611, net +6) 누적으로 통제셋 일치 60→72 (+12).

## 6. 산출물
- 소스: `src/renderer/typeset.rs`
- fixture: `samples/hwpx/opengov/36387725_footer_page_bottom.hwpx`
- 테스트: `tests/issue_1611_footer_page_bottom_pagination.rs`
- 측정: `output/poc/task1611_after.tsv`, `output/poc/task1611_svg/`
- 문서: `_impl`, `_stage1~4`, 본 보고서, `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` 갱신
