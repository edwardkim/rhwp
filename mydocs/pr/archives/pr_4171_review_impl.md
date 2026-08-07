---
kind: review-implementation
status: completed-local
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-07
---

# PR #4171 통합·메인터너 보정 기록

## 적용 순서와 commit 경계

| 순서 | SHA | 역할 |
| --- | --- | --- |
| 1 | `b21a27de6` | #4097 CFB CLSID 코퍼스와 계획을 기록 |
| 2 | `b5513554f` | mini CFB 재포장에서 루트 CLSID 보존 |
| 3 | `61fe12598` | CLSID 보존 회귀 방향과 두 포맷 계약 확장 |
| 4 | `878f9c706` | HWP3 OLE 서브 스토리지 승격 시 CLSID 전달 |
| 5 | `85084818f` | 한컴 실측 및 C2b 게이트 기록 |
| 6 | `731f23772` | #4097 PDF 증적과 최종 보고서 |
| 7 | `395123e59` | #4141 HWP3 상대 크기 실측과 계획 |
| 8 | `7ca8b3496` | CharShape 상대 크기 기본값을 100으로 보정 |
| 9 | `3ca2f533d` | #4141 한컴 판정 보고서 |
| 10 | `c4f85c0a0` | CFB v4 루트 CLSID 디렉터리 오프셋 메인터너 보정 |

통합은 `upstream/devel` `9dbd3dc6c` 위에서 #4144 → #4160 순서로 수행했다. #4160의 본문이
#4144의 CLSID 보존을 선행 의존성으로 명시하므로 번호가 아닌 그 의존 순서를 적용했다. cherry-pick 충돌은
없었으며, contributor 원 commit의 history는 변경하지 않았다.

## 메인터너 보정 범위

1. CFB 루트 CLSID 조회의 파일 오프셋을 `512 + SID * sector_size`로 고쳤다.
2. v3와 v4가 모두 512바이트 헤더를 사용한다는 형식 규칙과, v3에서 기존 식이 우연히 맞던 이유를
   코드 문서에 남겼다.
3. CFB v4, 4096바이트 섹터, SID 1의 디렉터리 엔트리를 구성해 올바른 CLSID 위치만 읽는 단위 회귀를
   추가했다.
4. serializer의 기존 v3 mini CFB writer나 contributor의 보존 정책은 변경하지 않았다.

## 완료한 검증과 후속 단계

CFB v4 단위 1건, #4097 계약 3건, #4141 계약 5건, 전체 `release-test --tests`, fmt, clippy와 WASM
library check를 차례로 통과했다. GitHub code candidate `c4f85c0a0`에서도 CI, CodeQL, Render Diff의
모든 required 결과가 성공했다.

이 문서와 오늘할일은 code candidate 뒤의 review-only trailing commit으로만 push한다. push 뒤 최신
head의 preflight와 aggregate를 재확인하고, 작업지시자가 승인하면 #4171 병합, 이슈 자동 종료 확인,
원 PR #4144·#4160의 후속 처리, branch와 검토 전용 target 정리를 순서대로 수행한다.
