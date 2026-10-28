# task_m100_2563 처리결과 보고서 — 도형 이미지 채우기 왕복 정합

- **이슈**: [#2563](https://github.com/edwardkim/rhwp/issues/2563)
- **브랜치**: `task/m100-shape-imgbrush` (base `devel` @ `3c54abfd`)
- **범위**: `src/parser/hwpx/section.rs` `parse_shape_fill_brush`
- **분류**: 결함 수정 (이미지 도형이 빈 도형으로 왕복)

## 1. 문제

도형(`<hp:rect>` 등)의 `<hc:imgBrush>` 파서가 헤더(borderFill) 파서보다 얕았다.
**같은 파일·같은 IR** 인데 한쪽만 온전한 비대칭이다.

### 결함 1 — `<hc:img>` 자식 미처리로 그림 참조 유실

`b"imgBrush"` arm 이 `mode` 만 읽고 `<hc:img>` 자식 arm 이 없어
`binaryItemIDRef`·`bright`·`contrast`·`effect` 가 전부 버려졌다.

직렬화(`serializer/hwpx/shape.rs:837-877`)는 이 값들을 방출할 능력이 있지만
`img.bin_data_id` 가 항상 0 이라 `resolve_bin_id` 가 `None` 을 반환해
`<hc:imgBrush mode="…"/>` 만 나갔다. → **이미지로 채운 도형이 왕복 후 빈 도형**.

serializer 주석(`shape.rs:846-848`)이 이 결함을 이미 지목하고 있었다
("body shape 의 fill 파서가 bin_data_id 미캡처").

### 결함 2 — 채우기 모드 12종 중 8종이 TILE 로 붕괴

도형 파서는 4종만 매핑하고 나머지는 `_ => TileAll`. 헤더 파서
(`header.rs:1490-1503`)는 `TOTAL`, `TILE_HORZ_*`, `TILE_VERT_*`, `CENTER_*`,
`TOP_LEFT_ALIGN` 까지 전부 매핑한다. → `mode="TOTAL"`(늘여서 채우기)이 `TILE` 로.

## 2. 분석 — 같은 코드베이스의 선례를 그대로 이식

새 규약을 만들지 않았다. 헤더 파서가 이미 정답을 갖고 있어 그 매핑과 `b"img"` arm
처리를 도형 파서에 맞췄다. `b"color"` arm 이 "부모 뒤에 오는 자식" 처리 패턴을
이미 보여주고 있어 구조도 동일하다.

IR 근거: `Fill.image: Option<ImageFill>` → `ImageFill { bin_data_id, brightness,
contrast, effect, fill_mode }`(`model/style.rs:655-666`). 네 필드 모두 존재하며
헤더 경로에서는 정상 채워진다 — **도형 경로만의 결함**임이 증명된다.

## 3. 변경

`parse_shape_fill_brush` 한 함수:
1. `mode` 매핑을 헤더와 동일한 12종으로 확장
2. `b"img" | b"image"` arm 추가(헤더의 동명 arm 과 동형)

## 4. 검증

### red→green 실증

`test_shape_img_brush_preserves_image_ref_and_mode` 추가 —
`<hc:imgBrush mode="TOTAL"><hc:img binaryItemIDRef="image3" bright="10"
contrast="-5" effect="GRAY_SCALE"/>` 를 파싱해 5개 필드를 단언.

- `b"img"` arm 을 비활성화하면 → **FAILED** (`bin_data_id` left: 0, right: 3)
- 복원하면 → **passed**

### 회귀

`cargo test --lib parser::hwpx` 통과.

### 미실행 항목 (투명 고지)

- **왕복 직렬화 단언 미포함** — 파싱이 IR 을 채우면 직렬화는 기존 코드가 그대로
  방출하므로(그 경로는 이미 구현돼 있고 주석이 파서만 지목했다), red→green 이
  갈리는 지점인 파싱을 단언하는 데 그쳤다. 직렬화 왕복까지 원하시면 추가하겠다.
- **시각 검증 미실행** — 이미지 채우기 렌더 결과 비교는 visual sweep 이 필요하다.
  저장소 규약상 작업지시자 승인 사항이라 실행하지 않았다.
