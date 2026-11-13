# 실제 글자 추가 후 화면·저장 검증

제품 source `d66d5f05f9efa4d182228494cd970614d479fb7a`, base `959300e66c58241a47e6d46030b4b0854e59604b`.
원 공개 입력의 Body 끝에 공개 `insertTextInCell(0,0,3,0,0,4,"X")`를 호출했다.
Native/fresh 최적화 WASM의 편집 직후 전 2쪽 raw는 각각 100.00%이며,
HWP/HWPX 각 두 번 저장·재열기도 2쪽·내용 순서를 유지한다. 별도 편집 예외는 적용하지 않았다.

독립 기준은 [편집 모델](../../input-generation/edited-body-small-model.hwp)을 실제 한컴 2020 11.0.0.9136에서
method 0, one-up으로 출력한 [전 2쪽 PDF](../../../../../../pdf/pr9042/edited-body-small-hwp-2020.pdf)다.
이 입력은 한컴 저장본이 아닌 RHWP의 공개 합성 편집 모델이다. 이전 제품 47d5b71f32에서 현재 캐시로
저장한 원본을 읽고 X를 추가한 후 live IR을 직접 직렬화했다. 그때 잘못된 표 높이 writeback을
모델 작성에 섞지 않았으며 실제 제품의 공개 저장 검증과 구분했다.
[실제로 사용한 생성기](../../input-generation/edited-source-generator.rs.txt)는 생성 당시의 경로를 보존한다.
최초 입력은 원 공개 문서의 HWP 저장본이며 최종 검토에서는 공개 저장 진입점도 별도로 실행한다.

글꼴은 양쪽에 같은 HCRDotum/HCRBatang 프로그램과 명시 환경을 공급했다.
초기 WASM 진단에서 누락된 프로그램 때문에 Noto가 사용된 결과는 이 완료 측정과 구분한다.
384 DPI 독립 재출력에서 End of body의 하강부·괘선과 Body end의 포함 관계를 직접 판독했다.
Native의 편집 직후/첫 저장 재열기 PNG 두 쌍은 RGB가 같다. 100.00%는 기준 PDF와의 RGB 동일을 뜻하지 않는다.

TSV는 [Native](native.tsv)·[WASM](wasm.tsv), 각 live PNG는 같은 디렉터리에 있다.
전체 회귀·보호 검증과 제출 판정은 상위 최종 원장을 따른다.

현재 source의 [snapshot 복원](history.json), [실제 384 DPI 원본과 crop](actual384-provenance.json), [동일 글꼴의 실제 WASM 로드](wasm-capture-provenance.json), [저장 byte 그룹과 검증](saved-visual-results.json)을 함께 보존한다. 초기 공급 누락을 수정한 최종 확대 이미지이며 과거 Noto 캡처와 섞지 않았다.
