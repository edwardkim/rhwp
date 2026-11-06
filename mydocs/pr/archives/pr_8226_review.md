---
kind: report
status: active
last_verified: 2026-11-07
---

# PR #8226 리뷰 — #8286과 통합 준비

## 최종 판정

**머지 보류 — 최종 저장본 두 파일의 독립 한컴 PDF 검증을 위한 전송 승인 대기.**
#8226 단독 검증을 통합 전체의 완료로 대신하지 않았다. 통합 후보의 전체 회귀 12,777개와
Native Skia 3종, 편람 383쪽 게이트까지 통과했다. [공동 검토 기록](pr_8286_review.md)에
최종 code head, 원 커밋 대응표, 완료한 검사와 남은 게이트를 기록했다.

기여자 `@semanticist21`의 선행 SectionDef/ColumnDef 삽입 시 기존 제어 슬롯 보존 수정과
Native/WASM 증거 커밋을 author 및 `cherry-pick -x` 출처와 함께 포함했다.
request Storage 호환, 머리말·꼬리말 포함 inline table host 재조판, HWP 재개방의 표 앞뒤
본문 보존 회귀는 별도 메인터너 커밋이다.

기존 통합 [#8711](https://github.com/edwardkim/rhwp/pull/8711)의 보정과 추가 회귀를 재사용했으며
그 원격 브랜치와 본문은 수정하지 않았다. 이 브랜치에서 #8226 원본과 최종 HWP 저장본은
Native·fresh WASM 1쪽 모두 raw silhouette 100%다. 저장본 바이트도 이전 검증본과 같다.

원 #8226은 `maintainerCanModify=false`이므로 직접 push하지 않는다. 두 PR의 통합 후보가
전체 로컬 게이트 및 최신 CI를 통과하고 승인받은 뒤 병합한다. 원 PR과 #8711 close는 실제
대체 통합 병합 후 승인 범위에서 수행한다. #4680·#4687 전체 해결을 주장하거나 닫지 않는다.
