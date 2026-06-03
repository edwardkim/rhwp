# Task M100-852 최종 보고서 — HWPX → HWP Form 컨트롤 한컴 호환

- 이슈: [#852](https://github.com/edwardkim/rhwp/issues/852)
- 브랜치: `local/task852` (base: `local/devel = 7ec2e25f`)
- 기간: 2026-05-20
- 마일스톤: v1.0.0 (M100)
- assignee: @edwardkim
- 단계별 보고서:
  - `mydocs/working/task_m100_852_stage1.md` (Stage 1 진단)
  - `mydocs/working/task_m100_852_stage24.md` (Stage 2.4 Form 직렬화)
  - `mydocs/working/task_m100_852_stage25.md` (Stage 2.5 JavaScript)

## 1. 결과 요약

HWPX → HWP 변환 시 form-002.hwpx (10페이지 form 보유) 가 한컴 에디터에서 파일 손상 판정되던 issue 를 정공법으로 해소.

### 1.1 한컴 에디터 판정 (작업지시자, `feedback_visual_judgment_authority`)

| fixture | 변환 전 (devel) | Stage 2.4 후 | Stage 2.5 후 |
|---------|-----------------|--------------|--------------|
| form-01.hwp | 손상 | 성공 (JS 없음) | **성공 (JS 포함)** |
| form-02.hwp | 손상 | 성공 | **성공** |
| form-002.hwp (원 이슈 fixture) | **손상** | 성공 | **성공** |
| tbox-v-flow-01.hwp | 성공 | 성공 | 성공 |
| hy-001-rt.hwp (HWP roundtrip) | 성공 | 성공 | 성공 |

### 1.2 정답지 byte-level 정합

| 레코드 | 정답지 (form-01) | 변환 | 상태 |
|--------|------------------|------|------|
| 5 Form CTRL_HEADER+FORM_OBJECT pairs | 5 | 5 | 5/5 byte-perfect |
| %clk CTRL_HEADER (151B) | 1 | 1 | byte-perfect |
| 0x57 CTRL_DATA "myMsg01" (26B) | 1 | 1 | byte-perfect |
| Scripts/DefaultJScript (1580B uncompressed) | 1 | 1 | byte-identical |

## 2. 작업 분해

### Stage 1 — 진단 (`mydocs/working/task_m100_852_stage1.md`)

이슈 본문 가설 (Form 컨트롤 직렬화) 정정 — Stage 1 진단 결과 = **HWPX parser 의 contract 스트림 일반 누락 + Form 직렬화 갭**. HWP→HWP roundtrip 은 정상이나 HWPX→HWP 변환만 5+ 스트림 누락 (`troubleshootings/hwpx2hwp-rule.md` 5.A 위반).

### Stage 2.1+2.2 — Contract 스트림 (커밋 `2668d22a` + `95fdd8f8`)

HWPX 컨테이너 → HWP OLE 9 contract 스트림 변환 + 정적 fallback:
- Preview/PrvText.txt (UTF-8) → /PrvText (UTF-16 LE)
- Preview/PrvImage.png → /PrvImage (passthrough)
- Scripts/sourceScripts → /Scripts/DefaultJScript (zlib, Stage 2.1)
- HwpSummary / DocOptions/_LinkDoc / Scripts/JScriptVersion — `saved/blank2010.hwp` 정적 fallback

### Stage 2.4 — Form 컨트롤 직렬화 (커밋 `a93c0cf7`)

정답지 byte-level reverse engineering 기반 정공법 직렬화:
- `src/serializer/control.rs:115` 의 "미구현 컨트롤" 분기에서 `Control::Form` 분리 → 신규 `serialize_form_control()` + 5 헬퍼
- CTRL_HEADER "form" (46B): ctrl_id + attr 0x002a6211 + width/height + order + instance_id 0x7dcd59d6+order
- HWPTAG_FORM_OBJECT (가변): type_id × 2 + wchar_count + UTF-16 속성 문자열 (CommonSet + CharShapeSet + 타입별 Set)
- HWPX `<hp:formCharPr>` + form attributes 11 속성 보존 (autoSz / borderTypeIDRef / tabOrder 등)
- thread_local form_order 카운터 (section 단위 reset)

### Stage 2.5 — JavaScript 일관 직렬화 (이번 커밋)

작업지시자 관찰 ("JS 미포함") 해소:
- Scripts/DefaultJScript = `headerScripts` (var 선언) + `sourceScripts` (함수) **결합** + **raw deflate** + length-prefix
- BodyText `%clk` (ClickHere field) = `Control::Field` 직렬화 시 ctrl_id/properties/extra_properties/instance_id 정확 채움
- BodyText 0x57 (CTRL_DATA "myMsg01") = ClickHere field 의 CTRL_DATA 자식 레코드 자동 합성
- HWPX parser 가 `<hp:fieldBegin>` 의 type/name/id/editable attribute 완전 수집 → ctrl_id 매핑

### Stage 3 — 회귀 가드 + 최종 보고서 (이번 커밋)

`tests/issue_852_hwpx_to_hwp_contract_streams.rs` — 5 회귀 가드:
1. `form_01_keeps_nine_cfb_streams` — 9 contract 스트림 보유
2. `form_01_body_text_has_five_form_records` — 5 Form CTRL_HEADER+FORM_OBJECT pairs
3. `form_01_scripts_default_jscript_matches_golden` — Scripts/DefaultJScript byte-identical with golden
4. `form_01_body_text_has_click_here_with_ctrl_data` — %clk + 0x57 pair
5. `form_02_keeps_nine_streams_and_five_forms` — form-02 대칭 검증

## 3. CI 패턴 검증

| 항목 | 결과 |
|------|------|
| `cargo test --release --lib` | **1309 passed, 0 failed** |
| `cargo test --release --tests` (전체 통합) | **모두 passed, 0 failed** (issue_852 5/5 포함) |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --all-targets --release` | 본 PR 변경 파일 0 warnings |

## 4. 솔직한 한계 — 정답지 hardcode 영역

작업지시자 검토 후 명시한 정답지 reverse engineering 으로 hardcode 한 값들:

### 4.1 CTRL_HEADER "form" hardcode

- `attr = 0x002a6211` — 정답지 5 form 모두 동일, HWPX 에서 도출 불가
- `instance_id = 0x7dcd59d6 + order` — 한컴 내부 timestamp/PID 기반 추정
- zero padding 14 bytes — HWP5 spec reserved 추정

### 4.2 HWPTAG_FORM_OBJECT magic

- bytes 4..8 = type_id 중복 — spec 근거 없음, 정답지 관찰

### 4.3 ComboBox / Edit 기본값

- Edit MaxLength = 2147483647, BorderType 5 등 — 정답지 관찰값. HWPX attribute 우선이므로 fallback 만

### 4.4 %clk + 0x57 magic

- %clk byte 8 = 0x09 (extra_properties) — spec 근거 없음
- 0x57 bytes 0..2 = 0x021b, 6..8 = 0x4000 — magic marker
- ClickHere field instance_id = `0x7dcd59d6 + form_order_counter` — 정답지 패턴

### 4.5 Scripts/DefaultJScript trailing

- 12 bytes (8 zero + 4 0xFFFFFFFF) — EOS marker 추정

**위험 평가**: 본 hardcode 들은 정답지 5 form 의 일관된 값에 기반. 다른 fixture (form-002, tbox, hy-001) 5/5 한컴 성공 + 본 회귀 가드 통과로 1차 검증. 다양한 엣지 케이스는 향후 수집 후 대응.

## 5. 메모리 룰 정합

- ✅ `feedback_self_verification_not_hancom` — rhwp byte-perfect 후 작업지시자 한컴 게이트 (Stage 2.4 + 2.5 모두 5/5 통과)
- ✅ `feedback_visual_judgment_authority` — 작업지시자 한컴 에디터 시각 판정
- ✅ `feedback_diagnosis_layer_attribution` — Stage 1 진단에서 control.rs:115 ctrl_id=0 분기 정확 식별
- ✅ `feedback_hancom_compat_specific_over_general` — Form 5 타입 + ClickHere 명시 분기 (일반화 회피)
- ✅ `feedback_push_full_test_required` — cargo test --lib + --tests + clippy + fmt 전체 CI 패턴 통과
- ✅ `feedback_assign_issue_before_work` — assignee 지정 완료
- ✅ `reference_authoritative_hancom` — samples/form-01.hwp / form-02.hwp 정답지 baseline
- ✅ `feedback_search_troubleshootings_first` — Stage 1 사전 검색 (`hwpx2hwp-rule.md` 5.A 확인)
- ✅ `feedback_fix_scope_check_two_paths` — HWP roundtrip (정상) vs HWPX 변환 (누락) 두 경로 식별 + HWPX 만 정정
- ✅ `feedback_commit_reports_in_branch` — 보고서 + orders 갱신 타스크 브랜치 커밋
- ✅ `project_output_folder_structure` — `output/poc/task852/stage{N}/` 산출물

## 6. 후속 권고

1. **HWP5 spec 비트필드 명시화** — `0x002a6211` (CTRL_HEADER attr) / `0x021b` (0x57 magic) / `0x4000` (0x57 flag) 등의 비트 의미를 spec/hwp2hwpx 참조 라이브러리에서 확인
2. **byte-perfect 회귀 가드 확장** — 본 task 는 record 존재 + Scripts byte-identical 만 검증. 정답지와 byte-level diff 단언 추가 가능
3. **다른 FieldType byte-level 검증** — Date / MailMerge / CrossRef 등 ClickHere 외 필드의 정답지 정합
4. **HwpSummaryInformation 정밀 패치** — 현재 blank2010.hwp 정적 fallback, HWPX content.hpf opf:metadata 의 title/creator/date 반영

## 7. 변경 파일 요약

### Stage 2.4 (커밋 a93c0cf7)
- `src/serializer/control.rs` — Form 분기 + `serialize_form_control` + 5 헬퍼 + thread_local 카운터
- `src/serializer/body_text.rs` — section 진입 시 카운터 reset
- `src/parser/hwpx/section.rs` — HWPX Form 11 속성 보존
- `src/parser/hwpx/blank2010_assets/{prvimage,prvtext,scripts_default_jscript}.bin` — Stage 2.2 누락 자산
- `mydocs/plans/task_m100_852_stage24.md` + `task_m100_852_stage25.md`
- `mydocs/working/task_m100_852_stage24.md`

### Stage 2.5 + 3 (현 커밋)
- `src/parser/hwpx/contract_streams.rs` — Scripts/DefaultJScript = header+source+raw_deflate
- `src/parser/hwpx/section.rs` — Field type/name/id/editable attribute 완전 수집 + field_type → ctrl_id 매핑
- `src/serializer/control.rs` — Field ClickHere instance_id 정답지 패턴 + CTRL_DATA 자동 합성
- `tests/issue_852_hwpx_to_hwp_contract_streams.rs` — 5 회귀 가드
- `mydocs/working/task_m100_852_stage25.md`
- `mydocs/report/task_m100_852_report.md` (본 보고서)
