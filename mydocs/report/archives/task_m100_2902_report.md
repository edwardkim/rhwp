# Task #2902 처리 결과 — 각주/미주 삽입 시 문단 내 표/글상자 위치 미검증 결함 수정

## 이슈

https://github.com/edwardkim/rhwp/issues/2902

## 문제

`src/document_core/commands/object_ops/note.rs`의 `insert_footnote_native` /
`insert_endnote_native`가 신규 각주·미주 번호를 계산할 때, 같은 문단(`is_same`)
안의 `Control::Table` / `Control::Shape` 분기는 그 컨트롤 자신이 커서(`char_offset`)
보다 앞인지 뒤인지 확인하지 않고 무조건 안의 각주/미주를 선행으로 카운트했다.
바로 위 `Control::Footnote`/`Control::Endnote` 단독 분기는 `find_control_text_positions`
로 실제 위치를 구해 비교하는데, 표/글상자 분기만 이 검사가 빠져 있었다.

영향: 문단 뒷부분에 각주/미주 포함 표·글상자가 있고 그보다 앞에서 새 각주/미주를
삽입하면 번호가 부풀려진다. 삽입 직후 전체 재넘버링 루프는 본문 참조 마커
(`Footnote.number`/`Endnote.number`)는 바로잡지만, 각주/미주 내용 문단 맨 앞의
`AutoNumber` 표시 번호는 생성 시점 값을 그대로 쓰므로 본문 참조 번호와 내용 패널
표시 번호가 어긋나는 사용자 가시적 결함으로 이어진다.

## 수정

`Control::Table` / `Control::Shape` 가드에 `is_same` 조건일 때 `positions.get(ci) <=
char_offset` 위치 비교를 추가. `insert_footnote_native`, `insert_endnote_native` 양쪽
모두 동일하게 적용.

## 테스트 (red → green)

`char_shape_inherit_tests` 모듈에 회귀 테스트 추가:
`insert_footnote_ignores_table_footnote_positioned_after_cursor_in_same_paragraph`

- 시나리오: 문단 텍스트 "AB" 뒤에 표(셀 안 기존 각주 1개 포함)를 sibling control로
  배치. 문단 맨 앞(`char_offset=0`, 표보다 앞)에서 신규 각주 삽입.
- 기대: `footnoteNumber == 1`.
- 수정 전 코드로 되돌려 확인: `assertion left == right failed ... left: 2, right: 1`
  (RED 확인 완료).
- 수정 후: PASS (GREEN).

## 검증

- `cargo build --lib`: 성공
- `cargo test --lib object_ops::note`: 4 passed (신규 테스트 포함)
- `cargo clippy --all-targets --profile release-test -- -D warnings`: 통과
  (`Box::new(Footnote::default())` → `Box::<Footnote>::default()` clippy 지적 반영)
- `rustfmt --edition 2021 src/document_core/commands/object_ops/note.rs`: 적용

## 변경 파일

- `src/document_core/commands/object_ops/note.rs`

## 범위

작업 지시에 따라 `src/document_core/commands/object_ops/{picture.rs,note.rs}`만
검토·수정했다. `picture.rs`는 리뷰 결과 캐시 무효화(`invalidate_page_tree_cache`)
호출 누락처럼 보이는 지점들을 확인했으나, 모두 `recompose_section`/`paginate_pass`
내부에서 이미 캐시를 무효화하므로 실질적 결함이 아니었다 (관련 함수:
`document_core/queries/rendering.rs`의 `recompose_section`, `paginate_pass`).
