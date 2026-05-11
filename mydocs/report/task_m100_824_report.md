# Task #824 최종 결과 보고서

**제목**: HWP3 임베디드 그림이 외부 파일 참조로 잘못 표시됨 (그림 속성 dialog 파일 이름 오표시)
**마일스톤**: v1.0.0 (M100)
**브랜치**: `local/task824` (base: `local/devel`)
**이슈**: [#824](https://github.com/edwardkim/rhwp/issues/824)
**선행**: PR #753 (Task #741) — 본 결함의 원인 commit 포함

## 결론

**결함 정정 완료** — `pic_type` 별 분기 추가 (`pic_type == 0` 만 `external_path` 설정).
SVG 회귀 zero, 자동 테스트 PASS, 작업지시자 시각 판정 통과.

## 결함 본질

[src/parser/hwp3/mod.rs:935-952](../../src/parser/hwp3/mod.rs#L935-L952) — Task #741 에서 `external_path` 셋팅 시 HWP3 그림 종류 (offset 74) 구분 누락. 외부 파일/OLE/Embedded Image 3종 모두에 `external_path` 설정되어, 임베디드 그림 (`pic_type == 2`) 이 외부 파일로 오표시.

| 항목 | rhwp-studio (수정 전) | 한컴오피스 2022 |
|---|---|---|
| 파일 이름 | `E$$00000.jpg` | (빈 값) |
| 문서에 포함 | ☐ (해제) | ☑ (체크) |

## 정정 내용

### 단일 핵심 변경

[src/parser/hwp3/mod.rs:944-955](../../src/parser/hwp3/mod.rs#L944-L955) — `if pic_type == 0` 가드 1줄 추가:

```rust
if !pic_name.is_empty() {
    // [Task #824] pic_type == 0 (외부 파일) 만 external_path 설정.
    // pic_type == 1 (OLE) / 2 (Embedded) 는 pic_name 이 내부 참조명
    // (예: "E$$00000.jpg") 이므로 external_path 설정 시 그림 속성 dialog 가
    // 외부 파일로 오표시됨 (한컴오피스 2022 정합).
    if pic_type == 0 {
        pic.image_attr.external_path = Some(pic_name.clone());
    }
    let next_id = (pic_name_to_id.len() + 1) as u16;
    let id = *pic_name_to_id.entry(pic_name).or_insert(next_id);
    pic.image_attr.bin_data_id = id;
}
```

`bin_data_id` 매핑은 type 무관 유지 (pic_name HashMap 기반 ID 할당).

### 회귀 테스트

`tests/issue_824.rs` — 2 개 테스트:
- `issue_824_embedded_picture_no_external_path` — sample11 의 임베디드 그림 → `external_path == None`
- `issue_824_external_picture_keeps_external_path` — sample10 의 외부 file path 그림 → `external_path.is_some()` (회귀 가드)

## 검증

| 항목 | 결과 |
|---|---|
| `cargo test issue_824` | 2 PASS |
| `cargo test --release` 전체 | 1353 passed, 0 failed |
| `cargo clippy --release -- -D warnings` | clean |
| SVG 회귀 (5 sample × 1031 page) | bit-identical, diff 0 |
| 작업지시자 시각 판정 (rhwp-studio) | ✅ 통과 |

### 시각 판정 결과 (작업지시자)

- ✅ `samples/hwp3-sample11.hwp` 임베디드 그림 → 그림 속성 dialog "파일 이름 빈 값 + 문서에 포함 체크" 한컴오피스 2022 정합
- ✅ `samples/hwp3-sample10.hwp` 외부 file path 그림 → 정상 동작 (회귀 가드)
- ✅ Canvas 렌더링 — sample11 그림이 placeholder 가 아닌 실제 image 로 표시

## 단계 진행 요약

| 단계 | 내용 | commit |
|---|---|---|
| 1 (RED) | 회귀 테스트 + sample11 fixture + FAIL 확인 | `64af532` |
| 2 (GREEN) | `pic_type == 0` 만 external_path 설정 | `fc3aa5f` |
| 3 (회귀) | cargo test 1353/1353 + 5 sample 1031 page bit-identical | `dbba1b7` |
| 4 (시각 + 보고서) | 작업지시자 시각 판정 통과 + 본 보고서 + orders 갱신 + closes #824 | (본 commit) |

## 산출물

| 종류 | 경로 |
|---|---|
| 회귀 테스트 | `tests/issue_824.rs` |
| 소스 수정 | `src/parser/hwp3/mod.rs` |
| Fixture | `samples/hwp3-sample11.hwp` (+ -hwp5 + -hwpx 변환본), `pdf/hwp3-sample11-hwpx-2022.pdf` |
| 수행계획서 | `mydocs/plans/task_m100_824.md` |
| 구현계획서 | `mydocs/plans/task_m100_824_impl.md` |
| 단계별 보고서 | `mydocs/working/task_m100_824_stage{1,2,3}.md` |
| 최종 보고서 | 본 문서 |

## 후속 사항

### 추가 발견 결함 (별도 이슈로 분리 예정)
시각 검증 중 작업지시자 발견 — **머리말 영역의 그림에서 우클릭 → "개체 속성" 클릭 시 dialog 가 뜨지 않음**. 한컴오피스는 머리말 안으로 들어가서 그림 클릭하면 정상 동작.

본 결함은 Task #824 (HWP3 parser) 와 본질이 다른 UI 영역 (rhwp-studio context-menu / picture-props-dialog 분기) 결함 — **별도 이슈로 신규 등록**.

### OLE 그림 (pic_type == 1) 회귀 미검증
보유 sample 중 OLE 그림 케이스 부재 — 본 task 범위에서 별도 회귀 검증 불가. 본 수정은 안전한 방향 (`external_path = None`) 이므로 OLE sample 발견 시 별도 task 처리.

## 메모리 룰 정합

- `feedback_visual_judgment_authority` ✅ — 작업지시자 시각 판정 게이트 통과 후 최종 commit
- `feedback_pr_supersede_chain` (b) 패턴 — Task #741 (PR #753) 후속 결함 영역 영역 별 task
- `feedback_process_must_follow` ✅ — 수행계획서 → 구현계획서 → 4단계 RED/GREEN/회귀/시각 절차 준수
