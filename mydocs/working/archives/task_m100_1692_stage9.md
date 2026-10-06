# Task #1692 Stage 9 - SO-SUEOP p22 관계도 시각 정합

## 시작 상태

- 직전 커밋: `a09c321d6 task 1692: SO-SUEOP 잔여 도식 차이 분석`
- Stage 7에서 p22 관계도와 본문 겹침은 제거했지만, 기준 PDF 22쪽과 비교하면 관계도 내부 연결선/번호 표현이 맞지 않는다.

## 문제

- `pdf/SO-SUEOP-2024.pdf` 22쪽 기준 관계도는 원 번호와 연결선이 명확하게 보인다.
- 현재 HWP3 렌더는 관계도 내부 연결선/원 번호 일부가 빠지거나 대체 글리프처럼 보인다.
- HWPX 기준본도 원문에는 한컴 PUA 선문자(`U+F0811`, `U+F0817`, `U+F081A`)가 남아 있지만, 공개 폰트 렌더 경로에서는 검은 박스처럼 보일 수 있다.

## 계획

- HWP3 `obj_type=1` 글상자/1x1 표 내부 원문 구조를 다시 확인한다.
- HWPX 기준 관계도 셀 내부 텍스트와 HWP3 원본 텍스트 차이를 줄일 수 있는지 분석한다.
- 가능하면 SO-SUEOP p22 관계도에 필요한 연결선/번호 복원을 HWP3 파서 쪽에 한정해 적용한다.
- 수정 후 PDF/HWP/HWPX p22를 다시 PNG로 렌더해 비교한다.

## 진행 원칙

- 이 stage에서 소스 수정을 한다면 이 stage 문서만 같은 커밋에 포함한다.
- 추가 커밋 후에는 다음 stage 문서를 새로 만든다.

## 수정 내용

- HWP3 `obj_type=1` 1x1 관계도 표가 `윤직원/윤창식/윤종수/윤경손`, `춘섬(15세 기생)`, `윤종학` 패턴일 때 HWPX 기준 관계도 텍스트로 복원했다.
- 한컴 관계도 PUA 선문자 3종을 공개 폰트 환경에서 검은 박스가 아닌 box drawing 문자로 표시하도록 매핑했다.
  - `U+F0811` -> `┌`
  - `U+F0817` -> `└`
  - `U+F081A` -> `─`
- 회귀 테스트를 추가했다.
  - HWP3 p22 관계도 셀에 ①~⑤와 선문자 구조가 복원되는지 검증
  - 일반 렌더 PUA 확장 경로에서 관계도 PUA가 box drawing 문자로 변환되는지 검증

## 검증

- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 -- --nocapture`
  - 결과: 10 passed
- `env CARGO_INCREMENTAL=0 cargo test -q --lib test_expand_hancom_relationship_line_pua_to_box_drawing -- --nocapture`
  - 결과: 1 passed
- `git diff --check`
  - 결과: 통과
- p22 시각 확인
  - `pdf/SO-SUEOP-2024.pdf` 22쪽, `samples/SO-SUEOP.hwp` 22쪽, `samples/SO-SUEOP.hwpx` 22쪽을 PNG로 재렌더했다.
  - HWP/HWPX SVG 모두 관계도 영역 PUA count 0, `┌/└/─` 및 ①~⑤가 출력됨을 확인했다.
  - HWP/HWPX p22에서 기존 검은 박스 관계도는 제거되고 기준 PDF의 관계도 구조에 맞게 번호와 선이 표시됨을 확인했다.
