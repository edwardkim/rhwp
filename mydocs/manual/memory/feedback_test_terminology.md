---
kind: memory
status: historical
canonical: mydocs/manual/memory/MEMORY.md
last_verified: 2026-07-26
name: feedback-test-terminology
description: 테스트 아키텍처 설명 용어 — 스모크(경량 샘플 크래시 유무 선행) vs 회귀(전체 셋 이전 빌드 diff 정밀 비교) 구분 사용
metadata: 
  node_type: memory
  type: feedback
---

작업지시자 지침 (2026-07-19): CI/CD 테스트 아키텍처를 설명할 때 두 단계를
구분해 부른다 — **스모크 테스트** = 가장 가벼운 샘플 몇 개로 크래시 유무만
먼저 확인하는 단계, **회귀 테스트** = 수많은 테스트셋으로 이전 빌드와 diff 를
정밀 비교하는 단계.

**Why**: 동료·기여자에게 테스트 구조를 설명할 때 정확한 업계 표현이 되고,
rhwp 파이프라인의 실제 구분(대표 샘플 기동 확인 vs 8샤드 스위트·golden·
92셋·render-diff)과도 일치한다.

**How to apply**: ①PR 검토 기록·CI 문서·이슈에서 두 용어를 구분해 사용
②"스모크"를 전체 스위트 의미로 오용하지 않기 ③CI 개선 후보: 아카이브 빌드
직후 30초급 명시적 스모크 job(대표 샘플 파싱·렌더 크래시 확인)으로 조기 실패
신호 — 제안 시 이 용어로. 관련: [[feedback-nonfunctional-dx-investment]]
