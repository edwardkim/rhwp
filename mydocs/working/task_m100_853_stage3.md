# Stage 3-1 조사 보고 — Task #853 (M100) — `다단나누기` 구분 칸 band 간격 + 3쪽 overflow

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` · 상태: **조사 — 미수정, 설계·승인 대기**

## 측정 (shortcut.hwp 2쪽, `pdf/basic/shortcut-2022.pdf` ↔ rhwp SVG `output/svg/sc853/shortcut_002.svg`)

`mutool draw -r 100` PNG 픽셀 측정(@96dpi 환산) ↔ SVG 좌표:

| 요소 | 한컴 PDF (body_top=56.7px 기준) | rhwp | 차이 |
|------|------------------|------|------|
| "파일" 헤더 띠(표) 상단 | +19.1px (5.1mm) | +3.8px | rhwp ~15px 높음 |
| 본문 첫 줄 "새 문서" 상단 | +134.3px (35.5mm) | ~+27px | rhwp **~107px 높음** |
| 띠 ↔ 본문 사이 간격 | ~92px | ~2px | rhwp **~90px 부족** |

→ 사용자 보고("모든 구분 칸 위·아래 줄 간격 좁음")의 실체: 헤더 띠 자체가 ~15px 위로 + 띠↔본문 사이가 ~90px 부족. 1쪽 stage5 측정(헤더↔본문 ~20px)보다 2쪽은 훨씬 큼 — 2쪽은 `쪽나누기`로 시작하는 페이지.

## IR 구조 (pi=36 "파일" 헤더 문단)

```
--- 문단 0.36 --- cc=19, text_len=2("파일"), controls=2 [쪽나누기]
  [PS] ps_id=2 align=Justify spacing: before=0 after=0 line=100/Percent  margins: left=0 right=2000 bf=3
  ls[0]: ts=0,  vpos=0,    lh=1200, th=1200, bl=1020   ← 텍스트 줄 (16px)
  ls[1]: ts=10, vpos=1200, lh=2332, th=2332, bl=1982   ← 표 줄 (31px = 표 1766 + outer_margin 283×2)
  [0] 단정의: 1단, 유형=일반, 간격=0.0mm
  [1] 표: 1행×1열, 쪽나눔=RowBreak, padding=(283,283,283,283), size=69448×1766(245×6.2mm), bf=4
       outer_margin (283,283,283,283)=1mm, 셀[0] paras=1 text="파일"
```
- 한컴은 pi=36 을 **line0=텍스트("파일", 16px) → line1=표(31px)** 순으로 배치(총 47px). rhwp 는 표를 line0 에 놓고 텍스트 line0 을 흡수 → ~27px (15~20px 부족).
- 표 셀 안에도 "파일" 이 들어 있어 PDF 상 띠에 "파일" 1개만 보이지만, 문단 텍스트 "파일"(line0)은 띠 위쪽 16px 줄에 별도로 흐른다(시각적으로는 띠와 겹치거나 거의 붙음).

## 미규명 — 띠↔본문 ~92px 간격의 출처

pi=36 의 LINE_SEG 2 줄(16+31=47px)로는 ~92px 가 설명되지 않는다. pi=36 과 pi=37("새 문서") 사이에 ① 빈 문단/추가 LINE_SEG, ② 1단 zone → 2단 zone 전환 시 한컴이 두는 고정 간격, ③ `쪽나누기` + `다단나누기` 조합의 누적 offset 해석, ④ TAC 표 `wrap=위아래`(TopAndBottom)가 글자처럼 취급이면서도 위아래 어울림으로 예약하는 추가 높이 중 하나로 추정 — 정확한 규칙 미확정. RFC #774("한컴 paragraph/zone spacing 알고리즘 정밀 분석") 영역.

## 3쪽 overflow

3쪽 단3 `<편집 화면 분할에서>`(pi=94)·"화면 이동"(pi=95) 둘 다 `vpos=0` 겹침 — 닫힌 #768 패턴. 위 zone-transition 규칙 규명에 종속될 가능성 큼.

## 권고

Stage 3 구현 전 **RFC #774 분석 문서 선행** 권고 — pi=36 의 line0/line1 배치 + zone 전환 간격(~92px) + TAC `wrap=위아래` 예약 + `쪽나누기` 누적 offset 을 PDF·IR 대조로 명세화한 뒤 composer/`layout_table_item`/`build_columns` 를 정정해야 안전(`feedback_essential_fix_regression_risk` — composer 변경은 전 문서 회귀 위험). 추측 구현 시 광역 회귀 위험 큼.

판단 요청:
- A. RFC #774 분석 문서(`mydocs/tech/`)를 본 타스크에서 작성 → 승인 → 구현
- B. composer 의 헤더 띠 line0 텍스트 렌더만 우선 시도(~15px 회복, 부분) + 나머지(~90px gap)는 후속 — `cargo test` 가드
- C. Stage 3 보류, 본 타스크는 Stage 2(제목 정정)로 마무리 + #853 에 band/overflow 잔존 기록

(현재까지 커밋: Stage 2 제목 정정 + golden 2건 갱신 — `f0d34713`. 소스 추가 변경 없음.)
