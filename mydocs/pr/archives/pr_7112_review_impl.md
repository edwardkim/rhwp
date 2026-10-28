---
kind: investigation
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7112 체리픽·검증·후속 계획

[개별 review](pr_7112_review.md)의 판정은 **승인**다. 통합 번호를 미리 만들지 않았으며
원 저자의 commit·SHA를 `-x`로 보존했다. 이 계획은 local 검토 완료와 아직 하지 않은 원격 조치를 구분한다.

## 이미 적용한 commit

| 원 SHA | 로컬 SHA | 제목 |
| --- | --- | --- |
| `5c1fca6c30b2ec0a5e5da46c4e00c35b6a450d6f` | `e04bf753a4e0d28a8027a07a78a8e29eb69e094e` | 수정: 한양중고딕의 가운뎃점 메트릭이 남의 글꼴 값을 빌려 쓴다 (#7092) |
| `1326ceec171c915ecd7018cb367b2f8e78b27251` | `6b2d81eaa3bad3dd9e9c67647d91b4a07da811c5` | 수정: 한양신명조의 가운뎃점도 자기 메트릭으로 잰다 (#7092) |

기준 `93ffc3dd59c120bd54df4c2ac6d1ddbe630f8a2d` 위에서 PR 순서 7094 → 7100 → 7104 → 7111 → 7112 → 7113 → 7115 →
7116 → 7117 → 7120 → 7131 → 7132로 누적했다. 이 PR의 마지막 local SHA는 `6b2d81eaa3bad3dd9e9c67647d91b4a07da811c5`다.
추가 fixture/PDF 보존은 `6933852a11b7e5998429eeb15708fdaeed7db626`, 최신 upstream 정렬 후 기록 기준은 `28d702d8f84bacb7ecec4f2a6dafd049e09bfcfa`다.

## 보정과 규칙

다른 face의 Latin1 메트릭을 공유하지 않고, 각 한양 글꼴의 독립 한컴 PDF glyph advance에 맞는 B7 값을 적용한다. HanYangSin 393/1024, HanYangJung 390/1024이며 다른 글자의 overlay를 확대 변경하지 않는다.

이번 검토 범위에서 새 실행 회귀 또는 수용을 막는 코드 문제를 발견하지 않았다. 아래 잔여·미검증 범위는 승인 대상에 포함하지 않는다.

#7111의 text_measurement 충돌에서는 최신 supplemental_metrics와 신규 font_metric_trusted를 함께 남겼다.
이외 생산 코드에 메인터너 수정은 추가하지 않았다. #7115 원인 확인용 임시 함수 rollback은 별도
대조 바이너리에만 사용하고 검토 source를 바이트 단위로 복구했다. 정식 보정으로 세지 않는다.

## 단계와 실행 상태

| 단계 | 상태 / 다음 작업 |
| --- | --- |
| source inventory·최신 devel·reviewer | 완료; draft #7098 제외, reviewer jangster77; owner 자동 요청 없음 |
| 16 commit 체리픽·충돌 보정 | 완료; 원 SHA·저자·적용 순서 위에 보존 |
| 원본·한컴 PDF 확보 | 완료; Git 내 동일 SHA 재사용, 신규 자료만 별도 보존 commit |
| focused/full/Native/Clippy/WASM/실물 sweep | 완료; 정확한 결과·제약은 review 참조 |
| 발견 결함 처리 | #7104·#7113·#7115 보류. 원인 보정 또는 범위 분리 후 필요한 검사 재실행 |
| 통합 PR | 아직 생성하지 않음. 준비가 확정되면 upstream 임시 head → devel; reviewer owner 자동 지정 안 함 |
| CI와 trailing 기록 | 새 code candidate CI 통과 후 review·오늘할일을 동일 PR에 포함; source/fixture를 docs-only trailing에 섞지 않음 |
| merge·원 PR/issue 후속 처리 | 최종 SHA·CI·MERGEABLE/CLEAN·승인 확인 후 수행. 원 PR comment에 통합 merge와 반영 SHA, 미해결 issue 범위를 남김 |
| devel·cleanup | 실제 merge 후 devel 동기화. 실행 중 Cargo/Rust가 없는지 확인하고 전용 target만 post_merge 7.7.1에 따라 정리 |

사용자 선택이 필요한 다음 범위는 발견 결함을 보정해 12개를 함께 진행할지, 보류 PR을 제외한 별도
수용 묶음을 만들지다. 지금 reviewed history에서 임의로 source PR을 빼거나 원격에 게시하지 않았다.

## rollback 경계

현재 branch는 `review/planet6897-20260914`다. 작업공간은 공유하므로 reset/clean으로 다른 변경을 버리지 않는다.
범위 분리가 승인되면 당시 최신 `upstream/devel`에서 새 branch를 만들고 이 표의 승인된 원 SHA와 필요한
메인터너 보정만 순서대로 적용한다. 특히 #7111·#7112·#7115는 같은 측정 코드, #7115·#7117은
form-002 golden을 공유하므로 중간 commit만 취소한 상태를 검증 결과로 재사용하지 않는다.

[후속 처리 정본](../../manual/pr_review/post_merge.md)을 따른다.

## PR #7138 code CI 완료 후 trailing 단계

- 통합 code candidate `a7898ff72a0b22e1e4071b682616a26dc82fd801`의 Full CI·CodeQL·Render Diff·Adapter·Proptest·Policy 성공을 확인했다.
- [개별 review](pr_7112_review.md)의 최신 절이 이전 조사 단계·SHA보다 우선한다. 원래 조사·검증 이력은 보존한다.
- 원 PR별 review와 [오늘할일](../../orders/20260914.md)을 같은 통합 PR의 single-parent 문서 trailing commit으로 반영한다. 코드 변경·base merge/rebase는 하지 않는다.
- push 전 최신 base/head merge-tree, 공백·문서 링크·기존 오늘할일 보존을 검사하고 push 뒤 exact trailing head의 required CI와 재사용 결과를 확인한다.
- 병합·원 PR 댓글/close·이슈 잔여 확인·devel 동기화·duration refresh·소유 산출물 정리는 승인된 merge 이후 단계다.
