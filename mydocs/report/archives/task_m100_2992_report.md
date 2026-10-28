# Task M100-2992 완료 보고서 — EMF POINTS16 파싱 unbounded allocation 수정

## 이슈

- 원본 이슈: [#2992](https://github.com/edwardkim/rhwp/issues/2992)
- 분류: 파일에서 읽은 검증되지 않은 count를 `Vec::with_capacity`에 그대로
  넘겨 거대 할당을 시도하는 클래스(`#2722`, `#2751`류와 동일)

## 문제

`src/emf/parser/records/drawing.rs`의 `parse_points16`은 EMR_POLYLINE16,
EMR_POLYGON16, EMR_POLYBEZIER16 등 POINTS16 계열 EMF 레코드에서 공통으로
사용되는 파서다. 여기서 포인트 개수 `count`를 EMF 레코드 바이트에서 그대로
읽어 `Vec::with_capacity(count)`에 넘기고 있었는데, 이 값은 검증되지 않은
u32라 조작된 값(예: `0xFFFFFFFF`)이 들어오면 실제 레코드에 남은 바이트와
무관하게 최대 약 17GB(`u32::MAX × 4바이트/포인트`)의 예약 할당을 시도한다.

HWP/HWPX 문서에 EMF 이미지가 삽입돼 있으면 렌더링/변환 경로에서 이 파서가
호출되므로, 조작된 EMF를 포함한 문서를 열기만 해도 트리거될 수 있는
서비스 거부(DoS) 성격의 문제였다.

## 원인

```rust
// 수정 전
let count = c.u32()? as usize;          // 검증 없는 파일 값
let mut pts = Vec::with_capacity(count); // count가 크면 거대 할당 시도
```

`Cursor`는 이미 `remaining()`을 제공하고 있어 다른 파서들
(`src/parser/doc_info.rs`의 `parse_tab_def`, `src/parser/control/shape.rs`의
커넥터 컨트롤 포인트 파싱)에서 이미 같은 패턴으로 방어하고 있던 것과
동일한 방식으로 상한을 씌울 수 있었다. POINTS16 한 개는 4바이트(u16 x +
u16 y)이므로 `count.min(remaining() / 4)`로 제한하면 된다.

## 수정 (red → green)

**Red**: `count = c.u32()? as usize;`(상한 없음) 상태에서
`emf::tests::polyline16_rejects_unbounded_point_count` 테스트를 실행하면,
악의적 count(`u32::MAX`)와 실제 데이터 0바이트로 구성한 EMR_POLYLINE16
레코드를 파싱할 때 `Vec::with_capacity`가 그대로 거대한 값을 예약하려
시도해, 남은 바이트가 0인데도 파싱이 실패(`UnexpectedEof`)하는
비정상적인 흐름이 관찰되었다(`recs[1]`이 아예 만들어지지 못함).

**Green**: `src/emf/parser/records/drawing.rs`의 `parse_points16`에서
`count`를 읽은 직후 `c.remaining() / 4`로 상한을 씌우도록 수정.

```rust
// 수정 후
let count = (c.u32()? as usize).min(c.remaining() / 4);
let mut pts = Vec::with_capacity(count);
```

정상적인 EMF 파일에서는 `count`가 항상 남은 바이트로 커버되는 값이므로
`min()`이 실질적인 영향을 주지 않아 기존 동작(라운드트립)은 그대로
유지된다.

## 테스트

`src/emf/tests.rs`에 `polyline16_rejects_unbounded_point_count` 추가:
- EMR_POLYLINE16 레코드에 count = `u32::MAX`, 실제 포인트 데이터는
  0바이트로 구성.
- 수정 전: `Vec::with_capacity(count)`가 상한 없이 그대로 거대 용량을
  예약하려 시도(실제 환경에서는 OOM abort로 이어질 수 있는 지점).
- 수정 후: `count`가 남은 바이트 기준(0/4=0)으로 상한이 걸려, 정상적으로
  `Ok`가 반환되고 `points`가 빈 벡터로 채워짐을 확인.

## 검증

- `cargo test --lib emf::tests::polyline16_rejects_unbounded_point_count`:
  수정 전 FAILED → 수정 후 ok 확인.
- `cargo check --lib`: 통과.
- `rustfmt --edition 2021 src/emf/parser/records/drawing.rs src/emf/tests.rs`:
  적용 완료.

## 변경 파일

- `src/emf/parser/records/drawing.rs` — `parse_points16`의 `count`에 남은
  바이트 기준 상한 적용 (3줄 순증가).
- `src/emf/tests.rs` — red→green 테스트 1개 추가.
