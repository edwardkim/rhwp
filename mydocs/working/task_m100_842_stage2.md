# Stage 2 완료 보고서 — Task #842 (M100)

목표: 결함 #4 — cross-run 우측탭 정렬에서 탭 다음 콘텐츠가 여러 composed run 으로 쪼개진 경우의 오버플로 수정.

## 변경

`src/renderer/layout/paragraph_layout.rs`:
- `right_tab_block_width()` 헬퍼 추가 — 탭 직후 run 부터 `\t` 를 포함하지 않는 연속 run 들의 `estimate_text_width` 합산.
- est 패스 / render 패스의 cross-run 우측·가운데 탭 정렬 시작 x 계산을, 단일 run 폭 → 블록 전체 폭(`right_tab_block_width`) 기준으로 변경.
- est 패스 run 루프를 `enumerate()` 로 변경 (`run_idx_est` 필요).

## 결과

shortcut.hwp 8페이지 우측탭 정렬:
- `Ctrl+(회색)5` 류(스크립트 경계로 `["Ctrl+(", "회색)", "5"]` 분할): `5` 우측 끝 1005px → 967px 로 교정. 페이지 1 우측열 16개 항목 정렬 폭 [966.0, 967.3] 으로 수렴.
- `(회색)+/-`, `(쉼표)` 등 한 run 으로 합쳐지는 항목: 기존대로 정상(변화 없음).
- 회귀: `cargo test` 전건 통과 (8/8 svg_snapshot 포함).

## 미해결 잔여 (결함 #4 일부 — `Alt+P/Ctrl+P` 계열)

증상: char-shape 경계가 `\t` **앞**에 놓여 run 이 `\t` 로 시작하는 항목(`끝`(id7) + `\tAlt+X`(id8) 등 — 단축키 우변이 별도 bold run 이고 그 run 이 `\t` 로 시작) 은 cross-run 핸들러("`\t` 로 끝나는 run 의 다음 run 정렬")가 트리거되지 않는다. 이 경우 in-run 탭 처리(`compute_char_positions`)가 `compute_char_positions`/`available_width` 기준으로 우측 정렬하나 실제 정렬 위치가 ~28~32px 우측으로 어긋남(`Alt+X`/`Alt+F9`/`Ctrl+W,H`/`Ctrl+K,M`/`Alt+Shift+*` 등 ASCII 로 끝나는 단축키).

시도한 수정 — composer 단계에서 선행 `\t` 를 직전 run 끝으로 이동 정규화(`split_runs_by_lang` 후처리): shortcut.hwp 는 정합되나 **KTX.hwp 목차 / aift.hwp 3페이지 svg_snapshot 회귀**(목차 leader-dot run 이 `\t` 로 시작하는 케이스를 잘못 병합 → leader 추출/우측정렬 깨짐). 메모리 룰(`feedback_essential_fix_regression_risk`)에 따라 폐기.

→ 별도 처리 필요. paragraph_layout 의 run 루프에서 "`\t` 로 시작하는 run" 을 "직전 run 이 `\t` 로 끝난 것"과 동등하게 다루되 leader run 케이스를 회피하는 핀포인트 수정이 필요. Stage 2b 또는 후속 이슈로 분리.

## 산출물
- `output/svg/task842_s2c/shortcut_00{1..8}.svg` — Stage 2 적용 후 SVG.

다음: (논의 후) Stage 2b — `Alt+P/Ctrl+P` 계열 잔여, 또는 Stage 3 — 단 구분선 점선.
