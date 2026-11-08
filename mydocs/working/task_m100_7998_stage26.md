# #7998 셀 구간별 실선 굵기의 구조 편집

- 시작 SHA `47f9f57b7`, 다음 절편 진행 승인에 따른 R1 범위다.
- 같은 물리 직선도 셀 경계에서 pen이 달라질 수 있다. 원본은 각 실제 셀 변의 구간을
  증명하지만, 이후 선 전체의 한 pen으로 축약하는 조건이 정상 입력을 거절한다.
  source의 각 구간 → 셀 자체 pen → 실제 native 템플릿 → 구간별 공유 변의 합집합 →
  안/바깥 반쪽 → 최종 paint·양 저장을 공통 결과로 연결한다.
- 같은 색·불투명·연속 실선·완전한 직사각 격자·증명된 원점/순서를 유지한다. 셀 변 안에서
  pen이 바뀌거나 부분 선·틈·dash/alpha·다른 색·병합/지역 축·미증명 원점은 별도다.
  저장 v5의 가려진 셀 pen 계약을 재사용하며 새로운 저장 형식으로 확장하지 않는다.
- 기존 외곽 세로 구간 입력과 독립 가로/공유 변 대조군으로 두 축의 삽입·삭제·끝 삽입,
  양 저장·재열기·문자 편집과 Snapshot 복원을 확인한다. 단일 셀 변 안의 변화는 반례다.
  Native/fresh WASM 전쪽 90%·독립 한컴 Print·직접 판독 뒤에만 정식 회귀/fixture를 등록한다.
- 증거는 ignored `output/pr-review/issue7998/p4-interval-border-inheritance/`다. 이전 stage25
  source/runtime은 보존하고, 변경되지 않은 입력·기준 PDF는 재사용한다. 전체 suite/최신 devel
  통합과 PR 제출은 P5이며 이번에 원격 변경은 하지 않는다.
  `parser_allocation_bounds_proven=false`를 유지한다.

## 실제 소비와 구현

- `border_paint_structure::Proof::complete`가 증명한 `(가로/세로, 경계, 셀 구간)`별
  pen을 `Bands`까지 유지한다. 선 전체를 하나로 축약하던 조건을 제거하고, 기존 1HU
  crop 불확실성 안의 정규화 순서·색·불투명·연속성·원점/paint 순서 증명은 유지한다.
  `physical_pens`의 같은 네 변을 셀 크기 수용과 실제 반쪽 paint 배치가 소비한다.
- 실제 native 템플릿이 선택한 셀 자체의 네 폭을 새 셀이 상속한다. `Bands::resolve`는
  같은 색 불투명 공유 변의 두 소유 폭을 합치며, 가려진 얇은 폭은 기존 v5 계약으로
  보존한다. 삽입/삭제 뒤에도 구간별 결과를 사용하고 내용·자식 표를 복제하지 않는다.
  줄 구성·측정·pagination/이어받기는 변경하지 않는다. 별도 cut/예약 경로는 비해당이다.

## 직접 대조에서 발견한 저장 손실

- 첫 후보는 전쪽 90% 조건을 만족했으나 HWP 저장본을 다시 열어 삽입한 첫 열을
  삭제한 HWPX의 한컴 Print에서 `OUTSIDE ABOVE`가 사라졌다. Native에는 남아 있었다.
  점수 통과를 내용 보존 증거로 대신하지 않고 첫 후보를 **미수용**으로 보존한다
  (`first-candidate-disposition.json`). 원본·첫 저장본·인쇄본을 덮어쓰지 않는다.
- HWP LIST_HEADER의 실제 네 0 여백 → 일반/그룹 도형 파서의 TextBox → HWPX
  `write_draw_text`의 `textMargin` 발행 → 한컴 배치로 추적했다. 파서가 0 여백의 존재를
  잃어 HWPX에서 속성이 생략됐다. [HWP 표 90](../tech/한글문서파일형식_5.0_revision1.3.md#표-90-그리기-개체-글상자용-텍스트-속성)의
  네 HWPUNIT16 여백과 기존 HWPX
  absent/explicit-zero 계약을 근거로, 실제 네 필드가 있는 경우만 explicit로 보존한다.
  HWPX writer나 좌표를 강제로 바꾸지 않는다.
- 누락된 0 여백 두 요소만 별도 ZIP/XML 진단본에 복원했을 때 한컴 Print에서 앞뒤
  문구가 모두 나타났다(`margin-diagnostic/`). 일반/그룹 파서의 존재 계약은 수정 전
  2 FAIL → 수정 후 2 PASS이며 기존 HWPX의 absent/explicit-zero 구별도 유지한다.
  이 제품 변경 뒤 최종 Native/fresh Docker WASM을 새로 빌드하며 첫 후보의 캡처를
  최종 head 결과로 계산하지 않는다.

## 최종 검증 진행

- 최종 Native로 16종 구조 편집·양 저장 32개와 재열기/삽입 선두 셀 삭제 4개를 만들었다.
  새 36개 중 HWP 경유 재저장 2개만 바이트가 달라 독립 한컴 Print를 새로 확보했다.
  나머지 34개와 기존 대조군 50개의 동일 입력은 증명 후 기존 기준을 재사용했다.
  새 36개 Print 모두 앞뒤 문구가 존재한다(`final/independent-content-check.json`).
- 최종 전쪽·실제 Studio·직접 확대 판독과 정식 회귀 등록을 완료했다. 메인테이너 수용과
  단계 커밋을 아래에 연결한다.

### 빌드와 실제 command

- 최종 Native SHA-256은 `86652870e8caf3fddc4db1230be93f560794d1f137a9cfc4bc3c62a2eb77410c`,
  fresh Docker WASM은 19,721,374바이트,
  `72a78d5aa5a552be1b05b0521c56b1c7fa55f1e6cd2301a8819f75c30b71def2`다.
  glue는 기존 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`다.
  Native/Docker의 제품 source 3개와 pkg·Studio public·보존 pkg의 바이트를 대조했다
  (`final/source-runtime.json`). 이후 제품 코드는 바꾸지 않고 같은 빌드를 사용한다.
- 공용 `target/pr-review`에서 `cargo build --locked --profile release-test --features
  native-skia --bin rhwp --lib`, canonical Docker `scripts/wasm-pack-locked.sh --target web
  --out-dir pkg`를 실행했다. jobs 14·WASM codegen-units 16이며 각 빌드는 2m45s/7m33s다.
  이슈별 Cargo target이나 별도 캐시를 만들지 않았다. 전체 suite·CI는 실행하지 않았다.
- 최종 Native 100입력·100쪽은 최저 97.45731%, 새 편집 36쪽은 최저 98.95186%다.
  기존 59쪽 PNG는 같은 backend의 stage25 수용본과 전부 같다
  (`final/native/compat-png-identities.json`). 최신 devel/P5 비교를 대신하지 않는다.
- 실제 새 Studio 16종·기존 일정 pen 7종/평행 pen 6종/충돌 12종이 SnapshotCommand,
  문자 삽입/삭제·undo/redo·양 저장/재열기를 통과했다. 내보낸 82개는 Native와 바이트 같다
  (`final/actual-identical-inputs.json`). HWP/HWPX 재열기 후 삽입한 선두 행/열 삭제 4종도
  같은 Native 저장본과 같고 undo/redo가 통과했다(`final/reopen-studio.json`).
- 실제 클릭은 중첩 A/B/C/D 셀의 빈 그림 0개·2단계 cellPath를 확인했다. 실제 내용 그림은
  계속 선택 가능하다(`final/click-after.json`). 단일 셀 변 내부의 두 변화 각 6종과 정상
  입력의 잘못된 인덱스 2종도 실제 WASM에서 원자 거절했다(`final/reject-studio.json`).
- 이전 stage25 source의 보존 WASM에서 같은 셀 구간 입력의 행/열 API가 의도한 PDF
  경계 오류로 무변경 거절된 증거는 `before-wasm.json`이다. 새 지원의 수정 전 실패이며
  빌드/환경 오류가 아니다. 글상자 0 여백은 별도 `margin-contract-before.log`의 일반/그룹
  2 FAIL → 최종 `final/margin_contract-run.log`의 2 PASS로 원인을 분리했다.

## 전쪽 검증과 메인테이너 수용

- Native/fresh Docker WASM 각각 **100입력·100쪽**, raw 최저는 **97.45731% / 96.19767%**다.
  새 편집 36쪽 최저는 **98.95186% / 96.69192%**다. 기존 59쪽·새 원본 5쪽·새 편집
  36쪽이며 미달·누락·전체 쪽수 차이 0이다(`final/final-validation-summary.json`).
  같은 backend의 기존 대조군 59쪽 PNG는 양쪽 모두 stage25 수용본과 바이트 같다.
- WASM 전쪽 실행은 완료한 43개를 보존한 뒤 나머지 57개를 두 독립 작업으로 나눴다.
  마지막 실행 중 캡처의 exit status 0을 확인하고 대기 중인 부모 orchestration을 종료했다.
  부모의 137 종료는 제품/캡처 실패가 아니다. input/PDF/producer 해시와 성공한 전체
  산출물을 대조해 재개했으며 성공한 캡처는 반복하지 않았다(`final/parallel-resume.json`,
  `final/wasm-parallel-validation.log`). 각 작업의 별도 summary와 최종 100행 원본을 보존했다.
- 각 backend 6대표 쪽의 compare·standalone overlay·review를 만들고, 같은 물리 좌표의
  외곽/공유 변과 굵기 변경 접점·자식 표·앞뒤 문구·삽입/삭제 후 배치를 직접 확대 판독했다.
  상단 문구는 새 독립 인쇄에서도 복구됐다. 기존 Helvetica의 한컴 대체 글꼴과 가는 선/
  공유 선 농도 차이는 확대에 명시하며 해결 완료로 표시하지 않는다(`final/reviews/`).
- 기존 자동 수집 test source에 새 구간 상속·셀 자체 pen 복원 검사 2개를 연결했다.
  지원 입력을 거절하도록 요구한 기존 검사는 단일 셀 변 내부의 가로/세로 변화에 대한
  12종 원자 거절로 갱신했다. 새 셀의 빈 내용, 원문 소속/순서·앞뒤 문구의 유일 보존,
  양 저장 후 반복 편집·자식 표·Snapshot 복원을 검사한다. source PDF의 선언 pen과
  독립 한컴 공유 변 규칙이 기대값이며, 기존 1HU crop 불확실성을 유지한다. 새 px 핀이나
  전체 SVG 해시를 고정하지 않았다. 기존 일반/그룹 HWP 파서 계약에 명시적 0 여백을
  보완했고 HWPX absent/explicit-zero 검사도 유지했다. 최종 **9모듈 68 passed / 0 failed /
  0 filtered**다(`final/guards-final.log`). 전체 suite·CI를 실행한 결과로 확대하지 않는다.
  대상 원본의 자동 수집 참조는 이 기존 test source에만 있으며 기존 검사도 source pen·
  내용/소유·관계 검사다. 실제 출력의 절대 px 핀을 교체하거나 허용치를 완화한 범위는 없다.
- 공개 생성기의 기존 6개와 신규 5개 모두 별도 폴더에서 바이트까지 재현했다
  (`final/generator-reproduction.json`). 새 5개 원본과 양 형식의 독립 Print 36개,
  `pdf/issue7998/retained-border-structure/print/interval-provenance.json`을 연결했다.
  새 렌더링 fixture/검사 등록은 두 backend의 전체 90%·직접 판독 뒤에만 수행했다.
- 이동 가능한 비교 화면은 `E:\tmp\issue7998-interval-border-review\index.html`이다.
  Studio 7793의 `?review=issue7998-interval-{v-outer-row-before,h-outer-col-before,
  v-shared-col-before,h-shared-row-before,reopened-v-shared-from-hwp}&lang=ko` 다섯 새
  탭에 실제 최종 WASM SHA·1쪽·자식 4셀/빈 그림 0개를 확인했다. 기존 탭/WIP는 보존했다.
  마지막 탭은 HWP 재열기 뒤 삽입한 첫 열을 삭제하고 HWPX로 저장한 수정본이다.

2026-10-19 메인테이너가 **“시각·편집 구조 통과”**로 판정했다. `final/maintainer-verdict.json`에
동일 source/runtime·새 입력 SHA·정식 68건·전쪽 결과를 연결하고 현재 절편을 로컬 단계
커밋으로 보존한다. 검증 뒤 제품 source는 그대로이며 새 빌드/캡처를 반복하지 않는다.
다른 색/스타일·dash/alpha·병합/지역 축·미증명 원점, R2/P5와
`parser_allocation_bounds_proven=false`는 유지한다. 원격 push·PR 변경은 하지 않았다.
