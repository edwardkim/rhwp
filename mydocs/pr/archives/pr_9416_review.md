# PR #9416 검토 · Word 호환 문단의 줄 높이와 형식 변환

hyunmyungJaneLee의 원 기여 `ae243b34c1744e68dfbe32ab2866148228b9a53b`와 메인테이너가 보완한 제품 소스 `9e601ad6a862a30b019ad9563a1d597f6a33222b`을 검토했다. 대조 기준은 `devel`의 `cbebdbeeca060efe670cb8702355e4d79cb7e70e`다. [원 PR](https://github.com/edwardkim/rhwp/pull/9416)

같은 `gulim.ttc`를 한글 이름과 영문 이름으로 지정한 한컴 인쇄본은 96 DPI에서 픽셀이 같았다. 세 원 기여만 병합한 Native에서는 한글 이름 63.37%, 영문 이름 99.54%였다. 보완본은 문서의 글꼴 슬롯과 이름이 가리키는 실제 글꼴을 따라 같은 줄 메트릭을 사용한다. 두 이름과 표 안 문단은 Native에서 각각 100.00%다. [글꼴과 인쇄본의 대조](../assets/hyunmyungjanelee-20261126/font-check.tsv)

HWP 5의 호환 대상·조판 플래그와 HWPX의 `compatibleDocument`를 양방향으로 보존한다. 표준 HWPX만으로 복원할 수 없는 플래그는 해당 호환 헤더에 결합해 보존하고, 저장 줄이 없는 비표준 Word 설정에서는 실제 배치한 본문 줄의 시작·높이·기준선·줄 간격을 HWPX에 기록한다. 외부 편집으로 호환 헤더를 바꾼 대조군도 검사했다. 한컴으로 저장·인쇄한 굴림 대조군에서 HWP 변환 뒤의 마지막 줄이 원래 위치로 돌아왔다. [재현·보완 근거](../hyunmyungjanelee_20261126_review_impl.md) · [한컴 재인쇄의 쪽별 결과](../assets/hyunmyungjanelee-20261126/final/desktop-pixels.tsv)

#8582에는 Word 탭 위치의 문제가 남는다. 이 PR의 처리 범위는 줄 높이와 호환 설정 보존이다. [연결 이슈](https://github.com/edwardkim/rhwp/issues/8582) · [통합 검증](../assets/hyunmyungjanelee-20261126/final/README.md)
