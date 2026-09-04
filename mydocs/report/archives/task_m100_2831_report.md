# 완료 보고서 — Task M100-2831

- 이슈: #2831
- 제목: HWP3 하이퍼텍스트 정보의 책갈피 필드가 16바이트 언더리드되어 후속 그리기 개체 파싱이 어긋남
- 작성일: 2026-07-22
- 브랜치: `task/m100-2831-hypertext-bookmark-underread`

## 1. 배경

직전 태스크(#2824, 변형된 호 오버리드)와 같은 스크루티니를 `Hwp3DrawingObject::read`의
나머지 `object_type` 분기(Line/Rectangle/Ellipse/Arc/Polygon/TextBox/Curve/
ModifiedEllipse/ExtendedCurve/ClosedPolygon)와 `src/parser/hwp3/**`의 형제 레코드
리더에도 적용해, 각 분기의 바이트 소비량이 스펙 인용과 정확히 일치하는지 재검증했다.

## 2. 검증 결과

- `Hwp3DrawingObject::read`의 object_type 0~11 각 분기: `mydocs/tech/한글문서파일구조3.0.md`
  §11.3.1~11.3.10(공통 헤더, 선/호/다각형/글상자/곡선/변형된 타원/확장·닫힌 다각형 세부 정보)과
  바이트 단위로 대조한 결과 전부 일치했다. 특히 공통 헤더의 기본 속성(44바이트)/회전
  속성(32바이트)/그라데이션 속성(28바이트)/비트맵 패턴 속성(278바이트)도 표 69~72와
  정확히 일치함을 확인했다.
- 그 과정에서 `Hwp3DrawingObjectHypertextInfo::read`(drawing.rs:49-76)의
  `jump_bookmark` 필드가 **16바이트만 읽는** 것을 발견했다. 코드 주석조차
  "16 hchar(32바이트)로 명시되어 있으나 오프셋 계산상 16바이트로 처리함"이라고
  스스로 모순을 인정하고 있었다.
- 스펙 §8.3 "하이퍼텍스트(HyperLink) 정보" 표 21은 이 구조체와 동일한 필드 구성을
  절대 오프셋과 함께 별도로 명시한다: 책갈피 필드는 오프셋 264→296 사이(32바이트)이며,
  전체 길이 공식 `617 = 256(파일이름) + 32(책갈피) + 325(매크로) + 1(종류) + 3(예약)`도
  32바이트를 요구한다. 16바이트로는 601바이트가 되어 공식과 16바이트 어긋난다.
- `parse_drawing_object_tree`(drawing.rs:581-587)는 `frame_header.header_length > 24`일
  때 이 구조체를 `seek` 없이 순차 `read_exact`로 읽으므로, 16바이트 언더리드는 하이퍼링크가
  걸린 그리기 개체를 포함한 HWP3 문서에서 이후 모든 형제/자식 그리기 개체 레코드의 파싱을
  통째로 밀어버린다(#2824와 동일한 버그 클래스).

## 3. 수정 내용

- `src/parser/hwp3/drawing.rs`
  - `jump_bookmark_buf`를 `[0u8; 16]` → `[0u8; 32]`로 수정
  - 구조체 필드 주석과 read 함수 내 주석을 스펙 §8.3 표 21의 절대 오프셋·전체 길이
    공식 근거로 갱신
  - 회귀 테스트 `hypertext_bookmark_underread_tests::hypertext_info_consumes_full_32_byte_bookmark`
    추가: 하이퍼텍스트 정보 뒤에 마커 바이트를 두고, 32바이트 소비 후 정확히 마커
    위치에 커서가 도달하는지 확인. 수정 전 코드로 되돌려 실행하면
    `left: 605, right: 621`로 16바이트 차이가 나며 실패하는 것을 확인했다(red).
    수정 후에는 통과한다(green).

## 4. 검증 결과

통과:

- `cargo build --lib`
- `cargo test --lib hypertext_bookmark_underread_tests`
  (수정 전 코드로 되돌려 재실행 시 실패 확인 → red→green 검증 완료)
- `cargo clippy --all-targets --profile release-test -- -D warnings`
- `rustfmt --edition 2021 src/parser/hwp3/drawing.rs` (변경된 파일만 대상, 이후
  `git diff --name-only` 결과 해당 파일만 존재)

## 5. 그 외 분기에 대한 결론

Line(1), Rectangle(2), Ellipse(3), Arc(4), Polygon(5), TextBox(6), Curve(7),
ModifiedEllipse(8), ExtendedCurve(10), ClosedPolygon(11) 분기와 공통 헤더 하위
속성 리더는 모두 스펙과 바이트 단위로 일치하여 추가 수정이 필요하지 않았다.
