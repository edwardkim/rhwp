# #1950 Stage 1 — 원인 분해 + 정답 확정

- 브랜치: `local/task1950` / 방법: render-diff·convert·ir-diff·export-render-tree 실측 +
  #1915 secd 임시 비활성 대조(측정 후 원복)

## 1. 결론 — #1915 무관, 7건은 **이질적 군집**(단일 원인 아님)

### 1-1. #1915(secd) 는 원인이 아니다 (디커플)
`RHWP_NO_SECD` 로 #1915 secd 삽입을 끈 뒤 render-diff 재측정: 2955331 OVER 376px,
15047877 PAGE_MISMATCH **secd ON/OFF 완전 동일**. → **#1915 와 상충 없음**(계획서의
"#1915 재설계 수반" 우려 해소).

### 1-2. 7건은 서로 다른 기전
| 파일 | 변위 | 기전(실측) |
|---|---|---|
| **2955331** | **376px** | **탭(TAB) run 분할** — 유일한 유의 결함 |
| 15047877 | PAGE 1→2 | 탭 없음. run 46→45(1개 소실) razor-thin 페이지 경계 |
| 2957835/2943007/2912183/20054041/20335533 | 1~13px | razor-thin 줄/행높이 드리프트(#1759/#1842 계열) |

## 2. 유의 결함(2955331) 정밀 원인 — 직렬화기의 탭 문단 char_count 팽창

- export-render-tree TextRun 대조: 원본 138 run vs 변환 139 run. 첫 발산 idx=137:
  - 원본: `x=123.2 '…4탭'`(탭 4개 한 run)
  - 변환: `x=123.2 '…3탭'` + 별도 run(탭 1개 분리) → **탭 run 이 3+1 로 쪼개져 376px 변위**.
- char 정합 실측: **원본 IR 은 문단 8개 전부 `cc==text_len`(정합)**. 그러나 **변환·재파스
  HWP5 는 controls=0 인데 `cc!=text_len` 인 문단 4개**(탭 보유 문단). 즉 **HWP5 직렬화기가
  HWP3-origin 탭 문단의 char_count 를 팽창**(cc>text_len)시키고, 이 팽창이 재파스 렌더에서
  탭 run 분할을 어긋내 376px 를 만든다.
- char_shapes 자체는 원본/변환 동일(pos 0,31), LINE_SEG 도 동일 → 원인은 **직렬화 char
  스트림(탭 표현)의 char_count 계약 위반**이지 char_shape/lineseg 손상이 아니다.

## 3. 정답 기준

원본 HWP3 IR 렌더가 정답(탭 4개 한 run). 변환 후에도 char_count 계약(cc==text_len,
control-free)이 지켜지면 탭 run 이 동일하게 유지되어 정합. → 직렬화기가 HWP3-origin 탭
문단을 원본과 동일한 char 스트림으로 쓰도록 정정.

## 4. Stage 2 제안 (승인 요청)

- **범위 집중**: 유의 결함 **2955331(탭 run 376px)** 를 대상으로 한다. 직렬화기가 탭 문단
  char_count 를 팽창시키는 지점(탭 char 표현/char_shape 오프셋 쓰기)을 특정하고, cc==text_len
  계약을 지키도록 최소 수정. → 탭 run 유지 → 376px 해소.
- **razor-thin 6건(15047877 포함)은 별개 축**(#1759/#1842 줄/행높이·페이지 경계). 본 이슈
  범위에서 분리(별도 이슈/후속) 제안 — 단일 수정으로 묶기 어렵고 값이 낮다.
- 검증: 2955331 render-diff PASS, 원본 HWP3 렌더 불변, hwp5/hwpx·hwp3 표본 무회귀.

## 5. 리스크

- 직렬화기 탭 char_count 수정은 HWP5 파일 계약에 영향 → hwp5-roundtrip·render-diff 양
  게이트로 검증. HWP3-origin 한정 분기가 필요하면 CLAUDE.md(HWP3 로직 parser/hwp3) 고려 —
  단 char_count 팽창은 공통 직렬화기 결함일 수 있어 Stage 2 에서 위치 확정.

→ Stage 2(2955331 탭 char_count 팽창 지점 특정 + 수정 설계) 진행 승인 요청. razor-thin 6건은
본 이슈에서 분리 제안.
