# #3507 Stage 1 — SectionDef CTRL_DATA 단일 소유와 focused 검증

- Issue: #3507
- 브랜치: `codex/issue-3507-sectiondef-ctrl-data`
- 착수 기준: `upstream/devel` `ef72fee5138e3c491b6d8f38e459dc94670284f6`
- 수행일: 2026-07-28

## 구현

SectionDef `CTRL_HEADER`에서 첫 중첩 `CTRL_HEADER` 전에 나타나는 첫 직접 자식 `CTRL_DATA`를
`Paragraph.ctrl_data_records`의 canonical ownership으로 고정했다.
`parse_section_def()`는 이 첫 레코드를 `SectionDef.extra_child_records`에 다시 넣지 않는다.
payload가 다른 추가 직접 자식, 중첩 control의 `CTRL_DATA`, 중첩 header 뒤의 직접 자식
`CTRL_DATA`는 raw record로 계속 보존한다.

과거 parser 또는 외부 생성 IR이 같은 직접 자식 레코드를 두 필드에 함께 보유한 경우를 위해
SectionDef serializer에도 방어를 추가했다. 문단 슬롯이 전달한 것과 tag·level·payload가 모두 같은
exact duplicate만 건너뛰며, canonical 사본은 공용 control 경로가 `CTRL_HEADER` 직후에 출력한다.

## red→green

수정 전 새 회귀 테스트 결과:

```text
parser::body_text::tests::test_section_def_direct_ctrl_data_has_single_owner_and_nested_records_are_preserved
  FAILED — 문단 소유 payload가 SectionDef extra에도 존재

serializer::control::tests::test_section_def_exact_ctrl_data_duplicate_is_serialized_once
  FAILED — expected 1, actual 2
```

수정 후 같은 필터 결과:

```text
6 passed / 0 failed
```

실물 통합 테스트 `tests/issue_3507_sectiondef_ctrl_data.rs`도 추가했다. 저장소의
`samples/복학원서.hwp`를 실제 `edit set-cell` CLI로 편집하고 다음을 한 번에 고정한다.

- `(table=0,row=1,col=1)` 재독 값 `강남대학교`
- SectionDef 직접 자식 `CTRL_DATA` 1개
- payload 280바이트
- 원본과 저장본 payload 완전 일치

## focused 검증

| 검증 | 결과 |
|---|---|
| `cargo test section_def_ --lib -- --nocapture` | 6 passed |
| `cargo test parser::body_text::tests --lib` | 23 passed |
| `cargo test serializer::control::tests --lib` | 24 passed |
| `cargo test --test issue_3507_sectiondef_ctrl_data` | 1 passed |
| `cargo test --test edit_set_cell_contract` | 5 passed |
| `cargo test --test issue_838_field_set_value` | 2 passed |
| `cargo test --test issue_852_hwpx_to_hwp_contract_streams` | 5 passed |
| `cargo test --test issue_1251_ole_chart_contents` | 10 passed |
| 빠른 IR field sweep(300KB 이하) | 608건, 회귀 0 |
| `cargo fmt --all -- --check` | pass |
| `git diff --check` | pass |

빠른 IR field sweep에서 기존 baseline의 `복학원서.hwp` 발산 중 다음 2개가 사라졌다.

```text
sections[].paragraphs[].controls[].section_def.extra_child_records.len
sections[].section_def.extra_child_records.len
```

이는 중복 소유 제거로 기대한 개선이다. 전체 sweep dump로 같은 범위의 stale allowance를 확정한 뒤
baseline을 갱신해야 한다.

## 실물 CLI와 외부 판정

편집 저장본:

| 항목 | 값 |
|---|---|
| 크기 | 110,080 B |
| SHA-256 | `2ced2123f59cf0b7e5aac6167138139b82d471ac4e81b3fe46b5fc881ab3ce1e` |
| `hwp5-roundtrip` | PASS, IR diff 0, round2 diff 0 |
| `diag` | pass |

`hwp5-ctrl-data-trace`:

| side | records | bytes | hash |
|---|---:|---:|---|
| 원본 | 1 | 280 | `8f6bfafab661230a` |
| 저장본 | 1 | 280 | `8f6bfafab661230a` |

macOS `한컴오피스 한글 Viewer`에서 저장본을 직접 열었다. 파일 손상 대화상자 없이 1쪽 문서가
정상 개방됐고, 첫 표의 `강남대학교` 값도 화면에 표시됐다.

Windows `한글 2024` 판정은 현재 macOS 작업 환경 밖이므로 작업지시자 인수 항목으로 남긴다.

## 다음 게이트

focused 결과를 공유한 뒤 저장소 승인 절차에 따라 다음 PR 직전 전체 검증을 순차 실행한다.

1. 전체 IR field sweep dump와 baseline 개선분 갱신
2. `cargo test --profile release-test --tests`
3. `cargo fmt --all -- --check`
4. `cargo clippy --all-targets -- -D warnings`
5. 최종 diff·문서·작업표 정리

위 게이트의 수행 결과와 전체 sweep baseline 판정은
[`task_m100_3507_stage2.md`](task_m100_3507_stage2.md)에 기록한다.
