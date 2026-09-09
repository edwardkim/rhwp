# Task #1745 최종 보고서 — 텍스트-anchor 어울림 표의 후속 문단 wrap 흡수 실패 수정

## 요약
hwpdocs 1000건 rhwp vs 한글(OLE) 페이지·PI 대조에서 발견된, 어울림(Square wrap) 표가
**텍스트 문단에 anchor 된 경우** 후속 문단이 표 옆 wrap 띠 대신 표 아래 페이지로 밀리는
결함을 규명·수정. 대표 `약사법 시행령 별표 2` pi1: rhwp 3쪽 → **1쪽** (한글 정합, MATCH 전환).

## 원인
- Task #362 wrap 흡수 로직이 wrap zone 기준 (cs, sw) 를 **표 문단의 첫 LINE_SEG** 에서 취함.
- 표가 제목 텍스트와 같은 문단에 anchor 되면 첫 LINE_SEG 는 전폭 제목 줄(cs=0) → 후속 문단
  (표 우측 잔여 띠 cs=45568, sw=2620)과 매칭 실패 → wrap zone 종료 → 표 아래 일반 배치.
- 부차: 매칭되더라도 흡수가 현재 column(분할 표 마지막 fragment 쪽)에 기록되어, 다쪽
  RowBreak 분할 표에서는 한글(첫 fragment 쪽)과 어긋남.

## 수정
1. `src/renderer/mod.rs` — `text_anchor_square_table_strip`: 텍스트 혼합 anchor 의 wrap 띠를
   표 geometry(`horz_offset + margin.left + width + margin.right`, 폭 = 첫 seg 폭 − cs)로 도출
   (+단위테스트 2). 한글 저장 LINE_SEG 와 정확 일치 확인.
2. `src/renderer/typeset.rs` — wrap zone 활성화 시 헬퍼 Some 이면 도출 띠 사용(기존 케이스
   무변경). `record_wrap_around_para` 신설: 분할 표의 흡수 문단을 첫 fragment column 에 소급 기록.
3. `src/renderer/layout.rs` — `layout_wrap_around_paras` 에 후속 문단 전용 `strip_x/strip_width`
   와 `render_host_text`(분할 표 호스트 텍스트 이중 렌더 방지) 추가. 표 단독 anchor 경로 무변경.

## 검증
| 항목 | 결과 |
|------|------|
| 재현 파일 (한글 OLE 대조) | PI_MISMATCH → **MATCH** (3=3쪽, pi1 1쪽 WrapAroundPara) |
| 1쪽 SVG 시각 | 수정 전과 동일 (제목 이중 렌더 없음) |
| cargo test --lib | 2050 passed / 0 failed |
| wrap 통합 (546/1440/1139) | 90 passed / 0 failed |
| byeolpyo1/4 · 승강기(#1718) · task1700 | 4 / 26 / 42 / 1쪽 무회귀 |
| 코퍼스 mismatch 39건 | 대상 1건 MATCH 전환, 악화 0 |
| 코퍼스 MATCH 표본 150건 | 150/150 유지 |
| rustfmt(변경 파일) / clippy --lib | 통과 |

## 한계 / 후속
- 사례 A(자리차지 다쪽 표의 anchor 캐럿 페이지 — 17991519 공항시설법 별표3)는
  `verify_pi_page_vs_hangul.py` 의 측정 의미 차이(표 시작쪽 vs 캐럿쪽)로 본 수정 범위 밖.
  도구 알려진 한계로 기록.
- Paginator(engine.rs, env fallback 경로)의 동일 시멘틱 반영은 후속.
- 우측 정렬/가운데 정렬 표의 텍스트 혼합 anchor(띠가 좌측)는 가드로 제외 — 발견 시 확장.

## 산출물
- 소스: `src/renderer/{mod,typeset,layout}.rs`
- 재현: `samples/task1745/table_text_anchor_wrap.hwp` + README
- 조사: `output/poc/hwpdocs_pipage/` (t4_*_pages.txt, bad39_recheck_1745.tsv, match150_recheck_1745.tsv)
- 배경 조사: hwpdocs 1000건 페이지·PI 대조 (`verify_20260702_s1000.tsv`, MATCH 96.1%)
