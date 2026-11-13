# 승인 후 추가 devel 병합 검증

제출 직전 base가 `9e2794796787b46815462fcecd1536ab74ac8528`로 전진했다. 후보 `0a3a2e4c41d10aabbc4da9b75b71d36c37f8f060`의 제품·테스트는 필수 검증 source `d66d5f05f9`와 같으며, 오늘할일의 이번 기록 위치만 옮겨 문서 충돌을 해소했다. 후보에 base를 merge/rebase하지 않고 실제 merge tree `1f4cb63a51e74dd35d8a0cb01801399cb8f5aef2`를 detached source `8136352a7121608e52068d61c31192d7911e67a2`에서 검증했다. 공개 base와 후보로 재구성할 수 있다.

- 최소 글꼴 Rust 집중 49건: PDF CMap 범위 10건·그림 셀 프레임 7건·HFT 글꼴 23건·이번 표 이월 9건을 통과했다.
- 호스트 글꼴 Node 검사 40건을 통과했다.
- 원본·BodyX 모델 4쪽과 새 그림 mixed/inline 2쪽의 Native 전쪽 raw는 모두 100.00%다. 원본·편집본 PNG 4쌍은 이전 판독 source와 RGB가 같다. 그림 2쪽은 같은 입력의 독립 Print와 직접 판독해 숫자·전체 프레임·누락 여부를 확인했다.
- 새 picture helper는 그림 전용 셀만 수용하며 이번 제목/본문 표와 저장 줄 경계를 대체하지 않는다. CMap은 PDF 입력 경로, 호스트 글꼴은 명시적 공급·선택과 수명 경로다. 기본 글꼴·비활성 동작과 CFF/TTC 계약을 실제 집중 검사로 확인했다.
- 이번 추가 확인을 전체 회귀·보호 3,542쪽·fresh WASM의 재실행으로 표시하지 않는다. 고정 후보의 기존 필수 검증과 추가 병합 검증을 구분한다.

[실제 source·명령·검사 수·RGB 대조](results.json), [원본](original.tsv), [편집본](edited.tsv), [mixed](new-picture-mixed.tsv), [inline](new-picture-inline.tsv).
