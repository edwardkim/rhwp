# Task #7957 13단계 — #7971 통합 source의 최종 검증

Runtime source는 `3b7a3662b3fbabac2e8544ce364497bd4c454b19`로 고정한다.
12단계의 JPEG 예약 경계 보정을 PR #7970의 공동 처리에 포함한다.
현재 Native Skia·fresh Docker WASM과 전쪽 검증을 진행한다. 실제 종료 코드·producer SHA·
원본/생성 대조군 결과를 얻은 뒤 아래에 기록하며 이전 source의 결과로 대신하지 않는다.

앞선 359쪽 시각 범위와 자원 전용 size 1쪽에 공개 생성 5쪽·비공개 원본 5쪽을 추가한다.
원본의 개인정보·그림·SHA·비교 화면은 ignored 로컬 증거에만 보존한다.
공개 PR의 시각 증거와 fixture에는 생성 대조군만 포함한다.
원본과 생성본을 서로의 출력 일치 증거로 대신하지 않고 각각 독립 PDF와 대조한다.

새 source는 default buffer/codec/누적 한도와 그림 geometry를 변경하지 않는다.
실제 JPEG storage 분기와 같은 `encoded_jpeg` 판정을 거절의 적용 범위에도 사용한다.
raw·JPEG2000·JBIG2 등 다른 분기는 기존 선행 guard를 유지한다.
실제 JPEG RGB/Gray/CMYK와 chained/inline 입력, 실제 pixel/codec cap·sticky 상태·새 요청
복구는 정식 자원 검사에 포함한다. 기준값 완화나 개인정보 원본의 공개로 검증을 대신하지 않는다.

증거는 ignored `output/pr-review/issue7957/extraction-buffer-20261017/`다.
`parserAllocationBoundsProven=false`와 사용자 승인 B1 자원 전용·Helvetica 백로그를 유지한다.
새 source의 원격 CI·PR review·병합·이슈 종료는 아직 완료하지 않았다.


## Producer와 작은 경계 완료

- Native Skia 자원 검사 123건 PASS. 기존 JPEG header 불일치·private metadata/MCU·CMYK,
  raw·sample/stream·누적 한도와 추가 RGB/Gray/CMYK·chained/inline 경계가 포함됐다.
- Native CLI SHA-256:
  `6fb1baf64195da8f28d934b825f6dc2004ba86aae540ebd47bd81933619a1408`.
  같은 source의 Native Skia 빌드가 생성한 CLI를 보존해 이후 Cargo 산출물 변경과 분리했다.
- Docker 빌드는 exit 0(9분 10초)이고 WASM SHA-256은
  `7dedc27fb948458999a4905f59b5487a2b6cea8361ff85473db59b4448d99c98`이다.
  JS SHA-256은 `e7bd20dd7ad8748c187fcb2864ebb3b27f1fb26bc7f43f8b72bf65037d77993a`로
  기존 binding과 같고 pkg/public/실제 Studio HTTP 응답이 일치한다.
- 공개 API 검사 115 assertion·실제 Worker 7개 PASS. 새 공개 스캔 5쪽의 직접 가져오기,
  Worker 및 HWP/HWPX 저장·재열기의 각 쪽 그림 bytes 보존을 포함한다.
  같은 새 검사에서 이전 9c6c48c091 WASM은 `image extraction buffer`로 FAIL했다.
  이를 빌드 실패나 새 source의 실패로 기록하지 않는다.

## 전쪽 시각 결과와 직접 판독

Native/fresh Docker WASM 370쪽씩을 새로 산출했다. 시각 범위는 16개 문서·369쪽이며
공개 15개 문서·364쪽과 비공개 원본 5쪽이다. 양쪽 모두 전쪽 raw 90% 이상·누락 0·독립
PDF 쪽수 일치다. 고정 devel 공통 지원 대조군의 0.01%p 이상 하락은 없다. 미지원 신규 입력의
기준 TSV 불가 범위는 기존 승인에 따르며 새 공개 스캔도 고정 devel에서 동일 거절을 확인했다.
자원 전용 size 1쪽은 Native 73.54167% / WASM 78.19253%를 그대로 남긴다.

새 공개 5쪽은 양쪽 전쪽 100%다. 비공개 5쪽 최저는 Native 98.72216%, WASM 98.72583%다.
기존 범위 최저는 Native 90.14458%, WASM 90.16683%로 유지됐다.
새 코드로 재출력한 기존 360쪽의 각 PNG는 이전 검증본의 PNG와 양쪽 모두 360개 전부
byte-identical이다. 이를 실행하지 않은 캡처 재사용으로 바꾸지 않으며 실제 새 출력과 비교했다.
`previous-head-png-comparison.json`, `visual-summary.json`, 각 원본의 전체 `silhouette.tsv`와
실제 export/score 로그를 ignored 경로에 보존한다.

원본/생성본 각각 같은 영역의 Native·PDF·WASM을 직접 판독했다. 그림 외곽·본문/괘선,
페이지 모서리·하단 표식·색상·내용의 잘림/누락을 확인했다. 원본의 미세한 raster 보간 차이는
남으나 그림 소속·geometry와 내용은 보존됐고 큰 배치 결함은 관측하지 않았다.
기존 360쪽의 알려진 글꼴·간격 직접 판독은 동일한 최신 출력 bytes 확인과 함께 재사용했다.
새 10쪽의 점수를 기존 Type3 glyph 또는 B1의 완료 근거로 확대하지 않는다.

실제 Studio는 `http://localhost:7797/?review=issue7971-extraction-buffer&lang=ko`에서
비공개 원본 5쪽을 열었고 새 WASM 응답 해시 일치·pageerror 0건을 확인했다.
로컬 전체 비교는 `/mnt/e/tmp/issue7971-extraction-buffer-review-20261017/index.html`이다.
이 폴더는 비공개 원본 이미지를 포함하므로 공개 asset에는 복사하지 않는다. Git asset은
`extraction-public`과 기존 공개 원본의 대표 PNG만 whitelist로 복사한다.

필수 전체 Rust 회귀는 동일 runtime에서 완료했으며 최종 결과는 아래에 기록한다.


## 검증 중 base 전진

2026-10-17 중간 확인에서 upstream/devel은 #7969의 merge
`04f8c2bd1617f0dbec6f10543913391ff088c4a7`로 전진했다. renderer/paint/Native host와
부속 자료 변경이며 `src/parser/pdf`·PDF backend·shared PDF worker는 변경되지 않았다.
`git merge-tree --write-tree HEAD upstream/devel`는 exit 0이다. 현행
`review_only_fast_pass.md` A.0 절차에 따라 base 전진만을 이유로 source를 다시 merge/rebase
하지 않는다. 기존 비교 기준 SHA는 9415b920으로 고정하며 이번 producer 검증과 구분한다.
최신 base의 자동 merge checkout을 사용하는 PR CI에서 통합 build/회귀를 별도로 확인한다.
이 기록을 통합 후 Visual Sweep 완료로 해석하지 않는다. PR 병합/이슈 종료는 별도 승인이다.


## 최종 로컬 판정과 원격 반영

전체 nextest는 **10,625 PASS / 45 skip**, 실제 테스트 실행 345.104초(slow 9건), exit 0이다.
추가 자원 계약 5건도 정식 전체 회귀에서 각각 PASS했다. Native Skia lib 4,043 PASS/12 ignored,
placeholder 2건·direct PDF 4건, workspace build·모든 Rust lint·manifest/unit-tier 등
필수 validation 단계 11개도 전부 exit 0이다. 기존 검사를 제거하거나 기준값을 완화하지 않았다.
명령과 각 단계의 source/base/종료 코드는 `validation-results.json`, 실행 로그는
`full-regression.log`·`validation-phase1.log`·`validation-phase2.log`에 보존한다.

변경하지 않은 font rule·projection 데이터와 Node 49건, Studio source/의존성 검증은 앞선
동일 bytes의 근거를 재사용한다. 새 제품 WASM의 공개 API·실제 Worker와 Studio HTTP bytes는
이번 source로 새로 검증했다. 이후 제출 commit은 test·문서·공개 PNG만 포함하며 runtime·font
rule·binding은 위 producer와 동일하다. 같은 source를 다시 빌드하지 않는다.

작업지시자가 승인한 PR #7970 포함 지시에 따라 원격 source branch를 갱신하고 #7971의
공개 생성 대조군·전체 공개 TSV 원문·대표 PNG·비공개 검증 집계를 한국어 PR 본문에 반영한다.
원격 새 head의 CI는 로컬 통과와 구분해 확인한다. exact code head CI 성공 전 PR review·오늘할일
trailing 문서는 만들지 않으며, PR 병합·이슈 종료는 이번 원격 반영의 완료 범위에 포함하지 않는다.
