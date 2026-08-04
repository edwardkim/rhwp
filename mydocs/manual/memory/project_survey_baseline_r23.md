---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: project-survey-baseline-r23
description: "10k 서베이 기준선은 r23(폰트-클린) — 하니스에 RHWP_FONT_PATH 필수, r22는 폰트 오염"
metadata: 
  node_type: memory
  type: project
---

10k 한글 오라클 서베이의 비교 기준선은 **r23**(2026-07-25, planet6897 PR #3298, `mydocs/report/survey_10k_r23_20260725.md`)이다. devel `4b5514457` 기준 쪽수 회귀 0, 픽셀 평균 93.89%, 측정 갭 65(역대 최소).

**Why:** #2898 폰트 조달 변경 후 `RHWP_FONT_PATH` 미설정 실행(r22)은 시스템 폰트 폴백으로 픽셀 기준선이 **양방향 오염**된다(하락 211건 + 우연 상승 ~119건). r22와의 직접 비교는 회귀 판정에 쓰면 안 된다.

**How to apply:** 서베이·픽셀 비교 하니스 실행 시 `RHWP_FONT_PATH=ttfs/hwp;ttfs/windows` 필수. 새 서베이는 r23과 비교. #3270(para-float 앵커-줄)은 r23 바이너리 미포함이라 차기 서베이에서 쪽수 변경 2건(오라클 일치 확인됨)이 나타나는 것이 정상. 관련 [[reference-font-path]] [[feedback-visual-judgment-authority]]
