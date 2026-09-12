# #3587 Stage 1 — 템플릿 자동화 계약 조사

- 일자: 2026-09-12
- 상태: 조사 결과·구현계획 초안 보존, 기여자 응답 대기. 제품 구현·전체 회귀·한컴 시각 검증 전.
- 수행계획 승인 기록: `b3f9e4796`
- 조사 source: `59a11f180ad1bd5cadcbbf0a6dc9d0162f4a0a21`과 동일한 제품 코드
- [수행계획](../plans/task_m100_3587.md), [포크 검토](task_m100_3587_fork_review.md), [구현계획](../plans/task_m100_3587_impl.md)

## 1. 결론 — 무엇을 먼저 고쳐야 하는가

이번 목표는 **원형 블록을 복제하고 데이터를 채우되 앞뒤 문단을 보존하는 공통 자동화**다.
기존 코드에는 필드·셀 채우기, 문서별 일괄 생성, clipboard, 문서 간 서식 이식이 있다.
그러나 이를 곧바로 안전한 반복 블록 생성이라고 부를 수는 없다.

1. **실행으로 확인한 결함:** 표·글상자를 복사하면 개체 ID까지 복제된다. 중첩 표·그림·필드도
   포함되며, HWP/HWPX 저장·재열기에서 중복이 남는다. 먼저 개체 복제의 신원을 보정해야 한다.
2. **계약 차이:** 기존 붙여넣기는 커서 중심 편집이다. 후속 빈 문단 생성, 앞뒤 문단과의 merge가
   의도적으로 있다. 원형 블록을 독립 문단 묶음으로 삽입하는 자동화에는 별도 명시적 계약이 필요하다.
3. **재사용할 기능:** 단일 필드·셀 채우기와 문서별 mail merge는 다시 만들지 않는다.
   다중 작업 실행기도 기존 `run`/MCP를 확장하는 방향으로 설계한다.
4. **혼동하면 안 되는 것:** `beginBatch`는 페이지네이션 지연이며 자동 rollback이 아니다.
   기존 DSEL 주소도 변경 후 자동 추적되는 영구 ID가 아니다.

네 후처리 함수를 먼저 붙이고 2pt 압축으로 빈 페이지를 줄이는 방향을 권고하지 않는다.
**신원·참조 → 경계 보존 블록 삽입 → 반복·채우기 → 공개 실행 경로** 순서가 필요하다.

## 2. 실행 증적

기존 Docker WASM을 재사용했다. 제품 source를 바꾸지 않았으므로 불필요한 재빌드는 하지 않았다.

- WASM SHA-256: `5cd0f9fe37155593aa5e89afebbc903923935eceefbf4c50827452f1172ceba8`
- 재현 probe: [template-contract.mjs](../tech/investigations/issue-3587/probes/template-contract.mjs)
- 실행: 저장소 루트에서 `node mydocs/tech/investigations/issue-3587/probes/template-contract.mjs`
- 상세 결과: 로컬 `output/3587/stage1/probe-result.json` (생성물, 커밋하지 않음)
- 원본 샘플과 Studio의 WASM·서버는 변경하지 않았다. 생성 문서는 메모리 안에서만 저장·재열기했다.

### 2.1 기존 실물 샘플 — 복제와 ID

| 입력·대상 | 원본 중복 | 복제 후 관찰 | HWP/HWPX 저장 후 rhwp 재열기 |
| --- | --- | --- | --- |
| `samples/hwp_table_test.hwp`, s0/pi3/ci0 | 검사 대상 중복 0 | 표 `1559451900`이 원본·복제본에 2회 | 두 경로 모두 중복 유지 |
| `samples/table-in-tbox.hwp`, s0/pi0/ci2 | 검사 대상 중복 0 | rect·중첩 tbl·pic·fieldBegin 11개 종류/ID 쌍이 각각 2회 | 두 경로 모두 중복 유지 |

각 샘플을 rhwp에서 HWPX로 내보내 다시 열어 **HWPX 입력 경로에서도** 같은 복제를 실행했다.
결과가 같았다. 이 HWPX는 rhwp 변환본이지 한컴이 만든 독립 오라클이 아니다.

첫 표의 복제본에 `COPY_ONLY`를 삽입하고 해당 구간을 bold로 바꿨다.
원본 셀 텍스트·원본 첫 글자 속성은 유지됐고 복제본에만 문구가 들어갔다(두 입력 경로 모두).
따라서 **좌표 기반 개별 편집 성공**과 **ID 중복 결함**은 동시에 성립한다.
ID 중복으로 한컴에서 실제 오선택·파일 손상이 일어났다고까지 실증한 것은 아니다.

probe는 `tbl/rect/pic/container/ellipse/line/fieldBegin`의 XML `id` 중복을 검사한다.
모든 컨트롤 종류의 ID·참조를 전수 검증한 결과가 아니며, 서로 다른 종류 사이의 충돌도 별도 검사가 필요하다.
HWP 저장 결과는 rhwp로 재파싱한 뒤 HWPX로 투영해서 관찰했다. 원본 HWP 바이트 직접 비교는 미실행이다.

입력 SHA-256:

- hwp_table_test: `dc3e57d7577447ae4b47f59bdb0f499a5ac29066b62af8a650bcdd8d70eee32a`
- table-in-tbox: `02b0ced8b96cec5b6e1cad6dd1926b701a2875ea9591c589e57edf7653daf4af`

### 2.2 경계와 batch — API 계약용 합성 입력

`createEmpty` 다음 `createBlankDocument`로 정상 기본 스타일을 준비하고,
BEFORE 문단·표·AFTER 문단을 만들었다. 이 입력은 한컴 시각 판정용 샘플이 아니다.

- 표 2회 추가 전 본문 문단 4개 → 추가 후 8개. 각 paste가 표 문단과 후속 빈 문단을 넣었다.
- BEFORE/AFTER의 직접 텍스트·문단 속성·컨트롤 종류·run 서식 참조는 편집 직후 동일했다.
  모든 내부 구조와 저장 후 불변식을 통과한 것으로 확대하지 않는다.
- 생성 표의 HWPX `id=0`이 3회, HWP 재열기에서는 `2081831960`이 3회였다.
  `create_table_native`의 IR common ID와 raw ID를 함께 다룰 필요가 있다.
- `beginBatch` → 정상 편집 → 잘못된 구역 인덱스의 편집 실패 뒤, 앞의 정상 편집은 남았다.
  명시적으로 `restoreSnapshot`을 호출한 뒤에는 텍스트가 복구됐다.
- 첫 진단 시 `createBlankDocument`를 빠뜨려 미등록 스타일 0 참조로 HWPX export가 실패했다.
  초기화 절차를 수정해 다시 실행했다. 이 실행 준비 실패를 템플릿 복제 회귀로 세지 않았다.

## 3. 원인과 계보

### 3.1 빈 문단은 이번 fork가 새로 만든 현상이 아니다

현행 `clipboard.rs::paste_control_native`는 clipboard 문단을 clone한 뒤 커서 조건에 따라
치환/삽입/분할하고 **후속 빈 문단을 항상 추가**한다. `paste_internal_native`는 첫·마지막
clipboard 문단을 대상 문단의 좌우 조각과 merge한다.

후속 빈 문단 추가는 저장소 최초 commit `f0f7f1a4b4`(2026-03-27)의 같은 함수에도 있다.
포크와 현재 devel 모두 이를 이어받았다. 이것은 기존 대화형 붙여넣기 동작이며, 빈 문단 자체를
일괄 회귀로 규정하거나 기존 사용자 Enter까지 지우면 안 된다.
원 기여자의 실제 호출 순서·92쪽 산출물은 확보하지 못했으므로 그 증상의 전체 원인은 미확정이다.

### 3.2 복제 신원의 보존/재발급 책임이 분산돼 있다

- `clipboard.rs::copy_control_native`와 paste는 `control.clone()`/문단 clone을 사용한다.
  해당 복제 경로에 재귀 신원 재발급이 보이지 않으며 §2에서 저장까지 중복이 확인됐다.
- `object_ops/table.rs`의 표 생성은 크기 기반 값을 raw INSTANCE_ID에 기록하지만
  `common.instance_id`는 기본값이다. 같은 크기의 독립 생성에도 충돌 가능성이 있는 설계다.
  이번 실행은 독립 생성 3회가 아니라 하나를 2회 복제한 경우이므로 두 재현을 혼동하지 않는다.
- `table_ops.rs::split_table_native`는 별도로 뒤 표 ID를 계산해 common/raw에 쓴다.
  이처럼 각 연산의 개별 해시만으로 문서 전체의 무충돌을 보장할 수 없다.
- HWP 저장 `serializer/control.rs::serialize_table`은 raw/seal에 따라 raw 또는 common을 쓴다.
  HWPX 저장 `serializer/hwpx/table.rs::write_table`은 common ID를 쓴다.
  한쪽만 수정하는 방식은 두 출력이 달라질 수 있다.

이 단계에서 확인한 것은 **기존 제품의 복제 신원 보완 필요성**이다. 이번 타스크의 코드 변경은
없으며, 모든 ID 문제의 최초 도입 commit을 확정했다고 주장하지 않는다.

## 4. 식별자·참조 책임표

공식 HWP 5.0 문서의 [표 67·69](../tech/한글문서파일형식_5.0_revision1.3.md)는 컨트롤 종류와
문서 내 개체의 고유 instance ID를 구별한다. 두 값을 통째로 같은 규칙으로 재발급하지 않는다.

| 항목 | 현재 경로/의미 | 자동화에 필요한 계약 |
| --- | --- | --- |
| ctrl ID (`tbl `, `$rec` 등) | 개체 종류 | 그대로 보존 |
| `common.instance_id`, table raw ID | 개체 신원, HWP/HWPX 저장 경로 차이 | 복제본에 새 ID, common/raw 일관성, 기존 개체는 변경하지 않음 |
| drawing `inst_id`·그림 고유 값 | 공통 ID와 별도 저장 슬롯이 존재 | 무조건 동일 번호로 합치지 않음. 각 역할을 보존한 재매핑 |
| 연결선 SubjectID | `object_ops/connector.rs`는 drawing.inst_id로 연결점 조회 | 블록 안 연결은 새 대상에, 블록 밖 연결은 명시 정책 또는 거부 |
| 문단 ID | HWP raw_header_extra, HWPX `next_para_id` | 저장 포맷 정책 유지. HWPX 재채번 값을 세션의 안정 주소로 쓰지 않음 |
| 필드 ID/필드 짝 | 복제 시 중복 관찰, foreign paste는 field_id 증가 | begin/end·내부 참조를 쌍으로 매핑, 이름 중복과 ID 중복 구별 |
| 글자·문단·스타일 참조 | 같은 문서에서는 유효한 참조 공유 가능 | 값까지 무조건 복제하지 않음. 원본의 서식이 변하지 않는 편집 계약 |
| 테두리·번호·폰트·BinData | 외부 문서에서는 번호 의미가 다름 | 기존 DocInfo/BinPool 매핑 재사용, 없는 참조는 명시 실패 |

HWPX 문단 ID는 probe의 재열기 결과에서 모두 유일했다(표 샘플 152/152, 글상자 샘플 509/509).
이는 개체 ID의 유일성을 대신하지 않는다. HWP 문단 ID는 0 값 처리 등 기존 호환 정책을 따로 검토한다.

## 5. 전체 사용 흐름별 기존 기능과 차이

| 기능 축 | 근거 | 분류 및 결정 제안 |
| --- | --- | --- |
| 고정 필드·셀·문구 채우기 | `cli/commands/edit/fields.rs`, `form_filling_guide.md` | **재사용**. 동명 필드 `[N]`, notFound/ambiguous를 소비자가 검사 |
| 한 데이터 행 → 한 문서 | `cli/commands/batch_fill.rs`, `tests/batch_fill_contract.rs` | **재사용**. 한 문서 내 반복 블록 API가 아님 |
| 표 행 삽입 | `table_ops.rs`, `model/table.rs::insert_row/new_from_template` | **재사용+추가 계약**. 새 빈 셀은 첫 문단 서식 일부를 상속하며 원형 행의 모든 문단/컨트롤 복제가 아님 |
| 표·글상자 복사 | `clipboard.rs` 및 §2 | **보정**. 신원 재매핑 및 strict 블록 삽입으로 연결 |
| 제목+표+글상자 묶음 반복 | 기존 selection paste의 경계 merge | **신규 공통 기능**. 문단 범위를 독립 블록으로 복제, 임의 후속 빈 문단 생성 금지 |
| 다중 생성 영역 지정 | `src/agent/dsel/node.rs`의 구조 경로 | **재사용+보정**. 입력 snapshot의 주소로 계획 후 이동 대응표 제공. 영구 앵커로 오인 금지 |
| 문서 간 가져오기 | `foreign_paste.rs::merge_foreign_doc_info/RemapCtx` | **재사용+보완 검증**. 표·글상자·캡션·그림 등의 서식/자원 재귀 매핑. 구역 평탄화·경계 merge는 블록 계약과 다름 |
| 계획·dry-run·저널 | `cli/protocol/plan/execution.rs`, `plan_schema.rs` | **확장 재사용**. 현 action 4종은 채우기/치환/체크/셀 쓰기, 구조 복제 없음 |
| 원자성·실패 | run의 로컬 문서·저장 전 검증, core snapshot/batch | **계층별 계약 보완**. run은 step 실패 전 파일 미작성이나 파일 write의 crash 원자성까지는 보장하지 않음 |
| agent/MCP 노출 | `cli/metadata/mcp/protocol.rs::hwp_run_plan`, `src/agent/mod.rs` | **기존 실행기 확장**. agent/mod의 8층 설계 주석과 달리 실제 export는 dsel만 있음 |
| 이미지·선택 항목·데이터 변환 | 기존 그림 삽입/조건부 step | **업무 정책과 분리**. 엔진이 60% 높이·9pt·6×3 형태를 추측하지 않음 |
| 일반 구역 병합/신규 DSL/서비스 | 이슈 원래 제안과 확장 목표 | **이번 구현 제외 제안**. 근거·범위를 승인받고 제외, 자식 이슈 자동 생성하지 않음 |

`foreign_paste`의 스타일/BinData 및 필드 재매핑 코드는 정적 조사했다. 이번 probe는 외부 문서
가져오기까지 실행하지 않았다. 단순히 재귀 코드가 있다는 이유로 모든 참조·raw·실패 원자성의
완성을 선언하지 않는다. 이것은 후속 묶음의 선행 검사다.

## 6. 기존 검증 재사용과 미검증 경계

- `tests/edit_fill_fields_contract.rs`: field-01, dry-run/잘못된 입력/값 저장 재독.
- `tests/batch_fill_contract.rs`: 행별 생성, 이름 충돌, 실패 행 보존, 출력 순서, 재독, MCP 선언.
- `tests/run_plan_contract.rs`, `run_plan_dry_run_contract.rs`, `run_plan_cas_contract.rs`:
  선검증·실행·입력 지문·저장 전 검사.
- `tests/cases/foreign_paste.rs`: 외부 폰트/문단/테두리 참조, 동일 서식 재사용, 셀 문단.
- `tests/cases/issue_4275_nested_table_paste_style.rs`: 중첩 표 HTML 교환의 스타일.
- 기존 `src/wasm_api/tests.rs`의 copy/paste 및 표 저장 검사는 참고하되 새 테스트는 그 안에 넣지 않는다.
  일부 옛 검사는 `pasts/` 파일 부재 시 skip하므로 존재만으로 보장이라고 보고하지 않는다.

위 Rust suite는 **내용과 위치를 조사했으며 이번 Stage 1에서 실행하지 않았다**.
API probe 6경로(합성 경계, batch, 실물 2개×입력 2형식)를 실행했다.
원 기여자 원형/완성본, 다른 문서 가져오기, 블록 외부 연결 참조, 복제 후 삭제, 전수 스타일/자원
무결성, 전체 회귀, 한컴·브라우저 시각 대조, 성능 계측은 아직 미실행이다.
기존 실물 fixture를 우선 사용하고 부족한 경우에만 검증 목적을 정한 샘플을 추가한다.

## 7. 원 제안 네 함수의 처리 제안

| 제안 | 권고 |
| --- | --- |
| orphan 정리 | 구역 전체 추측 삭제 대신 신규 블록 생성 시 잉여 문단을 만들지 않음. 이미 생성한 항목 삭제는 반환된 명시적 범위로 수행 |
| last-table 이후 trim | 마지막 표가 아니라 생성 영역 끝을 기준으로 완료. 기존 뒤쪽 Enter·문단은 유지 |
| 2pt compact | 이식하지 않음. 원래 서식을 보존하고 삽입 경계를 교정 |
| section merge | 별도 구역 편집 의미가 있으므로 반복 자동화의 자동 후처리에서 제외. 구역 정의를 포함한 원형은 처음에는 거부 |

이는 구현계획에 포함한 **승인 요청안**이다. 네 API가 이미 구현됐거나 필요 없다고 확정한 결과가 아니다.

## 8. 다음 승인 요청

[구현계획](../plans/task_m100_3587_impl.md)의 전체 순서와 **첫 묶음 A(복제 신원·참조 기반 보정)**를
검토 요청한다. 첫 묶음만으로 #3587을 종료하지 않는다. B/C/D의 상세 계약 확장은 해당 묶음 전에
결과와 함께 재검토하며, 이번 승인을 원격 게시·PR 생성 또는 전 기능 무제한 구현 승인으로 사용하지 않는다.

## 9. 원본 템플릿·재현 시나리오 확인 대기

2026-09-12 메인테이너 지시에 따라 구현 착수 전 실제 사용 자료를 추가로 요청했다.

- [#3587 요청 댓글](https://github.com/edwardkim/rhwp/issues/3587#issuecomment-5642920496):
  사용자 승인 후 `gh issue comment --body-file`로 게시하고 API로 본문을 다시 확인했다.
- 요청 내용: 실제 6×3 원형 템플릿 또는 공개 다운로드 URL, 입력 DOCX/PDF·데이터,
  복제·삽입·쪽나눔·후처리의 호출 순서와 조건, 정상/문제 산출물, 버전·저장·보정 환경.
- 목적: v0.7.13 기반 포크와 현재 코드의 차이를 고려해 메인테이너 측에서 필요 연산과
  실제 재현을 먼저 검토한다. 기여자에게 현재 devel로 직접 이식할 것을 요구하지 않았다.
- 로컬 확보 자료: `samples/rnote/연구노트(분야).hwp` 7개는 **매뉴얼**이다.
  `samples/rnote/labnote-001.hwp`~`labnote-003.hwp`는 추가 확보한 검토 자료이며,
  아직 내부 구조 및 기여자의 실제 입력 템플릿과의 동일성을 확인하지 않았다.
  10개 모두 `/mnt/e/hwpsamples/rnote/` 원본과 복사본의 SHA-256 일치를 확인했다.
- 현재 상태: 구현 미착수. 기여자의 자료·답변과 메인테이너 검토를 반영한 뒤
  구현계획을 재검토한다. 앞선 조사용 fixture를 실제 기여자 템플릿으로 간주하지 않는다.
- 메인테이너가 현재 작업 상태의 로컬 커밋 후 응답 대기를 지시했다.
  조사 문서·진단 probe·확보 자료 10개를 보존하며, 원격 push·PR 생성·제품 구현은 진행하지 않는다.
