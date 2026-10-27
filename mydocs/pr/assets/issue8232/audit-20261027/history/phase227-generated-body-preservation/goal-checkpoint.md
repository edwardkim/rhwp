---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase227-generated-body-preservation/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase227-generated-body-preservation/goal-checkpoint.md` · 원문 SHA-256: `bc8730f7cb318502f3c02984104d7d87bd6b6690083dde58dc92102ac04766be`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 227차 기록

생성 root 두 개를 실제 실행해2 PASS를 확인했다. 그 HWP의 진단 사본에서 원본 문장40글자만 공백으로 바꿨고, 같은 CFB stream 길이·다른 stream·레코드 header/길이·압축/구역 메타는 보존했다. 현재 Native info는 모두9쪽/2구역이며, 전9쪽 export-text에서 원 문장1회가0회로 사라졌다. 형식/쪽수만으로 내용 보존을 대신하는 검출 공백이다. 제품 serializer가 실제로 본문을 잃거나 원 Rust root에 변형 출력을 주입해 PASS했다는 뜻은 아니다.

처음 분석기의 압축 level6이 CFB stream보다 커지는 오류는 level9로 보정했다. 이미 완료된 원 Native2 PASS를 재실행하지 않고 복구 영수증·초기 driver·로그를 남겼다. 이를 제품 실패나 소스 보정으로 세지 않는다.

증거 보류430→428개, 보정·정리 함수 지점319→321개다. Stage54 ignore 생성기 한 건은 보류를 유지한다. 이번 turn 누적 보류12개를 판정했고 직접 검토15,750개는 중복 증가시키지 않는다. 새 빌드·캡처·소스 수정·개명·외부 게시 없이 전체 감사는 진행 중이다.
