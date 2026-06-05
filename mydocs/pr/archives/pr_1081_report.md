# PR #1081 처리 결과 보고서

- PR: <https://github.com/edwardkim/rhwp/pull/1081>
- 제목: `refactor: CommonObjAttr raw_ctrl_data 바이트 오프셋 상수 모듈 (#698 후속)`
- 처리일: 2026-05-26
- 작성자: Codex

## 1. 처리 요약

PR #1081은 수용 가능하다고 판단했다.

다만 PR 원안은 `CommonObjAttr` raw byte offset 상수 모듈만 추가하고 실제 사용처는 바꾸지 않았다.
최근 #1077, #1078에서 같은 4바이트 오프셋 오해가 반복되었으므로, 단순 cherry-pick으로 끝내지 않고
소스의 고위험 사용처까지 상수 기반으로 변경했다.

## 2. 반영 커밋

PR 커밋 cherry-pick:

```text
99f3a8ab refactor: CommonObjAttr raw_ctrl_data 바이트 오프셋 상수 모듈 추가
a01debb9 refactor: Copilot 피드백 반영 — pub(crate) 가시성 축소 + MIN_LEN 자동 동기화
```

유지보수자 보강 커밋:

```text
bacb7484 refactor: apply CommonObjAttr offset constants
```

## 3. 추가 적용 내용

`src/model/shape.rs`의 `common_obj_offsets`를 실제 소스 경로에 적용했다.

적용 범위:

```text
src/document_core/html_table_import.rs
src/document_core/commands/object_ops.rs
src/document_core/commands/table_ops.rs
src/document_core/converters/hwpx_to_hwp.rs
src/model/table.rs
```

주요 변경:

```text
1. CommonObjAttr flags, width, height, margin, instance_id 직접 slice 숫자를 상수로 대체
2. table ops의 offset/keepWithAnchor 읽기와 쓰기를 상수 기반으로 변경
3. hwpx_to_hwp adapter의 table raw_ctrl_data 접근을 상수 기반으로 변경
4. prevent_page_break 영역을 위한 crate 내부 상수 추가
5. 기존 offset 정합 테스트를 상수 기반으로 보강
```

직접 인덱싱 잔존 여부:

```text
rg "raw_ctrl_data\\[[0-9]+(\\.\\.[0-9]+)?\\]" src/document_core src/model -n
=> 매칭 없음
```

## 4. 검증

실행한 검증:

```text
cargo fmt
cargo fmt --check
cargo check
cargo test raw_ctrl_data_offsets_match_parser
cargo test update_ctrl_dimensions_writes_correct_slots
cargo test test_paste_html_table_as_control
cargo test test_parse_table_html_save
```

결과:

```text
success
```

확인된 warning은 기존 경고로, 이번 변경에서 새로 만든 경고는 확인되지 않았다.

## 5. 판단

컨트리뷰터의 상수 모듈 도입 주장은 타당하다.

하지만 원안만으로는 재발 방지 효과가 제한적이었으므로, 이번 처리에서는 상수 추가와 함께 실제
CommonObjAttr raw byte 사용처를 마이그레이션했다. 이 상태라면 PR #1081의 의도인 “4바이트 오프셋
오해 재발 방지” 효과가 코드에 직접 반영된다.

## 6. 남은 절차

보고서 승인 후 다음 절차를 진행한다.

```text
1. 보고서 문서 커밋
2. PR #1081에 처리 결과 코멘트 작성
3. PR #1081 close
4. 필요 시 관련 이슈 상태 확인
5. local/devel -> devel 병합
6. devel에서 검증 후 origin/devel push
```
