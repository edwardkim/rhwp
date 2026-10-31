# #8174 v2-S38 — 요청 수명·정책·링크의 공개 API 회귀 이관

Issue: #8174. 2026-10-26, 기준 로컬 커밋 `637fd6799a`.
T04의 기존 테스트 정책 보정 첫 절편이며 전체 unit-tier 통과는 아직 아니다.

## 변경과 보존

세 source-side 신규 테스트 모듈의 10개 계약을 `tests/cases/`의 공개 API 검사 10개로 옮겼다.
생산 코드의 API·연산은 바꾸지 않았다. 원본 파일 전체·source commit·Git blob·SHA-256과
제거 전 테스트 수를 ignored `output/pr-review/issue8174/v2-s38-20261026/original/` 및
`migration-inputs.json`에 보존했다. `production-prefix-comparison.json`에서 세 파일의 생산
부분이 바뀌지 않았음을 확인했다. 새 private 지원 API·harness·정책 기준선을 만들지 않았다.

| 이전 source 모듈 | 공개 경로와 유지한 계약 |
| --- | --- |
| `document_core/pdf_import_session.rs::tests` 7개 | `issue_8174_pdf_session_lifetime.rs`: parser와 Core의 동일 요청, 정상 편집의 별도 수명, HWP/HWPX 첫 저장·진단의 누적 계상, 정확한 진단 backing, 출력/decoded 초과의 실패 고정·live IR 보존·새 요청 복구, 초기화 거절·자동 감지 일치 |
| `parser/pdf/policy.rs::tests` 1개 | 같은 공개 session 파일의 선택 7쪽 실제 가져오기·HWP 첫 저장. 상세 표 추론을 포함해 256 MiB를 넘는 누적 작업을 정상 수용하고 명시적인 64 MiB host cap은 거절한다. private `for_selected_pages`/가짜 `charge_decoded` 호출을 쓰지 않는다. |
| `model/hyperlink.rs::destination_tests` 2개 | `issue_8174_hyperlink_destinations.rs`: 공개 목적지 분류·command 왕복의 원문 보존과 실행 scheme·제어 문자·인증 정보를 담은 URL 거절 |

private `request.restrict_work_limits`는 초기 공개 policy의 낮은 cap으로 바꿨고, private
snapshot 호출은 실제 공개 HWP export로 같은 생산 경계를 실행한다. 기존 source test가
확인하던 상태 보존·실패 뒤 재사용 거절을 제거하거나 성공으로 기대값을 바꾸지 않았다.

## 실행 증거

- 공개 API integration 원본 **10 PASS**. 기존 공유 target의 Native rlib로 직접 연결해
  실행했다. 해당 생산 API는 이관 전후 동일하며 S36 Native 생산 source와 대조했다.
  집중 실행이며 최종 자동 suite/정확한 전체 head 검증은 T08에서 수행한다.
- 선택 쪽 사례의 첫 실행은 상세 추론 옵션이 빠져 256 MiB 초과 조건을 재현하지 못했다.
  이미 S34에서 측정한 실제 사례와 같은 `include_table_inference=true`로 바로잡았다.
  입력 조건을 고친 뒤 모든 검사를 재실행했다. 최초 실패는 `initial-runner-failure.txt`에 보존했다.
- 현재 최적화 WASM(S37)의 선택 7·9쪽은 직접/HWP/HWPX 세 경로의 최초 열기와 삽입·삭제
  되돌리기를 확인했다. **12개 상태**의 글자 원점 차이는 0이며 첫 열기 원점도 같다.
  독립 기준과 새 전체 페이지 TSV의 최저 raw 일치율 **99.85041%**를 확인했다.
- S36 Native의 실제 선택 쪽 저장본을 현재 다시 렌더링한 HWP/HWPX **4개 상태**도
  독립 기준과 최저 **99.96192%**다. 최신 Native 제품을 새로 빌드했다고 표시하지 않는다.
  CFF의 양쪽 100% 증거와 함께 공개 계약 입력의 시각 선행 조건을 유지한다.
- 이전에 고정한 글꼴 프로그램 subset/style만 같은 공급 자원으로 재사용했다. 현재 생산
  SVG의 글꼴 family를 대조하고 새 geometry에 공급했으며 이전 geometry·PNG는 재사용하지
  않았다. font style·현재 raw SVG hash를 `font-program-reuse.json`에 기록했다. 현재
  compare·overlay·review에서 본문 줄·표 외곽·사진/설명과 표제·그림 경계를 직접 대조했다.
- S37 대비 unit-tier 검사 통과: **4,140 tests / 304 modules**. cached devel 대비 전체
  검사는 여전히 **4,140 > 4,100**과 나머지 신규 모듈/지원 항목으로 실패한다. 부분 통과를
  PR 전체 통과로 올리지 않는다. 나머지 40개 증가분과 지원 항목의 이관을 계속한다.

현재 TSV·PNG·실행 명령·원시 로그·입력/제품 식별은 위 ignored evidence 경로에 있다.
렌더링 생산 코드를 바꾸거나 새 px/baseline을 추가하지 않았다. 자동 수집 파일은 직접 생성·
stage하지 않았다. 전체 이슈·PR 준비의 완료 보고가 아니다.
