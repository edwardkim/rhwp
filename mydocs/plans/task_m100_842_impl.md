# 구현계획서 — Task #842 (M100)

대상: shortcut.hwp PDF 정합성 잔여 결함 4건 (Issue edwardkim/rhwp#842)
브랜치: `local/task842`

회귀 위험이 가장 큰 #1(헤더 표 spacing, layout 본질 정정)을 마지막에 배치. 작은 위험 → 큰 위험 순.

---

## Stage 1 — 진단 및 회귀 기준 고정

목표: 4건 각각의 IR/레이아웃 근거를 확정하고, 수정 전 기준 산출물을 캡처한다. **소스 수정 없음.**

작업:
- `rhwp export-svg samples/basic/shortcut.hwp -o output/svg/task842_before/` — 8페이지 기준 SVG.
- 결함 #4: `dump -s 0 -p {해당}` 으로 `Ctrl+(회색)5`, `Alt+P/Ctrl+P` 단락의 char-shape run·tab_def 확인. composer가 해당 run 을 스크립트 경계로 쪼개는지 로그/코드로 확정.
- 결함 #3: ColumnDef 파싱 결과(`src/parser/.../column*` 및 IR)에서 단 구분선 종류 필드가 존재/보존되는지 확인. 없으면 파싱 추가 필요 범위 식별.
- 결함 #2: 파일·편집·보기·입력·서식·기타 섹션 본문 첫 단락 `dump -s N -p M` → ParaShape margins, 소속 단/zone, 헤더 표와의 관계 확인. PDF 와 left x 차이를 수치로 기록.
- 결함 #1: `dump-pages -p {각 헤더 페이지}` 로 헤더 표 앞뒤 단락 간격 측정, PDF 대비 부족분 수치화. 구 #770/#773/#776/#774 문서 재확인.
- 회귀 비교 대상 샘플 목록 확정(다단/표분할/목차 류 — `samples/` 내).

산출물: `mydocs/working/task_m100_842_stage1.md` (진단 결과 + 원인 확정 + 수정 범위 + 회귀 대상 목록).

---

## Stage 2 — 결함 #4: cross-run 우측탭 폭 합산 수정

목표: 우측/가운데 탭 뒤 콘텐츠가 여러 composed run 으로 쪼개져도 단 우측 끝에 정확히 정렬되도록 한다.

작업:
- `src/renderer/layout/paragraph_layout.rs` render 패스(1419~1480행 부근, est 패스 992~1069행도 동일 처리):
  - 탭 뒤 첫 의미있는 run 부터 **다음 탭 또는 줄끝까지** 의 composed run 들의 폭을 합산하여 정렬 시작 x 산출.
  - 빈 공백 run carry-over 로직과 일관되게 유지(공백 run 은 합산 단위 포함 여부 검토 — 한컴 동작 기준).
  - leader end_x 보정 로직이 합산 폭 기준으로 동작하도록 조정.
- est 패스(높이 측정)와 render 패스가 동일 규칙을 쓰는지 확인.

검증: shortcut.hwp 8페이지에서 `Ctrl+(회색)5`, `Alt+P/Ctrl+P`, `(회색)+/-`, `Shift+(회색)+/-`, `Ctrl+(회색)+`, `Ctrl+(회색)-` 등 모든 혼합 스크립트 우측탭 항목이 일반 항목과 같은 우측 끝(±1px). 회귀: 목차 류 우측탭(페이지번호) 샘플 SVG diff 무변화. `cargo test`.

산출물: `mydocs/working/task_m100_842_stage2.md`.

---

## Stage 3 — 결함 #3: 단 구분선 점선 반영

목표: 다단 구분선을 ColumnDef 에 지정된 선 종류(점선 등)로 렌더링한다.

작업:
- (Stage 1 결과에 따라) ColumnDef 파싱에 구분선 종류 필드 보존 추가 — HWP5/HWPX 양쪽.
- 레이아웃→렌더 노드로 구분선 종류 전달.
- SVG/렌더러에서 선 종류 → `stroke-dasharray` 매핑(점선/파선/실선 등). SVG export 에 단 구분선이 누락돼 있다면 함께 추가.

검증: shortcut.hwp 다단 페이지에 PDF 와 동일한 점선 세로 구분선. 회귀: 실선 구분선 사용 다단 샘플 무변화.

산출물: `mydocs/working/task_m100_842_stage3.md`.

---

## Stage 4 — 결함 #2: 섹션 본문 좌측 여백 정정

목표: 파일·편집·보기·입력·서식·기타 섹션 본문의 좌측 들여쓰기를 PDF 와 일치시킨다.

작업:
- Stage 1 에서 확정한 원인에 따라 수정:
  - (a) ParaShape `margins.left` 적용 기준이 단 안쪽이어야 하는데 본문 영역 기준이면 → 단(column) 기준으로 보정.
  - (b) 다단 zone 진입 첫 단락 들여쓰기 처리 차이면 → 해당 경로 수정.
- 룰/휴리스틱 구분(메모리 `feedback_rule_not_heuristic`): HWP 명세상 기준이 명확하면 단일 룰로, 분기 도입 전 자문.

검증: 해당 6개 섹션 본문 left x 가 PDF 와 일치. 회귀: 단일 단 문서 + 다른 다단 문서 본문 들여쓰기 무변화.

산출물: `mydocs/working/task_m100_842_stage4.md`.

---

## Stage 5 — 결함 #1: 헤더 1×1 TAC 표 앞뒤 단락 간격 보정

목표: 섹션 헤더 바 위/아래 간격을 한컴 PDF 와 일치시킨다. **layout 본질 정정 — 회귀 위험 최고, 광역 검증 필수.**

작업:
- Stage 1 진단 + RFC #774 분석 기반으로 TAC 1×1 표 앞뒤 단락 간격(before/after spacing) 보정 규칙 구현.
- 메모리 `feedback_essential_fix_regression_risk`: 다단/단일단/표분할 상호작용 회귀 광범위 검증.

검증: shortcut.hwp 8페이지 전 헤더 위아래 간격이 PDF 와 일치(±수 px). 회귀: TAC 표 포함 샘플 전수 + 표분할 샘플 + 한컴 2010/2020 정답지 대비 비교(가능 범위). `cargo test` 전건.

산출물: `mydocs/working/task_m100_842_stage5.md`.

---

## Stage 6 — 종합 검증 및 최종 보고서

작업:
- `cargo test` 전건 통과, `cargo clippy --all-targets` 신규 경고 0.
- shortcut.hwp 8페이지 SVG ↔ PDF 4건 모두 정합 최종 확인. `output/svg/task842_after/` 캡처 + before/after diff.
- 회귀 대상 샘플 SVG diff 최종 점검.
- 최종 보고서 `mydocs/report/task_m100_842_report.md` 작성.
- merge 전 `git status` 로 미커밋 파일 확인.

---

## 커밋 규약
- 각 Stage 소스 + `working/task_m100_842_stage{N}.md` 함께 커밋, 메시지 `Task #842: ...`.
- 최종 보고서 커밋 후 승인 → `local/task842` → `local/devel` merge (원격 push 금지).

---
승인 요청: 위 6단계 구현계획으로 진행해도 되는지 확인 부탁드립니다. 승인 시 Stage 1 진단부터 착수합니다.
