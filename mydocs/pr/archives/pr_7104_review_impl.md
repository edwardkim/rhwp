---
kind: investigation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7104 메인터너 보정·후속 단계

[개별 review](pr_7104_review.md)의 현재 판정은 **메인터너 보정 후 수용 가능**이다. 원 head의 보류 사유는 명시한 통합 보정 코드에서 해소했다.

## 적용 이력

- branch: `review/planet6897-20260914`
- rebase 당시 기준: `upstream/devel` = local `devel` = `037e4906a93e99896daa145a5ee5517824bfeaf4`
- source: `add3a01b4a786da31663cc35746576495a96d357` → rebase 후 `b9af4dc2377b27baa66e66d65f383a60dee6a2bd` (`-x`·원 저자 유지)
- 메인터너 보정: `9f3b1897c + fb1b46e23`
- 최종 검증 코드: `2f59c89373f497068f9a0bcb2c22730ec7dc7e51`
- 입력/PDF: 기존 Git/LFS 파일 재사용; 보존 commit `ab3184254`

## 해결한 원인과 확인

앞 단의 문단에 연결된 그림이 원본 수평 오프셋에 따라 다음 단에 놓여도, 단 전환에서 배제 영역이 지워졌다. 또한 NO_LS 일반 본문은 그 영역을 줄 채움에 전달하지 않았다.

같은 물리 페이지에서는 그림 배제 영역을 단 사이에 보존하고 새 페이지에서는 비운다. 원본 NO_LS 그림의 문단 기준 좌표를 해석하고, 일반 본문의 frame이 만든 행과 높이를 fit와 paint가 함께 소비한다. 원본 IR의 LineSeg는 바꾸지 않는다. 빈 후속 lane은 다음 가시 글줄이 아니므로 마지막 제목을 양쪽 정렬로 벌리지 않는다.

원본 첫 쪽 그림 pi=6/12/19의 bbox를 보존하면서 가로·세로 교집합이 각각 0.5px를 넘는 가시 TextLine 교차가 각각 4→0, 총 12→0이다. 종전 review의 1→3은 교차한 그림 개수이고 이 12는 교차한 글줄 개수다. 그림 3개와 본문을 유지했으며 BehindText 대조군에서는 겹침을 허용한다. 2쪽 제목 pi=88/93/99는 12pt 전각 자연 폭 64/32/48px를 유지한다.

문서는 한컴과 같은 3쪽이지만 전체 시각 동등성을 뜻하지 않는다. 기존 제목·문단 페이지 소속 차이는 남는다. #6970 종료 근거로 사용하지 않는다. 새 clipping·좌표 clamp·fixture 허용치 완화로 교차를 감추지 않았다. Sweep의 square_wrap_text_overlap flag 자체는 1~2쪽에 남는다. 1쪽 pi=6/12/19는 본문 관통에서 edge_clearance_loss로 바뀌었고, 그림 오른쪽과 글줄 시작의 반올림 좌표 차이는 0.0/0.0/-0.1px다. 독립 PDF의 첫 그림도 오른쪽 경계와 본문 시작이 x=672.2px로 맞닿는다. 이를 관통 0건과 구분하며 Sweep 전체 flag=0으로 보고하지 않는다. 2쪽 pi=78/ci=2 그림의 기존 physical_overlap은 보정 전 4행에서 최종 3행으로 줄었으나 남는다. 이번 F1의 1쪽 추가 교차 보정과 구분하며, #6970 전체 해결을 위해 후속 검토해야 한다.

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
- [개별 review](pr_7104_review.md)의 최신 절이 이전 조사 단계·SHA보다 우선한다. 원래 조사·검증 이력은 보존한다.
- 원 PR별 review와 [오늘할일](../../orders/20260914.md)을 같은 통합 PR의 single-parent 문서 trailing commit으로 반영한다. 코드 변경·base merge/rebase는 하지 않는다.
- push 전 최신 base/head merge-tree, 공백·문서 링크·기존 오늘할일 보존을 검사하고 push 뒤 exact trailing head의 required CI와 재사용 결과를 확인한다.
- 병합·원 PR 댓글/close·이슈 잔여 확인·devel 동기화·duration refresh·소유 산출물 정리는 승인된 merge 이후 단계다.
