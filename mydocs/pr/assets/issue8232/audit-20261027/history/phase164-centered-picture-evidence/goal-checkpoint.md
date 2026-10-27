---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase164-centered-picture-evidence/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase164-centered-picture-evidence/goal-checkpoint.md` · 원문 SHA-256: `581762cf4eb40a429b9dbbde303b1489c24d46dca7c8ff9873deb714a0bf8ab8`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 164차 기록

현재 head의 제품규격서 전6쪽을 Native/fresh WASM으로 새로 비교했다. 최저 raw는 각각98.78547%와98.78482%다. 원본 CFB PageDef와 그림 바이너리, 모델의 소유, 독립 Print의 이미지 내용·좌표를 대조했다. 그림 x 핀의 독립 근거는 확인했지만 그림3 캡션은 약20 CSSpx, 그림2 캡션은 약53 CSSpx 왼쪽으로 어긋난다. 정본 review·overlay·3배 확대에서 두 backend를 직접 판독했다. 점수가98% 이상이어도 완료 페이지가 아니므로 px 자격을 승인하지 않는다.

기존 로더 오류는 보존하고 캡션 x 생산·소비 연결을 기록했다. 원인은 registered_inline_pos의 y만 재사용하고 캡션에는 별도 pic_x를 넘기는 경로다. 전수 조사 뒤 공통 원점과 정상 소유·내용 관계를 보정하고 다시 검증한다. 재검토한 root는 중복 집계하지 않고 새 nested helper1개만 직접 점검 수에 더했다. 보정 필요항목199개이며 전체 감사는 미완료다. 추가 빌드·제품/테스트 변경·개명·외부 게시 없음.
