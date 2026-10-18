# #7998 P4 열한 번째 절편 — 텍스트 클립의 이동과 저장 소유

기준은 승인된 B19 첫 절편 `90b4ffbe937ac7ffddd9cfe31f6e39dcfad3c651`이다.
2026-10-19 다음 절차 지시에 따라 clip의 실제 배치 후처리와 문단 목록의 저장 소유를
확인한다. stage11의 시각·편집 구조 판정과 원본/빌드/출력은 그대로 보존한다.

## 구현 전 경계와 기대 계약

- `TextBox.paint_clip`은 원래 shape-local HU이다. `layout_textbox_content`가 실제
  원점에서 page-px clip을 생산하며 실제 LayerBuilder/각 backend가 이를 소비한다.
  이후 내용 subtree를 옮겼다면 같은 source clip도 같은 이동을 받아야 한다.
- `layout.rs::translate_subtree_y/translate_node_payload_y`와
  `table_layout.rs::translate_render_subtree_y`는 bbox·일부 payload를 옮기지만 새 clip은
  처리하지 않는다. 단순 bbox/viewport 확장과 실제 내용의 이동은 구분한다.
- `model/text_clip.rs`의 보조 metadata 순회는 본문/셀/도형 안 문단에 한정된다.
  다른 문단 목록에 넣은 같은 TextBox도 저장·재열기 후 원문과 clip의 유일한 소유를
  보존해야 한다. 한컴 기본 path clip 대응 여부와는 별도 rhwp 왕복 계약이다.

먼저 공개 PDF의 원래 clip/원문을 가진 실제 TextBox를 정상 꼬리말 정렬에 연결해
후처리 이동의 실패를 실행한다. 독립 PDF의 graphics translation과 같은 원문의
clip/기준선 상대 관계로 기대값을 정한다. 임의 Lineseg/vpos로 발동 조건을 맞추지 않는다.
검증된 작은 경계 뒤에 관련 정상 대조군과 fresh Docker WASM을 확인한다.
회전/크기 변경·한컴 native 비사각 표현 전체를 한 번에 완료한다고 표시하지 않는다.
긴 전체 Rust/lint/package 및 최종 devel 비교·원격 제출은 P5에 유지한다.
원시 증거는 ignored `output/pr-review/issue7998/p4-text-clip-placement/`에 보존한다.

## 원인·실제 소비와 보정

`layout/shape_layout.rs::layout_textbox_content`의 source-local HU → 절대 page-px
생산은 맞았지만, 실제 Footer 가운데/아래 정렬이 내용의 bbox와 glyph를 옮긴 뒤에도
clip은 옮기지 않았다. `layout.rs:6655`의 Footer 호출 → `translate_subtree_y` →
`translate_node_payload_y` → 공통 layer의 ClipPath와 backend 소비를 대조했다.
현재 줄 메트릭이나 요구 높이를 바꾸는 변경이 아니며, 이미 정해진 내용 원점의
후처리 이동에 절대 paint 경로를 포함시킨다. 측정의 별도 offset 추측은 추가하지 않는다.

| 구현 주장 | 적용/비적용 경로와 실행 근거 |
| --- | --- |
| 내용과 clip이 함께 이동 | 공통 `RenderNode::translate_paint_clip_y`를 layout의 실제 subtree 이동 payload에서 호출한다. 원본 첫 text-show의 보통 Footer TOP/CENTER/BOTTOM을 실행했다. `before-final/observed.json`은 내용 Δ62.37333px/clip Δ0px로 FAIL, `after/observed.json`은 같은 Δ62.37333px로 PASS다. `before-score/results.json`에서 가운데/아래 실제 출력 0% → 현재 각각 100%다. |
| 단순 viewport 확장과 구분 | `relax_textbox_clip_to_content`·TableCell viewport 확장에는 이동을 덧붙이지 않았다. 여유 없는 Footer와 clip 없는 Header/Footer 정상 대조군을 실행했다. |
| 다른 실제 이동 소비 | `table_layout.rs::translate_render_subtree_y`도 같은 paint 이동 연산을 호출한다. 이 경로의 line/seam/continuation 호출은 코드로 대조했지만 **clip이 있는 실제 table continuation 재현은 아직 미검증**이다. 이 절편의 Footer 실행으로 입증했다고 쓰지 않는다. |
| Header/Footer 저장 소유 | `model/text_clip.rs`의 collect/visit 양쪽에서 실제 Header/Footer 문단 목록을 순회한다. 제품 양 adapter와 Studio 양 export/reload에서 원문과 clip chain을 보존한다. 본문과 Header/Footer에 같은 ID가 있으면 metadata의 유일 결합을 거절한다. |

변경은 위 4개 제품 파일의 16행이다. line 구성·높이·분할 컷이나 렌더러별 좌표를
별도로 보정하지 않았으며, clip이 없는 node는 같은 no-op 경로다. `layout.rs`의
다른 실제 사용인 floating table/anchor 이동도 호출 위치로 확인했다. 해당 경계에
source-clipped TextBox를 넣은 실물 검증까지 했다는 주장은 하지 않는다.

기존 immutable CLI와 현재 CLI에 같은 `footer-bottom-hole.hwpx`를 넣어 실제 SVG
출력을 비교했다. `metadata-before-after.json`은 보조 metadata 누락으로 path clip
0개 FAIL → 1개 PASS를 기록한다. 이는 저장 소비의 실패 전후이며, 위 live Footer
이동의 실패와 구분한다. 빌드 실패를 원인 재현으로 세지 않는다.

## 입력·기준 PDF와 source 고정

새 공개 fixture는 `pdf/issue7998/text-clip-placement/`에 보관한다. 원래 공개 PDF의
첫 text-show·font·metric·clip을 보존한 3개 source와, 이를 보통 Header/Footer로 작성한
9개 HWPX 및 독립 PDF translation 기준이다. `contexts-origin.json`의 용지/여백과
source shape 높이로 cm를 결정했다. 현재 render tree의 실제 결과에서 기대 원점을
역산하지 않는다. 원문 MERGED는 paint가 전부 가려져도 실제 Paragraph에 남는다.
정상 unclipped 입력도 같은 생성·저장 경로를 사용한다.

이 기준은 **원래 한컴 Print에서 유래한 PDF 연산 + 독립 PDF raster**의 이동 계약이다.
새 Header/Footer를 한컴에서 인쇄한 결과나 한컴 native 비사각 clip 저장 계약이 아니다.
그 미지원은 두 writer의 실제 content-loss 보고에 계속 남긴다. 세부 생성과 파일별
bytes/SHA는 [fixture 설명](../../pdf/issue7998/text-clip-placement.md)에 연결했다.

초기 기준 생성에서 가로 용지의 저장 height를 물리 높이로 잘못 읽어 참조 위치를
52px 아래로 계산했다. 정상 unclipped 대조군의 불일치로 발견해 물리 높이 계산을
바로잡았다. 무효 자료는 `invalid-reference-orientation/`에 보존하고 최종 TSV에서
제외했다. 이 정정 때 제품 코드는 바뀌지 않아 빌드를 반복하지 않았다. 원래 전체
두 줄을 넣었을 때 정렬 이동이 발생하지 않은 사전 probe도 최종 실패 증거와 구분한다.

빌드와 캡처는 아래 제품 source 지문에 고정했다. 이후 정식 검사·fixture·문서만 추가했고
2,796개 runtime 파일 SHA가 그대로임을 다시 확인했다.

| 대상 | SHA-256 |
| --- | --- |
| runtime source manifest | `fe629132b299a33bb5e04be6792fa09e5d50bdce1b85445195334b5bd19a9d6e` |
| Native immutable candidate | `63e7cbe16922361ef2525b28ad0b38faec2b2dd56562721522e9a0df05248c99` |
| fresh Docker WASM, 19,374,476 bytes | `9ac6b2d17cd8e4d88d9ad05a8e80f79a812e2ca2f4288cc08390eb10d253108e` |
| JS glue, 476,233 bytes | `2f715f0325d1292c52a84165d08477868c979eb94900174231a6a02aa6a8de11` |

`native-build.log`는 shared target의 `--locked --profile release-test --features
native-skia --bin rhwp` 실행이다. `docker-wasm.log`는 기존 `rhwp` Compose project와
named cache를 사용한 fresh WASM 실행이다. JS glue는 이전 절편과 바이트가 같고,
WASM은 새 source로 빌드·실제 instantiate한 SHA를 검증했다. 메인 Studio 7700의
JS/WASM 지문은 시작 때와 같고 작업 Studio 7793만 갱신했다.

## 이번 출력·편집 검증과 증거 재사용

| 범위 | Native | fresh WASM | 판정/출처 |
| --- | --- | --- | --- |
| 새 9개 × PDF/HWP/HWPX 전쪽 | 27건/27쪽, 최저 93.33333% | 27건/27쪽, 최저 93.33333% | `native/results.json`, `wasm/results.json`과 각 `visual/*/silhouette.tsv` |
| 영향 있는 기존 clip 8개 × 세 형식 | 24건/24쪽, 최저 99.81269% | 24건/24쪽, 최저 99.51172% | `prior-clips-{native,wasm}/`; 이전 stage11의 같은 backend PNG 24쪽씩 모두 바이트 동일 |
| 이번 head 재출력 합계 | 51쪽, 90% 미만/누락 0 | 51쪽, 90% 미만/누락 0 | `validation-summary.json`, `native-all-current-pages.tsv`, `wasm-all-current-pages.tsv` |
| 이전 정상 99개/330쪽 | 기존 결과 재사용 | 기존 결과 재사용 | 107개 source HWPX의 Header/Footer 목록 0개와 clip 없음 경로를 감사했다. `prior-scope-audit.json`. 이번 head 재출력으로 세지 않음 |

모두 Arial 공급 경로·compat2022·96dpi 조건이다. `validate.py`와 `validate_prior.py`에
실제 전체 명령을 보존했고 제품 코드가 같은 시각 결과를 중복 생성하지 않았다.
최저는 곡선으로 가린 작은 glyph 조각의 경계 차이이다. source/독립 PDF 위치를
바꾸거나 기준 DPI를 조절해 점수를 올리지 않았다. parent 교집합 입력의 빈 화면은
원래 PDF 계약이며, 원문 존재·순서·편집 가능한 소유를 별도 확인했다.

대표 Header·Footer TOP/CENTER/BOTTOM·여유 없는 경우와 곡선을 같은 원점·배율로
확대 판독했다. baseline·clip 경계는 함께 이동하며 Native 잉크 농도 차이는 남는다.
대표 `review/{native,wasm}/{header-hole,footer-bottom-hole,footer-bottom-curve}/`에
compare/standalone overlay/review PNG도 만들고 실제 열어 도구 라벨까지 판독했다.
이는 전쪽 overlay 재생성이 아니라 영향 경계의 대표 판독이다.

fresh WASM TSV는 **공통 layer WebCanvas** 출력이다. 별도 실제 Studio **CanvasKit**에서
원문·clip·양 export/reload와 Header/Footer host 문단 X 삽입/삭제 복원을 9개 모두
확인했다. `studio-result.json`/`studio-restoration-summary.json`에 actual WASM SHA,
원문 유일 소유, 초기/복원 clip 동일과 page error 0을 기록했다. `wasm-actual/`은
실제 export 바이트다. Native의 TextBox 내부 M 편집도 양 adapter에서 확인했지만
Studio가 그 TextBox 안에 직접 타이핑한 검증으로 보고하지 않는다.

2026-10-19 작업지시자가 `E:\tmp\issue7998-text-clip-placement-review\index.html`과
Studio 7793의 header-hole/footer-center-hole/footer-bottom-hole 탭을 확인하여
**“시각·편집 구조 통과”**로 판정했다. 앞 절편의 승인 범위와 이 추가 판정을 구분한다.

## 정식 회귀와 남은 경계

`tests/cases/issue_7998_pdf_text_clip_placement.rs`는 자동 수집 suite에 등록된다.
같은 첫 text-show의 보통 Footer 실제 정렬, 여유 없는 반례, source-local 경로와
최종 원점의 관계, Header/Footer 원문 유일 소유·양 adapter/편집/복원,
문단 목록을 넘는 중복 ID 거절을 보호한다. 절대 px pin·전체 SVG 해시 기대값은 없다.
실행 결과는 아래 최종 검사 항목에 기록한다.

정식 자동 수집 경로를 `node scripts/rust-test-suite-manifest.mjs --prepare`로 갱신한 뒤
`run-rust-test.mjs <case> -- --cargo-profile release-test --target-dir
/home/edward/mygithub/rhwp/target/pr-review --features native-skia`로 실행했다.
새 파일 3건, 기존 `issue_7998_pdf_text_clip` 3건·`issue_7998_pdf_cell_paint_frame` 2건·
`issue_7998_pdf_table_composition` 8건, **총 16건 통과**다.
`focused-new-final.log`와 `focused-existing-results.json`에 실제 command/선택 건수를
보존했다. 이전 절편 집중 57건 전체를 재실행했다고 쓰지 않는다. 제품 변경이 닿는
source clip/metadata·cell paint·중첩 합성을 이번에 실행하고 나머지 증거는 재사용했다.

초기 새 검사는 host 문단의 X가 떠 있는 글상자의 MERGED보다 paint tree에서 먼저
나온다는 잘못된 기대를 뒀다. 실제 결과 MERGEDX를 편집 순서 회귀로 오인하지 않고,
원래 Paragraph/TextBox의 소유·원문과 X의 유일 보존을 검사하도록 바로잡았다.
실패 로그는 `focused-new.log`에 보존했다. 이때도 제품 source·WASM·시각 출력은
바뀌지 않았으며 기존 시각 판정을 반복 요청하지 않았다. 새 검사만 먼저 FAIL 1건을
확인했고, 올바른 소유 계약으로 고친 최종 실행은 3건 모두 PASS다.

테스트 실행 중 nextest 권장/설치 버전 차이와 기존 설정 키 경고는 남지만 실제 선택한
검사의 실행과 통과를 확인했다. 긴 전체 Rust/clippy/package 검사는 실행하지 않았다.

한컴 native 비사각 가림/외부 재저장, footnote/master/caption metadata 순회,
회전/resize·cache, source clip이 있는 table continuation/floating 실제 호출,
strict glyph-only V2 resource 검증은 B19 backlog에 남긴다. P4 전체 완료·P5 최종
devel 대비/전체 회귀·PR 준비 완료로 승격하지 않는다. 원격 push/댓글/PR 상태 변경은 없다.
