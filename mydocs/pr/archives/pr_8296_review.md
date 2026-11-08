# PR #8296 리뷰 — 한컴 날인 기호의 1em 합성

## 최종 판정

**승인 — CanvasKit 안쪽 face·glyph coverage 보정을 반영한 code head의 로컬 검증과 Full CI를 완료했다.**
원 기여의 1em 날인 기호 복원을 수용하며, 최초 head의 P1은 보정 `3ddf77af61`에서 해결했다.
2026-11-01 작업지시자가 수용 처리를 승인했다. 아래 최초 검토는 원 head의 진단 이력으로 남긴다.
최신 문서 후행 head의 필수 검사와 병합 상태는 push 후 다시 확인한다.

## 접수와 최신 devel

- PR: [#8296](https://github.com/edwardkim/rhwp/pull/8296), 작성자 `johndoekim`, 기존 merge 기여 20건.
- reviewer `edwardkim`이 이미 지정돼 있음을 API로 재확인했다. maintainer 일반 경로와 렌더링·fixture,
  실제 backend 검증·대형 PR 보조 절차를 적용한다. 첫 기여자 절차는 비해당이다.
- 검토 head: `f3e2128edca7b188080b2a94f25e0988e26c9699`, fork `johndoekim/rhwp`, base `devel`.
- 최신 devel: `20c9678b01c5a564bb510c9fccd652cceafcd212`.
  최신 코드의 F012B → `(인)` 확장은 그대로다. 동일 제품 내용의 최적화 WASM을 실제 실행한 SVG에서도
  `인감(인)또는서명`을 확인했다. [관측값](../assets/pr8296-maintainer-20261101/latest-devel-result.json).
- 최신 devel과 head의 merge-tree는 `d9dc04fdf031f6c68d9c3b7aa1015097fad22ed9`이며 충돌이 없다.
- head의 CI는 35 SUCCESS·6 조건부 SKIPPED다. [API 원문](../assets/pr8296-maintainer-20261101/pr-checks.json).
  녹색 CI를 실제 glyph/face 선택의 근거로 대신하지 않았다.

## P1 — 준비된 공개 안쪽 글꼴을 CanvasKit에서 소비하지 않음

[원 head 코드 L3534–3574](https://github.com/johndoekim/rhwp/blob/f3e2128edca7b188080b2a94f25e0988e26c9699/rhwp-studio/src/view/canvaskit-renderer.ts#L3534)의
seal 합성 분기는 `geometry.face`인 Haansoft Batang을 찾은 뒤, 없으면 본문 regular face를 바로 고른다.
Rust의 `enclosed_pua_geometry::inner_faces`가 지정한 `RHWP Public Batang` 후보를 소비하지 않으며,
선택한 face가 일반 ‘인’을 실제로 가지고 있는지 확인하지 않는다.

이 때문에 본문 face에 한글이 없고 Public Batang은 이미 준비된 유효한 fallback 조건에서도
`Canvas.drawText('인')`에 glyph ID `0`이 전달됐다. 실제 PNG에는 타원 안에 .notdef 네모가 보인다.
동시에 진단은 `lastRenderCompleted=true`, unsupported 없음, software surface로 보고한다.
Public Batang을 직접 지정한 정상 대조군은 같은 문자에 glyph ID `10638`을 사용하고 정상 표시한다.

- [원 문서 textRun의 글꼴 커버리지 반례](../assets/pr8296-maintainer-20261101/canvaskit-original-op-missing-inner.png)
- [단일 기호 반례](../assets/pr8296-maintainer-20261101/canvaskit-inner-missing.png) /
  [Public Batang 정상 대조군](../assets/pr8296-maintainer-20261101/canvaskit-public-face-control.png)
- [실제 drawText glyph ID·준비한 글꼴·진단·폰트 응답 해시](../assets/pr8296-maintainer-20261101/canvaskit-probe-result.json)

별도 브라우저에서 실제 제품 `CanvasKitLayerRenderer`와 공급한 font bytes를 사용했다. 원 HWP에서
생산한 textRun을 추출해 본문 face의 Hangul coverage만 바꾼 통제 반례와 단일 기호 대조군이다.
실제 렌더링을 대역으로 교체하지 않았으며 drawText 호출을 관찰했다. 정상 문서의 전체 페이지
한컴 비교나 Studio 전체 화면을 이 통제 실험으로 대체했다고 주장하지 않는다.

공개 바탕 face 요청·적재만으로 glyph가 최종 paint에서 소비되지는 않는다. CanvasKit도 공통
안쪽 face 후보 정책을 소비하고 glyph coverage를 확인한 뒤 그려야 한다. 없을 때는 누락을
성공으로 보고하지 않아야 한다. 기존 renderer-contract 검사는 ellipse와 drawText 호출은 확인하지만
실제 ‘인’ glyph가 0인지와 준비된 Public face 선택을 확인하지 않아 이 반례를 놓친다.

## 수용 가치와 보존 확인

원본은 F012B 한 문자이며 tcps 글자겹침이 아니다. `(인)` 세 문자 확장을 1em 기호로 되돌리는
방향은 입력의 편집 의도와 맞는다. #937/PR #947의 기대값, #2020/#4384의 날인선과
#6127/#7642의 네모 숫자 경로를 함께 검토했다. 측정·paint가 같은 `EnclosedPua` 분류와 em
메트릭을 쓰도록 일반화한 방향도 타당하다. 다만 위 CanvasKit의 최종 face 선택에서는 공유가 끊긴다.

입력의 PUA → display/측정 → 공통 PaintOp → SVG/Native/Canvas 소비를 추적했다.
문서 ID·페이지 수 맞춤 분기나 표 pagination 변경은 없으며 표 이어받기 컷/예약 높이 검토는 비해당이다.
렌더링 기대값은 독립 한컴 PDF를 근거로 교정했고, 새 관계 검사로 핵심 문자·seal·내용 보존을 다룬다.

최초 진단에서는 기존 검증 작업공간의 Native/fresh WASM 전쪽 TSV를 source/input 해시 확인 후
재사용했다. 아래는 그때의 원 head 수치이며, 최신 base의 재출력과 수용 보정 결과는 뒤 절에 구분한다.

| 문서 | 쪽수 | 원 head Native 최저 raw | 원 head fresh WASM 최저 raw |
| --- | ---: | ---: | ---: |
| 위임장 | 2 | 99.94% | 99.91% |
| 복학원서 | 1 | 97.23% | 97.13% |
| issue4680 장부 | 1 | 99.12% | 99.10% |

[보관된 base/head TSV와 신원](../assets/pr8296-maintainer-20261101/manifest.json)의 비교 base는 `da39c53661`이다.
최초 진단 당시에는 최신 base `20c9678b01`의 전쪽 Native/WASM 재측정과 보정 후 핵심 glyph 확대 판독이
남아 있었다. 뒤의 수용 보정 단계에서 모두 완료했으며 P1을 점수만으로 면제하지 않았다.
[Native 대표 review](../assets/pr8296-maintainer-20261101/native-seal-form-review.png)에서 실제 서식의
내용과 셀 경계를 직접 확인했고, CanvasKit 반례 PNG도 직접 판독했다.

재현 입력과 독립 Print PDF는 원 head commit에 포함된
[위임장 자료](https://github.com/johndoekim/rhwp/tree/f3e2128edca7b188080b2a94f25e0988e26c9699/samples/issue8201),
기존 `samples/복학원서.hwp`·`pdf/복학원서-2022.pdf`, issue4680 원본·2020 PDF다.
실제 사용한 파일과 commit의 동일성은 입력 manifest에 기록했다. 통제한 Font coverage 반례는
독립 Print 측정의 추가 fixture/golden으로 등록하지 않았다.

## 완료한 검사와 다음 절차

실제 브라우저 재현과 최신 devel SVG 확인을 완료했다. 원 head의 광범위 GitHub CI는 재사용하고
로컬에서는 날인·네모 숫자·PUA 정규화·기존 저장 보존의 집중 검사 28건을 실제 재실행해
모두 통과했다. 개인·시스템 글꼴을 가린 Native Skia 경로이며 같은 suite의 비선택 1,710건은
통과 수에 포함하지 않았다. [증거 manifest](../assets/pr8296-maintainer-20261101/manifest.json)에
실행 head·격리 조건·로그 해시를 기록했다. 문서 링크·스크립트 구문·공백 검사도 통과했다.

메인테이너의 제한된 보정안은 CanvasKit 안쪽 face 후보·coverage 선택, 실제 glyph 기반 회귀,
해당 backend 및 최신 base 전쪽 비교다. 다른 조판 리팩터링으로 범위를 넓힐 필요는 없다.
최초 진단 당시 원격 보정 제출·댓글·승인·머지는 후속 범위 결정 대기였다. 이후 작업지시자의 수용 승인으로
보정 제출과 검증을 진행했으며, 정확한 원격 조치는 뒤 절과 원 PR의 메인테이너 댓글에 기록한다.

## 수용 보정과 최종 code head 검증

- 원 head `f3e2128edc` 위에서 로컬 검토 기록을 replay한 `9f6b7ff6ef`, 제품·회귀 보정 `3ddf77af61`을 추가했다. 기여자 커밋의 rebase/amend/force-push는 없다.
- 변경 제품은 CanvasKit TypeScript 두 파일이다. Rust와 기존 Canvas2D/SVG 제품 내용은 원 head와 같으며 원 기여의 검증된 최적화 bundle을 재사용한다.
- 공통 JSON의 안쪽 face 우선순위를 소비하고 실제 glyph coverage를 확인한다. 모든 후보가 없으면 glyphMapping 진단으로 보고하고 부분 테두리를 성공 출력으로 남기지 않는다. 준비된 Public face·본문 대체·전체 후보 누락을 검사했다.
- 원 renderer를 Vite 로더에 주입한 음성 대조에서는 새 실제-glyph 검사가 실패했고, 보정 후 renderer-contract와 TypeScript, Studio 2,700 PASS/기존 skip 2가 통과했다. 앞선 Native 집중 28건의 Rust 내용도 같다.
- [최신 base/후보의 실행 신원·로그 해시·전쪽 비교](../assets/pr8296-maintainer-20261101/acceptance-validation.json): 최신 devel `20c9678b01`을 실제 재출력했다. Native/WASM 8개 page/backend 쌍 모두 90% 이상이며 하락·누락은 없다. 위임장 2쪽 99.94/99.91%, 복학원서 1쪽 97.23/97.13%, 장부 1쪽 99.12/99.10%가 각 경로 최저다.
- [실제 glyph 선택 보정 후](../assets/pr8296-maintainer-20261101/canvaskit-probe-after.json), [동일 반례의 보정 PNG](../assets/pr8296-maintainer-20261101/canvaskit-original-op-after.png), [Print·Native·WASM·CanvasKit 확대](../assets/pr8296-maintainer-20261101/seal-closeup-final.png)를 직접 판독했다. 안쪽 ‘인’과 끝 ‘명’이 셀 안에 보존된다.
- 세 문서의 전체 PageLayerTree도 실제 CanvasKit software로 재생했다. 공개 글꼴 aliases와 U+2024를 포함한 DejaVu 대체를 명시한 환경이며 unexpected unsupported op가 없다. 원 한컴 글꼴과의 점수로 승격하지 않는다. 기존 네모 숫자의 Native 전체 PNG와 CanvasKit 글꼴·장평 대조 5개는 바이트가 동일하다.

### code head CI — `3ddf77af61b10a9ace10224588ad68841e79b2f1`

| 검사 | 결과 |
| --- | --- |
| CI: lint·frontend·Native Skia·Archive A–E·집계 | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41277462618) |
| CodeQL 전체 | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41277462643) |
| Render Diff | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41277462671) |
| Adapter inter-diff | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41277462622) |
| Proptest roundtrip | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41277462612) |
| 최종 CI Impact Policy | [SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/41279110227) |

[API 원문](../assets/pr8296-maintainer-20261101/code-ci-final.json)을 보존했다. 성공한 같은 code head의 전체 Rust 검사를 다시 로컬에서 반복하지 않는다.

### 최신 devel 결합 검증 — `5d455015cb`

code CI 성공 뒤 문서 기록을 추가하는 과정에서 오늘할일만 add/add 충돌했다. 현재 base
`20c9678b01`과의 bridge `5d455015cb`는 두 parent를 가지며 수동 해소 경로는
`mydocs/orders/20261101.md` 하나다. #8417·#8420 기록을 보존했다. 문서 외 tree가 녹색
code candidate와 최신 base의 자동 병합 결과와 같고, current-base의 검사기도
`current-base-merge-resolution-mydocs-only`로 판정했다. 최종 원격 재사용 여부는 새 head에서 확인한다.

[결합 실행 manifest](../assets/pr8296-maintainer-20261101/combined-validation.json)에 Native
`release-test`와 실제 release·wasm-opt 빌드, 명령·해시·원본 TSV를 기록했다.
새 WASM SHA-256은 `a65db888312503f7c16ed1c3dfad59d30c01972106bacfdcdae7286a82b1b28f`이며
`pkg`와 Studio public bundle이 같다. 문서 후행 commit은 이 제품 내용을 바꾸지 않는다.

- 세 문서 4쪽의 Native/fresh WASM 8개 PNG는 앞서 검증한 후보와 바이트까지 같고, 위 전쪽
  raw·쪽수·내용 보존 결과가 유지된다. 결합 상태의 TSV와 대표 review·overlay를 새로 산출했다.
- [실제 CanvasKit probe](../assets/pr8296-maintainer-20261101/combined-canvaskit-probe.json)의
  안쪽 ‘인’ draw 7회 모두 유효한 glyph다. [전체 PageLayerTree 3개 재생](../assets/pr8296-maintainer-20261101/combined-canvaskit-full-page.json)도 완료·unexpected unsupported 없음이다.
- 위임장의 날인과 마지막 ‘명’, [복학원서의 두 날인](../assets/pr8296-maintainer-20261101/combined-bokhak-closeup.png),
  [장부의 성명·날인](../assets/pr8296-maintainer-20261101/combined-issue4680-closeup.png)을
  Print·Native·WASM 같은 영역에서 확대 판독했다. 원형 안의 ‘인’과 주변 내용이 보존된다.
  기존 본문 글꼴의 획·선 표현 차이를 완전한 픽셀 일치로 주장하지 않는다.
- 최신 base의 PDF 네 샘플은 곡선 clip 1·2·12쪽, 고해상도 이미지 1쪽, 암호화 원본
  10·21·43쪽, f87pro 1쪽을 실제 가져오고 저장했다. base/후보의 HWPX 네 파일은 바이트가 같다.
  결합 TypeScript 검사도 통과했다. 이는 해당 대표 쪽의 기능 보존 검사이며 127쪽 시각 검증을
  다시 수행했다는 뜻이 아니다.
- 추가로 시도한 32쪽 PDF 전체 가져오기·저장은 이 `release-test`/Native Skia 환경에서
  base와 후보 모두 exit 1이었다. base의 strace에서 CPU soft/hard 90초와 SIGKILL을 확인했다.
  이 실행은 통과 수에 넣지 않았다. PDF parser/backend/CLI는 base와 같고 대표 쪽 저장 결과도
  같음을 확인했으며, 이번 PR에서 기존 실행 한도나 PDF 구현을 변경하지 않았다.

### merge 후 contributor comment 계획

원 기여의 1em 복원에 감사하고 CanvasKit 보정의 사유·범위를 구분한다. 실제 merge SHA와 위 CI URL, 전쪽 수치와 최신 검토 기록을 연결한다. merge SHA에 존재하는 Native/WASM review·overlay와 보정 후 CanvasKit PNG를 포함하고 Visual Sweep 정본을 링크한다. 한국어 존댓말·UTF-8 본문을 파일로 전송하고 게시 뒤 API로 확인한다. fork branch는 보존하며 관련 #8201의 종료 여부를 확인한다.

이미지는 결합 상태의 `combined-native-seal-form-review-001.png`,
`combined-native-seal-overlay.png`, `combined-wasm-seal-form-review-001.png`,
`combined-wasm-seal-overlay.png`, `combined-canvaskit-original-op.png`를 사용한다.
