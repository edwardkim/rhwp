# Stage 1 — Task #866 v2: 정밀 측정 + pi=94 회귀 수정

GitHub Issue: edwardkim/rhwp#866 · 브랜치: `pr-task853` (PR #868 위)

## 1. pi=94 `<편집 화면 분할에서>` 회귀 수정

`typeset.rs::paginate_section` 의 `[단나누기]`(ColumnBreakType::Column) 처리:
- 종전: `!has_diff_col_def` 면 무조건 `advance_column_or_new_page()` → 다단 zone 의 **마지막 컬럼**에서 단나누기를 만나면 곧장 새 페이지(`push_new_page`).
- 수정: 마지막 컬럼이면 `process_multicolumn_break()` 로 라우팅 — 현재 ColumnDef 를 유지한 채 새 zone 밴드를 시작하고, 같은 페이지에 여유가 있으면 이전 밴드 아래에, 부족할 때만 새 페이지로(이미 `process_multicolumn_break` 가 그 판정 보유).

결과 (`dump-pages` / `export-svg`):
- pi=94 "<편집 화면 분할에서>" + pi=95 "화면 이동" 이 3쪽 zone_y_offset≈203 에 2단으로 배치 → 한컴 PDF 3쪽 정합.
- **shortcut.hwp 페이지 수 8 → 7** — `pdf/basic/shortcut-2022.pdf` 가 7쪽이라 정합.
- 부작용: `LAYOUT_OVERFLOW` 4 → 6 (3쪽이 더 채워져 `<글상자에서>` 그룹이 3쪽 하단 ~11~31px 초과). 한컴 PDF 3쪽은 `<그림 넣기에서>` 그룹까지만 → Stage 2 의 전환 간격 정정으로 3쪽이 한컴처럼 일찍 끊기면 해소 예상.

## 2. PDF ↔ rhwp SVG 측정 (96 dpi PNG 기준, y 좌표 px)

### 본문 항목 줄간격
rhwp SVG: 본문 항목들 ~20px 간격 (예: 1쪽 빈칸삽입 206 → 문단나누기 226 → … 20px 등간격). 한컴 PDF 도 ~20px. → **정합. `dump-pages` 의 `diff=-86.7px` 등은 metadata 측 measurer artifact (렌더 현실 아님).**

### 띠 ↔ 본문 전환 (1쪽, pi=1 형 빈문단 띠)
| 구간 | rhwp SVG Δy | 한컴 PDF Δy | 차이 |
|---|---|---|---|
| 제목 → "커서 이동" 띠 | ~60px | ~64px | rhwp ~4px 부족 |
| "커서 이동" 띠 → "빈칸 삽입" 본문 | ~40px | ~50px | rhwp ~10px 부족 |
| "오른쪽 단으로"(본문 끝) → "지우기" 띠 | ~30px | ~38px | rhwp ~8px 부족 |

→ pi=1 형 띠(ColumnDef 1단 `간격`=10mm) 위·아래 전환이 한컴 대비 ~4~10px 부족. 현 `build_columns` 의 `prev_zone_design_px/2 + new_zone_design/2` 미러가 적용은 되나 양이 부족(또는 띠 zone 높이 산정이 약간 작음).

### pi=36 형 띠("파일"·"보기"·"입력"·"서식"…, 셀 텍스트 + 표-as-char)
- **rhwp 는 띠 문단 텍스트("파일")를 띠 위에 한 줄(line0) 더 그린다** (예: 2쪽 "파일" y=70.3 + 셀 텍스트 y=93.8 → "파일" 2회). **한컴 PDF 는 띠 1개만** (220dpi crop 확인 — 띠 위 공백, 별도 "파일" 없음). → 띠 문단 line0 텍스트 별도 렌더가 한컴과 불일치. Stage 2 에서 이 line0 텍스트 렌더 제거(또는 표-as-char 와 동일 줄로 흡수) 검토 필요.
- 이 차이 때문에 pi=36 형 띠는 `dump`/`dump-pages` 상 띠 zone 이 line0(16px)+line1(31px)=47px 로 잡혀 본문이 그만큼 더 내려감 → rhwp 2·3쪽 본문이 한컴보다 ~14px 아래(종전 #866 보고의 "~4.5~9px 초과"의 일부).

### `<...>` 소제목 ↔ 인접 본문 (2쪽 예)
| 구간 | rhwp SVG Δy | 한컴 PDF Δy(96dpi 추정) | 차이 |
|---|---|---|---|
| "저장하기"(파일 본문 끝) → "<미리 보기 상태에서>" | ~21px | ~32px | rhwp ~11px 부족 |
| "<미리 보기 상태에서>" → "편집 용지"(본문) | ~21px | ~40px | rhwp ~19px 부족 |
| "문서의 끝으로"(<미리보기> 본문 끝) → "편집" 띠 | (zone 전환) | ~94px | — (rhwp 측 별도 측정 필요) |
| "편집" 띠 → "되돌리기"(본문) | ~52px | ~37px | rhwp ~15px 초과 |

→ `<...>` 소제목 자체 zone(1단 ColumnDef 간격=0) 위·아래가 한컴 대비 ~11~19px 부족. "편집" 띠처럼 pi=36 형 띠는 line0 텍스트 흡수 문제로 오히려 ~15px 초과. 즉 **방향이 띠 유형별로 다름** — Stage 2 에서 (a) pi=36 형 띠 line0 텍스트 정리, (b) `<...>`·pi=1 형 띠 전환 간격 보강, 두 갈래로.

## 3. 점선 단 구분선 (Stage 3 대상)
- 1쪽 ColumnDef `구분선: type=3`(점선). rhwp SVG 에 단 사이 세로선 자체가 안 보임 → 미렌더. Stage 3 에서 점선 렌더 추가.

## 결론 (Stage 2 방향)
1. pi=36 형 헤더 띠 문단의 line0 텍스트("파일" 등) 별도 렌더 제거 — 한컴은 띠(표) 1개만 그림. 이걸 고치면 2·3쪽 본문 ~14px 초과가 줄고 "편집"↔"되돌리기" ~15px 초과도 해소.
2. pi=1 형 띠 / `<...>` 소제목 zone 의 위·아래 전환 간격 ~8~19px 보강 — 한컴은 띠/소제목 zone 앞뒤로 추가 여백을 둠. `build_columns` + `process_multicolumn_break` 의 zone 전환 간격 규칙 정밀화.
3. `LAYOUT_OVERFLOW` 0 확인 (1·2 정정 후 3쪽이 한컴처럼 `<그림 넣기에서>` 까지만).

## 미해결
- 한컴 PDF y 측정은 96dpi PNG 육안 추정 → ±3~5px 오차. Stage 2 진행 시 220dpi crop 으로 핵심 전환만 재측정 권장.
