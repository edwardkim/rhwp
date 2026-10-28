# issue6448 — HWPX CELL/TAC 누락 host band

`tac_cell_leftover_fits.hwpx` 는 `samples/task2097/rowbreak_midpage_declared_fits.hwpx`의
표 `treatAsChar="1"` 판이다. HWPX `pageBreak="CELL"`은 모델에서 `RowBreak`이며,
`repeatHeader="0"`인 3행 x 1열·3셀 표의 선언 높이는 68000HU지만 빈 host에는
1000HU `LINE_SEG` 하나만 저장되어 있다.

Hancom 2020 기준 PDF는 `HEAD LINE BEFORE TABLE`을 1쪽, 표를 2쪽,
`AFTER TABLE`을 3쪽에 둔다. 짧은 host 줄을 표 물리 높이로 오인하면 표와 TAIL이
같은 쪽에 과밀 배치되는 회귀를 재현한다.
