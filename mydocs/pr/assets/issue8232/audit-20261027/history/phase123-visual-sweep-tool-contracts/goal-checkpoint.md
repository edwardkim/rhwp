---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase123-visual-sweep-tool-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase123-visual-sweep-tool-contracts/goal-checkpoint.md` · 원문 SHA-256: `7a5adf4e19ba3230f68dce7bc406b91998d5f0f77f4d04b8309009a8e458ceeb`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 123차 기록

직접 검토는 11,687개 함수·helper, Rust 테스트 7,045개 / 870개 파일입니다. 미검토 9,861개·증거 대기 581개이며 전수 조사는 미완료입니다. 보정 확정은 111개에서 112개 함수 항목으로 늘었습니다. 함수·helper 중복과 독립 결함 수를 구분합니다.

이번에는 Visual Sweep 원 모듈 16개, 1,489줄, 96개 함수(검사 75개 / helper 21개)를 분석했습니다. 현재 원본 Python unittest는 상속 4개를 포함해 79 PASS / 0 FAIL / 0 SKIP입니다. 새 Rust 빌드·제품/테스트 수정·이름 변경은 없습니다.

새 검출 공백은 독립 한컴 Print의 그라데이션 보존 검사입니다. 실제 기준 3쪽은 794×1122, 파랑 화소 15,805개입니다. 같은 크기의 흰 페이지에 파랑 1점만 둔 진단 출력을 원 검사에 공급해도 1 PASS였고, 완전 흰 대조군은 1 FAIL이었습니다. 원 PDF·제품·테스트를 수정하지 않았고 현재 PDFium이 그라데이션을 잃는다는 주장이 아닙니다. 실제 색 변화·채움 면적/소유를 관계로 보호하도록 보정할 항목입니다.

다른 항목의 작성된 PNG 크기·alpha/RGB·Type3 pt·font UPEM·cmap/GID·쪽 번호·worker 수는 도구 입력·형식 계약으로 구분했습니다. 합성 글꼴 3개의 cmap을 독립 SFNT reader로 대조했습니다. 명시적 과거 #8064 Native 품질 도구의 179조건·2,082쪽 TSV 검사를 새 HEAD의 전쪽 수용으로 쓰거나 자동 강제 연결을 복구하지 않습니다. 현재 사용자 0.1%p 지시도 과거 수동 도구의 2%p로 대체하지 않습니다.

원 검사의 72a982/991bbc/c8e69/f703d와 실제 PR #8064 merge 조상을 확인했습니다. 같은 커밋이 PR #8196에도 조회되나 미병합이므로 통합 근거로 쓰지 않았습니다. CI unittest discover 자동 수집을 확인했고 mock WASM·Node 프로토콜을 실제 fresh WASM/Chrome 실행으로 표시하지 않았습니다.

전수 조사 뒤 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
