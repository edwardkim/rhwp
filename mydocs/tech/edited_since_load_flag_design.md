---
kind: decision
status: active
canonical: mydocs/tech/document_ir_lineseg_standard.md
last_verified: 2026-08-03
---

# 로드 이후 편집 여부 플래그 설계 (#3777)

> **상태: 제안.** 아직 채택되지 않았고 코드도 없다. 이 문서는 조사 결과와 설계
> 선택지를 정리해 판단을 받기 위한 것이다.

저장 `LINE_SEG` 를 **권위**로 쓰는 판별자가 성립하려면, 렌더러가 "이 문서가 로드 이후
편집되었는가" 를 알아야 한다. 이 문서는 그 신호를 어디에 둘지, 무엇을 신호로 삼으면
안 되는지, 어떤 계약으로 못박을지를 정리한다.

## 1. 왜 필요한가

`#3746` 은 자리차지 표에서 **저장 사다리가 선언 높이를 뒷받침하면 선언이 권위**라는
판별자였다. 렌더 게이트 3종(92셋 · 10k 모집단 586건 · 표 코호트)을 모두 통과했으나
`nextest` 가 편집 계약 회귀를 잡아 철회했다.

```
issue_1951_table_cell_cursor_clip::one_depth_path_insert_reflows_the_same_as_direct_cell_insert
```

셀에 글자를 넣으면 내용이 늘어난다. 그런데 저장 사다리는 **편집 이전**을 적어 둔 값이라,
그걸 권위로 삼으면 새로 넣은 글자가 잘린다.

같은 IR 형상이 문맥에 따라 반대 진실을 갖는다.

```
1051000 pi=59 (파일 그대로)   셀 426.7px · 선언 253.1 · 호스트 줄 256.9   -> 선언 유지가 맞다
#1951    (셀에 삽입한 뒤)     셀 474.3px · 선언 302.3 · 호스트 줄 306.0   -> 성장이 맞다
```

셀 줄 수(`stored=36 measured=36`) · 셀 내용 높이 · 호스트 줄 높이 **어느 것으로도 안
갈린다.** 이 축(`#2148` 계열)이 오래 환원 불가로 남은 이유가 그것이다.

## 2. 왜 지금 IR 로는 못 가르나

편집 경로가 셀 `LINE_SEG` 를 재생성하는데, `composer/line_breaking.rs::reflow_line_segs`
가 **원본 `tag` 를 그대로 물려받는다.** 그래서 "한글이 클리핑해 저장한 표" 와
"rhwp 가 편집 후 재배치한 표" 가 IR 상 구별되지 않는다.

## 3. `table.dirty` 는 쓸 수 없다 — 그리고 그 실패가 설계 제약이다

의미상 맞는 신호지만 **페이지네이션 수렴 후 초기화**된다.

```
document_core/queries/rendering.rs:3268   table.dirty = false;
document_core/queries/rendering.rs:4056   table.dirty = false;
```

편집 직후 측정에서는 `true` 라 클램프를 막지만, 그 뒤 전체 재측정이 한 번 더 일어나면
`false` 로 돌아가 클램프가 되살아난다. **렌더할 때마다 결과가 달라지고**, 결국 셀
글자가 잘린다. 부분 수정은 오히려 더 나쁘다.

여기서 이 설계의 **1급 제약**이 나온다.

> 플래그는 **렌더·페이지네이션을 거쳐도 살아남아야 한다.**
> 렌더가 플래그를 바꾸면 같은 문서가 렌더 횟수에 따라 다르게 보인다.

## 4. 후보 신호 조사 (2026-08-03 실측)

### 4.1 `invalidate_page_tree_cache` — 편집 전용이 아니다

명령 계층에서 23곳이 부르지만, `queries/cursor_rect.rs`(6) · `queries/field_query.rs`(2)
에서도 불린다. 캐시 무효화는 편집의 필요조건이지 충분조건이 아니다.

### 4.2 `mark_all_sections_dirty` — 너무 좁다

호출처가 **4곳**(`queries/rendering.rs` 2 · `commands/document.rs` 2)뿐이다. 글자 삽입 ·
표 조작 같은 일상 편집은 더 잘게 쪼갠 `mark_paragraph_dirty` 경로를 타므로 안 걸린다.

### 4.3 `document_mut()` 게이트 — 지금은 비용이 크다

`document` 는 `pub(crate)` 이고, 크레이트 안에서 변형 패턴으로 만지는 지점이 **444곳**
이다. 접근자로 좁히는 것은 옳은 방향이지만 이 이슈 하나로 감당할 리팩터링이 아니다.

### 4.4 변형 표면의 실제 분포

| 위치 | `&mut self` 공개 함수 | 성격 |
| --- | --- | --- |
| `commands/` (11개 모듈) | 편집 API 전부 | **내용 편집** |
| `queries/rendering.rs` | 17 | 대부분 **레이아웃 상태** (`paginate` · `mark_*_dirty` · `recompose_*` · `compute_render_normalized`) |
| `queries/rendering.rs::set_section_def_all_native` | 1 | **내용 편집** (구역 정의) |
| `queries/field_query.rs::set_field_value_by_name` | 1 | **내용 편집** (필드 값) |
| `queries/field_query.rs::clear_active_field` | 1 | UI 상태 |
| `queries/changed_pages.rs` | 1 | 레이아웃 상태 |

**단일 관문은 없다.** 그러나 내용을 바꾸는 지점은 `commands/` 전부 + 명시적으로 꼽을 수
있는 **2개**뿐이라, 열거해서 관리할 수 있는 규모다.

## 5. 제안

### 5.1 상태

`DocumentCore` 에 로드 이후 한 번이라도 내용이 바뀌었는지를 담는 단조 플래그를 둔다.

```rust
/// 로드 이후 문서 **내용**이 바뀌었는가.
///
/// 렌더·페이지네이션은 이 값을 바꾸지 않는다 — 바꾸면 같은 문서가 렌더 횟수에 따라
/// 다르게 보인다(`table.dirty` 가 그렇게 실패했다, #3746).
edited_since_load: bool,
```

한 번 `true` 가 되면 되돌리지 않는다. 저장(`save`) 도 되돌리지 않는다 — 저장 후의 IR 은
여전히 "rhwp 가 재배치한 사다리" 를 갖고 있어 파일 그대로가 아니기 때문이다.

### 5.2 설정 지점

`commands/` 의 각 공개 편집 명령 진입부에서 `self.mark_edited()` 를 부른다. 더해서
4.4 의 내용 편집 2건(`set_section_def_all_native` · `set_field_value_by_name`)도 부른다.

명령 계층이 곧 편집 API 이므로 이 규칙은 **grep 가능하고 리뷰 가능**하다.

### 5.3 소비 지점

```rust
HeightMeasurer::new(dpi)
    .with_hwp3_variant(..)
    .with_stored_ladder_authoritative(!core.edited_since_load)
```

- 편집 전(파일 그대로): 저장 사다리가 권위 -> `#3746` 판별자 성립
- 편집 후: 사다리는 과거 기록 -> 종전대로 내용으로 성장

`with_hwp3_variant` 와 같은 주입 방식이라 새 배선 관례를 만들지 않는다.

## 6. 계약 — 양성보다 **음성**이 중요하다

테스트로 못박아야 할 것은 "편집하면 켜진다" 보다 **"렌더로는 안 켜진다"** 이다. 후자가
깨지면 `table.dirty` 와 같은 실패를 반복한다.

| # | 계약 | 왜 |
| --- | --- | --- |
| 1 | 로드 직후 `edited_since_load == false` | 기준 상태 |
| 2 | `paginate()` · `repaginate_if_needed()` · `render_page_svg()` 를 여러 번 불러도 `false` 유지 | **1급 제약** — 렌더 횟수 독립성 |
| 3 | 표시 옵션 변경(문단부호·조판부호·투명선)은 `false` 유지 | 내용이 아니다 |
| 4 | 글자 삽입 · 표 조작 · 개체 삽입은 `true` | 양성 |
| 5 | 한 번 `true` 면 이후 어떤 렌더로도 `false` 로 안 돌아감 | 단조성 |
| 6 | `save` 후에도 `true` 유지 | 저장본 사다리는 rhwp 가 쓴 것이다 |

계약 2·3·5 가 이 설계의 값어치를 지킨다.

## 7. 열린 질문

1. **저장·재로드 후는?** 재로드하면 `false` 로 시작한다. 그때 사다리는 rhwp 가 쓴
   것이지 한글이 쓴 것이 아니다. 판별자가 rhwp 저장본에서 어떻게 행동하는지는 별도
   측정이 필요하다. (`renderer_internal_fields_lost_on_roundtrip` 계열 계열 주의)
2. **부분 편집을 구역/표 단위로 좁힐 가치가 있는가?** 문서 하나만 고치면 문서 전체에서
   판별자가 꺼진다. 처음에는 문서 단위로 두고, 실측으로 손실이 크면 좁힌다.
3. **WASM/편집기 세션 경계.** 브라우저에서 문서를 열어 편집 중인 상태가 이 플래그로
   충분히 표현되는지 확인이 필요하다.

## 8. 범위와 규모

- 상태 1개 · 설정 지점 `commands/` 공개 명령 + 2 · 소비 지점 1 · 테스트 6
- 렌더 동작은 **플래그를 소비하는 판별자를 켜기 전까지 완전히 불변**이다. 즉 이 PR 은
  배선만 넣고 `#3746` 판별자는 후속 PR 로 분리할 수 있다 — 그러면 배선의 안전성을
  독립적으로 검증한 뒤 판별자를 얹게 된다. **그 분리를 권장한다.**

## 9. 되짚기 — 무엇이 이 축을 오래 막았나

`#2148` · `#3746` 은 "같은 IR 형상, 문서별 반대 진실" 로 여러 번 환원 불가 판정을 받았다.
이번 조사의 결론은 **판별자가 IR 안에 없다**는 것이다. 필요한 정보(파일 그대로인가)는
IR 이 아니라 **세션 상태**에 있고, 그래서 IR 을 아무리 들여다봐도 안 갈렸다.

관련: [편집 액션 실행 취소 아키텍처](edit_action_undo_redo_architecture.md),
[Document IR LineSeg 표준](document_ir_lineseg_standard.md)
