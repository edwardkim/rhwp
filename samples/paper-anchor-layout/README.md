# 종이·쪽 기준 개체 배치의 합성 입력

공개 빈 문서 [`samples/hwpx/ref/ref_empty.hwpx`](../hwpx/ref/ref_empty.hwpx)(A4, 본문 폭 42520 HU)에서
[`scripts/generate_paper_anchor_layout_fixtures.py`](../../scripts/generate_paper_anchor_layout_fixtures.py)
로 만든 독립 입력이다. `Contents/section0.xml` 만 새로 쓰고, 그림 입력에는 생성기가 만든 2×2 회색 PNG 하나를
넣는다. 사용자 서식·실제 문서의 글·그림은 포함하지 않는다. 생성기는 표준 라이브러리만 쓰고 ZIP 시각을
1980-01-01 로 고정하므로 다시 실행하면 같은 바이트가 나온다(`MANIFEST.json` 의 sha256).

이 입력들은 **조판 기하 계약**이며 한컴 PDF 일치 증거가 아니다. 기대 동작의 근거가 된 한글 출력 실측은
비공개 서식에서 얻은 것이라 저장소에 넣지 않는다 — 여기서는 그 형상만 합성으로 재현한다.

## `square-table-next-line.hwpx` — 옆 레인이 없는 종이 기준 어울림 표

- 문단 0 `Heading` · 문단 1 빈 앵커 문단 · 문단 2 `Next paragraph`.
- 문단 1 의 2행×1열 표: `textWrap=SQUARE`, `treatAsChar=0`, `vertRelTo=PAPER`·`horzRelTo=PAPER`,
  `vertAlign=TOP`, 세로 12000 HU · 가로 8504 HU(= 본문 왼쪽), 폭 42520 HU(= 본문 폭), 바깥 여백 0.
  그래서 좌·우 레인이 모두 0px 이고 글이 표 옆으로 흐를 수 없다.
- 저장 줄은 흐름 그대로(문단 1 vpos 1600, 문단 2 vpos 3200)다.
- 계약: 다음 문단 `Next paragraph` 는 표 바닥 **아래**에서 시작한다.
  수정 전 rhwp: 다음 문단 y=174.9px, 표 y=160.0..240.0px — 표 첫 행 위에 겹쳤다.
  수정 후: 다음 문단 y=261.3px.

## `cell-picture-page-paper.hwpx` — 칸에 앵커된 종이·쪽 기준 그림

- 글자처럼 놓인 1행×2열 표(칸 폭 21260 HU). 칸 (0,0) 문단에 종이 기준 그림, 칸 (0,1) 문단에 쪽 기준 그림.
- 두 그림 모두 `textWrap=IN_FRONT_OF_TEXT`, `treatAsChar=0`, `flowWithText=0`, 3000×3000 HU, 정렬 TOP/LEFT.
  종이 기준 오프셋 (가로 15000, 세로 30000) HU, 쪽 기준 오프셋 (가로 6000, 세로 24000) HU.
- 계약: 종이 기준 그림은 용지 원점 + 오프셋 = (200.0, 400.0)px, 쪽 기준 그림은 본문 원점 + 오프셋 =
  (본문 x + 80.0, 본문 y + 320.0)px 에 놓인다(허용 0.5px). 칸 시작점이 기준 원점과 떨어져 있어
  칸 기준으로 재면 크게 어긋난다.
  수정 전 rhwp: 종이 기준 그림 (313.4, 555.5)px — 칸 시작점 (113.4, 153.6)에서 오프셋을 한 번 더 더했다.

## `paper-table-fallback-padding.hwpx` — 대체 칸 여백을 가진 종이 기준 표

- 종이 기준(세로 20000 · 가로 8504 HU) 자리차지 3행×1열 표, 행마다 선언 높이 1800 HU(24.0px).
- 표 안 여백(`hp:inMargin`) 네 축 0(= 미지정), 칸은 `hasMargin="0"`, 저장 `cellMargin` 위·아래 850 HU.
  이 칸 여백은 저장값이 아니라 #2195 의 «표 여백 전축 0 = 미지정» 대체값이다.
- 칸마다 저장 줄 하나(1000 HU)가 선언 높이 안에 든다.
- 계약: 행 높이는 선언 24.0px 그대로다(허용 0.5px).
  수정 전 rhwp: 행마다 36.0px(= 줄 13.3 + 대체 여백 11.3×2) — 떠 있는 표가 선언보다 커졌다.

## `residual-cell-padding-pair.hwpx` — 잔재 칸 여백 쌍을 가진 흐름 표

- 문단 기준(`vertRelTo=PARA`·`horzRelTo=COLUMN`) 자리차지 1행×1열 표, 선언 높이 1740 HU(23.2px).
  종이·쪽 기준이 아니므로 위 대체 여백 규칙이 아니라 칸 여백 선택 규칙만 검사한다.
- 표 안 여백 네 축 0, 칸은 `hasMargin="0"`, 저장 `cellMargin` 위 20424 · 아래 1287 HU.
  위 축은 위생 한도(2500 HU)에 걸려 이미 버려진다. 한 축이 잔재면 그 쌍은 같은 잔재다.
- 계약: 행 높이는 선언 23.2px 그대로다(허용 0.5px).
  수정 전 rhwp: 30.5px(= 줄 13.3 + 살아남은 아래 축 17.2).

검사: [`tests/cases/paper_anchor_float_reference.rs`](../../tests/cases/paper_anchor_float_reference.rs)
(앞 두 입력), [`tests/cases/paper_anchor_table_row_height.rs`](../../tests/cases/paper_anchor_table_row_height.rs)
(뒤 두 입력).
