---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase120-font-advance-source-contracts/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase120-font-advance-source-contracts/goal-checkpoint.md` · 원문 SHA-256: `c91260c861189a0d23c6f5f9c004f9b578d9d99c4d73df6183b0732cd6484e85`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 120차 기록

직접 검토는 11,499개 함수, Rust 테스트 6,996개 / 866개 파일입니다. 미검토 9,986개·증거 대기580개·보정 확정109개 함수 항목이며 전수 조사는 미완료입니다.

두 글꼴 검사 모듈 전체402줄·18함수를 읽고 새17개를 판정했으며 기존 평균 helper1개는 원본 근거를 보강했습니다. 설치 프로그램6개의 cmap/hmtx, 독립 한컴 PDF8개 선택10쪽의 ToUnicode와 /W 또는 /Widths×FontMatrix, 원본7개의 언어별 대체 face/유형을 직접 확인했습니다.

KoPub·KoPubWorld Light의 바탕936/1000em·돋움872/1000em과 TTF H2gtrM/H2gtrE의 middle dot1024/1024em은 원본 프로그램으로 확인됐습니다. 독립 Print의 HFT Type3 middle dot은 약0.381em입니다. ±0.05px는 authored10pt 글꼴 전진폭 환산 계약이고, middle dot의0.03/0.05는 em 허용치입니다. 임의의 실물 페이지 px 핀으로 제거하거나 ±2px로 일괄 바꾸지 않습니다.

KoPub의 기존 동일 HEAD suite4개는 실제4 PASS / 0 FAIL / 0 ignore입니다. middle dot5root는 보유 등록 binary가 없어 새로 빌드하지 않았고 현재 실행 통과로 세지 않았습니다. 전체 문서 시각·fresh WASM은 이번에 실행하지 않았습니다.

초기 독립 관측기가 78494의 모든 언어 슬롯을 한글 슬롯과 같다고 가정해 종료됐습니다. 원본 ID_MAPPINGS로 언어 소속을 복원해 바로잡았고 초기 로그/driver를 보존했습니다. 그 전에 Rust 검사는 시작되지 않았으며 보정 때문에 Rust를 중복 실행하지 않았습니다. 원본 face 이름은 같아도 기타/사용자 슬롯의 대체 face는 다를 수 있습니다.

원 commit의 -x 출처와 통합 #8148의 실제 merge 조상은 확인했습니다. 원 commit→별도 원 PR 번호 API 응답은 비어 있어 연결 완료로 과장하지 않습니다. 제품·테스트 수정·이름 변경·추가 Cargo 빌드는0이며 사용자 JS 변경을 보존했습니다. 전수 조사 후 최종 이슈 갱신과 필요한 보정·issue_8232 명명·재검증을 계속합니다.
