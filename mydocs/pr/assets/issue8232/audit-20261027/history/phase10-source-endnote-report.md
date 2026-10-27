---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase10-source-endnote-report.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase10-source-endnote-report.md` · 원문 SHA-256: `18fae4d6c8291b10376b2271f746dad3fd45ebd81c8e6459d08c59745bb6b960`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

## #8232 추가 조사 — 수치 목적 구분과 미주 간격의 독립 대조

조사 head는 `238de6de41a208648f4ade6b494fe433a81c1fce`, 브랜치는 `audit/8232-px-regressions-20261025`입니다. 제품·테스트 수정·이름 변경·커밋·PR은 아직 없습니다. 이번 기록도 전수 조사 완료 보고가 아닙니다.

### 기존 검사의 보존 범위

기존 검사 자체가 잘못된 출력을 정답으로 고정했다면 그 동작을 무조건 보존하지 않습니다. 기존 목적과 기대값을 현행 지침·독립 근거로 재검증하여 유효한 결함 검출을 보호하고, 부적합한 단언은 보정·대체하거나 제거합니다. 실제 보정 대상만 `issue_8232`로 새로 명명하며 이전 이름 wrapper로 중복 등록하지 않습니다. 현 이슈 본문에도 이 원칙이 반영되어 있음을 다시 확인했습니다.

### source 내부 900그룹 추가 대조

직전 댓글 이후 source 묶음 24–30의 직접 단언 의미와 필요한 입력 문맥을 대조했습니다.

| 묶음 | 함수/helper 수 | 직접 수치가 실물 페이지 절대 px 핀이 아닌 항목 | 독립 계약 확인을 남긴 항목 |
| --- | ---: | ---: | ---: |
| 24–26 | 480 | 480 | 0 |
| 27 | 160 | 120 | 40 |
| 28–29 | 160 | 160 | 0 |
| 30 | 100 | 95 | 5 |
| 합계 | 900 | 855 | 45 |

855항목에는 HWP/HWPX 원본 HU·bit/byte·문자/자원 id·저장/파싱·font em·상대 자간·뷰포트·clip·색상 채널·내용 컷·행/줄 소속·분할 상태가 포함됩니다. 숫자, px 환산 또는 작은 오차가 있다는 이유만으로 삭제하지 않습니다. 이는 실제 테스트 PASS 수나 전체 helper 호출의 적합 판정이 아닙니다.

45항목은 합성 입력이라고 해서 현재 구현 규칙을 독립 정답으로 승인하지 않았습니다. compact 미주 HeightCursor 39항목과 대각선 비율 1항목, 밑줄 0.17em의 출처·저장 행 경계 잔여 25px·저장 꼬리줄 수 제한 2개·각주 안전마진/흐름 허용 1개를 추가 대조 대상으로 남겼습니다. 이 항목 모두가 부적합으로 확정된 것도 아닙니다.

전체 후보 원장은 여전히 9,666그룹·2,103파일입니다. 현재 미분류 6,328그룹 외에도 직접 의미 분류 후 helper·독립 근거·실제 실행 경로 대조가 남은 항목이 있어 전수 완료나 보정 완료로 표시하지 않습니다.

### 미주 `+40px`·`+18px` 기대값의 원 해결 추적

`src/renderer/height_cursor.rs`의 두 검사는 합성 cursor 결과를 `원본 줄간격 +40px` 또는 `+18px`로 계산하여 `1e-6` 오차로 비교합니다. 작은 오차만으로 실물 절대 좌표 핀이라고 판정하지는 않지만, 생산 코드의 상수와 같다는 사실만으로 올바른 단위 계약이라고 보지도 않습니다.

- `+40px` 검사 도입 커밋 `c8b81708c244205fae4dd73a12bbb2e3c19f963a`는 실제 병합 PR #1194, 통합 `6f42d2005ac330a8345cfda6bcaf201d01841ab8`에 연결됩니다.
- `+18px` 검사 도입 `7b5b3dbfcd6af7fde63a9fae1c75f2701319e4ba`는 현재 HEAD의 ancestor이지만 commit→PR 조회에서 연결 PR이 없어 임의로 추정하지 않았습니다.
- 현재 생산 코드의 두 상수 변경 `996e780fe3266b87bc482e81f9de61e3b4b93c24`는 실제 병합 PR #1410, 통합 `a2a1b3837f36083f7bcf2de0653bab4aa94ad78e`에 연결되며 현재 HEAD에 포함됩니다.
- 과거 stage14 기록은 2023년 9월 19쪽 문29 tail의 본문 경계 넘침을 줄이기 위한 변경을 설명합니다. 당시 SVG sweep의 flagged=0을 현재 raw 일치율 증거로 재사용하지 않았습니다.

### 현재 원본·새 Print·Native/fresh WASM 대조

세 원본의 실제 저장 제품은 한컴2022이므로 MCP Print는 engine2020, Print method0, timeout1800초를 지정했습니다. PDFium5.13.0·96CSS DPI·2px raw와 정본 Visual Sweep을 사용하며 전체 캡처는 8개로 제한합니다. fresh WASM은 Mac locked wrapper web/no-opt 로컬 대체 빌드이며 Docker 최적화 결과가 아닙니다.

첫 문서 `samples/3-09월_교육_통합_2023.hwp`는 새 Print와 Native/fresh WASM 모두 20쪽입니다.

| 범위 | Native raw | fresh WASM raw | 90% 미만 |
| --- | ---: | ---: | ---: |
| 전20쪽 최저 | 97.67133% | 97.50439% | 양쪽 0쪽 |
| 관련 19쪽 | 99.18789% | 99.16450% | 양쪽 없음 |

원본 SHA-256: `47a503ea0e92a63ee58b552e661fbde27f8a611afffd67e822be5928319e3c87`.
새 Print PDF SHA-256: `36b7d4aded9a6ff8e1205e9d9f95eb6bd888cb7ade24300da429cb4ddeabcfd3`.
MCP job: `353a7e1f-ca53-4a9c-bdd1-ae359b01e986`, 한컴 `11.0.0.9136`, one-up Print method0.

19쪽 Native review와 Native/fresh WASM overlay를 직접 확인했습니다. 문28/문29 제목·풀이·그림이 같은 단에 이어지고 문29 tail이 하단 프레임 안에 있습니다. 일부 글리프·수식·선의 미세 차이는 남습니다. 이 판독과 수치만으로 `+18px` 또는 `+40px`의 보편적 단위 계약이나 다른 모든 페이지의 px 검사 자격을 확정하지 않았습니다. 독립 기대 좌표·원본 소유·검출력의 연결이 필요합니다.

같은 원인과 연결된 `3-10월_교육_통합_2022.hwp` 18쪽의 새 Print는 완료됐고 Native 전쪽 캡처가 실행 중입니다. 이후 fresh WASM과 `3-11월_실전_통합_2022.hwp` 전21쪽을 순차 대조합니다. 실행 중인 범위를 완료나 PASS로 세지 않습니다.

로컬 증거는 `output/issue8232-audit/phase10-endnote-evidence/` 아래 input/PDF SHA·MCP 영수증·실행 명령·Native/fresh WASM 전쪽 TSV·review/overlay PNG로 보존합니다. 독립 검증·실제 보정·새 명명·필수 재검증을 계속 진행하며 PR은 나중에 처리합니다.

### 이후 추가 조사 — 소스280그룹과 미주 관계 검사

소스 묶음31–32의280그룹을 추가 대조했습니다. 276그룹의 직접 단언은 원본 문자/control offset·HU 저장 치수·byte/bit·XML/ZIP 자원 소유·내용 순서·합성 본문 포함 관계이며, 실물 페이지의 잠정 px 원점 고정과 구분했습니다. 큰 그림 분배 정책2그룹과 이름/실제 검출 범위가 다른 저장 검사2그룹은 독립 근거 대조 대상으로 남겼습니다.

`tests/cases/issue_7642_oct_endnote_relations.rs`도 전체 helper와9개 검사 본문을 읽었습니다. 기존 원장13그룹은 실제 단 내부 포함·선택 영역 교차·미주/수식 원본 소유·내용 순서/비겹침 관계입니다. 숫자 단언 없이 원본 loader·재귀 순회·결과 수집·앞뒤 점유 비교를 담당하는 helper4개도 원장에 연결했습니다. 이에 현재 후보 원장은 **9,670그룹·2,103파일**이고, 미분류는 **6,035그룹**입니다. 직접 의미 분류와 전체 호출 경로 적합 판정을 구분합니다.

### 오래된 Hyperlink 드롭 전제와 실제 검출력

- 원 제안 #4425는 CLOSED·미병합이며, 실제 반영은 #4445의 `baf6ef7ff13ebbb782277b8391b81b81f1250a5f`입니다. 원 커밋 `f6dbf96b47482e13457da1d5df8eebfd0f99c857`와 후속 `f1e7c5706c17b7b51a7f4bf953837430a46470a5`를 확인했습니다.
- 후속 #4775의 실제 통합 `4324eb0e4cf1a65f7efb305993a79ac44859a7ca`는 HWP3 Hyperlink를 HWPX `HYPERLINK` 필드와 Command로 보존하도록 개선했습니다. 두 실제 통합은 현재 HEAD의 ancestor입니다.
- 기존 `issue4388_hyperlink_control_drop_is_not_silent`는 여전히 “드롭이 현재 계약”이라고 설명하지만, 실제 단언은 재파싱 결과에 옛 `Control::Hyperlink` enum이 없다는 조건입니다. 대체 `Field`의 존재·URL은 검사하지 않습니다.
- 기존 두 검사와 같은 작성 입력을 현재 Native rlib에 연결한 관찰 프로그램에서, Hyperlink는 실제 `Field(Hyperlink)`1개와 URL `http://example.com`을 보존했습니다. 정상 보존 중이어도 옛 enum 개수는0이므로 기존 검사는 PASS합니다.
- 그 재파싱 결과에서 Field만 제거한 별도 음성 대조에서도 옛 enum 부재 조건은 그대로 참이었습니다. **이는 단언의 검출력 반례이며 과거 엔진이나 보정 후 엔진 실행 결과가 아닙니다.** 실제 정상 URL 보존을 보호해야 하며, 옛 드롭 설명을 유지하려고 보존 기능을 지우면 안 됩니다.
- Unknown 입력에서는 실제 stderr 경고가 관측됐지만 기존 테스트는 경고를 단언하지 않습니다. 이 관찰의 구조화된 내용 손실 보고 count0도 기록하되, 곧바로 별개의 제품 결함이나 전체 손실 보고 정책 변경으로 확대하지 않았습니다.
- 같은 source의 기존 release-test root binary에서 두 검사를 각각 exact 실행하여 각각1PASS/0FAIL·3812filtered·exit0을 확인했습니다. 전체 회귀나 검사 보정 후 통과가 아닙니다.

### 유지할 근거를 확인한 기존 ±2px 검사

`verified_september_2023_note_print_origins.rs`의 HWP caller는 14쪽 문20/문22 제목 기준선을 독립 Print의±2CSSpx로 검사하면서 원본 미주·본문·첫 수식의 소유/순서/누락/중복도 보호합니다.

새 Print의 PDFium `FPDFText_GetCharOrigin`을72pt에서96CSSpx로 환산한 결과, 문20은 `(34.0799993,415.2000326)`, 문22는 `(402.5599772,239.0400391)`이었습니다. 기존 기대값 `(34.08,415.20)`·`(402.56,239.04)`와 일치합니다. 해당14쪽의 현재 raw는 **Native99.34847%·fresh WASM99.27113%**이며 review/양쪽 overlay를 직접 판독했습니다.

실제 기존 검사 source를 현재 Native rlib와 standalone `rustc --test`로 연결해 HWP 경로1개를 실행한 결과, **1PASS/0FAIL·1filtered·47.39초·exit0**입니다. 자동 등록도 `tests/generated/regression_suite_024.rs`에 연결되어 있음을 확인했습니다. 이 실행을 integration suite 전체 실행으로 보고하지 않습니다. 로그의 다른 쪽 `LAYOUT_OVERFLOW` 진단은 남겨 두었으며 별도 실제 paint 대응 확인 대상으로 기록했습니다.

이 HWP caller의14쪽 px 검사에는 유지 근거가 있습니다. 같은 helper의 HWPX caller는 다른 원본이므로 별도 자격 확인이 필요합니다. HWPX 저장 제품이 한컴2024·20쪽임을 확인하고 engine2024의 독립 Print/전쪽 비교를 현재 캡처 종료 뒤 공유 lock으로 순차 실행하도록 대기시켰습니다. 아직 HWPX 파일이나 전체 helper의 적합 판정을 완료하지 않았습니다.

### 전쪽 비교 진행 갱신

2022년10월 원본은 새 Print·Native·fresh WASM 모두18쪽이며, 전쪽 최저는 각각 **98.10796%·98.05920%**, 90% 미만은 양쪽0쪽입니다. Native11쪽과 WASM17쪽의 review·overlay를 직접 확인했으며, 미주18→19→20의 간격과 문29의 풀이 tail이 대응 내용과 같은 단에 이어집니다. 일부 글리프·수식·정답 표시 위치 차이는 남습니다.

2022년11월 원본은 새 Print·Native·fresh WASM 모두21쪽으로 산출을 마쳤습니다. Native11쪽과 WASM19쪽의 review·overlay도 직접 확인했습니다. 문13→14 및 문24→25의 제목·풀이·그림과 후속 내용이 같은 단에 이어지고 프레임 안에 남습니다. 일부 수식·그림 선·글리프 차이는 남으며 전체 피델리티 개선 완료를 뜻하지 않습니다.

| 원본 | 양쪽 각각의 전체 쪽수 | Native raw 최저 | fresh WASM raw 최저 | 90% 미만 |
| --- | ---: | ---: | ---: | ---: |
| 2023년9월 | 20 | 97.67133% | 97.50439% | 양쪽0쪽 |
| 2022년10월 | 18 | 98.10796% | 98.05920% | 양쪽0쪽 |
| 2022년11월 | 21 | 96.48321% | 96.52279% | 양쪽0쪽 |

2022년11월11쪽은 Native97.87979%·WASM97.87183%이므로 절대 px 좌표 고정 자격은 없습니다. 대표 판독이나 다른 쪽의 높은 수치로 이 조건을 대신하지 않습니다. 이번 실행은 총59쪽/각 backend의 TSV 산출을 완료한 것이며, 여전히39개 미주 cursor 단위 기대 규칙의 독립 타당성 판정을 모두 마친 것은 아닙니다.

phase10은 실제 exit0과 최종 상태를 확인했고, 대기 중이던 HWPX 비교는 capture lock을 넘겨받아 engine2024 Print job `837f88e3-90c1-48c8-ab79-07c74847830b`를 시작했습니다. 전수 조사·독립 판정·실제 검사/출력 보정·이름/등록 변경은 계속 미완료입니다.

### 추가 조사 — 419개 함수와 쪽수 기대값 생산 helper 6개

소스 묶음33–35의 247개와 integration 묶음09의 172개를 추가로 대조했습니다. 직접 수치 의미가 실물 페이지의 절대 px 핀이 아닌 항목은 400개이며, 19개는 독립 계약 확인을 남겼습니다. 이는 실제 테스트 PASS나 파일 전체 적합 판정 수가 아닙니다.

- 표의 행·열·병합·텍스트 소유·undo/저장, HWP/HWPX의 XML·byte·HU, WMF의 bitmap byte 길이·overflow, CLI/JSON·통계 단위는 실물 페이지 위치 핀과 구분했습니다.
- `test_paste_cascade_floating_picture`의 567HU 기본 이동량은 원본 작성값이 아니라 붙여넣기 정책 상수이므로 독립 계약의 근거를 확인할 항목으로 남겼습니다.
- 합성 PDF 자원 검사 113개 그룹의 수치는 자원 한도·byte·codec·색상/alpha 샘플과 작성한 page tree의 leaf 수입니다. 원본 이미지의 decode 샘플 보존이나 같은 호출의 byte 동등을 전체 페이지 배치 golden과 혼동하지 않습니다.
- `tests/hwpx_to_hwp_adapter.rs`에서 쪽수 기준과 기존 위반 상태를 요구하는 18개 그룹을 추가 확인 대상으로 남겼습니다. 일부 probe의 고정 9쪽은 독립 Print가 아니라 rhwp 재로드 기준입니다. source LineSeg의 68258HU 저장 보존은 별개의 원본 단위 계약입니다.
- 숫자 단언이 없어 기존 목록에 없던 loader·쪽수 생산/소비 helper 6개도 전체 본문과 caller를 연결했습니다. `oracle_hwp_page_count_for_hwpx`는 한컴 저장 HWP를 **현재 rhwp로 조판한 쪽수**이고, 읽기/파싱 실패 시 caller의 rhwp HWPX 쪽수로 fallback합니다. 독립 Print 정답을 입증한 것으로 세지 않습니다. 이 중 3개 helper는 독립 기준 확인을 남겼습니다.

현재 원장은 **9,676그룹·2,103파일**, 미분류는 **5,616그룹**입니다. 직접 의미 분류 후에도 helper·독립 근거·실제 실행 대조가 남습니다. source 내부의 직접 숫자 의미 분류를 source 경로 전체의 적합 판정으로 확대하지 않습니다. 제품·테스트 수정·새 명명은 여전히 0건이며 PR은 만들지 않았습니다.

### HWPX caller의 기존 ±2px 검사 자격 대조 완료

`samples/3-09월_교육_통합_2023.hwpx`는 실제 저장 제품이 한컴2024이므로 engine2024로 별도 Print를 만들었습니다. 원본 SHA-256은 `9ff4ba0f8530621c0ad89be3bc563135cc88f75052949e4d444cfcd715375340`, PDF SHA-256은 `f63ea5e5c15c6bcf1208a29fe5c2845ad5413d1e390ac8d27b93311ce3a02371`입니다. job `837f88e3-90c1-48c8-ab79-07c74847830b`, 한컴 `13.0.0.3901`, 전처리 없음·one-up Print method0·20쪽을 확인했습니다.

| 범위 | Native raw | fresh WASM raw | 90% 미만 |
| --- | ---: | ---: | ---: |
| 전20쪽 최저 | 97.71868% | 97.55401% | 양쪽 0쪽 |
| 해당 14쪽 | 99.38688% | 99.31356% | 양쪽 없음 |

두 backend의 전체 TSV·쪽수 20쪽·종료 코드0을 확인했습니다. PDFium5.13.0의 문자 원점을 96CSSpx로 환산한 문20 `(34.0799993,415.2000326)`과 문22 `(402.5599772,239.0400391)`은 기존 기대값과 일치합니다. 같은 helper의 HWP와 HWPX를 서로 대신한 것이 아니라 각각 독립 Print와 대조했습니다.

Native와 fresh WASM의 14쪽 review/overlay 및 문20·문22 제목/후속 본문/첫 수식을 3배 확대해 직접 읽었습니다. 대응 내용과 위치·단·순서는 맞으며, Native의 미세 글리프 차이와 WASM의 제목/본문 두께·수식 모양 차이는 남습니다. 이번 판정은 제목 위치의 기존 ±2px 검사 유지 근거이며 글꼴 모양 전체의 일치 판정이 아닙니다.

현재 source와 Native rlib의 SHA를 다시 확인한 뒤 이미 컴파일한 실제 기존 HWPX caller를 exact 실행했습니다. 결과는 **1 PASS / 0 FAIL / 1 filtered, 167.55초, exit0**입니다. 실제 자동 등록은 기존 `tests/generated/regression_suite_024.rs`에 있습니다. standalone `rustc --test` 실행이므로 suite 전체·CI 통과로 보고하지 않습니다. 검사 로그의 다른 쪽 `LAYOUT_OVERFLOW` 진단은 보존했으며 실제 paint 대응을 별도로 확인할 항목으로 남겼습니다.

관련 전쪽 TSV·원본/PDF·실행/빌드 hash·독립 좌표·직접 판독 기록은 `output/issue8232-audit/phase11-existing-px-evidence/`와 `phase11-existing-px-page-qualification.json`, `phase11-existing-hwpx-test.log`에 보존합니다. 통합 시각 증거 목록은 87조건·각 backend 2,412쪽이며, 기존 무변경 코드의 capture 재사용과 현재 생성 증거를 구분합니다. 목록의 수치 자격은 전수 감사 또는 각 검사의 최종 적합 판정이 아닙니다.
