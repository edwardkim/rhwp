---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase15-survey-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase15-survey-report.md` · 원문 SHA-256: `18e0dcaca4c219e97dc39b7d21fd58f1e7ba7c74ef07d949280bdabafd1d9213`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## 15차 조사 — 빈 응답을 검사하는 MCP 회귀와 #6950 좌표 자격

기준은 `audit/8232-px-regressions-20261025`, head `238de6de41a208648f4ade6b494fe433a81c1fce`입니다. 작업장은 clean이며 제품·테스트 보정/개명은 아직 0건입니다.

### 1. MCP 검사의 실제 미검출 반례

`tests/mcp_server_contract.rs::boolean_false_does_not_inject_a_presence_flag`의 helper는 이미 `content[0].text`를 JSON으로 파싱한 **본문**을 반환합니다. 검사 함수가 여기에 다시 `result.content[0].text`를 적용하므로 빈 문자열을 얻고, `dryRun:true`가 없다는 부정 검사가 통과합니다.

같은 현재 서버에서 실제로 false/true 두 요청을 실행했습니다. false는 파일을 기록하고 본문에 false를 반환했으며, true는 파일을 기록하지 않고 본문에 true를 반환했습니다. 제품은 정상 동작했습니다. 그러나 기존 검사 식은 두 응답을 모두 PASS시켰습니다. 직접 본문의 boolean과 파일 기록 여부를 보호하는 계약은 반대 true 응답을 거부할 수 있습니다. 이는 현재 응답에 대한 검사 식의 반례 관찰이며, 과거 binary나 production mutation 전후 검증은 아닙니다.

원 PR [#3724](https://github.com/edwardkim/rhwp/pull/3724)는 미병합 CLOSED지만, 누적 commit `e22e60dfcd8120ce3e873e35ee0531781a46eeb1`이 통합 [#3742](https://github.com/edwardkim/rhwp/pull/3742)의 merge `cc382911684c05c041a6b5ae86a873f7321f2153`를 통해 현재 head에 포함돼 있습니다. helper의 최초 도입 때도 반환 형태는 파싱된 본문이었고, 잘못된 검사 본문은 도입 후 그대로입니다. 미병합 원 PR을 미반영으로 오인하지 않습니다.

### 2. #6950 원본 3쪽과 기존 px 검사

동일 원본 `samples/hwpx/20260909-para-table.hwpx`와 독립 한컴 PDF `pdf/hwpx/20260909-para-table-2024.pdf`를 전쪽 비교했습니다. 저장 제품 버전과 조판 호환 모드는 별개이므로, 검사 함수가 사용하는 기본 모드와 2024 호환 모드를 각각 실행했습니다.

| 조판 설정 | 쪽수 | Native 최저 raw | fresh WASM 최저 raw | 미달·누락·쪽수 차이 |
| --- | ---: | ---: | ---: | --- |
| 검사 기본 모드 | 3/3 | 99.13921% | 99.13741% | 모두 0 |
| 2024 호환 모드 | 3/3 | 99.13920% | 99.13740% | 모두 0 |

PDFium 5.13.0·96 CSS dpi·2px raw이며, Mac WASM은 locked wrapper의 web/no-opt 로컬 대체 빌드입니다. 두 설정의 PNG를 동일하다고 주장하거나 한 설정의 결과로 다른 설정을 대체하지 않았습니다. 양쪽 전쪽 review와 핵심 영역 확대를 직접 판독했으며, 글리프 획·점선 농도·화살표 이미지의 미세 차이는 남습니다.

- 3쪽 raw는 검사 기본 모드에서 **Native 99.79849% / WASM 99.79842%**입니다. 독립 PDF 원시 괘선의 y는 **307.8226666667 CSSpx**로 기존 기대 307.823과 일치합니다. 기존 ±0.6px는 현행 ±2 CSSpx로 보정할 대상입니다. 원본 최소 높이 54805HU·내용 52982HU·안 여백 282HU에서 얻는 **1541HU 정렬 공간**, 셀 포함·내용 소유도 함께 보호해야 합니다.
- 1쪽의 7085HU 본문 원점은 원본 XML의 top4251+header2834이며, 본문 높이70018HU와 마지막 줄 vpos68707/LH1200도 직접 확인했습니다. 이 검사는 불명확한 PDF 좌표 literal과 구분되는 원본 단위 계약입니다.
- 기존 정식 Native 모듈은 **27 PASS / 0 FAIL**입니다. 실행 summary 0.944초, 컴파일 포함 19.055초이며 필터 제외 217개는 실행 수에 합산하지 않습니다. 캡션 기대값의 동일 helper 의존과 수동 본문/정렬 변형의 근거는 별도 미완료입니다.

### 3. 조사 원장과 남은 범위

이번에는 5개 파일의 검사 104개·helper 53개, 총 157개 함수를 직접 분류했습니다. CLI/MCP의 byte·문자 수·논리 셀 주소·원본 HU·현재 단건/배치 동등성은 실물 px 핀과 구분했습니다. 자료가 없으면 그냥 반환하는 6개 경로와 위 MCP 검사도 기록했으며, 현재 자료의 존재를 미실행 PASS 경로의 해소로 간주하지 않습니다.

현재 통합 원장은 **10,024그룹·2,103파일**입니다. 미분류 4,795, 직접 의미 판정 4,449, 독립 증거 확인 중 438, 의미 메모 후 재판정 130, 파일 문맥만 확인 212입니다. 판정 수는 적합/보정 완료 수가 아닙니다. 시각 목록은 두 조판 설정을 구분한 **93조건·각 backend 2,499쪽**입니다.

원시 자료는 ignored `output/issue8232-audit/`의 `mcp-boolean-response-owner/observation.json`, `mcp-boolean-origin-pr-metadata.json`, `phase15-default-para-table-evidence/`, `phase15-para-table-evidence/`, `phase15-existing-tests.log`와 원장에 보존했습니다. 원본·기준 PDF·기존 증거는 변경하지 않았습니다.

전수 조사·독립 판정과 이후 실제 검사 보정·`issue_8232` 새 명명·등록/호출 갱신·필수 검증은 계속 남아 있습니다. PR은 생성하지 않았고 이슈를 완료로 처리하지 않습니다.
