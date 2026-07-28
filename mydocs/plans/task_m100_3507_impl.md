# 구현계획서 — #3507 SectionDef CTRL_DATA 중복 직렬화

- 이슈: [#3507](https://github.com/edwardkim/rhwp/issues/3507)
- 수행계획서: [`task_m100_3507.md`](task_m100_3507.md)
- 작업 브랜치: `codex/issue-3507-sectiondef-ctrl-data`
- 착수 기준: `upstream/devel` `ef72fee5138e3c491b6d8f38e459dc94670284f6`
- 최종 기준: `upstream/devel` `f32d964dfc5bdb1a28f41f9277e4aa2d7e4387ff`
- 승인 근거: 2026-07-28 작업지시자가 수행계획서에 포함된 구현 범위를 확인한 뒤 수정 작업 진행 승인

## 문서 분리 기록

착수 당시 수행계획서에 목표·원인·구현 범위·검증 방법을 함께 기록하고 승인을 받았다. 이 문서는
PR 준비 단계에서 그중 구체적인 변경 방법을 구현계획서 역할에 맞게 분리한 것이다. 별도의 승인
사건을 소급해 만들거나 작업 범위를 새로 추가하지 않는다.

## 변경 계약

1. SectionDef의 canonical `CTRL_DATA`는 `Paragraph.ctrl_data_records` 한 곳에서 소유한다.
2. canonical 대상은 SectionDef `CTRL_HEADER`의 직접 자식이면서 첫 중첩 `CTRL_HEADER` 전에
   나타나는 첫 레코드로 제한한다.
3. 추가 직접 자식, 중첩 control의 레코드, 중첩 header 뒤의 raw 레코드는 원래 순서와 payload를
   보존한다.
4. legacy 또는 외부 생성 IR이 canonical payload를 두 필드에 중복 보유해도 serializer는
   동일 level·payload의 exact duplicate만 한 번 출력한다.
5. 편집과 무관한 payload는 바이트 단위로 유지하고, 수정한 표 셀 값은 재독으로 확인한다.

## 파일별 변경 계획

| 파일 | 계획 |
|---|---|
| `src/model/document.rs` | `SectionDef.extra_child_records`와 문단 control 슬롯의 소유권 계약을 문서화한다. |
| `src/parser/body_text.rs` | canonical SectionDef `CTRL_DATA`의 위치를 판별하고 `extra_child_records`의 중복 소유를 제거한다. |
| `src/parser/body_text/tests.rs` | 직접 자식 단일 소유, 추가·중첩·경계 뒤 레코드 보존을 고정한다. |
| `src/serializer/control.rs` | 문단 슬롯 payload를 SectionDef serializer에 전달하고 경계 안 exact duplicate만 건너뛴다. |
| `src/serializer/control/tests.rs` | legacy 이중 소유 IR과 중첩 header 뒤 동일 payload 보존을 검증한다. |
| `tests/issue_3507_sectiondef_ctrl_data.rs` | `복학원서.hwp`의 실제 `edit set-cell` 저장 경로와 payload 불변성을 검증한다. |
| `tests/fixtures/ir_field_sweep_baseline.tsv` | 중복 소유 제거로 해소된 발산만 전체 sweep 실측 후 갱신한다. |
| `mydocs/tech/hwp_ctrl_data.md` | SectionDef canonical owner와 raw 보존 경계를 기록한다. |
| `mydocs/tech/hwp_save_guide.md` | SectionDef의 선택적 `CTRL_DATA` 직렬화 순서를 기록한다. |

## 단계와 게이트

### Stage 1 — 단일 소유와 focused 검증

- parser canonical ownership과 serializer 방어를 구현한다.
- 수정 전 red, 수정 후 green을 회귀 테스트로 확인한다.
- 실제 `set-cell` 저장본의 셀 값, 레코드 수, payload 길이·hash를 확인한다.
- macOS 한컴 Viewer에서 파일 손상 경고와 편집 값 표시를 확인한다.

완료 기록:
[`task_m100_3507_stage1.md`](../working/task_m100_3507_stage1.md)

### Stage 2 — 경계 보강과 전체 회귀

- 첫 중첩 `CTRL_HEADER`를 canonical 탐색과 중복 제거의 종료 경계로 고정한다.
- 전체 IR field sweep의 baseline 감소와 상한 슬롯 이동을 변경 전 기준과 대조한다.
- release-test, fmt, diff check, clippy를 실행한다.
- macOS Viewer 결과와 작업지시자의 Windows 한글 2024 판정을 최종 인수 근거로 묶는다.

완료 기록:
[`task_m100_3507_stage2.md`](../working/task_m100_3507_stage2.md)

## 중단 조건

- 서로 다른 payload를 가진 복수 직접 자식 `CTRL_DATA`가 손실되거나 합쳐진다.
- 중첩 `CTRL_HEADER` 뒤 raw 레코드의 순서 또는 payload가 바뀐다.
- 전체 IR sweep에서 중복 제거와 무관한 신규 발산이 생긴다.
- rhwp 재독은 성공하지만 한컴 Viewer 또는 한글 2024가 저장본을 거부한다.

위 조건 중 하나라도 발생하면 baseline을 갱신하거나 호환 판정을 완료하지 않고 원인 분리 단계로
돌아간다.
