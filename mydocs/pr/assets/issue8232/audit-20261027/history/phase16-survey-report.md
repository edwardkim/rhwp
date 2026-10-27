---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase16-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase16-survey-report.md` · 원문 SHA-256: `467344b313b77f82887e29ca70757f0fe6568bed3ad5ca411e974ec86d74e251`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# #8232 16차 전수 조사 진행 기록

현재 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`, 브랜치는 `audit/8232-px-regressions-20261025`입니다.

- batch25~29에서 14개 파일의 검사158개/helper101개를 직접 읽고 수치·선택·관계의 의미를 기록했습니다. 이는 실행 PASS/완전 적합 수가 아닙니다.
- 최신 원장:10,194함수/helper 그룹·2,103파일. 미분류4,655, 직접 수치 의미 판정4,755, 독립 근거 대기442, 의미 메모130, 파일 문맥만 확인212입니다. 호출 폐쇄 검증과 실제 보정은 여전히 진행 중입니다.
- Native view의1e-9는 작성한 호스트 좌표 변환의 산술 오차입니다. Region cache의 전체 raster/crop 비교는 같은 실행의 출력 동등성이며 한컴 일치나 과거 배치 golden을 뜻하지 않습니다. region cache는 native-skia 조건부입니다.
- 링크/누름틀/혼합 서식의 숫자는 주로 Unicode scalar·UTF-16·field/shape ID·원본 fontSize/색상입니다. all_text helper가 본문/표/Shape text_box만 순회하는 범위를 전체 container 보존으로 확대하지 않습니다.
- RowBreak의 캡션→글상자 비율은 원본 직접 lineseg의1000→11564HU/LH1100HU에서 확인했습니다. 첫 진단에서 중첩 글상자 lineseg까지 섞인 것은 정정했고, 최종 결과는 직접 p/linesegarray만 읽습니다. 두 독립 PDF는 각각18쪽이지만 현재 전쪽 시각 자격을 입증한 것은 아닙니다.

## 집계 연결 보정

기록을 원장과 대조하는 검사에서 직접 검토한78개 함수가 초기 숫자/렌더 검색에 없다는 이유로 원장에 빠진 것을 확인했습니다. 이전에 발견한 MCP 응답 선택 오류도 포함됐습니다. 통합 builder가 직접 검토 기록을 모두 연결하도록 보완했습니다. 전체5,197개 직접 검토 기록의 원장 누락은0개이며, 이는 새 결함/실행 PASS 증가가 아닙니다. 이번14개 파일의259개 함수도 source SHA·함수 전수 목록·원장 연결을 모두 대조했습니다. 미분류4,655개는 그대로입니다.

## 확인된 추가 보정 대상

`tests/cases/issue_5788_tac_table_anchor_line_spacing.rs:20`은 고정 y380..500 영역의 괘선과 다음 raw y attribute를 선택합니다.

| 관측 | 결과 |
| --- | --- |
| 현재 첫 표 아래 괘선 |464.826667px |
| 실제 다음 (주) 글줄의 translate baseline |484.893333px |
| 기존 검사가 선택한 뒤 표 본문 baseline |610.346667px |
| 기존 gap 검사 |145.52px로 PASS |
| 현재 SVG에서 모든 text를 제거한 반례 |대상 없음에도 f64::MAX 기본값으로 PASS |

따라서 고정 영역뿐 아니라 변환 좌표를 무시하는 선택, 유효한 다음 글줄 존재를 확인하지 않는 계약을 함께 고쳐야 합니다. 단순 허용폭 변경으로 해결하지 않습니다. 원 해결 #5829는 미병합 종료됐으나 #5844의 `1b8c39b6c32baf00574564016bd9d9f7d1d88198`로 실제 통합됐고 현재 조상입니다. 도입/현재 검사 SHA256은 `2472b97f95b6b3c8fdb7f67e065db0af9d8e336a9e368e15bdf7a19aea6448bb`로 같습니다.

현재 제품을 수정한 실행이나 과거 binary 반례가 아니라 **현재 SVG에 기존 predicate를 재현하고 문자 제거 대조군을 적용한 감사**입니다. 정식 새 검사 FAIL→PASS나 독립 전쪽 시각 수용으로 세지 않습니다. 첫 CLI -p1은 실제 두 번째 쪽을 출력했으므로 INVALID로 보존하고, 정확한 -p0 첫 쪽만 최종 진단에 사용했습니다.

제품·테스트 수정/새 명명은 아직0건입니다. 전수 조사 후 현행 지침과 독립 근거에 맞는 실제 보정·issue_8232 새 명명·등록/호출 갱신·필수 검증을 계속합니다. 잘못된 기존 단언 자체를 보존하는 조건은 적용하지 않습니다. PR/push는 하지 않았습니다.
