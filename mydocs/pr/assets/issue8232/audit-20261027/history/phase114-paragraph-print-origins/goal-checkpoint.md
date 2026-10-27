---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase114-paragraph-print-origins/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase114-paragraph-print-origins/goal-checkpoint.md` · 원문 SHA-256: `be80e74aa826e44c2985720addf412ed9411ec34a83404f9a5bab6443cd407f0`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 114차 기록

직접 검토11,291개 함수, Rust 테스트6,909개/856개 파일이다. 미검토10,102개, 증거 대기577개, 보정 확정103개 함수 항목이다. 전수 조사는 미완료다.

문단 원점 모듈449줄·13함수 전체를 읽고 새12함수와 기존 보류 helper1개를 판정했다. 원본 HWPX10개와 독립 Print17쪽의 문단·줄·쪽 소속 및214개 기대 문자 원점을 독립 파싱했다. PDF point→CSS96 변환 결과 최대 오차0.000040695px로 기대값이 직접 대응한다. glyph ink bbox를 기준선으로 오인하지 않았다.

현재 HEAD와 producer·등록·바이너리 해시가 같은 기존 suite011에서10개 검사를 실행해10PASS/0FAIL/0ignore, exit0을 확인했다. 추가 Cargo/빌드는0이다.

원 PR7825/7828은 미병합 CLOSED이고 통합7923 merge가 현재 HEAD의 조상임을 확인했다. 해당 원점 모듈은 소개 commit d80593과 바이트 동일하다. 원 통합 PR7923 본문에서20개 Native/fresh WASM TSV를 원문 복원해 SHA를 검증했다.10문서17쪽×2 전쪽 raw 최저 Native99.28751%, WASM99.80752%다. 입력/PDF/대표40PNG해시를 검증하고 indent matrix3쪽과 spacing prev2쪽의 양쪽 review+overlay4쌍을 직접 판독했다. 이는 당시 code39a634의 증거다.

따라서 근거 없는 px 핀으로 단정하거나 완화/제거하지 않는다. 최신 HEAD에서 양쪽98% 유지 여부는 별도 증거 대기로 남긴다. helper+10roots의 보류11건, 순수 관측/정규화 helper2건으로 구분했다. 현재 원본·테스트 수정 및 이름 변경0, 기존JS 변경 보존이다. 전수 조사와 증거 보완 뒤 최종 이슈 갱신·필요한 issue_8232 보정/명명·실제 재검증을 계속한다.
