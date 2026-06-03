# Task M100-1064 — el-school-001.hwpx 저장 (최종 보고서, 조사 + 분리 종결)

- 이슈: [#1064](https://github.com/edwardkim/rhwp/issues/1064)
- 마일스톤: v1.0.0 (M100)
- 브랜치: `local/task1064`
- 일시: 2026-05-22
- 종결 사유: **본질 식별 후 별도 task 분리 — 작업지시자 결정**

## 1. 작업지시자 보고 증상

1. **한컴 에디터 로딩 중 종료** (파일 손상)
2. **rhwp-studio 첫 표 0번 셀 이미지 비율 미처리**
3. **rhwp-studio 첫 표 0번 셀 도형 출력 안 됨**

작업지시자 추가 정보 (2026-05-22 13:10):
- "hwpx 를 처음 rhwp-studio 로 열었을 때 셀안의 도형이 출력되지 않았습니다"
- "hwp 에서는 도형의 방향도 잘못 배치"

## 2. 조사 결과 (Stage 1)

### 2.1 record-level diff (정답지 vs 저장본)

진단 도구: `examples/dump_table_ctrl_data.rs` 신규.

| Tag | 정답지 | 저장본 | 차이 |
|-----|--------|--------|------|
| **87 (HWPTAG_CTRL_DATA)** | **2** | **0** | 누락 |
| **82 (SHAPE_COMPONENT)** | size 40 ×4 | size 4 ×4 | **36 byte ×4 누락** |
| **66 (PARA_HEADER)** | size 24 (다수) | size 22 (다수) | 2 byte 누락 |
| 76 (TABLE) | size 252 | size 252 | byte_diff @204 |
| 77 (BORDER_FILL) | size 30 | size 30 | byte_diff @3 |
| 72 (LIST_HEADER) | 다수 | 다수 | 1-2 byte_diff |

### 2.2 본질 영역 분리

진단 결과 본 task #1064 의 한컴 로딩 실패에는 **다중 본질** 존재:

| 본질 영역 | 본질 | 결정 |
|----------|------|------|
| (A) 표 CTRL_DATA ParameterSet 누락 | 어댑터 — 표 메타데이터 | Stage 2 시도 후 작업지시자 한컴 시각 판정 — **로딩 실패 여전** → 부분적 정정만 |
| (B) **SHAPE_COMPONENT 36 byte 누락** | **HWPX 파서 — 셀 안 도형 처리 자체 누락** | **별도 task 분리** |
| (C) PARA_HEADER 2 byte 누락 | parser/serializer — instance_id 등 | Task #1058 의 영역과 유사, 별도 검토 |
| (D) hwp 도형 방향 회전 결함 | rhwp-studio HWP 렌더링 | **별도 task 분리** |

### 2.3 Stage 2 시도 결과 (롤백)

`adapt_table_ctrl_data_in_paragraph` 신규 함수로 셀 안 도형 있는 표의 CTRL_DATA
ParameterSet 합성 → repro_stage2.hwp 의 CTRL_DATA 2 records (size 104) 생성 정합.

그러나 작업지시자 한컴 시각 판정 보고: **"한컴 여전히 로딩 실패"**.

→ CTRL_DATA 합성만으로 부족. 진짜 본질이 (B) SHAPE_COMPONENT payload 누락 (HWPX 파서 영역).

**작업지시자 결정: Stage 2 롤백 후 본 task 종결, 본질 (B) (D) 별도 task 분리**.

## 3. 산출물

### 3.1 보존된 산출물

- `samples/hwpx/el-school-001.hwpx` (HWPX 원본)
- `samples/el-school-001.hwp` (정답지)
- `pdf-large/hwpx/el-school-001.pdf` (시각 정답지)
- `mydocs/plans/task_m100_1064.md` (수행 계획서)
- `mydocs/plans/task_m100_1064_impl.md` (구현 계획서)
- `mydocs/working/task_m100_1064_stage1.md` (Stage 1 보고서)
- `examples/dump_table_ctrl_data.rs` (진단 도구 — 후속 task 에서 재사용)
- `examples/repro_1064_save.rs` (reproduce 도구)
- 본 최종 보고서

### 3.2 롤백된 영역

`src/document_core/converters/hwpx_to_hwp.rs` 의 Stage 2 변경 (adapt_table_ctrl_data*)
모두 롤백 — 본 어댑터는 본 task 정정 없음.

### 3.3 본 task 의 조사 가치

진단 도구 + ParameterSet 정밀 분석은 **후속 task** 에서 직접 재사용 가능:
- ParameterSet 구조 (id 0x021B / 자식 id 0x0242 / 11 Integer4) 식별 완료
- 11 parameters 의미 추정 (vertical_offset, horizontal_offset, width, height, row_height, flag, ..., page_width, page_height)
- hwplib `ForParameterSet` 권위 자료 매핑

## 4. 별도 task 분리 — 신규 이슈 등록 예정

### Task A: HWPX 파서 — 셀 안 도형 처리 누락

- 증상: HWPX 의 표 셀 안 도형이 IR 화 안 됨
- 영향: rhwp-studio 자체 렌더링 + HWP 저장 후 한컴 로딩 실패
- 본질: HWPX `<hp:tbl>` 의 셀 안 `<hp:lin>`/`<hp:rect>` 등 처리 누락
- 산출물: 본 task 의 진단 도구 재사용 (SHAPE_COMPONENT size 40 vs 4 비교)

### Task B: rhwp-studio HWP 렌더링 — 첫 도형 180도 회전 결함

- 증상: HWP 의 첫 도형이 180도 회전된 상태로 출력
- 영향: rhwp-studio 자체 렌더링 (HWP 직접 로드)
- 본 task 와 별개 영역 (도형 렌더링 변환 행렬)
- fixture: `samples/hwpx/shape-001.hwpx` + `samples/shape-001.hwp` + `pdf-large/hwpx/shape-001.pdf`

## 5. 메모리 룰 정합

- ✅ `feedback_visual_judgment_authority` — 작업지시자 한컴 시각 판정 게이트
- ✅ `feedback_diagnosis_layer_attribution` — record-level diff 로 본질 영역 분리
- ✅ `feedback_self_verification_not_hancom` — Stage 2 적용 후 rhwp 자기 정합 정상 / 한컴 거부 여전
- ✅ `feedback_process_must_follow` — 계획 → 승인 → 단계별 실행 → 본질 식별 후 작업지시자 결정
- ✅ `feedback_search_troubleshootings_first` — 사전 검색
- ✅ `feedback_check_open_prs_first` — open PR 확인
- ✅ `feedback_assign_issue_before_work` — 이슈 등록 시 assignee 본인
- ✅ `project_hwpx_to_hwp_adapter_limit` — 단순 어댑터 한계 재입증 (CTRL_DATA 정정만으로 부족)

## 6. 학습

1. **다중 본질 task 의 분리 결정**: 진단 결과 단일 본질이 아니면 우선순위에 따라 분리 — 작업지시자 결정 후 진행
2. **Stage 2 즉시 검증의 가치**: 작업지시자 시각 판정 후 부족 확인되어 단일 본질이 아님이 명확화 됨. 시각 판정 게이트가 task scope 결정의 결정적 근거.
3. **부분 정정의 위험**: CTRL_DATA 합성 단독은 부분적 정정 — 한컴 호환 미달성 시 회귀 위험 존재. 롤백 결정의 합리성.
