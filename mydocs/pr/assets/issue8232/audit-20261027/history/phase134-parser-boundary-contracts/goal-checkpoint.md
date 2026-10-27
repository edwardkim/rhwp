---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase134-parser-boundary-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase134-parser-boundary-contracts/goal-checkpoint.md` · 원문 SHA-256: `04fdc1bb47500b73fe5434ac4d70f88d3eb1748b63917c1924cf7a5e4bb561ed`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 134차 기록

새 Rust 검사 37개와 test 전용 helper 17개를 직접 분석했다. 기존 helper 1개는 재판정했으며 중복 합산하지 않았다. 누적 직접 분석 11,993개 함수·Rust 검사 7,144개, 미점검 9,624개·증적 대기 552개다. 부적합 확정은 124→125개 함수 항목이며 전수 조사는 미완료다.

현재 컴파일 소스 1,305개와 기존 빌드 영수증/바이너리 SHA를 대조해 재빌드 없이 기존 검사 37 PASS를 실행했다. 실제 빌드 head 2546c8b2와 현재 01ced3d427의 컴파일 입력이 같은 근거를 유지했으며 새 head에서 다시 빌드한 결과로 표현하지 않는다.

test_parse_document_dispatches_hwpx는 실제 PASS하지만 입력이 PK signature뿐인 8바이트여서 Unknown→UNSUPPORTED_FILE_FORMAT으로 끝난다. 중앙 디렉터리의 필수 두 엔트리를 가진 독립 ZIP 대조군은 같은 CLI 공통 경로에서 format=hwpx로 열린다. 초기 commit의 is_err 검사와 ZIP 오인식을 막은 #6547의 정상 변화 사이에 남은 검사 의미 drift다. 제품의 엄격한 ZIP 판별은 유지하고 이 root만 독립 route 계약·issue_8232 이름으로 보정할 계획이다.

다른 signature·DRM·압축 입력/출력 한도·암호·byte 보존은 페이지 px 핀과 구분했다. 24바이트 PNG 헤더는 실제 이미지 디코더에서 유효하지 않으므로 opaque preview 저장 단위로만 기록하고 실제 PNG/페이지 시각 통과로 승격하지 않았다. 실제 원본 CFB/Deflate의 압축 flag와 DocInfo 1260→7329바이트·Section0 80882바이트를 독립 확인했다.

#8232의 현행 본문과 별도 조판 이슈 #8266을 읽었고 이 parser 발견을 #8266 범위와 합치지 않았다. 원본·사용자 Studio JS를 보존했다. 제품/test 수정·개명·추가 빌드·새 그림 sweep은 모두 0이다. 전수 조사 완료 후 이슈 갱신과 실제 보정·새 명명·호출/등록 검증을 계속한다.
