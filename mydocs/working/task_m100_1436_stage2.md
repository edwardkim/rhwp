# Task 1436 Stage 2

## 목적

Stage 1에서 Studio 속성 JSON으로 노출된 `sizeProtect`를 개체 속성 대화상자와 직접 조작 경로에 연결한다. 한컴오피스처럼 크기 고정이 켜진 개체는 크기, 회전, 기울기 입력과 마우스 핸들 조작을 비활성화하고, 설정 버튼을 눌렀을 때 `sizeProtect` 값을 저장한다.

## 범위

- 그림 개체 속성 대화상자의 `크기 고정` 체크박스 초기화/저장
- `크기 고정` on 상태에서 너비/높이 입력 비활성화
- `크기 고정` on 상태에서 회전각/기울기 입력 비활성화
- 체크박스 변경 시 대화상자 내 활성/비활성 상태 즉시 갱신
- `크기 고정` on 상태에서 마우스 리사이즈/회전 핸들 조작 차단
- `크기 고정` on 상태에서 Shift+방향키 크기 변경 차단
- 툴바 회전/대칭 명령이 크기 고정 개체를 변경하지 않도록 방어

## 제외

- 모달 외부 클릭 정책 정리

모달 외부 클릭 정책은 크기 고정과 별도 이슈로 분리한다.

## 검증 계획

- `npm --prefix rhwp-studio run build`
- `cargo test --test issue_1436_size_protect_properties -- --nocapture`
- `git diff --check`

## 수행 결과

- 그림 개체 속성 대화상자의 `크기 고정` 체크박스가 `sizeProtect`와 동기화되도록 연결했다.
- `크기 고정` on 상태에서는 너비/높이, 그림 확대/축소, 회전, 대칭, 기울기 입력을 비활성화했다.
- `크기 고정` on 상태에서는 마우스 리사이즈/회전 핸들 드래그 시작을 차단했다.
- 이미 드래그 상태가 시작된 예외 상황에서도 리사이즈/회전 적용 함수가 `sizeProtect`를 재확인해 변경을 중단하도록 방어했다.
- Shift+방향키 크기 변경과 툴바 회전/대칭 명령도 `sizeProtect` 개체에는 적용하지 않도록 막았다.

## 검증 결과

- `npm --prefix rhwp-studio run build`: 통과
- `cargo test --test issue_1436_size_protect_properties -- --nocapture`: 통과, 2 tests
- `git diff --check`: 통과

## 시각 판단 대기

- 사용자가 크기 고정 on 상태에서 회전 핸들/크기 핸들이 실제로 동작하지 않는지 Studio 화면에서 확인한다.
