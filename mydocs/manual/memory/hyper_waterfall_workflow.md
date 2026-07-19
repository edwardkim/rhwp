---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-17
name: Hyper-Waterfall 워크플로우 필수 준수
description: 타스크 진행 시 수행계획서→구현계획서→단계별 보고 승인 절차를 반드시 거쳐야 함
type: feedback
---

타스크 사이클에서 **승인 없이 다음 단계로 넘어가지 않는다**.

1. 타스크 등록 (GitHub Issue + 오늘할일)
2. **수행 계획서** 작성 → **승인 요청** → 승인 후 진행
3. **구현 계획서** 작성 → **승인 요청** → 승인 후 구현
4. 단계별 구현 → **단계별 완료 보고서** → **승인 요청** → 승인 후 다음 단계
5. 최종 보고서 → 승인 → 커밋/이슈 클로즈

**Why:** 승인 단계를 건너뛰면 Hyper-Waterfall이 바이브 코딩으로 전락한다. 컨트리뷰터들의 작업 추적과 품질 관문이 무력화된다. Task #88에서 구현계획서 승인 없이 바로 코딩한 실수가 있었다.

**How to apply:** 매 타스크 시작 시 이 순서를 확인한다. "승인합니다"를 받기 전까지 코드 작성을 시작하지 않는다. 참조: `mydocs/manual/hyper_waterfall.md`
