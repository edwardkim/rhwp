# Task #598 Stage 4-1 완료보고서 — 본문 각주 삭제 Rust API

## 작업 개요

- **Issue**: [#598](https://github.com/edwardkim/rhwp/issues/598)
- **브랜치**: `local/task598`
- **단계 범위**: 2차 작업 중 Rust/WASM 삭제 API 기반 구현

본 단계에서는 본문 각주 마커 앞/뒤 커서 위치를 조회하고, 해당 각주 컨트롤을 삭제하는 Rust native API 및 WASM 공개 API를 추가했다.
rhwp-studio 키보드 입력 연결은 다음 단계(Stage 4-2) 범위로 남겨둔다.

## 구현 내용

### Rust native API

- `get_footnote_at_cursor_native(section_idx, para_idx, char_offset, direction)` 추가
  - `direction="backward"`: Backspace 기준으로 커서 바로 앞 각주 마커 조회
  - `direction="forward"`: Delete 기준으로 커서 바로 뒤 각주 마커 조회
  - 반환: `hit`, `sectionIndex`, `paragraphIndex`, `controlIndex`, `charOffset`, `footnoteNumber`

- `delete_footnote_native(section_idx, para_idx, control_idx)` 추가
  - 본문 `Control::Footnote` 검증
  - 각주 마커의 텍스트 위치 복원
  - HWP 컨트롤 슬롯 8 UTF-16 code unit 제거에 맞춰 `char_offsets` / `char_count` 조정
  - `controls` / `ctrl_data_records` 동시 제거
  - 남은 각주 번호를 문서 순서대로 재계산
  - 본문 문단 reflow, section recompose, pagination, page tree cache 무효화 수행

### WASM API

- `getFootnoteAtCursor(sectionIdx, paraIdx, charOffset, direction)` 추가
- `deleteFootnote(sectionIdx, paraIdx, controlIdx)` 추가

### 보조 변경

- 컨트롤 삭제 후 문단 reflow helper 를 각주 삭제 경로에서도 재사용할 수 있도록 `pub(crate)` 로 조정했다.
- 각주 삽입 시 `control_mask` 에 각주/미주 비트(`1 << 0x0011`)가 설정되도록 보정했다.
- `DocumentEvent::FootnoteDeleted` 를 추가했다.

## 검증 결과

실행 명령:

```bash
cargo test --test issue_598_footnote_marker_nav
cargo test navigable_text_len_counts_trailing_footnote_marker
cargo build
git diff --check
```

결과:

- `cargo test --test issue_598_footnote_marker_nav`: 3 passed
- `cargo test navigable_text_len_counts_trailing_footnote_marker`: 1 passed
- `cargo build`: 통과
- `git diff --check`: 통과

## 테스트 추가

`tests/issue_598_footnote_marker_nav.rs` 에 삭제 회귀 테스트를 추가했다.

확인 항목:

1. 첫 번째 본문 각주 마커가 Backspace/forward 조회 API에서 정상 탐지되는지
2. `delete_footnote_native(0, 3, 0)` 실행 후 해당 문단의 각주 컨트롤 위치가 사라지는지
3. 기존 마커 좌표 hit test 가 삭제 후 `hit:false` 로 바뀌는지
4. 두 번째 각주 번호가 `1` 로 재계산되는지

## 남은 작업

다음 단계(Stage 4-2)에서는 rhwp-studio 쪽 키보드 입력을 연결한다.

- Backspace: 커서 바로 앞 각주 마커 삭제
- Delete: 커서 바로 뒤 각주 마커 삭제
- `SnapshotCommand` 기반 Undo/Redo 연결
- 브라우저 수동 검증 준비
