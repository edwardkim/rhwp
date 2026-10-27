---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase130-float-anchor-px-evidence/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase130-float-anchor-px-evidence/goal-checkpoint.md` · 원문 SHA-256: `dd16a4ada73fd8d16a5ff937148f0e39e1c9647e1e168b7c6e374e2d17a85c20`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 130차 기록

누적 직접 점검은 11,925개 함수·helper, Rust 테스트 7,102개 / 870파일입니다. 보정 필요 확정은 119→121개 함수 항목이며, 이번 2개는 경험적 PDF px 허용 오차 보정 대상입니다. 원본 HU 계약을 잘못된 px 핀으로 분류하지 않았습니다. 미점검 9,666개·증거 대기 580개가 남아 전수 조사는 미완료입니다.

#6312 전체 파일의 검사 6개와 helper 6개를 분석했습니다. 검사 4개·helper 6개는 새 직접 판정이고 검사 2개는 기존 판정을 보강했습니다. 독립 ZIP/XML에서 7085HU 본문 원점·저장 vpos·줄간격·그림 높이·셀 여백을 재확인했고 현재 Native 6 PASS를 완료했습니다. 재정동향 4쪽은 최저 98.63756%, 간 기증자 연구 11쪽은 99.64607%입니다. 과거 238de의 WASM 수치를 현재 통과로 재사용하지 않았으며 현재 fresh WASM은 미검증입니다.

독립 PDF 세로 괘선 centerline은 46.1906738CSSpx로 기존 46.1907의 근거가 맞았습니다. stroke 포함 ink 높이 47.15063으로 잘못 대체하지 않습니다. 경험적 ±0.3/±0.5 핀은 현행 ±2CSSpx와 내용·소유 관계에 맞게 보정해야 하며, source HU·셀 공간 점유의 정확한 계약은 유지합니다. Native 통과를 규칙 충족이나 새 WASM 자격으로 확대하지 않았습니다.

원 제안 #6436/#6445는 미병합이며 실제 source-vpos 보정은 #6485, 형제 셀·빈 줄·후속 쪽수 전제 이관은 #7505의 커밋임을 GitHub 주자료와 Git 조상으로 확인했습니다. a6c의 215쪽 핀은 복원하지 않고, 과거 미달 수치를 현재 11쪽 결과로 혼동하지 않습니다.

현재 review/overlay 1·3·11쪽을 직접 판독했고 그림 5/6·후속 본문·각주 보존을 확인했습니다. 기존 동일 head의 빌드 영수증과 해시로 suite028·CLI를 재사용해 추가 빌드 0, 소스 수정 0입니다. 전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
