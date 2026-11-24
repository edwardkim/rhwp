# PR #9377 HWPX 본문 저장 줄 좌표 검토

**접수 당시 판정은 보완 필요였다.** 원 PR이 다룬 문단 24의 저장 좌표는 복원됐다. 같은 문서의 문단 112에서는 한 글자를 넣었다 지우고 저장하면 rhwp가 한 쪽을 추가한다.

## 검토 범위

기여자 `semanticist21`, 리뷰어 `edwardkim`, 목표 브랜치 `devel`. 2026-11-24에 원 PR `b3301e1905edd7fe504fe400c3560e27c421ad33`와 기준 `2c0c1d205a54b8d5bd262241bac1391cffe0d801`을 확인했다. 단독 병합 tree는 `5f9f74cd4b536bb2623dd192db6679b1c384a41d`이다. 원 PR의 Build & Test와 CI Impact Policy는 성공했다. 연결된 자동 종료 이슈는 없다. 기존 #5847·#8310의 원본 줄 저장 계약과 `77984f4837`·`f6251b3639`·검증 기록 `4d08a990ca`를 대조했다. [원 PR](https://github.com/edwardkim/rhwp/pull/9377)

## 검증과 재현

- 기여자의 문단 24 회귀 검사가 통과했다. 메인테이너는 저장소 공개 원본 `samples/HWP5-nopassword-123456.hwpx`의 26개 문단에서 같은 입력을 독립적으로 반복했다. 같은 글자를 넣었다 지워 편집 후 본문 내용이 원래와 같은 상태에서 HWP·HWPX로 저장하고, 다시 연 문서의 각 본문 문단이 속한 쪽을 비교했다. [문단별 측정](../assets/semanticist21-9337-9354-9377/reopen-pages.tsv)
- 0부터 세는 문단 112에서 편집 중 24쪽, HWPX로 저장해 재열기 후 25쪽이었다. 231개 문단의 쪽 소속이 편집 시점과 달라졌다. 원래 첫 줄의 저장 세로 위치 3,768 HU가 −35,864 HU로 기록됐다. 문단 36에서도 원본 0 HU가 −85,920 HU가 됐다.
- 동일한 원본과 문단 24·36·112의 HWPX 저장본을 한컴 2024 13.0.0.3901에서 PDF 드라이버로 한 쪽씩 인쇄했다. 네 파일은 모두 24쪽이었다. 원본과 각 저장본의 24쪽 전체를 72 DPI로 래스터화하면 쪽별 픽셀이 모두 같다. `BodyLineFrame`의 앞 문단 보정량을 현재 문단의 쪽·단 좌표가 다시 시작하는 경계에서도 재사용하는 경로가 남아 있다.

보완 범위는 저장 좌표계가 바뀌는 쪽·단 경계에서 원본의 문단 소속을 지키고, 원본·편집 상태·재열기와 한컴 출력을 다시 대조하는 것이다.

## 메인테이너 보완 결과

HWPX의 저장 본문에 원본 쪽 좌표가 있으면, 편집으로 그 좌표의 근거가 사라진 문단에서는 선택적인 줄 캐시를 생략한다. 같은 구역의 손대지 않은 문단은 자기 원본의 좌표를 내보낸다. HWPX 구역 저장기와 저장 후의 문서 구조를 대조하는 검사에서 같은 판정을 사용한다.

원본의 서로 다른 26개 문단에서 한 글자를 넣었다가 지우고 HWP·HWPX 두 형식으로 다시 열었다. 52개 결과가 모두 24쪽, 본문 일치, 문단 소속 변경 0개였다. 원본과 문단 24·112의 최종 HWPX를 한컴 2024에서 다시 인쇄해 96 DPI로 대조했다. 세 인쇄본은 24쪽 모두 픽셀이 같다. 원본의 글자 모양이 지정한 한양신명조 등 HFT 글꼴 프로그램도 실제 공급 순서와 해시로 기록했다. [52개 편집·재열기 결과](../assets/semanticist21-9337-9354-9377/corrected-reopen.tsv) · [한컴 인쇄 24쪽의 픽셀 해시](../assets/semanticist21-9337-9354-9377/line-print-pixels-96dpi.tsv) · [글꼴 목록](../assets/semanticist21-9337-9354-9377/line-font-programs.tsv)

원본 및 문단 24·112 편집본 각각의 24쪽을 한컴과 96 DPI로 비교했다. 전쪽 실루엣 최저는 각각 Native 99.09%, fresh WASM 99.14%였다. 제목과 본문 줄, 글자 모양, 표 테두리를 확대해 문단의 쪽 소속과 줄바꿈이 유지되는 것을 확인했다. [쪽별 실루엣과 픽셀 해시](../assets/semanticist21-9337-9354-9377/edited-body-lines.tsv) · [문단 112 편집본 확대](../assets/semanticist21-9337-9354-9377/line-112-comparison.png)

원본 HWPX XML에서 문단 110·111에는 줄 정보가 없고, 문단 109의 마지막 저장 줄은 59,748 HU, 문단 112의 첫 줄은 3,768 HU다. 회귀 검사는 경계를 넘기 전 값을 직전 저장 줄에서 읽고, 편집한 112번 문단뿐 아니라 다시 연 모든 문단의 쪽 소속과 원본 글자를 대조한다.

최종 제품 `a1bc2e5fe6`에서 빌드한 Chrome WASM으로 26개 문단의 편집을 다시 실행했다. 편집 직후와 HWP·HWPX로 다시 연 52건을 원본과 대조했다. 모두 24쪽이며 365개 문단 각각의 텍스트와 쪽 소속이 일치했다. [형식별 Native·브라우저 결과](../assets/semanticist21-9337-9354-9377/corrected-reopen.tsv)

## Merge 후 contributor PR comment 계획

원 기여 커밋을 포함한 통합 PR의 병합 SHA를 확인한 뒤, #9377에 감사 댓글을 게시한다. 문단 112를 편집한 HWPX 8쪽을 한컴 인쇄에 대조한 아래 두 출력과 쪽별 TSV를 병합 SHA로 고정해 첨부한다. 편집 위치 앞뒤의 줄과 문단 소속을 비교한다. 26개 문단을 HWP·HWPX로 저장한 52건의 실제 쪽수·텍스트·쪽 소속 결과를 알린다.

- Native Skia의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/native/line-112.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/line-112-p008-native-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/line-112-p008-native-overlay.png)
- 브라우저 Canvas2D의 [전쪽 TSV](../assets/semanticist21-9337-9354-9377/page-tsv/target/wasm/line-112.tsv) · [인쇄 비교](../assets/semanticist21-9337-9354-9377/print-review/line-112-p008-wasm-review.png) · [겹쳐 보기](../assets/semanticist21-9337-9354-9377/print-review/line-112-p008-wasm-overlay.png)

## 최신 devel 병합 후 원 기여와의 최종 대조

원 PR 최종 커밋 `b3301e1905edd7fe504fe400c3560e27c421ad33`에서 Native와 브라우저용 WASM을 각각 빌드했다. 원 PR 본문과 변경 파일에 시각 TSV가 없어 같은 원본·한컴 인쇄본·글꼴·96 DPI로 측정했다. 기준 `e961a78771`을 병합한 후보 `ff9194594e`과 쪽별로 대조한 144행에서 0행이 개선됐고 144행은 같은 일치율을 유지했다. 누락과 하락은 0행이었다. [원 커밋·기준·후보의 쪽별 측정과 PNG 해시](../assets/semanticist21-9337-9354-9377/original-candidate.tsv) · [전체 검증 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)

같은 Chrome 156에서 두 WASM 파일을 각각 불러와 실제 편집을 비교했다. 원본의 112번 문단에 글자를 넣었다 지우고 HWPX로 다시 열면 원 기여는 24쪽에서 25쪽으로 바뀌고 231개 문단이 다른 쪽에 놓였다. 후보는 24쪽과 365개 문단의 쪽 소속을 지켰다. [두 실행 파일의 해시와 편집 결과](../assets/semanticist21-9337-9354-9377/local-checks.json)
