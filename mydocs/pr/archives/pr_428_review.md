# PR #428 검토 — 그룹 내 그림(Picture) 직렬화 구현

## PR 정보

| 항목 | 값 |
|------|-----|
| PR 번호 | [#428](https://github.com/edwardkim/rhwp/pull/428) |
| 작성자 | [@oksure](https://github.com/oksure) (Hyunwoo Park) — 신뢰 컨트리뷰터, PR #395/#396/#427 머지 통과 |
| 이슈 | (직접 연결 이슈 없음 — 자체 발견 결함) |
| base / head | `devel` ← `oksure:contrib/fix-group-picture-serialize` |
| 변경 규모 | +14 / -3, **1 file** (`src/serializer/control.rs`), 1 commit |
| mergeable | `MERGEABLE` (BEHIND) |
| 검토 일자 | 2026-04-29 |
| 처리 결정 | **옵션 A — 작성자 한컴 호환 검증 자료 보강 요청** (작업지시자 직접 결정) |

## 본질

`serialize_group_child::ShapeObject::Picture` 분기가 빈 TODO 였음 → 그룹 내 그림이 저장 시 유실되는 결함. 본 PR 이 Chart/OLE 자식 패턴을 따라 채움.

### 변경

| 항목 | Before | After |
|------|--------|-------|
| `serialize_shape_control::Picture` ctrl_id | `0x24706963` 매직넘버 | `tags::SHAPE_PICTURE_ID` 상수 |
| `serialize_group_child::Picture` 분기 | `// TODO: 그룹 내 그림 직렬화` (빈 분기) | `SHAPE_COMPONENT` + `SHAPE_COMPONENT_PICTURE` 레코드 추가 (Chart/OLE 패턴 동일) |

## 이슈 #429 와의 관계 점검

작업지시자가 PR #428 검토 시 "이슈 #429 (표 안 이미지 미조판) 와 궤를 같이 할 수 있다" 코멘트.

| 측면 | 이슈 #429 | PR #428 |
|------|----------|---------|
| 경로 | **렌더링** (SVG 출력) | **저장** (HWP 직렬화) |
| 위치 | `aift.hwp` 41페이지 표 셀 안 이미지 | 그룹 도형 내 중첩 Picture |
| 코드 | `layout.rs::layout_table_cell_*` 또는 `paragraph_layout.rs` | `serializer/control.rs::serialize_group_child` |
| 결함 | 표시 단계 누락 | 저장 시 그림 데이터 유실 |

→ **본질이 다른 결함** (서로 다른 경로). 그림 (Picture) 처리 누락 공통 문맥은 있으나 정정 위치/방법 다름.

## 트러블슈팅 사전 검토 (메모리 `feedback_search_troubleshootings_first`)

직접 관련 트러블슈팅:

| 문서 | 본 PR 과의 관계 |
|------|---------------|
| `picture_save_hancom_compatibility.md` | **권위 문서** — 그림 저장 호환성 5가지 결함 정정 이력. 본 PR 이 호출하는 함수들 (`serialize_picture_data`, `serialize_shape_component`, `serialize_common_obj_attr`) 에 정정 적용 확인됨 |
| `task_96_group_child_matrix_composition_order.md` | 그룹 자식 행렬 합성 순서 정정 (렌더링 측). 본 PR 은 행렬 직렬화 자체를 건드리지 않음 — 충돌 가능성 낮음 |
| `task_96_group_child_textbox_vertical_align.md` | 그룹 자식 텍스트박스 vertical-align. 본 PR 무관 |
| `cell_split_save_corruption.md` | 셀 분할 저장 손상. 본 PR 무관 |

## 호출 함수 호환성 정정 적용 확인

본 PR 의 코드:
```rust
data: serialize_shape_component(tags::SHAPE_PICTURE_ID, &pic.shape_attr, false),
data: serialize_picture_data(pic),
```

| 함수 | 위치 | 트러블슈팅 정정 적용 |
|------|------|---------------------|
| `serialize_picture_data` | `control.rs:728` | ✅ `border_x/y`, `crop`, `raw_picture_extra` (line 734-775) |
| `serialize_shape_component` | `control.rs:1391` | ✅ `local_file_version`, `raw_rendering` (line 1442, 1464) |
| `serialize_common_obj_attr` | `control.rs:1378` | ✅ `prevent_page_break` (line 1378) |

→ 호환성 정정 적용된 함수들을 호출하므로 코드 자체는 안전 정황.

## 검증 게이트 (cherry-pick dry-run 후)

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | ✅ 1062 passed (회귀 없음) |
| `cargo test --test svg_snapshot` | ✅ 6/6 passed |
| `cargo test --test issue_418` | ✅ 1/1 (Task #418 보존) |
| `cargo clippy --lib -- -D warnings` | ✅ warning 0건 |

## 검증 한계 — 작성자 보강 요청 사유

### 1. 그룹 내 Picture 라운드트립 단위 테스트 부재

본 PR 은 **기존에 없던 코드를 채우는** 정정이라 회귀 테스트가 추가되지 않음. 향후 회귀 가능성 점검이 어려움.

### 2. 한컴 호환 검증 자료 부재

PR 본문에 한컴 편집기 열람 증빙 / PDF 비교 자료 없음. 메모리 `feedback_self_verification_not_hancom` 원칙 — 자기 라운드트립은 항상 참인 무의미 테스트가 될 수 있음.

작업지시자 결정 메모: "처음에는 AI 가 자동으로 만들어주는 test case 와 성공을 믿다가 (자동 회귀테스트) 실제 시각 검증으로 돌아선 이유. 자동 테스트 케이스가 필요없다는 의미는 아니고, **자기 라운드 트립 코드 즉, 무조건 참인 케이스가 아닌지 리뷰** 해야 한다는 의미."

## 처리 방향 — 옵션 A (작성자 보강 요청)

PR #400 (HWPX 수식 직렬화) 와 같은 패턴.

### 보강 요청 항목

1. **한컴 origin 기반 회귀 테스트 추가**:
   - 입력: 한컴 편집기가 만든 hwp 의 parse 결과 IR
   - 검증: rhwp serialize → 재 parse → 그룹 내 Picture 핵심 필드 비교
   - 자기검증 패턴 (`Document::default + 수동 push`) 회피
   - PR #400 의 `equation_roundtrip_from_hancom_origin_hwp_sample` 참고

2. **한컴 편집기 호환 검증**:
   - rhwp serialize 결과 hwp 를 한컴 한글 2010/2022 에서 정상 열람
   - 파일 손상 메시지 없음 / 그룹 내 그림 정상 표시 / 비정상 큰 크기 (425.20%) 아님 확인
   - 한컴 편집기에서 PDF 내보내기 + PDF 첨부

### 후보 샘플

- `samples/hwpspec.hwp` (1개 group 컨트롤)
- `samples/hwpctl_Action_Table__v1.1.hwp` (1개 group 컨트롤)
- 작성자가 직접 한컴 편집기로 "그림 + 도형 → 그룹화" 한 hwp 를 samples/ 에 추가 (PR #397 / #400 패턴)

## dry-run cherry-pick 결과

`local/pr428` 브랜치 (`local/devel` 분기) 에서 단일 commit cherry-pick — 작성자 attribution 보존:

| commit (cherry-pick) | 작성자 | 내용 |
|---------------------|--------|------|
| `7bbd1dc` (← `6d93573`) | @oksure | fix: 그룹 내 그림(Picture) 직렬화 구현 + ctrl_id 매직넘버 제거 |

cherry-pick 결과: 충돌 없이 자동 적용. 코드 자체는 머지 가능 상태.

## 다음 단계

1. ✅ 작성자에게 [한컴 호환 검증 자료 보강 요청 댓글](https://github.com/edwardkim/rhwp/pull/428#issuecomment-4340354495) 작성
2. PR #428 OPEN 유지 (작성자 재제출 대기)
3. 작성자 보강 commit 추가 후 재검토 → cherry-pick 머지 진행
4. `local/pr428` 브랜치는 보강 commit 흡수 시 재사용

## 참고

- PR: [#428](https://github.com/edwardkim/rhwp/pull/428)
- 이슈 #429 (표 안 이미지 미조판) — 본 PR 과 다른 경로의 결함
- 트러블슈팅: `picture_save_hancom_compatibility.md`, `task_96_group_child_matrix_composition_order.md`
- 같은 작성자 머지 PR: [#395](https://github.com/edwardkim/rhwp/pull/395), [#396](https://github.com/edwardkim/rhwp/pull/396), [#427](https://github.com/edwardkim/rhwp/pull/427)
- 보강 요청 패턴 origin: PR #400 (HWPX 수식 직렬화) — `equation_roundtrip_from_hancom_origin_hwp_sample` 참고
