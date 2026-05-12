# Stage 5 조사 보고 — Task #842 (M100) — 결함 #1 (헤더 1×1 TAC 표 앞뒤 spacing 압축)

상태: **조사 진행 — 미수정**. RFC #774 영역, 본질 정정 위험 큼. 작업지시자 판단 요청.

## PDF ↔ rhwp 정밀 비교 (shortcut.hwp 1페이지)

`pdftotext -bbox` (PDF, pt) + `mutool draw -r 150` (PNG 시각 확인):

| 요소 | PDF (pt → px ×1.335) | rhwp (px, SVG) | 차이 |
|------|---------------------|----------------|------|
| 본문영역 상단 | 15mm ≈ 56.7px | 56.7px | 0 |
| 제목 "흔글 2010 단축키 일람표" 텍스트 top | 62.65pt ≈ 83.6px | ~58px (baseline 79.4 − ascent) | rhwp ~25px 높음 |
| "커서 이동" 헤더 텍스트 (yMin~yMax) | 108.24~120.24pt ≈ 144~160px | bar rect 103.1~126.7px, 텍스트 baseline 121 | rhwp ~40px 높음 |
| "빈칸 삽입" 첫 본문행 텍스트 top | 145.93pt ≈ 194.8px | baseline 142 → top ~131px | rhwp ~64px 높음 |
| 헤더 바 아래 → 첫 본문행 간격 | ~34px (텍스트bottom→텍스트top) | ~15px (bar bottom→text baseline), 실질 ~4px (bar bottom→text top) | rhwp ~20~30px 부족 |
| 본문 행 pitch | 15pt ≈ 20px | ~20px | ≈ 동일 |

→ rhwp 의 콘텐츠가 위로 갈수록 누적적으로 위쪽으로 압축됨: 제목 위 ~25px 부족, 제목↔헤더 ~15px 부족, 헤더↔본문 ~20px 부족. 본문 행 pitch 자체는 정상. `mutool` 렌더 시각 확인 — 제목이 헤더 바에 거의 붙어 있고, 헤더 바 위·아래 여백이 PDF 대비 1/3 수준.

`dump-pages -p 0` 관찰: 단0(제목 zone) `used=69.1px hwp_used≈53.1px diff=+16px`(rhwp 16px 초과), 단2/3(본문) `used=186.7 hwp_used≈273.3 diff=−86.7px` — 단 `used` 값은 line-spacing gap 미포함으로 실제 렌더(SVG 본문 ≈ 280px ≈ hwp_used)와 불일치, 즉 본문은 정상. 제목 zone +16px / 헤더 바 주변 spacing 누락이 핵심.

## 미규명 — spacing 출처
헤더 문단(0.1): `spacing before=0 after=0 line=100%`, 표 outer_margin 1mm(≈1.9px), 셀 pad 0.5mm. 제목 문단(0.0): `spacing before=0 after=0 line=140%`. 본문 첫 문단(0.2): `spacing before=0` + `다단나누기` + 새 `2단 ColumnDef`. → 명시 spacing 어디에도 PDF 의 ~20px 가 없음. 한컴이 (a) zone 전환(1단↔2단) 시, 또는 (b) TAC 표 문단의 `line=100%`/표 높이 기반으로, 또는 (c) 제목/헤더 문단의 LINE_SEG vpos 해석으로 암묵 간격을 넣는 것으로 추정 — 정확한 규칙 미확정. 이게 닫힌 이슈 #770/#773/#776 + RFC #774("한컴 PDF paragraph spacing 알고리즘 정밀 분석")의 주제.

## 부수 발견 (별개 결함)
1. **제목 첫 글자 누락**: rhwp 가 "흔글 2010..." 을 "글 2010..." 로 렌더 — 제목 첫 글자 "흔"(PUA? 옛한글?) 이 빠짐. char run 분할/PUA 처리 의심.
2. **페이지 3 → 4 column-break 행 밀림** (`<편집 화면 분할에서>` 의 "화면 이동" 행): 작업지시자 언급. 닫힌 이슈 #768 과 동일 — 다단 zone 분할 결함, 별개 영역.

## 권고
결함 #1 은 RFC #774 분석을 끼고 다뤄야 안전 (zone 전환/TAC 표 spacing 본질 정정 — 메모리 `feedback_essential_fix_regression_risk`: 다단/단일 단/표분할 상호작용 회귀 위험 큼, 광범위 샘플 + 한컴 2010/2020 정답지 검증 필요). 현 타스크에서 임의 수정은 회귀 위험이 큼.

→ **본 타스크는 #4·#3·#2 (4건 중 3건, 완료·검증·커밋) 로 마무리** 하고, **#1(헤더 표 spacing/RFC #774) + 부수 발견 2건(제목 첫 글자 누락, 페이지 3→4 밀림) 을 별도 후속 이슈로 등록** 권고. #1 을 본 타스크에서 강행 시 회귀 위험을 감수해야 함.
