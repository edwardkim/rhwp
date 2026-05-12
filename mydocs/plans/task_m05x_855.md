# 수행 계획서 — Task #855

## 대상 이슈

[#855] 21_언어_기출_편집가능본.hwp 14p 우측 단: Square-wrap 표 뒤 문단(pi=300) 렌더링 누락

## 현상 요약

`samples/21_언어_기출_편집가능본.hwp` 14페이지 오른쪽 단(`단 1`) 하단부가 렌더링되지 않음.

`dump-pages -p 13` 결과 (단 1):
```
단 1 (items=8, used=919.0px, hwp_used≈1219.3px, diff=-300.3px)
  ...
  PartialParagraph  pi=299  lines=0..9  vpos=51892..66420
  Table             pi=299 ci=0  3x2  22.7x220.7px  wrap=Square tac=false
  PartialParagraph  pi=301  lines=0..1  vpos=90345..0 [vpos-reset@line1]
```
- 문단 `pi=300` ("최근에는 기존의 법학방법론적 논의와…") 이 페이지 레이아웃 결과에서 **통째로 누락**.
- 페이지 15 첫 항목은 `pi=301 lines=1..22` → `pi=300` 은 다음 페이지로 넘어간 것도 아님 (완전 소실).
- `단 1` 누락 높이(약 300px)가 `pi=300` 분량(line seg 12개, 약 58mm)과 일치.

## 1차 조사 (코드 미수정)

- `pi=299` 에 `wrap=어울림(Square)`, `쪽나눔=RowBreak`, 크기 6.0×58.4mm 인 3×2 표가 문단 기준 위치(세로 오프셋 1.7mm)로 앵커되어 있음.
- 이 표는 `pi=299` 의 9개 라인(약 51mm)보다 길어서(58mm) 표 하단이 `pi=300` 의 첫 라인(vpos 68236)보다 약간 아래(≈68442)까지 내려옴.
- 라인 세그먼트 자체는 정상: `pi=300 ls[0]` 만 표를 피해 narrow(`sw=27581`), 나머지는 full(`sw=30184`).
- **추정 원인**: Square-wrap 개체가 앵커 문단보다 아래로 확장될 때, 레이아웃이 그 개체의 하단 y를 커서로 잡고 "그 y보다 위쪽 vpos 를 가진 문단"을 이미 배치된 것으로 간주해 건너뛰는 것으로 보임 → `pi=300`(시작 vpos 68236 < 표 하단 68442)이 스킵, `pi=301`(vpos 90345 > 68442)은 정상 배치.

## 작업 범위

1. 레이아웃(`src/renderer/layout.rs` 등) 에서 Square/어울림 wrap 개체 처리 후 다음 문단 진입 로직 정밀 조사 — 어디서 `pi=300` 이 누락되는지 확정.
2. 원인 지점 수정 (개체 하단 y 와 무관하게 후속 문단은 정상적으로 큐에서 소비되도록).
3. 회귀 검증: `dump-pages -p 13/14`, `export-svg -p 13` 으로 `pi=300` 렌더링 확인. `pdf/21_언어_기출_편집가능본-2022.pdf` 14페이지와 시각 대조.
4. 다른 샘플 회귀 없음 확인 (`cargo test`, 주요 샘플 SVG diff 스팟체크).

## 산출물

- 구현 계획서: `mydocs/plans/task_m05x_855_impl.md`
- 단계별 완료 보고서: `mydocs/working/task_m05x_855_stage{N}.md`
- 최종 보고서: `mydocs/report/task_m05x_855_report.md`

## 브랜치

`local/task855` (from `local/devel`)

---

승인해 주시면 구현 계획서 작성으로 넘어가겠습니다.
