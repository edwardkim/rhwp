# 한컴 paragraph/zone 수직 spacing 모델 (shortcut.hwp 정합) — RFC #774 후속 분석

작성 배경: Task #853 Stage 3-1. 닫힌 RFC [#774](https://github.com/edwardkim/rhwp/issues/774)("한컴 PDF paragraph spacing 알고리즘 정밀 분석")의 후속. shortcut.hwp ↔ `pdf/basic/shortcut-2022.pdf` 측정 + IR 구조 기반.

## 1. LINE_SEG.vertical_pos 의미 (확인됨)

LINE_SEG 의 `vertical_pos` = **zone(단/페이지 흐름 구간) 상단 기준 누적 절대값** = Σ(lh_i + ls_i) (선행 줄들의 line_height + line_spacing).

근거: 본문 행 pi=37 `vpos=0, lh=1000, ls=500` → pi=38 `vpos=1500` (= 1000+500). pi=39 `vpos=3000`. 즉 한 zone 안에서 문단들의 첫 줄 vpos 는 누적된다.

## 2. 섹션/페이지 첫 문단 — spacing_before 클램프 (Stage 2 에서 정정 완료)

`para_index == 0` 이면서 column-top 인 문단(예: 제목 pi=0)은 한컴이 `spacing_before` 를 적용하되 **그 문단 첫 LINE_SEG.vertical_pos 로 상한 클램프**한다.

- 제목 pi=0: `PS spacing_before=3968 HU (52.9px)`, `LINE_SEG[0].vertical_pos=1984 HU (26.45px)` → 적용값 = min = 1984 HU = 26.45px. 한컴 PDF 제목 텍스트 top = body_top(56.7px) + 26.9px ≈ 83.6px ≈ vertical_pos. ✓
- rhwp 정정: `paragraph_layout.rs` 에 `is_column_top && para_index==0 → y += spacing_before.min(hwpunit_to_px(line_segs[0].vertical_pos))` 적용(커밋 `f0d34713`). `height_measurer`(이미 vertical_pos 반영)와 정합.

## 3. `다단나누기`(ColumnDef) zone 진입 top spacing — **미규명, 정정 대상**

shortcut.hwp 의 각 구분 칸 섹션은 `다단나누기`(ColumnDef control)로 새 zone 을 연다. ColumnDef 의 `간격`(column spacing) 필드 값:

| 문단 | ColumnDef | `간격` |
|------|-----------|--------|
| pi=1 (1쪽 "커서 이동" 헤더 띠) | 1단, 일반 | **10.0mm (2835 HU = 37.8px)** |
| pi=36 (2쪽 "파일" 헤더 띠) | 1단, 일반 | 0.0mm |
| pi=2 / pi=37 (본문, 2단 배분) | 2단, 배분 | 1.0mm (283 HU) |

- RFC #774 가설 B 검증 결과: ColumnDef.spacing 이 rhwp 의 `zone_y_offset` 에 반영 안 됨.
- 1단 ColumnDef 의 `간격`(원래는 단 사이 가로 간격이나 1단이라 가로 간격 무의미)이 **세로 zone 진입 간격으로 해석**되는지 확인 필요. 1쪽 "커서 이동" 띠 앞 ~38px 디자인 여백의 출처 후보. 2쪽 "파일" 띠는 `간격=0` 이라 이 항목으론 설명 안 됨 → 2쪽 deficit 은 §4 가 주원인.

## 4. 헤더 띠 문단(TAC 표 단독 줄) — line0 텍스트 흡수, **정정 대상**

헤더 띠 문단의 IR 구조가 1쪽(pi=1)과 2쪽 이후(pi=36)가 다르다:

| 문단 | text_len | LINE_SEG |
|------|----------|----------|
| pi=1 (1쪽 "커서 이동") | 0 (빈 문단) | `ls[0]: vpos=0, lh=2332` (= 표 1766 + outer_margin 283×2). 줄 1개. |
| pi=36 (2쪽 "파일") | 2 ("파일") | `ls[0]: vpos=0, lh=1200` (텍스트 줄, 16px) + `ls[1]: vpos=1200, lh=2332` (표 줄, 31px). 줄 2개. |

- pi=36 류: 한컴은 line0(텍스트 "파일", 16px) → line1(표, 31px) 순으로 배치(총 47px). **rhwp 는 표를 line0 에 놓고 텍스트 line0 을 흡수해 ~27px** → ~16~20px 부족. (표 셀 안에도 "파일" 이 있어 PDF 상 띠 1개만 보이지만, 문단 텍스트 "파일"(line0)은 띠 위 16px 줄에 별도로 흐른다.)
- pi=1 류(빈 문단): line0 = 표 줄 1개(2332 HU = 31px). rhwp 정합. → 1쪽 헤더 띠 자체 높이는 OK, 1쪽 deficit 은 §3.

## 5. 측정 종합 (shortcut.hwp 2쪽, `mutool draw -r 100` → @96dpi 환산)

| 요소 | 한컴 PDF (body_top 기준) | rhwp | 차이 |
|------|------------------|------|------|
| "파일" 헤더 띠 상단 | +19.1px | +3.8px | rhwp ~15px 높음 (= line0 텍스트 줄 흡수) |
| 헤더 띠 하단 | +43.1px | +27.3px | ~16px |
| 본문 첫 줄 "새 문서" 상단 | ~+75px | ~+29px | rhwp **~46px 높음** |
| 띠 하단 ↔ 본문 사이 | ~32px | ~2px | ~30px 부족 |

- ~46px = ~15px(§4 line0 흡수) + ~30px(띠↔본문 gap). 띠↔본문 ~30px gap 의 출처: pi=36 zone 은 line0(16px)+line1(47px) = 47px 인데(즉 body zone 은 body_top+47px ≈ +47px 에서 시작해야 함) PDF 는 +75px → ~28px 미설명. 후보: ① 1단 zone → 2단 zone 전환 시 한컴 고정 간격, ② TAC `wrap=위아래`(TopAndBottom)가 글자처럼 취급이면서도 위아래 어울림으로 추가 예약, ③ 표 `쪽나눔=RowBreak` 처리, ④ 본문 첫 문단(pi=37)의 추가 leading. **추가 측정 필요** (3쪽 이후 다른 띠들의 PDF↔IR 대조로 패턴 확인).

## 6. 3쪽 본문영역 초과

3쪽 단3 `<편집 화면 분할에서>`(pi=94)·"화면 이동"(pi=95) 둘 다 `vpos=0` 겹침 — 닫힌 #768 패턴. §3/§4/§5 정정으로 zone 누적이 정확해지면 자연 해소될 가능성. 안 되면 page break 가드(누적 offset > body_bottom 시 다음 페이지) 보강.

## 7. 구현 계획 (Stage 3-2)

순서대로, 각 단계마다 `cargo test --release` + shortcut.hwp 7~8쪽 SVG↔PDF 확인:

1. **헤더 띠 line0 텍스트 렌더** (`composer.rs` / `layout/table_partial.rs` / `layout_table_item`): 헤더 띠 문단을 LINE_SEG 순서(text line0 → table line1)대로 배치. rhwp 가 표를 line0 에 올리는 경로 차단. 영향: pi=36 류 전부 +16px.
2. **추가 측정 → 띠↔본문 ~28px gap 규칙 확정** (3쪽 이후 띠들 PDF↔IR 대조). 확정되면 해당 경로(zone 전환 간격 / TAC wrap 예약 / RowBreak)에 적용.
3. **ColumnDef.spacing → 1단 zone 진입 top 간격** (`build_columns` `zone_y_offset`): 1쪽 "커서 이동" 띠 앞 ~38px. 단, 회귀 위험 — 다른 문서의 1단 ColumnDef 사용처(전 fixture sweep)로 영향 확인. 한컴 정답지로 1단 ColumnDef `간격`의 세로 적용 여부 재확인 필수.
4. 3쪽 overflow 재확인 → page break 가드 필요 시 보강.
5. 광역 회귀(`cargo test`, 전 fixture sweep, 한컴 2010/2020 정답지).

회귀 위험: §1·§2 = 낮음(완료). §4(composer 변경) = 중간~높음. §3(zone 진입 간격) = 높음 — `feedback_essential_fix_regression_risk` 정합, 전 fixture sweep + 한컴 정답지 필수.

## 미해결 / 추가 작업

- 띠↔본문 ~28px gap 의 정확한 출처 (§5 후보 ①~④) — 3쪽 이후 띠 추가 측정 필요.
- 1단 ColumnDef `간격`의 세로 적용 — 한컴 편집기(Windows) 또는 다른 샘플 cross-check.
- 본 모델은 shortcut.hwp 의 `다단나누기`+TAC 표 헤더 띠 패턴 한정. 다른 zone/spacing 패턴은 별도.
