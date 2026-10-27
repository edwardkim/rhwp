---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase132-current-paragraph-position-evidence/goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase132-current-paragraph-position-evidence/goal-checkpoint.md` · 원문 SHA-256: `043123de9cb3200cbe58cccfc509158510e45495f049e5074dacb2e50e485a49`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 전수 조사 132차 기록

현재 HEAD 01ced3d427에서 증적 대기 11개를 추가 판정해 572→561개로 줄였다. 직접 분석한 함수 11,939개와 Rust 검사 7,107개/873파일은 중복 합산하지 않았다. 보정 필요 확정은 124개 함수 항목, 미점검은 9,661개이며 전체 감사는 미완료다.

qualified_paragraph_print_positions의 전체 source와 검사 10개·helper 3개를 확인했다. Native 기존 검사 10 PASS, 현재 Native/fresh WASM 각각 전 17쪽 raw 100이다. ZIP 원문과 독립 Print의 전체 내용을 순서로 대조해 214개 문자 원점의 쪽·문단·줄 소속과 ±2CSSpx를 확인했다. 직접 확대 판독은 9개 접촉판의 80개 핵심 ROI이며 원본 review/overlay와 crop 출처를 연결했다. 정상 좌표·내용 관계 검사는 유지하고 이름만 바꾸지 않는다.

이전 재정동향 전 4쪽 Native와 현재 WASM은 backend 이외 실행 provenance가 같다. Native 최저 98.63756%, WASM 최저 98.63321%다. 간 연구 11쪽은 Native 99.64607%/WASM 99.64616%로 현재 증거를 보완했다. 215쪽 전체 통과나 과거 WASM 재사용으로 주장하지 않는다. 표 높이 ±0.3px·캡션 ±0.5px의 기존 보정 필요는 유지한다.

현재 build receipt·소스·package·입력/PDF·PNG·TSV를 확인해 추가 빌드 없이 진행했다. 첫 output 폴더 생성 및 silhouette-only 완료 manifest 이름을 진단 driver에서 정정했다. 완료된 Native 검사와 첫 캡처는 재실행하지 않았으며, review 디렉터리 생성 수정도 관측기에만 적용했다. 소스/test 변경과 이름 변경은 0이며 사용자 JS는 보존했다.

전수 조사 완료 후 최종 이슈 갱신과 실제 보정·issue_8232 명명·호출 경로 갱신 및 재검증을 진행한다.
