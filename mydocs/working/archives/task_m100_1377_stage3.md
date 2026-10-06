# Task #1377 Stage 3 — 발산 완전 규명 (between-notes gap 선택적 compaction) + 수정 방향 입증

- **이슈**: #1377 (M100) / 브랜치 `local/task1377`
- **단계**: Stage 3 — shape-reserved 경로 끝까지 추적, 근본 + 수정 방향 확정
- **결과**: 발산 **완전 규명**. 수정이 p22 를 실제 해소함을 **입증**(1100.59→1087.72 프레임 내). 단 선택적
  적용 필요(일괄 cap 은 14건 회귀). 코드 클린.

## 1. 발산 근원 (완전 규명)

`EN_LINE` 계측: **pi=1128 은 빈 para + TAC 수식 2개**(between-notes gap spacer). file 인코딩 line_height
2070HU(27.6px) + line_spacing 1984HU(**26.5px**) = **렌더 54.1px**. typeset 은 **33.6px 로 선택적 compact**.
- line_height(27.6) 는 수식 높이 보유 — 정상.
- **line_spacing(26.5) 이 phantom 과대 gap** (정상 미주 줄 spacing ~6px). 이 +20px 가 단 하단까지
  누적돼 pi=1156 프레임 초과.

## 2. 수정 입증 + 한계 (핵심)

`empty_tac_guide_line` 의 endnote line_spacing 을 8px 로 cap:
- **pi=1128 dy 54.1→35.6, p22 최하단 1100.59→1087.72(프레임 1092.3 내) = overflow 실제 해소** ✓
- **그러나 issue_1139 14건 회귀** — 회귀 전부 "미주 사이 gap/제목 tail 보존" 테스트. 즉 그 spacing 은
  **다수 케이스에서 진짜 between-notes gap** 이라 일괄 compact 불가.

→ **typeset 은 선택적**으로 compact(pi=1128 만, 14건 gap 은 보존)하는데, **render 는 file 원시값을 써서
그 선택을 모른다**. pi=1128 은 단 top(rel 18)이라 "단 하단" 게이트로도 분리 불가.

## 3. 근본 = render ↔ typeset 의 gap 결정 분리 (아키텍처)

- typeset(`compute_en_metrics`/between-notes 게이트)은 어느 빈-수식-spacer gap 을 compact 할지 **이미
  정확히 결정**(14 테스트가 그 정확성 보증, acc pi=1128=33.6).
- render(`layout_composed_paragraph`)는 file line_spacing 을 그대로 그려 그 결정을 **반영 못 함**.
- → **수정 = typeset 의 gap-compaction 결정을 render 로 전파**(예: PageItem/ColumnContent 에 per-para
  `compacted_gap` 플래그를 typeset 가 기록 → render 가 그 줄의 line_spacing 을 compact). 좁은 cap 이
  아니라 **결정 전파**(plumbing typeset→render)가 정답.

## 4. 성과 / 다음

- **완전 진단**: 증상(p22 overflow) → pi=1128 빈 수식 spacer 의 phantom line_spacing(+20px) → typeset
  선택적 compact vs render 원시값 → **결정 전파 필요**. 수정이 p22 를 실제 해소함을 입증.
- **다음(구현)**: typeset 의 빈-수식-spacer gap compaction 지점에서 per-para 플래그 기록 → ColumnContent
  로 전달 → render 가 해당 줄 line_spacing compact. 전 exam·골든 SVG 가드. (plumbing 작업, 중간 규모)

## 5. 코드 상태

src 클린(전 시도 revert). 유효 수정 = #1375 `821a8b32`. 진단 = #1377 Stage1/2/3.
