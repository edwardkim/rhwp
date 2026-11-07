# #8733 단색 배경 저장 검증 입력

이 디렉터리는 합성 문서와 Studio 대화상자 증적이다. 신고자의 원 문서를 재배포하지 않으며, 한컴 Print PDF나 독립 한컴 렌더링 결과는 포함하지 않는다.

- 기준 devel: `b8dd67c5001dfb0410e9e192528676396cdd6a4e`
- 검증한 소스: `e551fcc76e0794790e25a565f2509a4891eac6cb`
- 브라우저: Chrome 155, Rust 1.93.1 / wasm-pack 0.15.0의 optimized release WASM(`wasm-opt -O`). 실제 응답 WASM 해시를 산출물과 대조했다.
- 전체 결과와 파일별 SHA-256: [validation-summary.json](validation-summary.json)

## 입력과 재현

1. `cell-new-solid.*`, `table-new-solid.*`: 새 셀/표 단색 배경의 무늬 없음은 `-1`이다. HWP와 HWPX 저장 후 다시 열어 같은 값과 색상을 확인했다.
2. `imported-raw-zero.hwp`: 기존 가로 무늬 `0`을 합성한 대조군이다. 대화상자를 열어 테두리만 바꾸면 원본 무늬와 색상이 유지된다.
3. `cell-cleared-zero.hwp`, `table-cleared-zero.hwp`: 위 대조군에서 무늬 없음을 명시적으로 선택한 결과이며 `-1`로 저장된다. 실제 undo/redo도 확인했다.
4. PNG는 해당 Studio 대화상자의 미리보기다. 한컴 출력 또는 Native/Canvas의 전체 페이지 비교로 해석하지 않는다.

## 범위와 제한

수정 대상은 편집 속성의 기본값·부분 변경 보존과 직렬화 계약이다. compositor, 크기 계산, 페이지 배치, paint backend는 변경하지 않았다. HWP 원시 BorderFill 바이트와 HWP/HWPX roundtrip을 별도 Rust 회귀로 확인했다.

로컬 한컴 2020(11.0.0.9166)을 확인했으나 앱 실행 승인이 시간 초과되어 파일을 열거나 Print PDF를 만들지 못했다. 보안 우회나 원격 업로드는 하지 않았다. 한컴 직접 비교는 미검증이며, 앱 접근 승인 후 위 입력을 사용해 이어서 확인해야 한다.

양수 무늬 번호의 전면 매핑 변경과 이미 잘못 저장된 문서의 자동 복구는 포함하지 않는다. 너비·줄 간격 문제의 기존 #7608 수정은 유지하고 #7606 회귀 3건을 재검증했다.
