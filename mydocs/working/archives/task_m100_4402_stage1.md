# task_m100_4402 Stage 1 — HWP5 저장이 미기입 누름틀 안내문을 복원하지 않는다

- **이슈**: [#4402](https://github.com/edwardkim/rhwp/issues/4402)
- **브랜치**: `fix/issue-4402-hwp5-guide-residue`
- **분기 기준**: `upstream/devel` (0 behind)
- **상태**: 게이트 전부 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 결함

누름틀(필드)에 아직 아무것도 입력하지 않으면 본문에는 안내문이 남아 있다. HWP5 로 저장하면
이 안내문(`guide_residue`)이 복원되지 않는다. HWPX 축은 #3545 에서 이미 해소됐고 **HWP5 축만
남아 있었다.**

## 2. 검증 — 수정 전 4건 RED

```
form_02_survives_hwp_to_hwpx_to_hwp5_roundtrip
  form-02.hwp 안내문이 HWP→HWPX→HWP5 왕복에서 소실/중복됐다  left: 0  right: 1
initial_guide_text_save_reload_save_fixed_point
  저장→재적재→재저장 고정점에서 안내문이 소실/중복됐다
hwp_to_hwpx_to_hwp5_roundtrip_preserves_guide_text
sibling_field_edit_forces_reserialize_and_guide_text_survives
  다른 필드(작성자) 편집으로 재구성된 저장본에서 회사명의 안내문이 소실/중복됐다
test result: FAILED. 3 passed; 4 failed
```

같은 파일의 나머지 3건은 수정 전에도 통과한다 — `raw_stream` 통과 경로가 깨지지 않았음을
확인하는 비회귀 가드라서 그렇다. 즉 결함 범위가 재직렬화 경로에 한정됨을 함께 보인 것이다.

## 3. 게이트 (완료)

- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.
- `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` — 바이너리 **498개, 0 failed**.
  합계 **5,506 passed / 0 failed / 35 ignored**. `test result: FAILED` grep 0건.

## 4. 새 코드에 남긴 미검증 경로

`shift_char_shapes_for_residues`(`src/serializer/body_text.rs` 신규, ~850행)는 잔여물 삽입
위치의 동률 `char_shapes` 그룹에 `residue.char_shape_id` 와 일치하는 항목이 **없을 때** 그룹
전체를 민다. 즉 필드 자기 스타일 항목이 필드 경계에 정확히 놓이지 않고 그보다 앞에 있는 경우다.

**시험한 실제 샘플 3건은 전부 정확히 일치해서 이 폴백 경로를 타지 않는다.** 재현 사례를 찾지
못했으므로 이 분기는 미검증 상태다. 실물에서 걸리면 전용 픽스처가 필요하다.

## 5. 이 작업에서 고치지 않은 것

`parse_paragraph`(`src/parser/hwpx/section.rs:492`, `:496`)가 `SectionDef` 를 항상 먼저 push 해
HWP5 원본의 `[ColumnDef, SectionDef]` 순서를 왕복에서 뒤집는다. `samples/field-01-memo.hwp`
문단 0.0·1.0·2.0 에서 `ctrl[0] A=cold vs B=secd` 로 관측된다. **이 수정 전후 모두 재현되므로
기존 결함이고, 범위 밖이라 손대지 않았다** — [#4433](https://github.com/edwardkim/rhwp/issues/4433).

## 6. 미처리

GitHub Actions, 작업지시자 승인, merge.
