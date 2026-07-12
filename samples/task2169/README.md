# samples/task2169 — Issue #2169 통제 프로브 픽스처 (합성)

한글 NO_LS(저장 LINE_SEG 부재) 셀 회계의 실측 근거. 생성기·프로브 스크립트는
`output/poc/task2169/` (make_anchor_ladder.py / make_empty_ladder.py 등,
COM 직독은 tools/hangul_row_heights2.py 의 hangul_col0_heights).

## empty_ladder.hwpx — 빈 문단 em 승격 근거

trailing/중간/leading 빈 문단 사다리 (ls=160%, 10pt). 한글 실측: **모든
위치의 빈 문단 = 완전한 em 줄** (trailing 빈 1/2/3 = 2.001/3.000/4.001줄
역산 — em 모델 정확). 종전 placeholder 400HU=5.33px 는 과소.

## anchor_ladder.hwpx — 중첩 표 anchor/outMargin 근거

TAC 중첩 표 anchor 빈 문단 사다리 (10/11pt). 한글 실측: **anchor 빈 문단
몫 = 0** (row = nested + pad 정확). 80168 r6 검산: 텍스트 46.9 + 중첩 선언
99.7 + **outMargin 7.5** + pad 6.0 = 160.1 (한글 관측 160.2).

## 반영 수정 (#2169 부분집합)

- NO_LS 셀 중첩 표: text_height + Σ(max(실측, 선언) + outMargin) 가산
  (additive 경로 한정), anchor 빈 문단 0.
- NO_LS 순수 빈 문단: placeholder → 비마지막 fs×ls% / 셀 마지막 em.
- `para_has_no_stored_line_segs`: 원본 NO_LS 와 자기-export HWPX 재파싱본
  (전부 synthetic lineseg) 동일 취급 — 왕복 시멘틱 정합.
- issue_1891 픽스처(80168 hwpx) 재생성: 과거 export 의 셀 lineseg 소실본
  → 현행 보존 산출물 (원본 .hwp 직파싱은 157 그린).

게이트: issue_1891 8샘플·1842·1623·2146·byeolpyo4 통과, 359 recount
IMP3/WOR0, 전체 cargo test 통과.
