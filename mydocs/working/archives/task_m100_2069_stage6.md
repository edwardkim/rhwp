# Task M100 #2069 Stage 6: OLE 개체 속성 캡션 설정 반영

## 목표

`samples/한셀OLE.hwp`와 `samples/한셀OLE.hwpx`에서 OLE 개체 속성 대화상자의 `여백/캡션` 탭에서 캡션 위치, 크기, 개체와의 간격, 여백 확장 값을 설정하면 실제 문서 모델에 반영되도록 한다.

## 원인

Studio 대화상자는 OLE 개체에서도 캡션 버튼과 입력값을 활성화하고, `setShapeProperties`에 `hasCaption`, `captionDirection`, `captionVertAlign`, `captionWidth`, `captionSpacing`, `captionIncludeMargin` 값을 전달하고 있었다.

하지만 Rust `getShapeProperties`/`setShapeProperties`의 Shape 공통 경로는 OLE를 포함한 `ShapeObject` 캡션 필드를 JSON으로 노출하거나 적용하지 않았다. 그래서 대화상자에서 설정을 눌러도 전달된 캡션 속성이 문서 모델의 `Ole.caption`에 저장되지 않았다.

## 수정

- `ShapeObject` 공통 캡션 참조/가변 참조 helper를 추가해 drawing shape, group, picture, chart, OLE 모두 같은 경로에서 캡션을 다룬다.
- `getShapeProperties` 응답에 `hasCaption`, `captionDirection`, `captionVertAlign`, `captionWidth`, `captionSpacing`, `captionMaxWidth`, `captionIncludeMargin`을 포함한다.
- `setShapeProperties`에서 `hasCaption=true`가 전달되면 없는 캡션을 생성하고, 기본 그림 자동 번호 문단을 포함해 한컴의 개체 캡션 구조에 맞춘다.
- 기존 picture 속성 경로와 동일하게 `hasCaption=false`는 삭제로 처리하지 않고 보존한다.

## 검증 계획

- Rust 통합 테스트에서 HWP/HWPX OLE의 캡션 속성 roundtrip을 확인한다.
- Studio E2E에서 실제 WASM 브리지를 통해 `한셀OLE.hwp`를 로드하고, 30mm/3mm 오른쪽 아래 캡션 설정이 다시 `getShapeProperties`로 읽히는지 확인한다.
- 기존 OLE 선택, Enter, Backspace 재진입 회귀 테스트와 함께 실행한다.
