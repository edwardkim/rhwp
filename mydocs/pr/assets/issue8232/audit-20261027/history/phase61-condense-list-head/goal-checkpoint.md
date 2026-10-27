---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase61-condense-list-head/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase61-condense-list-head/goal-checkpoint.md` · 원문 SHA-256: `ba62b99b4e67397bf9052429c771784c6d1035d2693fd14fb4991236fb600663`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

61차 condense·목록 머리 감사 완료

현재 HEAD는 01ced3d42724285c8f9765a453393ab6af326617, 브랜치는 audit/8232-px-regressions-20261025입니다.
정본 원장은 /Users/tsjang/rhwp/output/issue8232-audit/phase61-condense-list-head/audit-ledger.json입니다.
누적 직접 점검은 8,304함수, Rust 검사 5,831개/750파일입니다. file-scope 판정 3개는 함수 수와 구분합니다. 원장 12,051그룹/2,139파일이며 미분류·문맥 3,744그룹, 근거 pending 501그룹이 남았습니다.

이번에는 17함수(검사 10/helper 7)와 생성기 file-scope 3개를 직접 판정했습니다. Native 10 PASS/0 FAIL, 0.353초/빌드 포함 14.545초이며 필터 제외 270건은 PASS가 아닙니다. 독립 ZIP XML 11쌍의 1,466개 줄 시작 위치와 본문이 현행 참조/재조판 모델 양쪽에 보존됐습니다.

같은 목록 머리 입력의 최신 Native Skia print/compat2022·독립 한컴2024 PDF 전4쪽은 canonical PDFium 96CSSdpi/2px raw가 모두100%입니다. 누락·90% 미달은0입니다. review/overlay contact sheet와 1·3쪽 원본을 직접 열었습니다. 엄격 RGB 차이는 남으며 fresh WASM 자격은 미확인입니다. 완전 px 자격으로 세지 않습니다.

list_marker_head_places_first_and_following_lines의 HEAD_CASES 43쌍은 원본 속성이 아닌 PDF 측정 위치입니다. 0.99894 척도·6HU 원점 조정 설명과 15HU=0.2CSSpx 핀이 있으며, 현행 ±2px·관계 조건의 재작성 대상으로 판정해 pending 1개를 추가했습니다. 다른 9개 root의 문자 소속 계약은 보존합니다. 파일 첫머리의 과거 7/6쪽 문구와 달리 현재 main fixture는 Native/참조/PDF 모두6쪽입니다.

작업 중 rhwp-studio/public/rhwp.js의 별도 97줄 추가를 확인했습니다. 소유자를 추정하지 않고 patch/바이트/SHA를 preserved-dirty-source.json에 보존했습니다. 이 파일은 Native 감사 경로에서 소비하지 않으며 현재 선택 소스·입력과 Rust 제품은 HEAD에서 불변입니다. 되돌리거나 커밋하지 않았습니다. 이후 작업에서 무조건 clean을 가정하지 마십시오. 현재 JS SHA는 c1c8d1820cc4375ab668427fa0a42c87d71c3d0d83955f4d300880559023ddc6입니다.

#8232는 2026-10-26T10:33:06Z에 갱신했고 EOF/본문 SHA를 확인했습니다. 본문65,461자로 한도에 가깝습니다. 다음 게시 전에 자체 최신 상태 또는 자체 이력을 전문 보존·역복구 입증 후 정리해야 합니다. 기존 실패·외부 본문은 유지합니다.

기존 57·58·60차 검출 누락과 신규 Rust95/JS schema2건 제외를 유지합니다. 이 감사의 제품/검사 수정0·개명0이며 전체 목표는 ACTIVE/미완료입니다. 남은 전수 조사 후 부적합 검사·필요 구현 보정, issue_8232 새 명명과 실제 재검증을 진행합니다. PR은 후속 승인이 필요합니다.
