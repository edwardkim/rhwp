# #3507 SectionDef CTRL_DATA 중복 직렬화 수정 최종 보고서

- 이슈: [#3507](https://github.com/edwardkim/rhwp/issues/3507)
- 브랜치: `codex/issue-3507-sectiondef-ctrl-data`
- 착수 기준: `upstream/devel` `ef72fee5138e3c491b6d8f38e459dc94670284f6`
- 최종 기준: `upstream/devel` `f32d964dfc5bdb1a28f41f9277e4aa2d7e4387ff`
- 상태: 로컬 구현·전체 검증·외부 호환성 확인 완료, `devel` 통합 대기

## 1. 결과

rhwp 브라우저 확장에서 `samples/복학원서.hwp`의 첫 표 `대학` 값을 수정해 저장한 파일이
macOS 한컴오피스 한글 Viewer와 Windows 한글 2024에서 모두 정상 개방된다. 저장본에는 수정한
`강남대학교` 값이 유지되고 SectionDef 직접 자식 `CTRL_DATA`는 원본과 같은 280바이트 payload로
한 번만 기록된다.

## 2. 원인과 수정

파서는 SectionDef의 canonical `CTRL_DATA`를 `Paragraph.ctrl_data_records`와
`SectionDef.extra_child_records`에 동시에 보존했다. serializer는 두 필드를 각각 복원해 같은
payload를 두 번 출력했다. rhwp는 이 파일을 다시 읽을 수 있었지만 한컴 구현은 control child
record 계약 위반으로 파일 손상을 판정했다.

수정 후 계약은 다음과 같다.

1. 첫 중첩 `CTRL_HEADER` 전에 나타나는 첫 SectionDef 직접 자식 `CTRL_DATA`는
   `Paragraph.ctrl_data_records`만 소유한다.
2. 추가 직접 자식, 중첩 control의 레코드, 중첩 header 뒤의 직접 자식은
   `SectionDef.extra_child_records`에 원본 그대로 보존한다.
3. legacy 또는 외부 생성 IR이 두 필드에 동일 레코드를 갖고 있어도 serializer는 경계 안에서
   level·payload가 모두 같은 exact duplicate만 제거한다.

## 3. 계획 대비 결과

| 계획 | 결과 | 차이 |
|---|---|---|
| parser canonical ownership | 완료 | Stage 2에서 첫 중첩 `CTRL_HEADER`를 탐색 종료 경계로 명시했다. |
| serializer exact duplicate 방어 | 완료 | 같은 경계 뒤 raw 레코드는 payload가 같아도 제거하지 않도록 보강했다. |
| parser·serializer 회귀 테스트 | 완료 | 단일 소유, 추가·중첩·경계 뒤 레코드 보존을 고정했다. |
| 실제 `edit set-cell` 통합 테스트 | 완료 | 셀 재독과 SectionDef payload 개수·길이·동일성을 한 테스트에서 검증한다. |
| IR baseline 개선분 확인 | 완료 | 전체 sweep에서 70개 길이 key가 제거되고 총 발산은 56건 감소했다. |
| macOS·Windows 외부 판정 | 완료 | 두 환경 모두 파일 손상 경고 없이 정상 개방됐다. |

전체 sweep에서 새로 보인 7개 key는 회귀가 아니라 문서당 진단 상한
`MAX_DIVERGENCES=2000`의 슬롯 이동이었다. 변경 전 `upstream/devel` 대조 실행으로 기존
baseline과 byte-for-byte 동일함을 확인한 뒤 개선분만 갱신했다.

## 4. 단계와 커밋

| 단계 | 커밋 | 산출물 |
|---|---|---|
| 승인된 계획 기록 | `e72ded63c` | 수행계획서, 오늘할일 착수 기록 |
| Stage 1 | `2abd6010d` | 단일 소유·serializer 방어, focused 테스트, macOS Viewer 확인 |
| Stage 2 | `0674965d9` | 중첩 header 경계 보강, 전체 IR sweep·회귀 검증 |
| 최종 정리 | `docs: finalize #3507 compatibility validation` | 구현계획 분리 기록, Windows 판정, 최종 보고서 |

착수 당시 하나의 수행계획서에 구체적인 구현 범위까지 포함해 작업지시자 승인을 받았다.
PR 준비 과정에서 문서 역할을 명확히 하기 위해 그 내용을 별도 구현계획서로 분리했으며,
존재하지 않았던 별도 승인 사건을 소급 기록하지 않았다.

## 5. 검증

### focused·실물 검증

| 검증 | 결과 |
|---|---|
| SectionDef focused parser·serializer 테스트 | 6 passed |
| parser body text 테스트 | 23 passed |
| serializer control 테스트 | 24 passed |
| `tests/issue_3507_sectiondef_ctrl_data.rs` | 1 passed |
| `edit_set_cell_contract` | 5 passed |
| HWP5 관련 focused 통합 테스트 | 실패 0 |
| `hwp5-roundtrip` | IR diff 0, round2 diff 0 |
| `hwp5-ctrl-data-trace` | 원본·저장본 모두 1개 / 280바이트 / hash 동일 |

### 전체 게이트

| 검증 | 결과 |
|---|---|
| 전체 IR field sweep | 2 passed, 803개 샘플, 회귀 0 |
| `cargo test --profile release-test --tests` | 전체 test binary 실패 0, lib 2,988 passed / 7 ignored |
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass |
| `git diff --check` | pass |

renderer·layout·WASM API 변경이 없으므로 별도 WASM build와 신규 시각 sweep은 수행 범위가 아니다.

전체 게이트는 `upstream/devel` `9c69bc3d3` 기준에서 다시 실행했다. 이후 반영된
`9c69bc3d3..f32d964df`는 CI workflow와 문서만 변경하며 Rust 소스·테스트·샘플 트리는
동일하다. 해당 기준 이동 후 #3507 focused 테스트와 diff 검사를 다시 확인한다.

## 6. 외부 호환성과 인수 파일

| 항목 | 결과 |
|---|---|
| macOS 한컴오피스 한글 Viewer | 파일 손상 대화상자 없이 정상 개방, `강남대학교` 표시 |
| Windows 한글 2024 | 작업지시자 수동 확인으로 정상 개방 |
| 후보 크기 | 110,080 B |
| 후보 SHA-256 | `2ced2123f59cf0b7e5aac6167138139b82d471ac4e81b3fe46b5fc881ab3ce1e` |
| SectionDef `CTRL_DATA` | 1개 / 280바이트 / 원본 payload와 동일 |

후보 파일은 세션 임시 경로의 검증 산출물이다. 장기 회귀 근거는 저장소에 포함된
`tests/issue_3507_sectiondef_ctrl_data.rs`가 담당한다.

## 7. 남은 절차와 위험

- 작업지시자가 `devel` 대상 PR 생성까지 승인했다.
- 최신 PR head의 CI와 merge 가능성을 확인한 뒤 review·통합한다.
- 이슈는 수정 커밋이 `devel`에 포함된 것을 확인하기 전까지 닫지 않는다.
- 서로 다른 payload를 가진 복수 직접 자식 `CTRL_DATA`의 의미 해석·정규화는 이번 범위 밖이다.

구현과 인수 검증에서 확인된 미해결 결함은 없다.

## 8. 문서 지도

| 역할 | 문서 |
|---|---|
| 수행계획서 | [`task_m100_3507.md`](../plans/task_m100_3507.md) |
| 구현계획서 | [`task_m100_3507_impl.md`](../plans/task_m100_3507_impl.md) |
| Stage 1 보고서 | [`task_m100_3507_stage1.md`](../working/task_m100_3507_stage1.md) |
| Stage 2 보고서 | [`task_m100_3507_stage2.md`](../working/task_m100_3507_stage2.md) |
| CTRL_DATA 기술 문서 | [`hwp_ctrl_data.md`](../tech/hwp_ctrl_data.md) |
| HWP 저장 가이드 | [`hwp_save_guide.md`](../tech/hwp_save_guide.md) |
