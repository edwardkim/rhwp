---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase121-table-edit-source-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase121-table-edit-source-contracts/goal-checkpoint.md` · 원문 SHA-256: `845cae7cb1feeacc0b6555291cf50ec8c927eff49433467415e925ff51faae09`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 121차 기록

직접 검토는 11,527개 함수, Rust 테스트 7,010개 / 869개 파일입니다. 미검토 9,971개·증거 대기 581개·보정 확정 109개 함수 항목입니다. 전수 조사는 미완료입니다.

표 편집 세 모듈 전체 718줄·28개 함수를 읽었습니다. 3,000/6,000/9,000과 셀 여백·높이는 직접 작성한 HWPUNIT이고, 566은 기본 바깥여백 283+283HU입니다. 원본 PAGE_DEF의 편집 폭 53,292HU와 25/25/50%의 13,323/13,323/26,646HU를 저장 결과에서 확인했습니다. 경로 tuple·32열·raw 13바이트 역시 화면 좌표 핀이 아니므로 일괄 제거하거나 ±2px로 바꾸지 않습니다.

동일 HEAD의 기존 binary에서 셀 여백/width_ref Rust 3개가 실제 PASS했습니다. 별도 frozen Native CLI 41프로세스와 그 안의 실제 MCP tools/call 2개에서 양 형식의 6셀 너비/정렬/머리행·기존 필드 11/1개 보존·잘못된 요청 32건의 목적 파일 불변·정상 dry-run 파일 부재를 확인했습니다. 수동 CLI 관측을 기존 Rust 5개 root 실행으로 세지 않았으며 중첩 표 6개 root도 미실행입니다. 새 빌드·제품/테스트 수정·이름 변경은 0입니다.

중첩 표 reflow root는 잠정 px 핀은 아니지만, 초기 저장 줄이 0이면 잘못된 외부 폭으로 1줄만 생겨도 after>before를 통과할 수 있습니다. 현재 정확한 root/변이 실행 근거가 없어 조건부 검출 우려 1개로 대기하며 실제 엔진 결함·보정 확정으로 세지 않습니다.

초기 추가 관측기의 PARA_HEADER level 가정을 실제 저장 record에 맞게 보정했습니다. 성공한 첫 CLI 출력은 hash로 검증해 재사용했고 Rust를 중복 실행하지 않았습니다. 원 PR #5567→통합 #5610, 원 PR #7214→메인터너 보정→통합 #7237, 직접 PR #7068의 계보와 실제 merge 조상을 확인했습니다. 과거 10k corpus·Studio drag·시각 검증을 현재 재실행한 것으로 확대하지 않습니다.

사용자 JS 변경과 원본을 보존했습니다. 감사 완료 뒤 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
