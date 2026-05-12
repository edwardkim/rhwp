# Stage 4 중간 보고 — Task #842 (M100) — 결함 #2 (헤더 바 좌측 위치)

상태: **조사 진행 중 — 미완료**. 본질 정정 위험군이라 작업지시자 판단 요청.

## 증상 재확인 (수정 전·후 동일)
- 페이지 1 `커서 이동` 헤더 바(1×1 TAC 표) rect x ≈ 94.5px (body 좌측 = 정상).
- 페이지 2~8 헤더 바(`파일`/`보기`/`입력`/`서식`/`기타` 등) rect x ≈ 122.5px → +28.0px 우측 이동.
- 본문 텍스트 x0 는 두 페이지 모두 ≈ 121.2px (동일, 정상).
- 페이지 1 body-clip width ≈ 933.5px, 페이지 2~8 ≈ 953.97px (+20.5px). 헤더 바 width 는 두 페이지 모두 925.97px (표 size 69448 HU 고정).
- 페이지 2: 헤더 x(122.5) + width(926) = 1048.5 = body-clip right(94.5+954). 즉 헤더가 body 우측 끝에 우측 맞춰진 모양 (28 = 954 − 926).

## IR 차이 (원인 후보)
페이지 1 헤더 문단 0.1 vs 페이지 2 헤더 문단 0.36:

| 항목 | 0.1 (page1) | 0.36 (page2) |
|------|-------------|--------------|
| 직전 컨트롤 | 구역나누기 + 자체 다단나누기 | **쪽나누기** |
| ColumnDef | 1단, 간격=10mm | 1단, 간격=0mm |
| para text | `(빈 문단)` text_len=0 | **`"파일"` text_len=2** |
| LINE_SEG | `ls[0]` 1개 (lh=2332, 표) | **`ls[0]` lh=1200(텍스트?) + `ls[1]` ts=10 lh=2332(표)** 2개 |
| ParaShape margins | left=0 right=2000 | left=0 right=2000 (동일) |
| 표 outer_margin/size | 1mm / 69448 HU | 1mm / 69448 HU (동일) |

→ 페이지 2 헤더 문단은 `쪽나누기` 후 새 1단 ColumnDef 로 시작 + 문단이 두 LINE_SEG (텍스트 줄 + 표 줄) 를 가짐. 표는 `ls[1]` (ts=10 HU ≈ 0.13px) 에 위치. ParaShape margins, 표 속성은 페이지 1 과 동일하므로 +28px 는 **ParaShape/표 속성 출처가 아님**.

가설(미검증): 2단 zone 내부에서 `쪽나누기` → 새 페이지 진입 시 새 `1단` ColumnDef 가 적용되는데, rhwp 의 column-area 재계산이 직전 2단 zone 의 좌측 컬럼 영역(또는 잔여 컬럼 offset)을 끌고 들어가 새 1단 area 의 좌측 시작이 ~28px 안쪽으로 잡히는 것으로 추정. (28px ≈ body-clip 폭 차이 20.5px + α — 정확한 산식 미확정.) 추가로 `comp_line.runs.is_empty()` 가 아닌(text "파일") 케이스라 TAC 표가 `run_tacs` 경로(`layout_table(..., Some(x), ...)`, paragraph_layout.rs:~2009)로 그려지며, 이때 `x = x_base + inline_offset + num_x_offset`, `x_base = effective_col_x + effective_margin_left`. `effective_margin_left = margin_left(0) + line_indent(ls.ts→~0)` ≈ 0 이어야 하나 실제 122.5 → `effective_col_x` 또는 `inline_offset` 에 ~28px 가 섞임.

## 미확정 사항 / 다음 단계
1. `RHWP_LAYOUT_DEBUG` 또는 임시 로깅으로 페이지 2 헤더 문단의 `col_area.x` / `effective_col_x` / `effective_margin_left` / `inline_offset` 실측 → 28px 의 출처 확정.
2. 출처가 column-area 재계산이면 해당 경로(page-break-then-ColumnDef in multi-column zone) 수정. → 다단 zone/표분할 상호작용 회귀 광범위 검증 필수 (메모리 `feedback_essential_fix_regression_risk`).
3. 페이지 2 헤더 문단의 para text `"파일"` (ls[0]) 가 표와 별개로 black 텍스트로 렌더되는지(이중 표시) 확인 — PDF 는 `파일` 1회뿐. 별도 결함 가능.

## 원인 추가 확정 (2026-05-12 후속 조사)
- 헤더 바 1×1 표는 `is_tac_table_inline()` 가 **false**(폭 ≈ 단 폭) → 블록 취급 → `PageItem::Table` → `layout_table_item` → `is_tac` 분기 → 표 x = `col_area.x + effective_margin + leading`, `leading = compute_tac_leading_width(...)`.
- 페이지 2 헤더 문단(0.36)은 text "파일" + TAC 표 가 별도 LINE_SEG 2개로 들어옴. `compute_tac_leading_width` 가 `composed.lines.first()` 전체(= "파일", ~28px)를 leading 으로 합산 → 표가 +28px 우측 이동. 페이지 1 헤더 문단(0.1)은 빈 문단이라 leading=0 → 정상.

## 수정 시도 2회 — 모두 회귀 (`issue_677_bokhakwonseo_page1` snapshot)
1. `compute_tac_leading_width` 에서 `composed.lines.len() > 1` 이면 0 반환 → 복학원서.hwp 표가 7px 좌측 이동(56.7 vs golden 63.7) 회귀.
2. `compute_tac_leading_width` 를 "표가 놓인 줄에서만 선행 폭 합산" 으로 재작성(`tac_char_pos` 전달) → 동일 회귀.
3. 호출 측에서 `lines.len() > 1 && line0 에 실제 텍스트` 일 때 leading=0 으로 우회 → 동일 회귀 (복학원서 표 para 가 예상과 달리 `lines.len() > 1` 로 판정되는 듯 — 미규명).

→ 복학원서.hwp 의 TAC 표 레이아웃이 헤더 바와 같은 코드 경로를 공유하며 `compute_tac_leading_width` 가 그 7px offset 에 의존. 둘을 안전하게 분리하려면 복학원서 표 para 의 composed line 구조 / 7px 출처를 먼저 규명해야 함 (시간 소요 큼, 회귀 위험 큼).

## 권고 (갱신)
**#2 와 #1 을 별도 후속 이슈로 분리** 하고, 본 타스크는 **#4(우측탭 정렬, 완료·검증·커밋) + #3(단 구분선 점선, 완료·검증·커밋)** 으로 마무리(최종 보고서 + merge) 하는 것을 권고. #2/#1 은 layout 본질 정정으로 다단/표분할/TAC 표 상호작용 회귀 위험이 커 정밀 검증과 함께 다뤄야 함(메모리 `feedback_essential_fix_regression_risk`).
