---
name: 보고서는 타스크 브랜치에서 커밋
description: 단계별 완료보고서, 최종 보고서, 오늘할일 갱신은 타스크 브랜치에서 소스와 함께 커밋해야 함
type: feedback
originSessionId: 1f035a49-cf55-4427-a5b6-ba6a493aa832
---
단계별 완료보고서(`_stage{N}.md`), 최종 결과보고서(`_report.md`), 오늘할일(`orders/`) 갱신은 반드시 **타스크 브랜치(`local/task{N}`)에서 소스 수정 커밋과 함께 커밋**한다. merge 전에 반드시 `git status`로 미커밋 파일이 없는지 확인한다.

**Why:** Task #106에서 보고서를 커밋하지 않고 merge하여 `devel` 브랜치에서 뒤늦게 커밋하는 비정상 흐름이 발생했다. 보고서는 하이퍼-워터폴의 핵심 산출물이므로 누락되어서는 안 된다.

**How to apply:** 각 단계 승인 후 → 소스 커밋 시 보고서도 함께 `git add` → merge 전 `git status` 확인 필수.
