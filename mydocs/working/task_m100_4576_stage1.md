# Task #4576 Stage 1 — 핫패치 무효화 계약을 `&mut self` 로 넓힌다

- Issue: [#4576](https://github.com/edwardkim/rhwp/issues/4576)
- 기준: `upstream/devel` `b66e3d79a`
- 작업 브랜치: `fix/issue-4576-subsecond-invalidation`
- 범위: 무효화 계약만. 핫패치 경계 추가(#4577), 진단·로깅(#4578), 수명(#4579),
  번들·명명(#4580)은 건드리지 않는다.

## 결함

(줄 번호는 기준 `upstream/devel` `b66e3d79a` 기준이다. 이슈 본문의 번호와 다르다.)

`invalidate_subsecond_render_caches`(`src/wasm_api.rs:909`)가 `&self` 라서 내부 가변성 뒤의
캐시 다섯 개(`page_tree_cache`, `layer_tree_json_cache`, `clear_layout_caches()` 의 셋)만 비울 수
있었다. 정작 `build_page_tree` → `find_page`(`rendering.rs:4991`)가 읽는 `pagination`,
`section_render_paragraphs`, `section_render_composed` 는 모두 `&mut self` 를 요구해 손댈 방법이
없었다.

그래서 핫패치 뒤 화면은 **새 코드가 낸 숫자를 패치 이전 코드가 잡은 페이지 박스에, 패치 이전
코드가 조합한 문단으로** 그린다 — 소스의 어느 버전에도 대응하지 않는 혼합물이다.

억지로 `paginate()` 를 불러도 소용없다. `paginate_pass`(`rendering.rs:3926`)는 깨끗한 구역을
건너뛰고(`:4059`), `compute_render_normalized`(`:4750`)는 revision 이 같으면
short-circuit 하며(`:4762`), 증분 측정은 `measured_sections` 와 `table.dirty` 로 옛 측정값을
재사용한다(`height_measurer.rs:2524` `measure_section_selective` — `dirty_paras=None` 이어도
`!table.dirty` 인 표는 이전 `MeasuredTable` 을 clone 한다). 문서가 안 바뀌었으므로 세 게이트
모두 "재사용" 을 고른다.

## 고른 방식 — 새 상태가 아니라 연산

"렌더러가 새로 빌드됐다" 는 사실을 담을 **플래그나 epoch 필드를 추가하지 않았다.** 대신 원본
IR 에서 파생 상태 전부를 다시 만드는 연산 하나를 이름 붙여 두고, subsecond 경계가 그것을
부른다.

```rust
// src/document_core/queries/rendering.rs
pub(crate) fn rebuild_derived_state(&mut self)   // 스타일 → 조합 → (정규화·측정) → 페이지네이션

// src/wasm_api.rs
pub fn invalidate_subsecond_render_caches(&mut self) {
    self.core.rebuild_derived_state();
}
```

### 왜 플래그가 아닌가

`code_epoch: u64` 를 `DocumentCore` 에 두는 안을 lifecycle 로 검증하면 셋 다 어긋난다.

| | `code_epoch`(가정) | `dirty_sections`(현행) |
| --- | --- | --- |
| 생성 | 프로세스 기동 시 1개 | 문서를 열 때마다 1개 |
| 변경 | dev server 가 패치를 보낼 때 | 키 입력마다 |
| 소멸 | 프로세스 종료 | 문서를 닫을 때 |

문서 3개를 연 세션이면 프로세스 단위 사실이 문서별 상태에 3벌 복제되고, 그중 2벌이 조용히
어긋날 수 있다. 그렇다고 `DocumentCore` 밖으로 빼면 게이트가 읽을 수 없다 — 게이트는
`DocumentCore` 안에서 돈다. 즉 필드로는 어느 쪽에 둬도 성립하지 않는다.

실제로 이 무효화에는 **수명이 없다.** 패치가 적용되는 순간 모든 파생본이 stale 이고, 동기
호출 한 번으로 전부 다시 만들어지며, 돌아온 뒤 기억해 둘 것이 남지 않는다. 플래그는 "누가
지워 줄 때까지 남는 상태" 를 표현하는 도구인데 여기엔 그런 상태가 없다. 그래서 필드가 아니라
메서드다.

### 왜 문서 편집과 혼동될 수 없는가

저장하는 상태가 없으니 나중에 오독될 상태도 없다. 그리고 "사용자가 문서를 바꿨다" 를 지고
있는 것들은 `rebuild_derived_state` 가 하나도 건드리지 않는다.

- `self.document` — 읽기만 한다
- `snapshot_store`(undo/redo) — 미접촉
- `event_log` — 미접촉
- `bin_data_epoch` — **일부러 올리지 않는다.** 이 세대는 "문서를 통째로 갈아끼운 연산" 에만
  올린다는 계약이고(`document.rs:1697-1705`), 핫패치는 그림 바이트를 한 바이트도 바꾸지
  않는다. `restore_snapshot_native` 는 이 bump 를 **공유 메서드 밖에서 자기가** 수행한다 —
  "문서가 교체됐다" 와 "파생본을 다시 만든다" 가 다른 사실이라는 것이 코드 배치로 드러난다.

`mark_all_sections_dirty()` 를 재사용하는 것은 두 개념을 합치는 것이 아니다. 그 함수가 뜻하는
바는 "구역별 파생본이 원본과 대응하지 않는다" 이지 "사용자가 편집했다" 가 아니며, 게이트도
"왜 stale 인가" 가 아니라 "stale 인가" 만 묻는다.

## 구현

### 단일 정의

`restore_snapshot_native`(`src/document_core/commands/document.rs:1709`)가 이미 같은 순서를
손으로 펼쳐 두고 있었다 — 스타일 재해소 → 재조합 → `mark_all_sections_dirty` → 측정 캐시
비우기 → 렌더 캐시 비우기 → `paginate()`. 그 몸통을 `rebuild_derived_state` 로 옮기고
호출로 바꿨다. 동작은 같다(`page_tree_cache.borrow_mut().clear()` → `invalidate_page_tree_cache()`
는 상위집합이고, `paginate()` 가 어차피 같은 것을 다시 부른다).

파생 순서는 소비 순서와 같아야 한다: 스타일 → 문단 구성 → (정규화·측정) → 페이지네이션 →
페이지 트리. 앞 단계를 뒤 단계보다 늦게 만들면 같은 패스 안에서 옛 값이 섞인다.

### 벤더 이름 비유입

`rebuild_derived_state` 는 subsecond 를 모른다 — 상황(파생본이 원본과 대응하지 않게 된 순간)만
안다. 확인:

```
$ grep -rn "subsecond\|HotFn" src/document_core/
(0 results)
```

subsecond 가 내일 사라져도 `restore_snapshot_native` 에는 여전히 필요한 메서드다.

## 검증

### RED before

수정 커밋의 몸통만 되돌리면(= `rebuild_derived_state` 가 `invalidate_page_tree_cache()` 만 하도록)
신규 테스트가 실패한다.

```
---- wasm_api::tests::issue_4576_rebuild_derived_state_recomputes_composition_and_pagination stdout ----
panicked at src/wasm_api/tests.rs:28474:5:
assertion `left == right` failed: 페이지네이션이 원본에서 다시 만들어져야 한다
  left: 2
 right: 3
test result: FAILED. 0 passed; 1 failed
```

테스트는 원본 IR 을 한 글자도 건드리지 않고 메모된 파생본만 "패치 이전 코드가 만들었을 값"
으로 바꾼 뒤,

1. `paginate()` 만으로는 아무것도 복구되지 않음을 고정하고(게이트가 살아 있다는 확인),
2. `rebuild_derived_state()` 뒤 조합·측정·페이지네이션·페이지 트리가 모두 기준선으로
   돌아옴을 확인한다.

CONTRIBUTING 의 `tests/issue_{번호}_{설명}.rs` 관례 대신 `src/wasm_api/tests.rs` 안의 unit
test 로 두었다. 파생본을 "패치 이전 코드의 값" 으로 바꾸려면 `composed`·`pagination`·
`measured_sections` 가 `pub(crate)` 라 통합 테스트에서 손댈 수 없고, `rebuild_derived_state`
자체도 `pub(crate)` 다. 같은 파일의 `issue_4325_...`(`src/wasm_api/tests.rs:413`)가 같은 이유로
같은 자리에 있다.

테스트는 `invalidate_subsecond_render_caches` 가 아니라 `rebuild_derived_state` 를 직접 부른다.
전자는 `#[cfg(feature = "subsecond-dev")]` 라 기본 게이트(`cargo test --profile release-test
--tests`)에서 컴파일되지 않기 때문이다. 경계가 그 계약을 실제로 부르는지는
`cargo check --lib --features subsecond-dev --target wasm32-unknown-unknown` 과 코드 두 줄로
확인한다.

### 게이트

| 게이트 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0 (경고 0) |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | exit 0 — `test result: ok` 블록 535개, 5,757 passed / 0 failed / 36 ignored |
| `cargo test --profile release-test --features native-skia skia --lib` | exit 0 — 58 passed / 0 failed |
| `cargo test --profile release-test --features native-skia --test issue_2225_missing_picture_placeholder` | exit 0 — 2 passed / 0 failed |
| `cargo test --profile release-test --features native-skia --test render_p37_direct_pdf_export` | exit 0 — 4 passed / 0 failed |
| `wasm-pack build --target web --out-dir pkg` | exit 0 — `Done in 2m 08s`, `pkg` 산출 |
| `cargo check --lib --features subsecond-dev --target wasm32-unknown-unknown` | exit 0 |

`cargo check --features subsecond-dev --target wasm32-unknown-unknown` 은 crate 전체로 돌리면
`bin "rhwp"`(`src/main.rs`)에서 7건 실패한다. `upstream/devel` `b66e3d79a` 무수정 트리에서
같은 7건이 그대로 재현되는 것을 확인했다 — CLI 바이너리는 wasm32 대상이 아니며 이번 변경과
무관하다. 이번 변경이 관여하는 feature-gated 코드는 `--lib` 로 통과한다.

### 검증하지 못한 것

**핫패치 실사용 경로는 검증하지 못했다.** `dx serve` 로 dev server 를 띄우고 브라우저에서
`renderer/composer.rs` 의 줄바꿈 규칙이나 `renderer/pagination/engine.rs` 의 임계값을 실제로
고쳐 넣는 왕복이 필요한데, 이 환경에서 실행할 수 없다. 그래서 이슈의 재현 절차(본문이 다시
감기는가, 쪽수가 바뀌는가)를 눈으로 확인한 것이 아니라, 그 재현이 실패하던 **원인**(원본이
그대로면 세 게이트가 재계산을 거부한다)을 Rust 테스트로 고정했다. 브라우저 왕복 확인은
작업지시자 환경의 몫으로 남는다.

## 발견했으나 고치지 않은 것

- `set_document`(`src/document_core/commands/document.rs:1646`) — 같은 전면 재구성의 세 번째
  사본인데 측정 캐시를 비우지 않는다. 이미 채워진 core 에 다른 문서를 넣으면
  `measure_section_incremental` 이 `!table.dirty` 인 표에 대해 **이전 문서의** `MeasuredTable`
  을 재사용할 수 있다(`dirty_paragraphs=None` 은 표를 덮지 못한다). `pub` API 의 동작 변경이고
  이번 이슈의 무효화 계약 밖이라 손대지 않았다.
- `set_respect_vpos_reset`(`src/wasm_api.rs:660`) — `mark_all_sections_dirty()` 를 부르지 않고
  `dirty_sections` 루프를 직접 돌려 revision bump 를 건너뛴다. 이 옵션은 normalization 에
  영향이 없어 현재는 결과가 같지만, 같은 뜻의 표현이 두 가지로 남아 있다.
- `invalidate_subsecond_render_caches` 라는 이름이 이제 하는 일보다 좁다 — 무효화가 아니라
  재구성이고, `pagination` 은 캐시가 아니다. 벤더 이름 경계는 #4580 소관이라 두었다.
- `RenderNormalizationState::document_epoch`(`src/document_core/mod.rs:111`) — 쓰기만 하고 읽는
  곳이 없다. `mark_all_sections_dirty`(`rendering.rs:3559`)가 올릴 뿐,
  `grep -rn document_epoch src/ tests/ examples/` 가 이 두 자리 말고는 잡지 못한다. 이번
  이슈에서 "코드가 바뀌었다"를 epoch 으로 표현하는 안을 검토할 때 발견했다. 죽은 상태이므로
  거기에 새 의미를 얹지 않았다.
- `rebuild_derived_state` 는 `batch_mode` 와 무관하게 `paginate()` 를 부른다. 기존 두 전면
  재구성 지점(`set_document`, `restore_snapshot_native`)과 같은 규약이라 새 불일치는 아니지만,
  batch 중 핫패치가 오면 그 batch 의 pagination 지연이 끝난다.


## 후속 이슈 (2026-08-11)

작업 중 발견했지만 범위 밖이라 손대지 않은 것을 이슈로 분리했다.

- **[#4582](https://github.com/edwardkim/rhwp/issues/4582)** — `set_document`
  (`commands/document.rs:1646`)이 전면 재구성 순서의 **세 번째 사본**이면서 측정 캐시
  정리가 빠져 있다. `!table.dirty` 인 표에 대해 이전 문서의 `MeasuredTable` 을 재사용할
  수 있다. `pub` API 동작 변경이라 호출부 전수 확인이 필요하다.
- **[#4583](https://github.com/edwardkim/rhwp/issues/4583)** — `document_epoch`
  (`mod.rs:111`)이 쓰기 전용이고, `set_respect_vpos_reset`(`wasm_api.rs:660`)이
  `mark_all_sections_dirty()` 와 같은 일을 다른 철자로 한다.

`invalidate_subsecond_render_caches`(`wasm_api.rs:909`)의 이름이 이제 하는 일보다
좁다 — 무효화가 아니라 재구성이고 `pagination` 은 캐시가 아니다. 벤더 이름 경계 자체가
#4580 의 범위라 그쪽에서 함께 정리한다.
