---
kind: guide
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-09
---

# 출력 백엔드 소비 경로 검증

조판·paint·글꼴·이미지 처리 변경이 실제 Canvas·SVG·Native PNG에서 같은 의미를 보존하는지
확인하는 작성자·reviewer 공통 절차다. 적용 범위는 변경한 값의 **실제 소비 경로**로 정한다.
단일 backend 수정이면 그 경로와 공통 결과를 공유하는 정상 대조군을 확인하고, 모든 backend나
전체 문서를 매번 전수 실행하지 않는다. 공통 producer를 바꿨다면 그 결과를 소비하는 제품 경로를
확인 대상에서 빼지 않는다. 미실행은 `미검증`이며 `비해당`은 호출 경로 근거가 필요하다.

최종 시각 판정·90% 기준·새 회귀 추가 조건은 [시각 검증 거버넌스](visual_verification_governance.md),
빌드·회귀 범위는 [로컬 검증 4.3](../pr_review/local_validation.md#43-변경-범위별-기본-검증)을 따른다.
이 절차는 해당 검증을 추가로 연결하며, 기존 필수 게이트를 대체하지 않는다.

## 1. 수정 전에 공통 결과와 소비자를 연결한다

PR 본문 또는 기존 작업 기록에 다음 사슬을 짧게 남긴다. 별도 보고서 양식을 늘리지 않는다.

`입력 속성·단위 → Document IR → 줄 구성·점유/원점 → PageLayerTree/PaintOp → backend 소비 → 최종 화면/파일`

- 줄 소속·쪽/셀 소유·기준선·점유 끝점·crop 기준·transform/clip·plane 순서는 공통 결과를
  소비해야 한다. backend가 문서 속성을 다시 추측하거나 임의 좌표 보정으로 결과를 맞추지 않는다.
- 글꼴은 요청 family 문자열만 공유해 끝내지 않는다. 측정의 face 또는 메트릭 근거, paint 후보·style,
  실제 glyph coverage, 사용할 font resource를 연결한다. 본문에서 선택한 공급 경로가 수식·폼·
  그림 안 텍스트 등 별도 paint에서도 전달되는지 확인한다. 같은 font manager만 받는 것은 충분하지 않다.
- 지원하지 않는 op/resource, glyph 누락과 fallback은 성공과 구분해 기록한다. 파일 생성·readiness·
  op 개수만으로 내용 보존을 판정하지 않는다. fallback은 기존 규칙에 따라 해소하고 실제 결과를 확인한다.
- 저장 LineSeg 재사용과 편집 후 재조판, Canvas 보충 메트릭과 portable 메트릭은 각각의 계약이다.
  같은 context에서는 공통 결과를 대조하고, 다른 context의 차이는 적용 규칙·범위·독립 기대값으로
  설명한다. export 중 context 전환과 finally 복원을 확인한다. 화면을 같게 만들려고 유효한
  보충 메트릭을 없애지 않는다. 전체 렌더러 이관은 이 절차의 선행 조건이 아니다.

## 2. 실제 산출 경로와 실행 신원을 고정한다

| 검증할 결과 | 실제 산출 경로 | 대신할 수 없는 증거 |
| --- | --- | --- |
| Studio Canvas2D | 실제 CanvasView/PageRenderer 화면: 본문·앞뒤 레이어·DOM 그림·비동기 paint | 명령 기록용 Canvas, flow canvas의 `toDataURL()`만으로 전체 화면 입증 |
| SVG | 지정 profile/context의 SVG와 공급 글꼴을 적용한 browser raster | SVG 문자열·텍스트 존재만으로 glyph 모양 입증 |
| Native PNG | `native-skia` 빌드의 CLI `export-png` | SVG를 PNG로 변환한 Visual Sweep 산출물 |
| CanvasKit | 실제 요청/선택 backend·surface와 준비된 Typeface/자원 | Canvas2D의 CSS face 성공, 숨겨진 다른 renderer의 출력 |
| 독립 한컴 일치 | 동일 원문·정상 Print PDF와 Native/fresh WASM Visual Sweep | backend끼리의 일치, CI 성공 |

기존 기록에 source/build SHA, 입력·PDF hash, 문서 revision, 쪽, profile, metric context,
OS/browser·backend·surface, zoom/DPR/DPI, 글꼴의 공급·선택 provenance를 연결한다.
font byte hash를 읽지 못했거나 실제 face가 모호하면 그대로 `미검증`으로 남긴다.
같은 FontFaceSet·family명·서버 포트는 동일한 실제 face/build의 증거가 아니다.

Rust/WASM 변경은 [Docker WASM 빌드](../dev_environment_guide.md#wasm-빌드)를 따른다.
worktree 루트 `pkg/`와 Studio/public의 JS·WASM hash를 확인한 뒤 새로고침한다.
직접 Studio 서버에 연결하는 진단은 `VITE_URL`을 명시하고 실제 초기화에 사용한 JS/WASM 응답의
hash를 빌드 기록과 대조한다. 초기화 뒤 별도 fetch만 확인하면 이미 열린 탭의 낡은 인스턴스를
입증하지 못하므로 새 탭/새로고침과 초기화 요청도 확인한다. 기본 포트의 과거 bundle로 연결되거나
build가 확인되지 않으면 현재 head 결과로 쓰지 않는다.
코드·빌드·출력 조건이 같고 기존 증거가 충분하면 재사용하며 동일 WASM을 다시 빌드하지 않는다.

## 3. 구조, paint, 독립 기준을 순서대로 비교한다

1. **구조:** 같은 입력·profile·context에서 쪽수, 줄/셀/개체 소속, 순서·누락·중복,
   기준선·원점·점유 끝점과 crop/transform/clip을 대조한다. producer와 소비 뒤의 덮어쓰기까지 따른다.
2. **paint:** 같은 쪽·영역의 실제 Canvas·SVG raster·Native PNG를 직접 연다. 별도 font 소비,
   glyph 네모·누락, 그림 crop·효과, 도형 화살촉·clip·plane과 수식/form을 변경 주장에 맞춰 확인한다.
3. **독립 기준:** 한컴 Print PDF와 수정 전후를 대조한다. 모든 backend가 같은 방식으로 틀릴 수
   있으므로 상호 일치만으로 올바른 조판을 확정하지 않는다. 편집 수정은 동일 편집 저장본을 사용한다.

고정 픽셀 차이는 후보 탐색이고 2px 실루엣은 래스터 차이를 구분하는 보조값이다.
둘의 분모·단위와 원값을 구분한다. 높은 점수로 글자 네모·누락·잘린 그림·잘못된 쪽 소속을
면제하지 않고, 미세한 색상 프린지만으로 줄 구성 결함을 단정하지 않는다.
profile 간 편집 가이드 차이도 문서 내용과 구분하며 차이를 숨기는 임의 마스킹은 하지 않는다.

## 4. 실제 호출 경계의 반례를 확인한다

| 변경한 계약 | 필요한 반례와 정상 대조군 |
| --- | --- |
| font 공급/선택 | 본문과 별도 수식 paint에 같은 한글; 유효한 custom face와 시스템 fallback; glyph 없는 후보와 있는 후보; style별 선택 |
| metric context/export | 보충 메트릭이 활성화되는 실제 입력과 비활성 입력; export 전/portable 중/복원 후 트리·쪽수·내용; 실패 뒤 복원 |
| crop/transform/clip | 자르지 않은 그림과 경계가 변하는 crop; 유효한 좌표 기준과 기준 누락; 원본 데이터→공통 source rect→최종 viewport |
| Studio 합성/갱신 | DOM 그림 또는 별도 plane이 있는 입력; 초기 안정화, zoom/DPR/resize 후와 원래 조건 복원; 편집 변경이면 부분 갱신·저장 재열기 |

실제로 변경한 계약에 해당하는 행을 사용한다. 예상값은 사양·독립 출력·정렬/소유 불변식에서 정하며
수정 구현의 계산값을 재인용하지 않는다. 내용 존재만 검사하고 위치·모양까지 검사했다고 보고하지 않는다.
최초 캡처와 안정화 후 캡처를 구분하고 이미지·글꼴 준비와 실제 backend를 확인한다.
Studio의 여러 페이지를 담은 parent 전체와 한 페이지를 혼동하지 않는다. 해당 page 사각형을
**브라우저 화면에서** 캡처해 겹쳐 그린 레이어·DOM도 포함한다.

## 5. 기존 도구로 시작하고 검출 범위를 기록한다

baseline 대표 입력은 [manifest](../../../scripts/renderer_baseline_manifest.json)를 사용한다.
변경 계약에 필요한 정상 대조군을 고르며 manifest에 없는 입력은 기존 기준을 복사한 로컬 manifest로
추가 조사한다. 허용치를 완화하거나 검사되지 않는 예외를 추가해 통과시키지 않는다.

```bash
# 검토 worktree의 저장소 루트. 공용 target과 해당 head의 Docker WASM을 먼저 준비한다.
export CARGO_TARGET_DIR="${rhwp_review_target_dir:?기본 작업공간의 공용 target을 먼저 고정하세요}"
python3 scripts/renderer_baseline.py \
  --scope representative --filter equation-inline \
  --browser-mode headless --profiles screen,print --canvaskit-surface software \
  --output output/pr-review/renderer-backend-check/equation-inline
```

headless Chrome 경로는 현재 host에서 확인한 `CHROME_PATH`로 지정한다. 하네스는 작업 checkout의
Studio를 별도 포트에 실행해 browser capture에 연결한다. 이 명령은 변경 경계의 **진단 시작 예시**이며
영향 페이지 전체 sweep이나 필요한 정상 대조군 실행을 대신하지 않는다.

| 현재 도구 | 확인하는 것 | 추가로 확인해야 하는 것 |
| --- | --- | --- |
| `renderer_baseline.py` | Native SVG/Skia와 Canvas2D/CanvasKit 화면, 준비 상태·선택 backend 등 | 모든 입력의 Canvas2D↔SVG↔Native PNG 의미 보존을 자동 판정하지 않음. report-only parity와 hard failure를 분리해 읽음 |
| `visual_sweep.py` | Native/fresh WASM **SVG → browser PNG**와 독립 PDF | Studio Canvas·Native PNG의 paint 검증을 별도로 수행 |
| trace/null/adapter 계약 검사 | op·생명주기·capability·구조 | 실제 glyph/이미지/합성 화면의 정확성 |

자동 출력 신원 검사·glyph coverage·공통 font/crop 결과의 소비 검사는 수정 범위에 맞춰 제품 진입점의
회귀로 보강한다. **아직 추가하지 않은 자동 검사를 CI gate가 있는 것처럼 보고하지 않는다.**
회귀는 수정 전 의도한 원인으로 FAIL, 수정 후 PASS와 정상 대조군을 확인한다.
새 렌더링 회귀/fixture/golden의 추가 전 시각 조건은 거버넌스를 따르며 현재 실패를 기준값에 고정하지 않는다.

### 구현된 소비 경계 회귀

아래 검사는 #7688에서 독립 Print와 수정 전 FAIL / 수정 후 PASS를 확인한 경계다.
변경한 소비 경로에 해당하는 검사를 실행하며, 새 문서·다른 형상·모든 OS의 시각 일치를 대신하지 않는다.

| 계약 | 정식 검사 | 직접 확인하는 것 |
| --- | --- | --- |
| 그림 crop | [`issue_7688_picture_crop`](../../../tests/cases/issue_7688_picture_crop.rs) | 공개 CLI SVG의 두 그림 source 선택과 내용 순서, screen/print |
| 화살촉 | [`issue_7688_arrow_heads`](../../../tests/cases/issue_7688_arrow_heads.rs) | 공개 SVG의 intrinsic 크기·끝점 소유와 실제 Native PNG의 shaft 밖 화살촉, screen/print |
| CanvasKit font 준비·style | [`canvaskit-font-preparation-issue7688`](../../../rhwp-studio/e2e/canvaskit-font-preparation-issue7688.test.mjs) | 실제 Studio 준비·자원 실패 fallback, 두 face의 공급 순서와 실제 regular/bold outline 소비 |

Rust 두 검사는 `node scripts/run-rust-test.mjs <case> -- --cargo-profile release-test
--target-dir <공용 target>`로 실행한다. 실제 PNG 검사는 `--features native-skia`를 추가해야 한다.
Studio 검사는 Docker WASM을 반영한 서버의 `VITE_URL`과 확인한 `CHROME_PATH`를 지정하고
저장소 루트에서 `node rhwp-studio/e2e/canvaskit-font-preparation-issue7688.test.mjs --mode=headless`로
실행한다. 회귀가 성공해도 영향 입력의 Visual Sweep과 실제 화면 확인을 생략하지 않는다.

## 6. 제출·review·후속 처리

기존 PR의 구현 주장별로 `소비 경로 → 독립 기대값 → 검사 항목 → 수정 전후 결과 → 대표 이미지`를
연결한다. 판정은 `충족 / 미충족 / 미검증 / 비해당`이며 실행 결함·코드 검토상 우려·의도된 계약 차이를
구분한다. 영향받는 소비 경로의 알려진 내용 손실이나 필수 근거 미검증을 다른 backend 성공으로 상쇄하지 않는다.
필수 검증 범위를 새 head에서 충족한 뒤 제출하며, backend 확인만으로 PDF sweep을 면제하지 않는다.

수정 후에는 바뀐 공통 결과와 모든 해당 소비자가 이를 사용하는지 코드와 실제 출력으로 다시 확인한다.
연결 이슈의 종료 표현도 해결 범위에 맞춘다. 새로운 소비 경로나 fallback을 추가할 때는 이 절차를 다시
적용한다. 원격 게시·push·merge는 기존 승인 절차를 따른다.

관련: [폰트 사고 대응](../font_incident_response.md), [공통 조판 원칙](../../../AGENTS.md#조판-수정과-검토-원칙),
[RenderBackend 계약](../../tech/render_backend.md), [#7688 조사](../../working/task_m100_7688_stage1.md).
`src/render_backend/` 어댑터의 생명주기·capability 계약과 실제 제품 paint 검증은 서로 다른 증거다.
