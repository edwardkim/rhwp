# task-m100-3146: HML CHARSHAPE/PARASHAPE 자식 요소 last_mut 귀속 결함 수정

## 이슈

#3146 HML 파서에서 CHARSHAPE/PARASHAPE 본체는 `Id` 속성 위치에 배치(set_indexed)되는데,
자식 요소(FONTID/RATIO/CHARSPACING/RELSIZE/CHAROFFSET, PARAMARGIN/PARABORDER)는
`last_mut()` 로 "마지막 원소"에 귀속된다. 두 배치 기준이 어긋나 있어:

1. `Id` 가 내림차순(비순차)으로 들어오면 자식 값이 엉뚱한 도형을 덮어쓰고,
   원래 도형은 기본값(`[0; 7]` 장평 등)으로 방치된다.
2. #2743 이 도입한 리소스 Id 상한 초과 건너뜀 경로에서는, 건너뛴 도형의 자식이
   직전 정상 도형을 오염시킨다 — BORDERFILL 에는 있는 "자식 폐기" 처리가
   CHARSHAPE/PARASHAPE 에는 빠져 있었다.

## 원인 분석

`src/parser/hml/reader.rs`:

- CHARSHAPE/PARASHAPE 시작 태그: `set_indexed(id, ...)` 로 Id 위치에 삽입.
- 자식 요소(RATIO 등): `char_shapes.last_mut()` / `para_shapes.last_mut()` 로 귀속.
- `set_indexed` 는 Id 가 현재 길이보다 작으면 기존 슬롯을 교체하므로,
  Id 내림차순 입력에서 "마지막 원소"와 "방금 삽입한 원소"가 달라진다.
- 상한 초과 건너뜀 시에도 `last_mut()` 는 직전 정상 도형을 가리켜 오염이 발생한다.
  (BORDERFILL 은 `current_border_fill: Option<usize>` 로 이미 방어하고 있었다.)

## 수정 내용

BORDERFILL 의 `current_border_fill` 패턴과 동일하게:

- `current_char_shape: Option<usize>` / `current_para_shape: Option<usize>` 필드 추가.
- CHARSHAPE/PARASHAPE 시작 시 삽입 인덱스를 기록, 상한 초과 건너뜀 시 `None`.
- 자식 요소는 `last_mut()` 대신 기록된 인덱스로 귀속, `None` 이면 폐기.
- 요소 종료 시 인덱스 해제.

## red→green 결과

재현 테스트 3건 추가 (`tests/issue_hml_shape_child_attribution.rs`), 수정 전 전부 FAIL 확인:

- `charshape_children_follow_out_of_order_ids` — Id=1,0 순 입력에서 RATIO 교차 오염
  (수정 전: `Id=1 의 RATIO 는 90 이어야 한다` assert 실패, 실제 `[0; 7]`)
- `skipped_charshape_children_do_not_pollute_previous_shape` — 상한 초과 건너뜀 도형의
  RATIO 가 직전 도형 덮어씀 (수정 전: `[100; 7]` 기대, 실제 `[10; 7]`)
- `parashape_margin_follows_out_of_order_ids` — PARAMARGIN 교차 오염
  (수정 전: `Id=1 의 PARAMARGIN Left` 700 기대, 실제 0)

수정 후 3건 전부 PASS.

## 검증 명령

```
cargo test --test issue_hml_shape_child_attribution
```

3 passed. 기존 HML 파서 테스트 회귀 없음 (`cargo test hml`).

## 관련 코드

- `src/parser/hml/reader.rs` — 귀속 인덱스 추적 (+34/-3)
- `tests/issue_hml_shape_child_attribution.rs` — 재현 테스트 3건 (신규)
- 기준 커밋: upstream/devel `49469c1f`
