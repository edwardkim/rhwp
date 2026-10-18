# #7998 P4 열 번째 절편 — 편집 가능한 텍스트 클립 계약

- 기준 checkpoint `e3bb62045b138160c0a9f1d2d34ae835471c46e7`.
- 다음 절차 지시로 B19 실패 원본부터 조사했다. 2026-10-19 메인테이너가
  **편집 가능한 IR 계약 확장**을 선택했다. 글자를 그림으로 바꾸는 해법은 선택하지 않았다.
- 이번 첫 경계는 source text-show의 실제 clip chain → 원래 Paragraph를 가진 TextBox의
  명시적 paint clip → 최종 공통 layer/Native·WASM·SVG paint → rhwp 저장 재열기다.
  한컴 기본 형식에서 같은 clip을 적용하는 것은 별도 호환성 판정이며, 통과로 추정하지 않는다.

## 구현 전 근거와 실패

보존 원본 `p4-cell-regions/diagnostic/nonrectangular-triangle.pdf`는 정상 한컴 Print의
첫 셀 clip을 명시적으로 삼각형 W*로 바꾼 PDF다. 옛 Native 86.64081% 실패는 보존한다.
현재 B20 producer와 올바른 Arial 공급으로도 Native 89.94524%로 실패한다.
원문 `MERGED `, `HEADER`의 source order [23], [33]이 실제 삼각 clip과 연결되고,
아래 정상 두 셀은 각각의 사각 clip을 사용함을 backend public API로 확인했다.
`source-text-clip-observations.jsonl`은 글자와 활성 graphics state의 실제 대응이다.
근처 clip의 bbox를 문자 소속의 근거로 추정하지 않았다.

Adobe PDF Reference 1.7 §4.4.3의 W/W*는 이후 graphics object의 paint에 적용되며
q/Q에 따라 복원된다. [공식 문서](https://opensource.adobe.com/dc-acrobat-sdk-docs/pdfstandards/pdfreference1.7old.pdf)를
기준으로 원문 문자와 paint 가림을 구분한다. 현재 `TextBox`/공통 layer에는 이 비사각
가림 표현이 없으며, 표 추론에서 얻은 clip 관측만으로 실제 출력을 보정할 수 없다.
로컬 OWPML `AbstractDrawingObjectType/drawText` 스키마와 HWP 스펙의 글상자 필드도
대조했다. 해당 필드에서 같은 path clip 매핑을 확인하지 못했으며, 이것을 한컴의 모든
가능한 표현이 없다는 증명으로 확대하지 않는다.

## 구현 계약

1. 원래 TextBox의 Paragraph·글자·메트릭을 보존하며, signed local HWPUNIT의 명시적
   `paint_clip`을 추가한다. q/Q·Form·CTM이 해소된 실제 clip chain을 local 원점으로 한 번
   투영한다. 줄 높이·글자폭·원래 문자를 가림에 맞춰 줄이거나 삭제하지 않는다.
2. 실제 textbox layout 원점에서 page px로 투영하고, 공통 `ClipPath` layer를 Native Skia,
   CanvasKit, SVG 및 기존 Canvas 경로가 소비한다. 각 경로가 clip을 bbox로 바꾸지 않으며
   even-odd/nonzero와 부모 교집합을 유지한다. source graphics clip은 layout clip 진단
   옵션을 껐다는 이유로 해제하지 않는다. source/캐럿·글꼴/variant 수집도 clip 자식을 방문한다.
3. source clip을 잃는 문단/표 승격은 보류하고 편집 가능한 원래 TextBox를 유지한다.
   비사각 clip을 새 셀의 작성 근거로 삼거나, root 실패로 다른 정상 내용을 버리지 않는다.
4. 기존 hyperlink-format의 검증된 보조 stream/part 패턴을 재사용해 typed clip을 저장한다.
   version·bytes·records·깊이·명령수·유한 좌표를 제한하고, ID와 현재 원문의 fingerprint가
   유일하게 일치할 때만 복원한다. 임의 HWP ParameterSet ID나 설명문에 코드를 숨기지 않는다.
   이는 rhwp 왕복 계약이며 한컴 기본 표현의 근거가 아니다. 대상 writer의 미지원 표현을
   기존 content-loss 보고 경계에서 명시한다. 한컴 실제 열기/Print·외부 재저장 보존은
   별도 검증 없이 통과로 기록하지 않는다.
5. 원본 삼각 clip·같은 clip의 q/Q 복원·evenodd 구멍·부모 교집합·Form 및 정상 무clip/사각
   대조를 순차 확인한다. 작은 실제 출력부터 확인하고 최종 producer에서 영향 전쪽과
   기존 공통 범위를 검증한다. 원본·독립 Print·Native/fresh WASM과 직접 판독 전에
   신규 실물 layout 회귀/fixture를 등록하지 않는다.

원시 증거는 로컬 `semantic-stage3/p4-text-clip` 및 ignored
`output/pr-review/issue7998/p4-text-clip/`에 보존한다. 기존 B20의 빌드·판정·원본을 보존하며
전체 CI/lint/package와 최신 devel 최종 검증은 P5다. 원격 반영·PR 변경은 하지 않는다.

## 원인에서 최종 출력까지의 연결

| 생산/소비 경계 | 실제 구현과 판정 |
| --- | --- |
| `extract.rs`의 ordered painted span → 실제 `text_uses` | source invocation order를 유일하게 연결한다. witness 중복·누락은 명시 오류이며 근처 bbox 추정으로 대체하지 않는다. |
| `pdf/text_clip.rs` → `lowering.rs`의 TextBox | q/Q·Form·CTM이 해소된 path chain을 실제 span 원점에 대한 signed HWPUNIT로 저장한다. Move/Line/Cubic/Close·evenodd·부모 교집합을 보존한다. |
| `shape_layout.rs::layout_textbox_content` | 실제 `base_x/base_y`와 DPI로 page px를 생산한다. 원래 문단의 줄 구성·높이·폭을 가림에 맞춰 변경하지 않는다. |
| `paint/builder.rs` → `LayerNodeKind::ClipPath` | 같은 자식과 원문 source table을 감싼다. `paint.textClipPath`를 필수 feature로 알리고 JSON schema minor를 27로 올린다. variant·font·source·plane 수집은 자식을 계속 방문한다. |
| Skia / CanvasKit / SVG / WebCanvas | 동일 path chain의 교집합과 winding을 소비한다. source clip은 layout clip 진단 옵션과 별개다. Native Skia·실제 Studio CanvasKit·SVG 및 direct PDF는 실행 확인했다. WebCanvas legacy는 컴파일 확인이며 이 절편의 실제 픽셀 검증으로 대체하지 않는다. |
| 저장·재열기 | bounded typed sidecar를 실제 두 writer에 연결한다. ID·현재 원문의 fingerprint가 유일하게 일치해야 복원하며, stale·중복·잘못된 version/과대 payload를 거절한다. 한컴 기본 표현의 제한은 기존 content-loss 보고에 남긴다. |

`structured_lowering`의 셀·본문 승격은 원문 비사각 clip을 잃는 후보를 보류한다.
이 보류를 표 인식 완료로 표시하지 않는다. 실제 편집 가능한 원래 TextBox를 유지하며,
문자 자체를 삭제하거나 raster 그림으로 바꾸지 않는다. leaf-only `RenderBackend` 보조
replay는 표현할 수 없는 path clip을 명시 오류로 반환한다. 해당 경로에서 완전한 paint를
지원한다고 주장하지 않는다. Canvas command recorder의 자식 방문은 진단 수집이다.

## 대조군의 수정과 직접 판정

추가 대조군의 첫 생성기는 첫 머리글 clip이 아닌, 앞에서 설정 후 Q로 해제되는 page
clip을 바꿨다. 구조 검사에서 활성 clip 수 0개로 드러났다. 초기 7개와 실행 로그는
`sources/`, `inputs.json`, `native-new/`, 원래 WASM shard에 보존하되 최종 증거에서
제외했다. 잘못된 대조군의 100%를 nonrectangular 지원 근거로 사용하지 않았다.

`generate_active_controls.py`로 실제 첫 text-show에 활성화된 두 번째 W/W*를 변경하고,
각 `*-active-observations.jsonl`에서 실제 원문/order/path chain을 확인했다. 최종
`sources-active/`와 `pdf/issue7998/text-clips/`는 같은 바이트다. 원래 B18 삼각 실패
원본은 바꾸지 않았다. 공개 한컴 Print를 PDF operator로 수정한 경계 대조군이며,
한컴 작성기가 이 비사각 text clip을 native 저장한 사례로 부르지 않는다.

최종 8개는 triangle, evenodd/nonzero hole, 부모 교집합, q/Q 복원, Form 이동,
파란 배경, cubic 경로다. 독립 PDF와 Native/fresh WASM의 같은 좌표/배율 확대에서
실제 가림·아래 정상 원문·배경·선 위치를 판독했다. 기존 선/글자의 backend별 미세
antialias 차이를 clipping 개선과 구별했다. `inspection/`에 확대 PNG와 대표 Native/WASM
compare·standalone overlay·review를 보존했다.

2026-10-19 메인테이너가 `E:\tmp\issue7998-text-clip-review\index.html` 및 Studio 7793의
triangle/evenodd/curve 탭에서 **시각·편집 구조 통과**로 판정했다. 이번 판정은
PDF→IR→paint 및 rhwp 저장·재열기 범위이며, 한컴 native 표현 완료의 승인이 아니다.

## 최종 제품과 전쪽 결과

- 제품 2,796개 파일의 manifest: `source-runtime.json`, SHA-256
  `032f0a5190a9bde1743370b568e9c1194c0fe12ecda47a4eab755d7a9caa6c18`.
  빌드 뒤 제품 hash를 다시 대조하여 변경 0개를 확인했다. 이후 추가·정리는
  회귀/fixture/문서뿐이므로 기존 Native/fresh WASM 제품과 캡처를 재사용했다.
- Native `release-test`/기본 feature+native-skia, jobs 14, 고정 host `target/pr-review`.
  실행 파일 SHA-256 `8f6b4806377e31d24aedfc25724cdf6c82721d8a7efe71ca618f3c8e61738b7d`.
- fresh WASM은 공통 Docker project `rhwp`, jobs 10/codegen-units 16.
  19,401,865 bytes, SHA-256
  `9d72b26f578977ca18cfff1301915efd36fcb83f32b8affffe4866734ffe96ba`.
  `pkg`/worktree Studio public의 JS·WASM hash와 실제 `instantiateStreaming`을 대조했다.
  main Studio 7700의 JS/WASM hash는 보존됐다. tracked `public/rhwp.js`의 4줄은 이번
  WebCanvas winding rule import/enum glue이며 무관한 변경이 아니다. `pkg`·WASM binary는
  로컬 생성물이다.
- backend별 **107입력 × 직접 PDF/HWP/HWPX = 321건, 354쪽**.
  오류·누락·쪽수 차이·90% 미달 0건. 최저 Native **97.47552%**, fresh WASM **97.65094%**.
  직전 B20의 공통 99입력/330쪽은 같은 backend에서 PNG가 전부 byte 동일하다.
- 새 8입력/24쪽 최저 Native **99.81269%**, fresh WASM **99.51172%**.
  각 backend의 HWP/HWPX 재열기 PNG는 해당 직접 PDF와 byte 동일하다.
  원래 triangle 실패는 Native 89.94524→99.97535%, WASM 89.61347→99.51172%다.
- 별도 SVG 전쪽 8쪽 최저 **99.52707%**. 저장 HWPX를 실제 direct PDF로 내보내어
  독립 원본과 비교한 8쪽 최저는 **99.96905%**다. 이 점수는 Hancom Print 왕복의
  점수가 아니라 rhwp의 SVG/direct PDF 출력 검증이다.

`native/`, `wasm/`의 `results.json`·각 `visual/<key-format>/silhouette.tsv`,
`native-comparison.json`/`wasm-comparison.json`, `native-active/`/`wasm-active/`,
`svg/`/`pdf-export/`에 명령·TSV·입력/출력 hash·로그를 보존한다. aggregate는 원래 shard에
symlink만 연결하여 scratch를 중복 복사하지 않았다. 96dpi·compat2022·기존 입력별 실제
font 공급을 유지했고, 새 입력은 검증된 Arial 공급을 사용했다.

## 실제 편집·저장과 정식 검사

`api.rs`/`api.log`와 `verify_studio.mjs`/`studio-result.json`이 8개 실제 원본을 열어
가려진 원문도 Paragraph에 남는지, clip owner/chain과 실제 layer가 대응하는지,
M 삽입/삭제·양 adapter 저장/rhwp 재열기 후 유지되는지를 확인했다. Native 명시 저장
보고서에서 한컴 기본 표현 제한도 확인했다. metadata f64의 JSON 왕복 차이는
1e-9 HWPUNIT 미만의 수치 허용으로 구별하며 path 수·command 종류·winding은 그대로 검사한다.

실제 fresh WASM의 V2 compatibility export/Studio normalizer에서도 source clip과
필수 feature가 유지된다(`studio-v2-result.json`). strict glyphRun/outline export는 이
source font의 complete resource-proven variant가 없어 `strictVisualVariantMissing`으로
거절된다. 해당 strict 경로의 완전한 paint 지원은 **미검증**이다.

신규 `tests/cases/issue_7998_pdf_text_clip.rs` 3건은 source 소유/내용 순서·actual layer,
실제 문자 편집과 두 writer, metadata 오결합/과대 입력 거절을 검사한다. 독립 근거와
전쪽 양 backend/직접 판독 후 등록했다. 이전 CLI가 성공적으로 생성한 같은 triangle
HWPX를 현재 reader로 열어 **정식 검사의 동일 check**를 실행하면 활성 clip 0개로 FAIL,
현재 adapter 출력은 PASS다(`before-after-proof.json`). 옛 라이브러리 전체를 재빌드한
검사라고 표시하지 않는다. 첫 파일 경로 누락 실행은 환경 실패로 제외하고 원래
CLI import 성공 뒤 다시 확인했다.

기존 집중 54건+신규 3건, **총 57건 PASS**. 관련 TypeScript 24건 PASS, `tsc --noEmit` PASS.
정확한 명령은 `focused-final-results.json`/`focused-summary.json`과 개별 로그에 있다.
새 enum으로 기존 검사 visitor에서 컴파일 누락이 검출되어 11개 기존 case의 순회만
ClipPath 자식까지 확장했다. 검사 기대값·baseline·허용치는 유지한다. case 수정 뒤
suite manifest를 다시 생성하지 않은 첫 실행의 0 tests는 제외했으며, 최종 stable
manifest에서 실제 PASS 수를 확인했다. 마지막 test-only rustfmt 뒤에도 manifest를
재생성해 P5 실행 준비를 맞췄다. suite 생성물은 커밋하지 않는다.

## 남은 경계와 다음 절차

B19의 첫 IR/paint·rhwp 왕복 절편을 checkpoint로 남긴다. **B19/P4 전체 완료는 아니다.**
한컴 기본 표현의 대응과 외부 재저장 보존은 남아 있다. 일반 header/footer/footnote/master
및 caption의 sidecar 순회, 회전/resize·후처리 subtree 이동/캐시 좌표 소비도 이번 PDF
본문 경로의 통과로 대신하지 않는다. 특히 `layout.rs::translate_node_payload_y`와
`table_layout.rs::translate_render_subtree_y`의 후처리 이동은 새 절대 paint clip과 함께
움직이는지 추가 감사/보완해야 하는 코드 검토 경계다. 현재 실행에서 검출한 회귀로
보고하지 않고 B19 backlog에 명시했다.

P5 전체 Rust/lint/package·최신 upstream/devel 통합/최종 gate와 원격 제출은 이후다.
기존 사용자 판정과 구현이 같은 범위는 재사용하며, 실제 코드가 바뀐 경계만 새 head에서
다시 확인한다. remote push·PR 상태 변경은 하지 않았다.
