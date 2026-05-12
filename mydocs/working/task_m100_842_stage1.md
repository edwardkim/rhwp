# Stage 1 완료 보고서 — Task #842 (M100)

목표: 4건 결함의 IR/레이아웃 근거 확정 + 수정 전 기준 산출물 캡처. **소스 수정 없음.**

## 기준 산출물
- `output/svg/task842_before/shortcut_00{1..8}.svg` — 수정 전 8페이지 SVG (회귀 비교 기준).

---

## 결함 #4 — cross-run 우측탭 오버플로 (원인 확정)

증상: `현재 낱말의 끝 글자로 ⟶ Ctrl+(회색)5` 의 `5` 우측 끝 ≈ x 1013px, 정상 우측탭(`Ctrl+Page Up` 등) ≈ x 973px → ~40px 초과.

원인:
- 단축키 문단은 `tab_def_id=1 auto_right=true` (단 우측 끝 자동 우측탭) + 텍스트 `"…\tCtrl+(회색)5"`.
- `src/renderer/composer.rs::split_runs_by_lang` 가 char-shape run `"Ctrl+(회색)5"` 를 스크립트 경계로 분할 → `["Ctrl+(", "회색)", "5"]` (`회`/`색` 만 Hangul, `(`·`)` 는 중립이라 인접 run 에 흡수, `5` 는 ASCII digit 이라 비중립 → 별도 run).
- `src/renderer/layout/paragraph_layout.rs` cross-run right-tab 처리(render 패스 ~1419~1480, est 패스 ~992~1069): `pending_right_tab_render` 소비 시 **탭 직후 한 개 composed run** 의 폭만 `estimate_text_width(&run.text, …)` 로 빼서 시작 x 산출. 따라서 `"Ctrl+("` 만 우측 정렬되고 뒤따르는 `"회색)"`·`"5"` (~38px) 가 좌→우 정상 진행으로 탭스톱 오른쪽으로 밀려나옴.
- 기존 빈-공백-run carry-over 분기(`run.text.trim().is_empty()`)로는 못 잡음.

수정 방향: 우측/가운데 탭의 정렬 단위 = **해당 탭부터 다음 탭(또는 줄끝)까지의 composed run 전체**. 그 합산 폭 기준으로 블록 시작 x 산출(leader end_x 보정도 합산 폭 기준). est/render 패스 동일 규칙. Task #279 목차(페이지번호) 케이스 회귀 점검.

영향 항목 (혼합 스크립트 우측탭): `Ctrl+(회색)5`, `(회색)+/-`, `Shift+(회색)+/-`, `Ctrl+(회색)+`, `Ctrl+(회색)-` 등. `Alt+P/Ctrl+P` (`"인쇄\t Alt+P/Ctrl+P"` — 탭 뒤 선행 공백) 도 같은 계열(공백 run carry-over → 다음 단독 run 정렬). 합산-폭 방식이면 함께 해소될 가능성 큼 — Stage 2 에서 재확인.

---

## 결함 #3 — 단 구분선 점선 (원인 확정)

증상: 두 단 사이 세로 구분선이 실선. PDF 는 원형 점선(`⋮` 형태).

원인:
- shortcut.hwp 의 2단 ColumnDef: `2단, 유형=배분, 구분선 type=7, width=7, color=0xaeaeae`. (`type=7` = HWP 선 종류 Circle/원형 점선 — `src/parser/doc_info.rs:303` 참조.)
- `src/renderer/layout.rs::build_column_separators` (~1029~1035): `separator_type` → `StrokeDash` 매핑이 `2=>Dash, 3=>Dot, 4=>DashDot, 5=>DashDotDot, _=>Solid` 만 처리. `6`(LongDash), `7`(Circle) 누락 → `7` 이 `_ => Solid` 로 떨어짐.
- 파서/IR(`ColumnDef.separator_type/width/color`)·SVG `<line>` 출력 자체는 정상 동작 (`output/svg/task842_before/shortcut_002.svg` 에 `<line … stroke="#aeaeae" …/>` 존재, dasharray 없음).

수정 방향: `build_column_separators` 의 line-type→dash 매핑을 `doc_info.rs` 의 line_type 의미(1=Solid, 2=Dash, 3=Dot, 5=DashDotDot, 6=LongDash→Dash, 7=Circle→Dot, …)와 일치시킴. 가능하면 `border_line_type_to_dash` 류 공용 변환 재사용. `width=7` → `border_width_to_px` 값(현재 ~1.9px)이 HWP 0.5mm 와 큰 차이면 같이 검토(부차).

---

## 결함 #2 — 섹션 헤더 바 좌측 위치 어긋남 (원인 미확정, Stage 4 에서 확정)

증상(SVG 좌표 재측정으로 정정):
- 페이지 1 `커서 이동` 헤더 바 rect x ≈ 94.5px, 헤더 글자 x0 ≈ 98.3px.
- 페이지 2 `파일` 헤더 바 rect x ≈ 122.5px, 헤더 글자 x0 ≈ 126.3px → 페이지 1 대비 ~28px 우측 이동.
- **본문 텍스트 x0 는 두 페이지 모두 ≈ 121.2px 로 동일** — 즉 어긋난 것은 헤더 바(1×1 TAC 표)뿐. (사용자가 말한 "왼쪽에 여백" = 헤더 바가 오른쪽으로 밀려 본문보다 들어간 상태.)
- 또 페이지 1 body-clip width ≈ 933.5px, 페이지 2+ ≈ 954.0px 로 ~20px 차이.

관찰: 페이지 1 `커서 이동` 헤더 문단(0.1)과 페이지 2 `파일` 헤더 문단(0.36) 의 ParaShape(`margins left=0 right=2000`), TAC 표 outer_margin(1mm), 표 size(69448 HU) 가 **동일**. 차이점: (a) 0.1 은 직전 `구역나누기` + 자체 `다단나누기`, 0.36 은 `쪽나누기`; (b) 직전 ColumnDef 가 0.1 은 `1단 간격=10mm`, 0.36 은 `1단 간격=0mm`. 다단 zone 안에서 쪽나누기로 새 페이지 진입 시 헤더 바 가로 위치/폭 계산이 어긋나는 것으로 추정. 정확한 원인은 Stage 4 에서 `--debug-overlay` + 레이아웃 로깅으로 확정.

---

## 결함 #1 — 헤더 바 1×1 TAC 표 앞뒤 단락 간격 압축 (원인 확정, 회귀 위험 최고)

증상: 각 회색 헤더 바 아래(와 위)에 PDF 가 두는 가시적 여백(~10~13px)이 rhwp 에서 0 에 가까움.

근거 (`dump-pages -p 0`):
- 페이지 1: 단1(헤더 표) `used=31.1px`, 단2/단3(본문) `zone_y_offset=100.2` — 헤더 표 끝(69.1+31.1=100.2)과 본문 시작이 **gap=0**.
- 본문 단2 `used=186.7px` vs `hwp_used≈273.3px` (diff −86.7px), 단3 `used=173.3px` vs `≈253.3px` (diff −80.0px) — dump-pages `used` 가 line-spacing gap 미반영이라 과소 표시이긴 하나(실제 SVG row pitch 는 20px 로 정상), 헤더 표 직후 zone 전환 spacing 누락이 누적 압축의 핵심.
- 헤더 문단 0.1: `spacing before=0 after=0 line=100%`, 본문 문단 0.2: `spacing before=0 after=0` + `[다단나누기]` + `2단 ColumnDef` → 헤더(1단 zone)에서 본문(2단 zone)으로의 **zone 경계**. 명시 spacing 어디에도 없음 → 한컴이 암묵적으로 추가하는 간격(RFC #774 zone-level / TAC 표 후속 spacing).

수정 방향: 구 #770/#773/#776 + RFC #774(`mydocs/...`) 분석 기반으로 TAC 1×1 표 문단 직후(또는 zone 전환 시) 한컴 동일 간격 보정. **layout 본질 정정 — 다단/단일단/표분할 상호작용 회귀 광범위 검증 필수**(메모리 `feedback_essential_fix_regression_risk`). 그래서 Stage 5(마지막)에 배치.

---

## 회귀 비교 대상 (Stage 2~5 공통)
- 목차/페이지번호 우측탭 샘플 (#4 관련)
- 다단 + 단 구분선 사용 샘플 (#3 관련)
- TAC 표 포함 샘플 전수 + 표분할 샘플 + 다단/단일단 혼재 샘플 (#1, #2 관련)
- 한컴 2010/2020 정답지 접근 가능 시 우선 (#1)

---

다음: Stage 2 — 결함 #4 (cross-run 우측탭 폭 합산) 수정.
