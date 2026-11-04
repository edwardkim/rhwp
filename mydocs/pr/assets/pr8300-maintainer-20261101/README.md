# PR #8300 메인터너 검증 증거

원 source는 `dc171d224bd6d1f5702554d7e171af0961dab755`, base는
`2adaa0859f94f30f7dc3b994ee27c34a872906ba`다. 실제 빌드 source와 수정 파일 해시는
[WASM 빌드 manifest](wasm-build-manifest.json)에 구분해 기록했다.

## 입력과 독립 출력

- `created/nested-empty.{hwp,hwpx}`: 새 3×3 표의 가운데 빈 셀에 2×2 표 생성.
- `created/nested-mid.{hwp,hwpx}`: 새 2×2 표의 첫 셀 ‘가나다라’ 사이에 2×2 표 삽입.
- 네 입력은 Native API 생성본과 실제 Studio 명령으로 생성·저장한 파일의 SHA-256이 같다.
- 독립 기준은 [pdf/issue8262](../../../../pdf/issue8262/)의 네 `*-2020.pdf`다.
  [인쇄 실행 기록](print-execution.json)에 입력·PDF 해시와 한/글 2020 `11.0.0.9136`의
  Print 실행 정보(방법 0, 한 쪽에 한 쪽)를 기록했다.
- 원 PR이 제출한 입력 3개와 기준 PDF는 [기존 입력](../issue_8262/input/)에 있다.

## 시각 결과

[전쪽 요약](visual-summary.json)의 7개 문서는 각각 1쪽이다. 모든 Native/fresh WASM 쪽이
96 DPI·Print 프로필·2022 호환 모드의 경계 보정 전 2px raw 실루엣 100.00%이며 PDF 쪽수와 같다.
`created/`·`supplied/` 아래에 각 backend의 TSV 원본과 실행 manifest를 보존했다.
`base/`의 제출 문서 3개도 양쪽 100.00%로, 기존 파일 열기의 보존 근거다.
새 표 생성 동작은 실제 생성·저장 입력과 별도로 검증했다.

각 `*-detail.png`는 같은 영역을 Print·Native·WASM 순서로 3배 확대했다. 새 HWP 대표 2개는
backend별 compare·overlay·review PNG도 있다. 외곽선·부모 셀 안쪽 여백·뒤 문단과
‘가나–표–다라’의 순서·간격을 직접 판독했다. 안티앨리어싱 차이를 제외한 여백 침범·누락·겹침은 없다.
글꼴 프로그램을 커밋하지 않으며, 실제 Canvas2D의 Noto Serif KR 대체 글꼴과 Print의 HCRBatang은
[글꼴 실행 기록](font-receipt.json)에 구분했다.

[배치 반례](placement-negative-control.json)는 초기 메인터너 보정안을 폐기한 이유를 기록한다.
빈 줄 메트릭의 구현 태그를 표 호스트에 복사하면 셀 위 여백과 표 바깥 여백을 침범했다.
원 기여의 호스트 초기화·재조판을 유지한 최종 후보는 해당 포함 관계를 만족한다.
두 실제 렌더 트리도 함께 보존했으며, 이 실패는 원 기여자의 결함으로 분류하지 않는다.

## Studio와 자동 회귀

[브라우저 결과](browser-result.json)는 실제 표 선택기·상세 대화창, 셀 내 생성,
캐럿 경로, Undo/Redo, 2단계 중첩, 잘못된 옵션의 문서 무변경 거절과 실제 한글 입력을 검사했다.
`studio-*.png`는 실행 화면이다. [실행 probe](browser-review.mjs)는 자동 CI suite가 아닌
메인터너 브라우저 재현 스크립트이며, fresh WASM을 적용한 Studio 개발 서버를 먼저 실행해야 한다.

```bash
VITE_URL=http://127.0.0.1:7705 \
CHROME_PATH=/path/to/chrome \
RHWP_WASM_BUILD_MANIFEST="$PWD/pkg/build-consumption-manifest.json" \
node mydocs/pr/assets/pr8300-maintainer-20261101/browser-review.mjs --mode=headless
```

자동 Rust 회귀는 `tests/cases/issue_8262_table_in_cell.rs`의 기존 7개 검사다.
실제 배치의 부모 셀 padding·중첩 표 margin·뒤 문단 관계를 추가했으며 절대 px 좌표를 고정하지 않는다.
최소 글꼴 환경에서도 실행하며 별도 CI 호출은 추가하지 않았다.
suite 배정은 파일 크기에 따라 바뀌므로 고정 suite 이름 대신 다음 명령을 사용한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs issue_8262_table_in_cell -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
```
