# #7998 공유 변의 굵기 충돌과 실제 셀 템플릿

- 시작 SHA `7eff6b0a1`, 다음 절편 진행 승인에 따른 R1 범위다.
- native `Table::insert_row/insert_column`이 실제 선택하는 이웃 템플릿을 공통 결과로
  전달한다. PDF 경계에서 별도로 가장 가까운 셀을 추측하지 않는다. 같은 불투명 실선·색의
  공유 변은 기존 `border_rendering::merge_border`처럼 더 굵은 pen을 사용한다.
- 보이는 공유 변만 다시 읽으면 충돌에서 진 셀의 원래 폭을 잃어 후속 삭제가 잘못된다.
  셀 자체 폭과 실제 공유 변 폭을 구분하고, 필요한 경우만 bounded ownership sidecar에
  원래 폭을 보존한다. HWP/HWPX 재열기·반복 편집까지 같은 계약으로 검증한다.
- 원본 연속성·같은 선의 interval별 동일 pen·원점·paint 순서 증명과 원자적 거절을
  유지한다. 일반 dash/alpha·다른 색·병합/지역 축·R2/P5는 이번 완료 범위가 아니다.
- 증거는 ignored `output/pr-review/issue7998/p4-border-template-conflicts/`다.
  독립 표준 셀 속성의 충돌 대조군과 양 형식 Print, Native/fresh WASM 전쪽·직접 판독을
  먼저 확인한 뒤 정식 회귀를 등록한다. 전체 suite와 원격 반영은 실행하지 않는다.
  `parser_allocation_bounds_proven=false`를 유지한다.

## 구현과 작은 경계

- `Table::insert_row_inheriting/insert_column_inheriting`은 native `new_from_template`이
  실제 선택한 셀을 observer에 전달한다. 기존 공개 삽입 API는 같은 본체를 no-op observer로
  호출한다. `StructureEdit::destination`은 생존 셀 좌표만 이동하며 템플릿을 추측하지 않는다.
- source ink → `Proof::complete`의 물리 pen → 셀 자체 pen과 실제 템플릿 → 공유 변의
  같은 색 불투명 합집합 → `Bands::apply`의 안/바깥 반쪽 → RenderTree와 양 저장으로
  연결한다. 같은 축의 각 interval이 같은 pen인지 계속 증명한다. 줄 구성·높이 측정·
  pagination·이어받기 알고리즘은 바꾸지 않으며 기존 Table의 크기 갱신을 소비한다.
- 보이는 폭보다 작은 셀 자체 pen만 `CellBorderPaint.cell_pen`에 남긴다. 필요한 문서에만
  bounded RBPO v5의 레코드당 4바이트를 추가한다. v1–v4와 최대 4MiB 예산을 유지하고,
  가려진 폭이 없는 기존 대조군의 저장 바이트는 그대로다. 재소비 시 셀 자체 폭들의 합집합이
  실제 원본 경계를 설명해야 하며, 메타데이터로 더 큰 잉크를 승인하지 않는다.
- 수정 전 보존된 stage24 WASM의 두 축 편집이 의도한 경계 띠 오류로 무변경 거절됐다
  (`before-wasm.json`). 첫 진단은 오류 문구를 일반 PDF 경계로 예상해 assertion이 실패했다.
  실제 경계 띠 오류를 기록해 바로잡았고 제품 수정 전의 실패 원인은 동일하다.
- 현재 Native의 앞/안/끝 삽입·첫/마지막 삭제·충돌 후 원래 이웃 삭제와 양 저장·재열기·
  Snapshot 복원이 통과했다(`probe.log`, `append-probe.log`). 끝 삽입은 실제 템플릿이
  앞 이웃으로 fallback하는 경계이며 같은 빌드에 새 4개 저장·인쇄만 추가했다.
- 표준 BorderFill 0.4/0.2mm의 별도 대조군 6개를 실제 한컴에서 인쇄했다. 같은 공유 변에
  삽입 뒤 0.60pt/1.08pt가 함께 그려지고, 원래 이웃 삭제 뒤 0.60pt만 남는 것을 PDF의
  실제 path에서 확인했다(`independent-shared-edge-rule.json`). 표준 폭을 PDF의 임의
  source pen으로 바꿔 쓰거나 화면을 맞추는 근거로 사용하지 않는다.

## 현재 검증

- Native SHA-256 `74c9e837b5cc884d5d67218ff176818b13c65ca709cce0da1aa393f10c919f7e`,
  fresh Docker WASM 19,650,162바이트 SHA-256은
  `ea0bb1d816977d2c416b93b8ce930baf0e15dbb0935b0b60009a2a18a55deeda`다.
  glue는 이전과 같은 `5671802fbb89ed060c3bab7a6f458a78833a7e5ca3e7df8e1699d45098cddeb6`다.
  Native·Docker의 여섯 제품 source와 pkg/Studio public/보존 pkg의 바이트를 확인했다
  (`source-runtime.json`). 이후 제품 변경 없이 검증을 계속하며 빌드를 반복하지 않는다.
- 새 편집 24개(HWP 12·HWPX 12)와 표준 속성 6개 모두 실제 한컴 2020 11.0.0.9136의
  Print method 0·한 쪽씩 driver 경로로 인쇄했다(`print-provenance.json`). 각 입력·PDF SHA와
  1쪽을 확인했다. 이 단계의 자동 서비스는 정상 동작하며 수동 변환을 요청하지 않았다.
- Native 59쪽은 전쪽 90% 이상이다. 실제 새 Studio SnapshotCommand 12종의 문자
  삽입/삭제·undo/redo·양 저장·재열기, 저장 후 원래 이웃 삭제 4종도 통과했다. 새 저장본
  24개는 Native와 바이트 같다. 실제 마우스 클릭은 중첩 A/B/C/D 셀의 빈 그림 0개·2단계
  cellPath로 들어가며 실제 내용 그림은 선택 가능하다(`click-after.json`).
- 현재 source에서 기존 평행/일정 pen 대조군의 저장본 26개와 Native/fresh WASM PNG
  각각 26개는 stage24 수용본과 바이트 동일하다. 이전 제품의 캡처를 새 실행으로 바꾸어
  세지 않으며 최신 devel/P5 비교를 대신하지 않는다.
- 기존 집중 7모듈 58건이 통과했다. 이번에 지원하는 축까지 거절하도록 요구한 기존 2건은
  제외 상태이며, 최종 시각 선행 검증 후 방향별 수용/거절과 저장 재편집 계약으로 갱신한다.
  실제 WASM에서도 같은 선의 interval별 변화 6종·유효한 source의 잘못된 인덱스 2종이
  무변경 거절됐다(`reject-studio.json`).
- uniform Studio 보조 스크립트는 7종 실행 뒤 footer의 보존 pkg 경로가 잘못돼 기록 단계가
  실패했다. 제품 변경 없이 그 경로만 바로잡아 runtime 신원·7종 기록을 완성했다. Native
  캐시 재사용 요약도 마지막 새 항목 뒤 생략된 기존 행이 파일에 기록되지 않는 문제를
  바로잡았으며, 전체 59개의 같은 입력·PDF·producer를 확인하고 캡처는 반복하지 않았다.
  진단 코드의 실패를 제품 회귀로 계산하지 않는다.
- 위 중간 상태의 전쪽 비교와 기록을 완료했다. 기존 R2 직접 문자 역삭제의 배치 차이를
  이번 절편의 완료로 표시하지 않는다.

## 최종 전쪽·직접 판독과 메인테이너 판정

- Native/fresh Docker WASM 각각 **59입력·59쪽**, raw 최저 **97.45731% / 96.19767%**다.
  새 구조 편집의 양 저장 24쪽 최저는 **98.66183% / 96.69192%**다. 원본 3쪽·기존 대조군
  26쪽·새 편집 24쪽·표준 속성 6쪽이며 미달·누락·쪽수 차이 0이다
  (`final-validation-summary.json`). 마지막 작은 추가의 요약 갱신도 동일 입력/producer
  캐시만 읽었으며 이미 성공한 캡처·빌드를 다시 실행하지 않았다.
- 독립 Print와 Native/fresh WASM의 같은 좌표에서 삽입·충돌·삭제 후 외곽/공유 변,
  자식 표·앞뒤 문장·빈 새 셀을 직접 확대 판독했다. 폭의 복귀와 위치/내용 관계는 일치한다.
  기존 Helvetica 대체 글꼴과 가는 선/공유 선 농도 차이는 2배/3배 확대에 공개했다.
  compare·standalone overlay·review는 각 backend 6대표 쪽이다(`reviews/`, `zoom-check/`).
- 실제 새 Studio 12종과 기존 평행 6종·일정 pen 7종이 SnapshotCommand·문자 편집·
  undo/redo·양 저장·재열기를 통과했다. 새 저장본 24개와 기존 26개는 Native 바이트와 같다.
  재열기 뒤 원래 이웃 삭제 4종도 Native와 같은 저장 바이트이며 undo/redo를 확인했다.
- 정식 기존 자동 수집 파일에 새 충돌·가려진 pen 보존과 유효하지 않은 v5 pen의 무변경
  거절 검사 2개를 연결했다. 지원하는 동작을 거절하도록 요구하던 두 기존 검사는 유효한
  인덱스/같은 선의 interval 변화 거절로 남기고 지원 동작을 새 긍정 검사로 연결했다.
  새로운 px 핀이나 전체 SVG 해시를 고정하지 않았다. 첫 검사 컴파일은 Serialize 대상이
  아닌 Section을 사용해 실패했으며 실제 그리기 노드를 포함하는 Paragraph 목록으로
  바로잡았다. 제품 코드·기대 폭·오차는 변경하지 않았다. 최종 기존 7모듈은
  **62 passed / 0 failed / 0 filtered**다(`guards-final.log`). 전체 suite/CI는 실행하지 않았다.
- 독립 Print 30개와 표준 속성 원본 HWPX 6개를 공개 fixture에 보존했다. 정확한 입력·PDF
  SHA·현재 runtime·인쇄 방식·전쪽 최저·수용 판정은 `print/conflict-provenance.json`에 있다.
  신규 지원·수정된 회귀 모두 전체 시각 선행 검증과 직접 확대 뒤에만 등록했다.
- 이동 가능한 검토 화면은 `E:\tmp\issue7998-border-conflict-review\index.html`이다.
  Studio 7793의 `?review=issue7998-conflict-{vertical-before,horizontal-before,
  vertical-restored,horizontal-restored}&lang=ko` 네 새 탭에서 실제 최종 WASM SHA·1쪽을
  확인했다. before의 자식은 4셀/빈 그림 0개이며 restored는 원래 첫 열/행을 삭제했으므로
  그 안의 자식도 함께 삭제된다. 기존 사용자 탭과 WIP는 보존했다.

2026-10-19 메인테이너가 **“시각·편집 구조 통과”**로 판정했다.
`maintainer-verdict.json`에 같은 source/runtime·새 입력 SHA·정식 검사 62건·전쪽 결과를
연결했다. 검증 뒤 제품 source는 그대로이며 통과한 빌드/캡처를 재실행하지 않는다.
현재 절편을 같은 로컬 단계 커밋으로 보존한다. 같은 선 길이 방향 pen 변화·다른 색/스타일·
dash/alpha·병합/지역 축·미증명 원점의 일반 상속, R2와 P5는 남는다.
`parser_allocation_bounds_proven=false`이며 원격 push·PR 변경은 하지 않는다.
