# Task #2824 Report — HWP3 그리기 개체 타입9(변형된 호) 8바이트 오버리드 수정

## 요약

`src/parser/hwp3/drawing.rs`의 `Hwp3DrawingObject::read`에서 그리기 개체
`object_type == 9`(변형된 호, 회전을 위해 확장된 호)를 처리할 때, 스펙에 없는
8바이트(`info1_len`, `info2_len`)를 읽어서 버리고 있었다. `mydocs/tech/한글문서파일구조3.0.md`
11.3.4절은 "추가 세부정보를 필요로 하지 않으며, 회전 속성의 평행사변형 세 점을 이용해
첫 점에서 끝 점 방향으로 그린다"고 명시하고, 열거형 정의(`Hwp3DrawingObject::ModifiedArc`)의
기존 주석도 "공통 헤더 외에 추가적인 세부 정보 없음"이라 밝히고 있었음에도, 실제 파서 구현은
이와 모순되게 사각형/타원(타입 2/3)에만 해당하는 "8바이트 placeholder" 관례를 잘못 차용해
스트림 커서를 8바이트 더 전진시키고 있었다.

## 근거

- 스펙: `mydocs/tech/한글문서파일구조3.0.md` 11.3.4절(변형된 호) — 추가 세부정보 없음.
- 대조: 11.3.8절(표 80, 변형된 타원)은 정보1길이(4)+호 좌표 4개(16)+정보2길이(4) = 24바이트를
  전부 읽고 보존하는데(`Hwp3DrawingModifiedEllipse`), 변형된 호는 그 자체로 세부 구조가 없다.
- 코드 자체 모순: 수정 전 `Hwp3DrawingObject` 열거형 주석 "`ModifiedArc(...) // 공통 헤더 외에
  추가적인 세부 정보 없음`"과, `read()` 구현이 실제로는 8바이트를 읽는 동작이 불일치했다.

## 수정 내용

`src/parser/hwp3/drawing.rs`의 `object_type == 9` 분기에서 `_info1_len`/`_info2_len` 읽기
2줄을 제거하고, 왜 추가 바이트를 읽지 않는지 스펙 근거를 주석으로 남겼다. 파일 전체를
`rustfmt --edition 2021`로 재포맷했다(변경 없음, diff 없음 확인).

## 검증 (red → green)

`src/parser/hwp3/drawing.rs`에 `modified_arc_overread_tests` 모듈을 추가했다. 공통 헤더만
있고 추가 필드가 없는 최소 buffer(92바이트, `options=0`으로 회전/그라데이션/비트맵 패턴
서브레코드 없음)를 만들고, 그 뒤에 "다음 형제 레코드의 선두 바이트"를 흉내 낸 8바이트
마커를 이어 붙인 뒤 `Hwp3DrawingObject::read`를 호출해 커서 위치가 정확히 공통 헤더
길이(92)에서 멈추는지 확인한다.

- 수정 전(8바이트 오버리드 코드 상태): `cargo test --lib modified_arc_does_not_overread_past_common_header`
  → **FAILED** (`left: 100, right: 92` — 8바이트 초과 소비 확인).
- 수정 후: 같은 테스트 **ok** (`test result: ok. 1 passed`).
- `cargo build --lib` 통과.
- `cargo clippy --all-targets --profile release-test -- -D warnings` 경고 없음.
- `rustfmt --edition 2021 src/parser/hwp3/drawing.rs` 이후 `git diff --name-only` 에 포맷
  전용 변경 없음(수정한 파일만 diff에 존재).

## 영향

변형된 호(타원 부채꼴을 회전시켜 그리는 호) 그리기 개체가 포함된 HWP 3.0 문서에서, 해당
개체 뒤에 이어지는 형제/자식 레코드가 8바이트 밀린 채로 파싱되어 헤더 필드가 오염되거나,
파일 말미에서는 `read_exact` 실패로 조기 종료될 수 있는 문제를 제거했다.

## 후속

- 다른 그리기 개체 타입(2, 3 등)의 "8바이트 placeholder" 자체는 스펙 표 73에 명시된 대로
  유지했으며, 이번 수정 대상에서 제외했다.
