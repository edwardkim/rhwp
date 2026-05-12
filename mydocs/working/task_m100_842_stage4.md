# Stage 4 완료 보고서 — Task #842 (M100) — 결함 #2 (헤더 바 좌측 위치)

목표: 페이지 2~8 섹션 헤더 바(1×1 TAC 표)가 +28px 우측으로 밀리는 문제 수정.

## 원인
- 헤더 바 1×1 표는 `is_tac_table_inline()` 가 **false**(폭 ≈ 단 폭) → 블록 취급 → `PageItem::Table` → `layout_table_item` 의 `is_tac` 분기 → 표 x = `col_area.x + effective_margin + leading`, `leading = compute_tac_leading_width(...)`.
- 페이지 2 헤더 문단(0.36)은 LINE_SEG 가 2개: `ls[0]` = 텍스트 "파일", `ls[1]` = TAC 표(자체 줄). `compute_tac_leading_width` 는 `composed.lines.first()`(= line 0 = "파일")의 run 폭을 전부 leading 으로 합산 → 표가 width("파일") ≈ 28px 만큼 우측 이동. 페이지 1 헤더 문단(0.1)은 빈 문단이라 line 0 폭 = 0 → leading=0 → 정상.

## 수정
`src/renderer/layout.rs` `layout_table_item`, `is_tac` 분기:
- 문단이 여러 줄(`composed.lines.len() > 1`)이고 **line 0 에 alphanumeric 글자**(한글 음절/라틴/숫자/한자 등 — `char::is_alphanumeric()`)가 있으면 → 표는 line 0 텍스트 *다음* 이 아니라 자체 줄 좌측에서 시작하므로 `leading = 0`.
- 그 외(빈 문단, 또는 line 0 이 HWP TAC 필러 `U+F081C`/`U+F012B` 등 PUA·공백뿐인 경우 — 예 복학원서.hwp pi=16, 한컴이 표 폭만큼 필러를 채워 줄바꿈시킨 케이스)는 종전대로 `compute_tac_leading_width` 사용.
- `is_alphanumeric()` 판정으로 PUA 필러는 자동 제외(PUA = Letter/Number 아님). 이전 시도(특정 필러 코드포인트 blocklist)는 `U+F012B` 같은 추가 필러를 놓쳐 복학원서 회귀가 났음.

## 결과
- shortcut.hwp 헤더 바 페이지 1~8 전부 rect x = 94.5 (= body 좌측, 페이지 1과 동일).
- 복학원서.hwp pi=16 표는 종전대로 leading 유지 (`issue_677_bokhakwonseo_page1` snapshot 통과).
- `cargo test` 전건 통과 (8/8 svg_snapshot 포함, 34개 test result ok, exit 0).

## 부수 관찰 (미해결, 별개)
- 페이지 2 헤더 문단(0.36)에는 표(ls[1]) 외에 line 0 = "파일" 텍스트가 black 으로 별도 렌더되는지(헤더 바의 white "파일"과 이중 표시) — PDF 는 "파일" 1회뿐. 확인 필요하나 본 결함과 별개. → 후속.

다음: Stage 5 — 결함 #1 (헤더 1×1 TAC 표 앞뒤 spacing 압축).
