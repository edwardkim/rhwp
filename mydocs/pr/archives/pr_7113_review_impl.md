---
kind: investigation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7113 메인터너 보정·후속 단계

[개별 review](pr_7113_review.md)의 현재 판정은 **메인터너 보정 후 수용 가능**이다. 원 head의 보류 사유는 명시한 통합 보정 코드에서 해소했다.

## 적용 이력

- branch: `review/planet6897-20260914`
- rebase 당시 기준: `upstream/devel` = local `devel` = `037e4906a93e99896daa145a5ee5517824bfeaf4`
- source: `13af3ecacd966792d0c04bbd66bfde2c32f34bdb` → rebase 후 `08381be17133451b24717968bc84247911fcc453` (`-x`·원 저자 유지)
- 메인터너 보정: `30835faa1`
- 최종 검증 코드: `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`
- 입력/PDF: 기존 Git/LFS 파일 재사용; 보존 commit `ab3184254`

## 해결한 원인과 확인

기존 EqParser 깊이 제한보다 먼저 실행하는 parse_sequence가 무제한 재귀했다. TAB 방언 감지와 정규화는 따옴표 내부 문자열을 명령처럼 해석했다.

정규화도 EqParser와 같은 MAX_EQ_DEPTH=64 예산을 사용한다. 초과하면 부분 변환 결과를 버리고 원문 Tokenizer로 돌아간다. 방언 감지는 따옴표 내부 TAB을 무시하고, 실제 레거시 수식에서도 닫히지 않은 따옴표를 포함해 literal 원문을 보존한다.

220002-byte, 20000중첩 BAR/TAB 입력의 tokenize를 2MiB thread에서 완료하고 원문 token 수·첫 token 보존을 검사했다. 현행/레거시 quoted literal과 미종결 따옴표 대조군도 통과했다. 실제 transistor 원본 11쪽의 SVG가 보정 전 통합 candidate와 동일해 기존 분수·첨자 개선을 유지한다.

큰 입력 검사는 tokenizer 경로이며 20000중첩 전체 AST의 의미 평가 성공을 주장하지 않는다. 별도 출력 byte 상한을 추가한 것은 아니며, 재귀 깊이 제한과 원문 fallback으로 이 정규화 실패를 차단한다. eqalign/RTN 잔여 및 OLE 실제 편집은 이번 범위에 포함되지 않아 #7105를 종료하지 않는다.

## 단계

| 단계 | 상태 |
| --- | --- |
| upstream/devel 동기화·rebase | 완료; 원 18 commit 재적용, 충돌 없음, 이전 branch 백업 보존 |
| 세 보류 항목 보정 | #7104 그림 배제/빈 lane, #7113 재귀·quoted literal, #7115 마지막 가시 행 공백 |
| 로컬 검증·Visual Sweep | 보류 사유 해소 — 원 PR의 부분 개선 범위 수용; 정확한 결과·입력·PNG는 개별 review에 기록 |
| 통합 PR | #7138 생성 완료; upstream `codex/planet6897-integration-20260914` → devel, owner 리뷰 요청 없음 |
| CI·trailing 기록 | 통합 code candidate CI 후 review·오늘할일 trailing 절차 적용. 이번 로컬 검증을 원격 CI로 표기하지 않음 |
| merge·원 PR/issue 후속 | 최종 SHA·CI·MERGEABLE/CLEAN 및 승인 범위 확인 후 수행. 부분 이슈를 완료로 닫지 않음 |
| devel·정리 | merge 후 동기화하고 실행 중 Rust/Cargo가 없는지 확인한 뒤 소유한 review target만 정리 |

원 PR에 게시할 comment에는 반영 source SHA, 메인터너 보정, 통합 merge SHA, Visual Sweep 직접 링크와
고정된 대표 PNG, 남은 이슈 범위를 포함한다. 현재 이 문서는 게시 완료 기록이 아니다.
다른 작업의 변경을 reset/clean으로 버리지 않는다. [후속 처리 정본](../../manual/pr_review/post_merge.md)을 따른다.

## PR #7138 code CI 완료 후 trailing 단계

- 통합 code candidate `a7898ff72a0b22e1e4071b682616a26dc82fd801`의 Full CI·CodeQL·Render Diff·Adapter·Proptest·Policy 성공을 확인했다.
- [개별 review](pr_7113_review.md)의 최신 절이 이전 조사 단계·SHA보다 우선한다. 원래 조사·검증 이력은 보존한다.
- 원 PR별 review와 [오늘할일](../../orders/20260914.md)을 같은 통합 PR의 single-parent 문서 trailing commit으로 반영한다. 코드 변경·base merge/rebase는 하지 않는다.
- push 전 최신 base/head merge-tree, 공백·문서 링크·기존 오늘할일 보존을 검사하고 push 뒤 exact trailing head의 required CI와 재사용 결과를 확인한다.
- 병합·원 PR 댓글/close·이슈 잔여 확인·devel 동기화·duration refresh·소유 산출물 정리는 승인된 merge 이후 단계다.
