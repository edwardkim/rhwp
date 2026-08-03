---
kind: investigation
status: completed
canonical: mydocs/manual/bug_hunting_playbook.md
last_verified: 2026-08-02
---

# Task #3738 Stage 17 — p78–p80 표 25 URL 각주 경계 복원

## 출발 상태

Stage 16 commit `b78928606`은 HWP p31–p32의 두 줄 각주 30을 fragment로 나눠 본문/각주 충돌을 복원했다.
이 Stage의 native HWP 출력은 220쪽, 기준 한컴 PDF는 215쪽이며 전체 pagination 정합은 아직 미완료다.

## 이전 Stage에서 이월한 미해결 기준 목록

Stage 15의 "다음 Stage"와 Stage 16의 "다음 단계"에 남은 항목을 아래처럼 **전부** 이월한다. Stage 16에서
해결한 p31 각주 30은 목록에서 제외했다. p76–p79는 Stage 15 native selected sweep에서 owner를 복원했지만,
사용자 UI의 관측과 아직 일치한다고 재판정하지 않았으므로 완료로 바꾸지 않는다.

| 출발 Stage | rhwp 관측 페이지 | 결함/검증 대상 | Stage 17 상태 |
| --- | --- | --- | --- |
| 15, 16 | 37 | 그림 두 개가 세 개로 보이는 중복 그림 paint 또는 fragment duplication | 미분석·이월 |
| 15, 16 | 43 | 본문과 각주 영역 overlap | 미분석·이월 |
| 15, 16 | 54 | 본문과 각주 영역 overlap | 미분석·이월 |
| 15, 16 | 66 | 표와 각주 영역 overlap | 미분석·이월 |
| 15, 16 | 76 | 표 24가 기준의 다섯 줄이 아니라 네 줄로 보이는 UI 관측 | native/WASM revision·semantic row owner 재확인 대기 |
| 15, 16 | 77–79 | 그림 51 단독 이월·caption owner·빈 표 page 관측 | native/WASM revision·semantic owner 재확인 대기 |
| 15, 16 | 80 | 표 셀의 reference 번호·URL·본문 다층 overlap | **Stage 17에서 해소** |
| 15, 16 | 83 | `para=897` FullParagraph overflow 후보 | 미분석·이월 |
| 15, 16 | 87 | 기준 PDF와 semantic 흐름 차이 | 미분석·이월 |
| 15, 16 | 90 | 기준 PDF와 semantic 흐름 차이 | 미분석·이월 |
| 15, 16 | 99–100 | 기준 PDF와 semantic 흐름 차이 | 미분석·이월 |

표의 "미분석·이월"은 결함 해소나 비결함 판정이 아니다. 각 행은 p80 커밋 후에도 다음 Stage의 출발 목록으로
그대로 계승한다.

사용자 화면의 rhwp p80에는 표 셀 경계 안에서 reference 번호·URL·설명 본문이 서로 위로 겹치고, 표 아래의
123)·124) 각주도 같은 셀 흐름과 분리되지 않는 현상이 보인다. 이 문제는 p31의 단일 body-footnote fragment와
표면상 비슷해도, 표 cell 내부의 line paint/row fragment/footnote reservation 중 어느 한 축이 원인인지 아직
판정하지 않았다.

## 분석과 원인

기준 PDF와 raw HWP `para=885`를 표 25/각주 번호로 맞췄다. 이 표는 `RowBreak`, 6행×3열, table 선언 높이
`448.1px`인 반면, 셀 원문을 모두 누적한 측정 높이는 `1184.1px`다. 따라서 p78의 첫 fragment와 p79의
continuation 표 자체는 기준 PDF의 경계를 이미 재현했지만, 셀 URL 각주 queue의 terminal 처리만 실제 경계와 달랐다.

- 기존 terminal queue는 일반 본문을 위한 40px safety margin과 HWP5 표 보호 간격 32px를 함께 적용했다.
  그 결과 p79에 들어가야 할 111번이 p80으로 밀렸고, p80의 각주 영역이 과대 예약돼 본문 flow가 틀어졌다.
- queue가 저장 `LINE_SEG`만으로 각주 높이를 추정하면 긴 URL의 실제 재래핑 높이보다 작아, 반대로 p80 본문이
  각주 112 구분선을 침범할 수 있었다.

이는 표 cell의 중복 paint나 URL 폭 overflow가 아니라, **실제 FootnoteArea 높이와 terminal queue budget의 불일치**다.
동일 페이지 번호만 대조하지 않고 표 25, 105–124번 각주, p889 문단을 anchor로 판정했다.

## 구현

1. `TableCellFootnote`가 compose된 실제 줄/줄간격 기준 높이를 보유하게 해, URL 재래핑을 queue reservation과
   `FootnoteArea` paint가 같은 높이로 계산하게 했다.
2. native HWP5의 다수 URL 각주(`>=8`)를 가진 고정 선언-height RowBreak 표만, continuation terminal fragment에서
   일반 safety margin을 중복 적용하지 않도록 한정했다. 표 하단 보호 간격도 32px가 아니라 기준 PDF의 한 줄 여백을
   반영한 12px로 낮췄다. 다른 표·HWPX·일반 footnote 경로는 유지한다.
3. regression은 p78=105·106, p79=107–111, p80=112–124와 p889 body bottom이 p80 footnote separator 위에
   있어야 한다는 contract를 render tree로 고정했다.

## 검증과 증적

- `cargo fmt --check`, `git diff --check` 통과.
- `CARGO_TARGET_DIR=target/review-planet6897-20260802 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3738_rowbreak_table_footnote_fragment -- --nocapture` — **7 passed**.
- `visual_sweep.py` 144 DPI selected sweep: `requested_pages=completed_pages=[78,79,80]`,
  `missing_pages=[]`, `run_state=complete`, structural flag 0건. rhwp SVG/render tree는 문서 전체 220쪽을 생성하고,
  raster/PDF/compare/overlay/review는 세 쪽만 생성했다. pixel match는 p78 `90.71027%`, p79 `88.00550%`,
  p80 `90.41151%`이다. 폰트 raster 차이 때문에 수치만으로 합격을 선언하지 않고 review PNG와 render-tree를 함께
  확인했다.
- p79 render tree: table bottom `838.5px`, FootnoteArea 시작 `851.9px`, 각주 107–111만 존재한다.
- p80 render tree: FootnoteArea separator `501.6px`; p889의 마지막 p80 본문 줄 bottom `469.8px`로 31.8px 위에
  있다. 각주 112–124만 존재하며 p80 본문이 p81로 통째 이월되지 않는다.
- 실행 임시 경로: `/private/tmp/rhwp-stage17-p078-080-sweep.41vbIY/issue3738-stage17-hwp-p078-080/`.
- 장기 보존 원본: HWP `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp`
  (`50094a3db2b2003b293c5cbf43014d001aa97929acb488cef0cb7ea0e16b3113`), 동일 문서 HWPX
  (`8ae9dc95643d0902fcced2af73badd732aea86c1cc5b875ef7b1272bccba862c`), 한컴 기준 PDF
  `pdf/pr3740/hwp/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-2020.pdf`
  (`7879ffee6313575132187c44c0090cd2e62c32c12c29b7eabd989181acf27b3a`). 세 파일은 이미 Git 추적 중이며
  LFS 대상이 아님을 확인했다.

상세 비교는 [Stage 17 visual sweep](task_m100_3738_stage17_visual_sweep.md)에, 대표 3-way review PNG는 아래에 보존했다.

- [p78](../pr/assets/pr_3740_issue3738_stage17/hwp_p078_review_after.png)
- [p79](../pr/assets/pr_3740_issue3738_stage17/hwp_p079_review_after.png)
- [p80](../pr/assets/pr_3740_issue3738_stage17/hwp_p080_review_after.png)

## 다음 단계

p37, p43, p54, p66, p76–p79의 별도 관측, p83, p87, p90, p99–p100과 전체 220/215쪽 pagination 차이는 이 수정으로
해소됐다고 주장하지 않는다. p80 결함만 완료 처리하고, 나머지는 다음 Stage의 새 분석 문서로 이월한다.
