---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase172-relative-zero-hancom-print/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase172-relative-zero-hancom-print/goal-checkpoint.md` · 원문 SHA-256: `582a55bed8cd8105498baf3961817f383de761d91d3f394e04a2508af655853a`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 172차 기록

보류 중이던 상대 크기 0% 검사 한 개를 독립 한컴 2020 Print 네 건과 대조했다. 정상 100%는 약 9.95pt, 정상 10%는 약 0.96pt, 작성 0% HWP/HWPX는 모두 약 0.12pt다. PDF 내부 Tf만 읽지 않고 cm·Tm을 적용했으며 glyph 경계도 작은 표시를 확인했다. 같은 입력의 현재 진단은 네 건 모두 clean이었다.

기존 검사의 상대 크기를 항상 무시해 정상 10pt로 본다는 전제는 이 입력의 실제 출력과 맞지 않아 보정 대상으로 확정했다. 다만 공식 범위 밖인 0%의 모든 legacy 문서를 대표하지 않으며, 보정 시 과거 HWP3 입력 내성을 별도로 보존해야 한다. 인쇄 양자화값을 새 px 핀으로 쓰지 않는다. 이전 동일 소스 Native PASS는 재사용했고 빌드·PNG 캡처는 추가하지 않았다.

이 한 개는 재검토이므로 직접 점검 수에 다시 더하지 않았다. 보정 필요 누계는 205개이며 제품/테스트 수정·개명·외부 게시가 없다. 166차 전쪽 검증은 여전히 별도 프로세스에서 진행 중이고, 최종 결과는 최신 원장에 병합한다. 전체 감사는 미완료다.
