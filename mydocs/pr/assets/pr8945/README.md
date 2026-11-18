# PR #8945 메인테이너 검증 자료

[검토 기록](../../archives/pr_8945_review.md). 원 기여 head `06f47c8a51`, base `99b9d7ec5c`,
실제 로컬 병합 source `e98d884fc3`의 관계는 [source 원장](source-association.json)에 있다.
제품 코드·검사·원본·기준 PDF 보정은 하지 않았다. 공개 승인 리뷰·push·merge는 별도다.

- [집중 검사의 실제 명령·결과](focused-results.json): 최소 글꼴 14 PASS.
- [수행한 검토 계획](review-plan.md).
- [실제 노출된 최소 글꼴](exposed-minimal-fonts.txt), [Fontconfig](minimal-fonts.conf),
  [격리 명령 prefix](minimal-font-prefix.json).
- [필수 편람 gate](handbook-gate/required-handbook-8153-gate.json): HWPX Native 383쪽 PASS.
- [목표 문서 전쪽 요약](focus/summary.json): 후보 Native/WASM 8쪽씩 raw 100.00%.
- [Native 직접 비교](focus/native-review/), [fresh WASM 직접 비교](focus/wasm-review/).
- [새 원본·PDF의 해시](new-inputs.json), [기존 수용 HFT 프로그램 7종의 대조](original-font-programs.json),
  [공개 편람 글꼴 공급](handbook-font-supply.json). 글꼴 프로그램은 포함하지 않는다.
- [sample10의 연결 그림 3개](linked-images.json): Native와 같은 원본 바이트를 WASM에도 공급한다.

각 `focus/<base|head>-<native|wasm>/<key>/silhouette.tsv`는 실행 원본이다. DPI는 96,
profile은 print, compat는 2022다. pypdfium2/PDFium RGB 래스터와 색상 경계 보정 전
2px 실루엣을 비교했다. 100.00%를 전체 RGB 픽셀 동등성으로 해석하지 않는다.

직접 비교 PNG는 같은 배율의 출력·독립 PDF를 같은 사각 영역에서 잘라 나란히 배치했다.
행/본문이 모두 보이도록 흰 여백만 줄였으며 내용 위치·모양을 변형하지 않았다.
짧은 영역은 양쪽을 같은 2배로 확대했다. `manifest.json`에 원 PNG 경로·crop·배율이 있다.
`b-full-review_*.png`는 정본 Visual Sweep이 생성한 전쪽 review 원본이다.

최적화 fresh WASM은 Rust release와 `wasm-opt -O`를 실제 완료한 frozen package이며,
브라우저는 Chromium 152 / revision 1660786이다. Native는 실제 Skia PNG다.
producer별 실행 파일·WASM·도구 SHA-256과 실제 build receipt는 source 원장에 포함돼 있다.

[보호 문서 전쪽 대조](protection/README.md)는 base/후보 3,542쌍의 PNG가 모두 같지만,
과거 정본 값의 미재현 911행을 별도로 기록한다. 이 특별 기준은 통과로 표시하지 않는다.
[원격 처리안](remote-action-plan.md)은 이 기존 차이의 PR 한정 결정과 원격 승인을 전제로 한다.
원시 실행 로그와 대형 중간 산출물은 ignored `output/`에만 두며 이 디렉터리에 commit하지 않는다.

[병합·duration 결과](post-merge-record.json). 원 PR merge는 `4ab860651b857bfe6d28b22b4565b2855bebb042`이다.
[독립 메인테이너 전쪽 review·overlay](focus/complete-visual/manifest.json)는 기존의 검증된 PNG에서 정본 함수로 생성했다.
