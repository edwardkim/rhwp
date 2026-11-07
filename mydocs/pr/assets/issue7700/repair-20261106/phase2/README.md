# #7700 2단계 — 빈 호스트 줄과 조각 내용 하한

원본/PDF는 [기존 이슈 자산](../../delegation-rules/README.md)의 바이트를 그대로 사용했다.
한컴 2020 Print 전103쪽과 새 Native/fresh WASM 전103쪽을 96DPI·compat2022·정본 PDFium·2px raw로 비교했다.
Native 최저 94.75432%, WASM 최저 94.75587%, 쪽수 일치·누락0·90% 미만0이다.
Native는 고정 base170b 대비 페이지별 하락0이며, WASM base 전쪽 대조는 아직 진행 중이다.
이 단계의 통과를 전체 회귀·최종 PR 적격 또는 이슈 완료로 확대하지 않는다.

빈 호스트 줄은 저장 사다리가 뒤 inline 표의 시작을 인증할 때 현재 쪽에서 한 번 소비한다.
76쪽 후속 표 상단이 367.7→399.7px가 됐고, 독립 Print는 399.24268px다.
이어받기 종료 행을 다시 풀 때 선언과 공통 내용 컷의 실제 물리 요구 중 큰 값을 유지한다.
30쪽 BRM 칸의 24.88px 내용 성장분을 잃던 현상을 복구했다. 30–33쪽 Native는 각각
99.36627/99.60282/100/100%, WASM은 99.36595/99.60264/100/100%다.

Native Skia 라이브러리 전체4039PASS/0FAIL/12ignore, PNG2PASS, 직접PDF22PASS를
Mac 최소 글꼴 sandbox에서 순차 실행했다. 실제 명령·요약·로그 hash는 결과 JSON에 있다.
로그·임시 probe·중간 render tree는 ignored output에 두고 여기에 포함하지 않는다.

- [Native 전103쪽 TSV](native-all103.tsv) · [fresh WASM 전103쪽 TSV](wasm-all103.tsv)
- [Native30 review](native-p030-review.png) · [overlay](native-p030-overlay.png)
- [WASM30 review](wasm-p030-review.png) · [overlay](wasm-p030-overlay.png)
- [Native76 review](native-p076-review.png) · [overlay](native-p076-overlay.png)
- [WASM76 review](wasm-p076-review.png) · [overlay](wasm-p076-overlay.png)
- source·artifact 검증 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7700/repair-20261106/phase2/phase2-validation.json`)

정본 review/overlay를 직접 판독했다. BRM 다섯 줄·안여백·내부 괘선과 다음 행을 보존하고,
76쪽 후속 표·77쪽 부동 표의 소속과 호스트 중복 방지를 확인했다. 미세한 글리프 가장자리·
수평 간격 차이는 남으며 raw100%도 픽셀 전체 동일을 뜻하지 않는다. 좌표 이동·문서 ID 예외나
시각 문턱 변경을 넣지 않았다. 원문 일반 셀14166개와 컨트롤 셀5개의 텍스트 소속 대조는
모두 일치했지만 이 텍스트 감사가 실제 clip·컨트롤 동작 검증을 대신하지는 않는다.
