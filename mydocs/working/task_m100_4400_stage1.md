# Task M100-4400 Stage 1 — HWP5 바이트 배치 지식 누출 조사, 5번 항목만 정리

## 배경

이슈 #4400: "HWP5 바이너리 바이트 배치 지식이 렌더러와 편집 커맨드로 새어 나왔다." 다섯 항목이
제기되었다. 이 stage는 다섯 항목을 적대적으로 검증하고, 그중 가장 작고 안전하다고 판단한 **5번**
(직렬화기가 `document_core::converters`를 import하는 역방향 의존)만 고쳤다. **1~4번은 손대지
않았다** — 조사 후 반증(1번)했거나, 확인은 됐지만 이번 stage 범위 밖으로 남겼다(2·3번, 5번의
마커 상수 잔여분).

## 1단계 판정

| # | 내용 | 판정 |
|---|---|---|
| 1 | 렌더러가 `raw_table_ctrl_height_px`로 CTRL_HEADER height 직접 디코딩 (`typeset.rs:441`) | **반증 — 시도했다가 되돌림.** 아래 "되돌린 시도" 절 참고 |
| 2 | `section_has_zero_high_attr_rowbreak_table`이 `raw_table_record_attr` 상위 바이트 검사 (`typeset.rs:3097`) | **확인됨 — 안 고침.** `table.attr`는 `CommonObjAttr.attr`의 동기화 값이라 실제로는 다른 필드다. `raw_table_record_attr`의 상위 바이트(및 대부분 비트)는 IR에 전혀 모델링돼 있지 않다. IR 표현력 부족이 원인인 진짜 누출 — 별도 이슈 대상 |
| 3 | 편집 커맨드가 PARA_HEADER 꼬리(`raw_header_extra`) 손조립 | **확인됨 — 안 고침.** 5개 파일 8곳 이상에서 중복. 부가 발견: `raw_header_extra[0..2]`/`[4..6]` 쓰기는 `serializer/body_text.rs:404`가 "건너뜀"이라 명시하고 실제로 안 읽는 죽은 코드다. 별도 이슈 대상 |
| 4 | 도형 편집마다 `pack_common_attr_bits` 재계산 (`object_ops/common.rs`) | **반증 — 정당한 설계.** `CommonObjAttr.attr`가 CTRL_HEADER attr의 passthrough 소스라, 편집이 decomposed 필드만 바꾸면 attr가 stale해지는 걸 막는 필수 동기화다(`serialization_passthrough_contract.md`가 경고하는 것과 같은 성격). 유일한 잔여 문제는 함수 위치 — 5번과 동일 사안으로 수렴 |
| 5 | 직렬화기가 `document_core::converters` import | **확인됨 — 이번 stage에서 고침(부분).** 아래 "적용한 변경" 참고. 마커 상수(`parser/mod.rs:362,547` ↔ `hwpx_to_hwp.rs:2074,2090`) 잔여분은 범위 밖으로 남김 |

## 적용한 변경 — 5번(부분)

`pack_common_attr_bits`(+ 8개 `_to_bits` 헬퍼)를
`document_core/converters/common_obj_attr_writer.rs` → `src/serializer/control.rs`로 이동했다.

근거: 이 함수는 `CommonObjAttr` enum 필드로부터 CTRL_HEADER attr u32 비트를 합성하는, 본질적으로
직렬화 로직이다. `document_core/converters/mod.rs` 자신의 모듈 헌장("잘 작동하는 직렬화기 어깨
위에 서자 — 직렬화기 자체를 수정하지 않고 IR만 정렬한다")과 정면 모순이었고, 정작 진짜
직렬화기(`serializer/control.rs`, `serializer/hml/preflight.rs`)가 이걸 역으로 import하고
있었다.

호출부 4곳은 import 경로만 바꿨다: `converters/hwpx_to_hwp.rs`(3곳),
`converters/common_obj_attr_writer.rs`의 `serialize_common_obj_attr`/`sync_anchor_bits`,
`commands/object_ops/common.rs`. 로직은 바꾸지 않았다.

커밋: `a62fffc15` (브랜치 `worktree-agent-a1cd33b0a5c6dfbd6`)

### 자기검증에서 나온 잔여 결함 (미해결로 기록)

gestell 방법론(`~/.claude/skills/gestell/references/maintenance.md`)의 "total
independence — zero imports, either direction" 기준으로 재검토한 결과, 이 이동은
**완전하지 않다**:

- `document_core/converters/common_obj_attr_writer.rs`에 남은 `sync_anchor_bits`는
  `pack_common_attr_bits`와 **같은 종류의 CTRL_HEADER attr 비트 지식**을 다룬다 —
  자체적으로 `0x01 | (0x03 << 3) | (0x03 << 8)` 같은 비트 마스크 리터럴을 하드코딩하고,
  이제 `serializer::control::{vert_rel_to_to_bits, horz_rel_to_to_bits}`를 `pub(crate)`로
  가져다 쓴다. `pack_common_attr_bits`를 옮긴 것과 같은 잣대(직렬화 로직이 document_core에
  있으면 안 된다)를 적용하면 `sync_anchor_bits`도 이동 대상이었는데 이번 stage에서는
  안 옮겼다.
- 그 결과 `document_core → serializer` 방향 import가 여전히 3개(`pack_common_attr_bits`,
  `vert_rel_to_to_bits`, `horz_rel_to_to_bits`) 남아 있다. `pack_common_attr_bits` 1개는
  `serialize_common_obj_attr`(IR 필드 backfill이라는 이 모듈의 정당한 역할)의 최소 의존이라
  방어 가능하지만, 나머지 2개는 `sync_anchor_bits`를 옮기지 않아서 생긴 잔여 결합이다.
- `vert_rel_to_to_bits`/`horz_rel_to_to_bits`를 `pub(crate)`로 넓힌 것도 이 잔여 결합
  때문이다 — `sync_anchor_bits`가 옮겨졌다면 이 둘은 `serializer::control` 안에서 완전히
  private로 남을 수 있었다.

후속 조치 후보(이번 stage에서는 안 함): `sync_anchor_bits`도 `serializer/control.rs`로
옮기고, `document_core/commands/object_ops/picture.rs`의 호출 3곳(478, 679, 740행)이 새
경로를 import하도록 바꾼다. 그러면 `common_obj_attr_writer.rs`에는 `serialize_common_obj_attr`
하나(및 그 1개의 방어 가능한 `pack_common_attr_bits` 의존)만 남는다.

## 되돌린 시도 — 1번

처음에는 1번(`raw_table_ctrl_height_px`가 CTRL_HEADER height 바이트를 직접 파싱)을 정적
분석만으로 "안전한 leak"이라 판단해 실제로 고쳤다 — `table.common.height`가 `Table`의 모든
뮤테이터(`update_ctrl_dimensions` 등)에서 `raw_ctrl_data`와 항상 함께 갱신되는 "의도된
dual"임을 `model/table.rs:795` 주석과 여러 호출부에서 확인했고, 같은 파일 안에 이미 정확히
같은 원칙을 쓰는 선례(`get_table_vertical_offset`, #178)도 있었기 때문이다.

그런데 `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`를 실제로 돌리자
**즉시 회귀**했다:

```
thread 'issue_2007_saved_frame_tail_nested_table_starts_before_next_frame' (119037489) panicked at tests/issue_2007_nested_cell_pagination.rs:191:5:
16쪽이 이해관계자 협의 저장 프레임에서 재개하지 않았다
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    issue_2007_saved_frame_tail_nested_table_starts_before_next_frame

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
error: test failed, to rerun pass `--test issue_2007_nested_cell_pagination`
```

`git diff`를 되돌리면(원래 raw-byte 읽기 복원) 통과, 다시 적용하면 실패 — red/green으로
직접 인과관계를 증명했다. 실 문서 fixture(`samples/basic/issue2007_nested_cell_pagination_42065.hwp`)
파싱 직후에는 `table.common.height`와 raw 바이트가 모든 표에서 일치함을 별도 진단
바이너리(임시, 이후 삭제)로 확인했으므로, 렌더링 파이프라인 어딘가(정확한 지점은 특정하지
못함 — `height_measurer.rs`의 "stretched"/"target_body_height" 계열 로직이 유력 후보)에서
`table.common.height`만 동적으로 바뀌는 실재 경로가 있다는 뜻이다. 즉 raw 우선 읽기는 leak이
아니라 그 mutation을 우회해 "원래 선언된" 높이를 되찾는 필수 로직이었다. **이 시도는
커밋하지 않고 코드를 원상복구했다** — `git diff upstream/devel..HEAD -- src/renderer/typeset.rs`
는 비어 있다.

## 회귀 없음 근거 (red→green 짝이 없는 이유)

실제로 커밋한 5번(부분) 변경은 **로직을 한 글자도 바꾸지 않는 순수 코드 이동**이다.
고친 버그가 없으므로 이 변경에 대한 전용 red→green 회귀 테스트는 없다 — 새 회귀 테스트를
작성하지 않았다.

대신 사용한 비회귀(non-regression) 논거:

1. `document_core::converters::common_obj_attr_writer` 유닛테스트 15개 — 무변경 통과
2. `serializer::control` 유닛테스트 25개 — 무변경 통과
3. `issue_3552_table_common_attr_save`, `hwpx_to_hwp_adapter`, `issue_1916`,
   `issue_1916_tbl_common_attr`, `issue_2724_passthrough_invalidation_guard`,
   `issue_1061_equation_serialize` — 전부 무변경 통과
4. `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` — 클린
5. `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` 전체 스위트(497개
   `test result: ok` 블록, 0 FAILED) 완주 — 한 차례(`--no-fail-fast`) 실행에서는
   `issue_2007_nested_cell_pagination.rs`의 다른 서브테스트가 실패했으나, 그 파일만 단독
   격리 실행하면 항상 6/6 통과했다. 두 번의 전체 실행에서 같은 파일의 *서로 다른*
   서브테스트가 실패한 것 자체가 "이 변경에 결정론적으로 연동된 회귀"가 아니라 동시 실행
   부하(다중 worktree 경합) 하 flaky라는 근거다 — 이번 변경은 페이지네이션 로직을 전혀
   건드리지 않는다.

## 미해결 — 4가지 (별도 이슈로 분리 예정)

1. `section_has_zero_high_attr_rowbreak_table` — `src/renderer/typeset.rs:3097` (항목 2:
   raw 상위 비트에 대응하는 IR 필드가 없음)
2. `table.common.height`를 렌더링 파이프라인 어딘가에서 동적으로 바꾸는 미확인 지점 — 정확한
   위치를 특정하지 못함(항목 1을 반증하는 과정에서 실재를 증명했으나 원인 함수는 못 찾음)
3. PARA_HEADER 꼬리 손조립(항목 3) — `html_table_import.rs:655,657`,
   `commands/object_ops/table.rs:593-594,627-628`, `commands/object_ops/shape.rs:1169-1170`,
   `commands/table_ops.rs:371-372`, `commands/text_editing.rs:732-733`
4. `HWPX_ORIGIN_STREAM_PATH`/`HWP3_ORIGIN_STREAM_PATH`(`converters/hwpx_to_hwp.rs:2074,2090`)를
   `parser/mod.rs:362,547`가 역으로 import — `model/document.rs`의 `HWP5_ORIGIN_HWPX_MARKER_PATH`와
   통일 필요(항목 5의 잔여분)

그 외 이번 자기검증에서 새로 발견한 것 — 위 "자기검증에서 나온 잔여 결함" 절 참고
(`sync_anchor_bits`가 옮겨지지 않아 남은 `document_core → serializer` import 2개).

## 검증

```
cargo build --lib
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --lib document_core::converters::common_obj_attr_writer
cargo test --lib serializer::control::
cargo test --test issue_3552_table_common_attr_save --test hwpx_to_hwp_adapter \
  --test issue_1916_tbl_common_attr --test issue_1916 \
  --test issue_2724_passthrough_invalidation_guard --test issue_1061_equation_serialize
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
```
