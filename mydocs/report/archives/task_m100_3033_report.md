# 완료 보고서 — Task M100-3033

- 이슈: #3033
- 제목: [hwpx] hh:bullet 이미지 글머리표가 실제 binaryItemIDRef 를 읽지 않고 항상 ID 1로 고정됨
- 작성일: 2026-07-22
- 브랜치: `task/m100-3031-bullet-useimage`

## 1. 완료 내용

`src/parser/hwpx/header.rs`의 `parse_bullet_hwpx()`가 `<hh:bullet>` 자식 요소를
전부 건너뛰던 것을, `<hh:img binaryItemIDRef="imageN" .../>`만 인식해 숫자 ID를
추출하도록 수정했다. HWP3 BULLET record(표 44)의 `image_bullet` 필드는
"0=문자, ID=이미지"로 정의되어 있어, 이미지 글머리표는 실제 참조 ID가 보존되어야
한다. 기존에는 `useImage="1"` 여부만 보고 상수 `1`을 대입해, 서로 다른 이미지를
참조하는 여러 글머리표가 전부 같은 BinData ID로 뭉개졌다.

`src/parser/hwpx/section.rs`에 이미 있던 `binaryItemIDRef` → 숫자 ID 추출
패턴(`val.chars().filter(|c| c.is_ascii_digit())`)을 그대로 재사용했다.

## 2. 주요 변경

- `src/parser/hwpx/header.rs`
  - `parse_bullet_hwpx()` 자식 순회 루프에 `<hh:img>` 매치 분기 추가,
    `binaryItemIDRef`를 파싱해 `bullet.image_bullet`에 대입
  - 단위 테스트 `test_parse_bullet_hwpx_image_id_from_binary_item_id_ref` 추가
    (`binaryItemIDRef="image3"` → `image_bullet == 3` 확인)

## 3. 검증 결과

통과:

- `cargo check --lib`
- `cargo test --lib bullet` (관련 4개 테스트 전부 통과, 신규 테스트 포함)
- `rustfmt --edition 2021 src/parser/hwpx/header.rs`

## 4. 남은 이슈

없음. `image_data`(대비/밝기/효과 4바이트)는 여전히 HWPX에서 채워지지 않으나,
이는 이번 이슈(binaryItemIDRef 자체 유실)와 별개의 후속 개선 대상이다.
