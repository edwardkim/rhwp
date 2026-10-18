# #7998 P4 네 번째 절편 — 실제 셀 테두리와 행·열 편집

- 시작 checkpoint: `cd6c995d399b3a2460b483b6642ee5d38ed3840f`.
- 승인: 2026-10-18 B14 통과 뒤 다음 절편 진행 승인.
- 이 보고서의 stage5는 실행 기록 번호이며 최종 통합 P5와 구분한다.
- 선행 [P4 기록](task_m100_7998_stage4.md)의 source·빌드·직접 판정은 보존한다.

## 이번 절편의 규칙·원인 계층과 반례

같은 family에 periodic 경계가 하나라도 있으면 현재 lowering은 연속 실선인 다른 표도
`BorderFill=None`으로 생성하고 셀 내부의 원본 drawing으로만 경계를 그린다.
원래 그림은 가깝지만 일반 테두리 편집은 그 drawing을 변경하지 않고, 새 행·열의 빈 셀은
내용 그림을 복제하지 않으므로 상속할 테두리가 없다. 실선 경계의 출처/coverage·스타일을
실제 Cell BorderFill로 발행할 수 있는지 먼저 대조한다.

이번 절편은 **실제 셀의 변별 연속성·단일 스타일이 입증되는 경계**의 native 경계와
그 행·열 삽입/삭제·테두리 변경·양 저장 형식에 집중한다. 실제 빈틈·혼합
스타일·bitmap/alpha·복합 경계는 원본 paint 소유를 유지한다. 픽셀 형태만으로
그림을 테두리로 지우지 않는다. background와 내용 그림, 중첩 표의 별도 경계는 보존한다.

생산 경로는 `structured_lowering::TableBuilder`의 Cell BorderFill 및 원본 path 소유 →
`table_paint::partition`의 실제 조각 배분 → 표 renderer의 Cell BorderFill이다.
행·열 편집은 `table_ops` → `Table::insert_row/insert_column` →
`Cell::new_from_template`의 테두리 상속을 소비한다. 경계를 native 속성으로 발행했으면
원래 path의 같은 소유 조각은 중복 paint하지 않아야 하며 나머지 path/셀 소유는 유지한다.

작은 실선/periodic 혼합 family·독립 새 행/열의 빈 영역·테두리 색상과 양 저장 형식부터
진단한다. 독립 PDF·한컴 Print와 Native/fresh Docker WASM 전쪽·직접 판독 선행 조건 뒤
정식 회귀를 추가/변경한다. 비표준 경계 스타일의 일반 편집, 병합/지역 격자의 남은 근거,
원래 글꼴 차이와 P5 최종 검증·원격 반영은 남는다.

## 실행으로 확인한 저장·소비 경계

`probe-before.log`와 실제 이전 WASM `before-studio.json`에서 실선 부모도 네 변이
`None`이며, 삽입된 빈 행·열도 같은 상태임을 확인했다. 이후 native 후보는 연속 실선
부모/자식의 실제 `BorderFill`을 발행하고 source path의 같은 소유 조각만 native 경계로
소비한다. 내용 그림·background·다른 표의 source paint는 원래 소유를 유지한다.
최초 시도는 표 전체의 네 변을 모두 요구했으나 아래 실제 한컴 소비 경계 때문에 변별 표현으로 나눴다.

최초 Native 대조군은 99% 이상이고 실제 API 구조 검사는 통과했으나, 독립 한컴 Print의
열 삽입은 87.80243%였다. 원인을 나눠 원본/모델/실제 인쇄를 대조했다.

1. 원본 MediaBox는 400×360pt인데, PDF lowering은 그 값을 세로 용지의 raw 폭/높이에
   그대로 넣었다. 한컴 Print는 360×399pt로 정규화해 오른쪽을 잘랐다. PaperDef의
   세로 기준 폭/높이와 landscape를 일관되게 만든 진단본은 399×360pt로 정상 인쇄됐다.
   `PageDef` → `PageArea::from_page_def_for_page`/renderer page layout → HWP/HWPX
   serializer의 동일 방향 계약을 적용한다. 특정 표/문서 폭을 clamp하지 않는다.
2. 셀 소유 source paint는 `Column/Para`, `BehindText`인데 `flowWithText=0`이었다.
   한컴에서 중첩 셀을 따르지 않아 경계가 쪽 원점 근처로 나갔다. 동일 진단본에
   `flowWithText=1`만 추가하자 실제 인쇄에서 중첩 표 경계가 복원됐다.
   `paper-2020.pdf`와 `paper-cell-flow-2020.pdf`, `anchor-diagnostics.png`를 보존했다.
   이제 `table_paint::anchor`가 셀 소유와 문단을 따르는 속성을 함께 발행한다.
   그림을 본문 흐름 개체로 바꾸거나 원래 그림 크기/원문을 줄이지 않는다.
3. 같은 두께·종류에서 이웃 셀의 색이 다르면 Native는 먼저 방문한 셀을 선택했다.
   독립 2×2 인쇄 입력에서 위/아래와 좌/우의 빨강·파랑을 서로 뒤집어 대조했다.
   한컴은 오른쪽 셀의 왼쪽 변/아래 셀의 위 변을 선택했다. 공통 edge collector는
   이 기하 소유를 tie 조건에 반영하며 셀 벡터 순서와 분리한다. 두께/서로 다른 종류의
   기존 우선순위는 유지한다. `border-tie.hwpx`, `border-tie-2020.pdf`가 독립 근거다.

`hp:pos`의 음수 offset을 signed 문자열로 바꾼 진단도 별도로 인쇄했으나 경계를 복원하지
못했다. 로컬 OWPML `ParaList XML schema.xml`은 해당 속성도 nonNegativeInteger로
정의하며 기존 wraparound 인코딩을 유지한다. 이 진단을 serializer 수정의 근거로 쓰지
않았고 제품 저장기에 부호 문자열 변경이나 0 clamp를 넣지 않았다.

초기 Native 219쪽·WASM package와 첫 인쇄 입력/결과는 `initial-*`로 구분해 보존했다.
이들을 저장 속성·edge 수정 후 최종 head의 증거로 재사용하지 않는다. 첫 WASM은 정상
빌드됐지만 후속 실제 인쇄에서 드러난 보정 이전 package이므로 최종 증거에서 제외한다.
같은 소스의 재검증이 아니라 위 실행으로 확인한 원인 보정 뒤 영향 경계 재검증이다.


### 셀 경계와 내용 점유의 구분

두 번째 한컴 대조에서 미달은 87.80%의 우측 잘림에서, 그림이 복원된 뒤의
89.18592%로 바뀌었다. 실제 한컴 2020 HWP 재저장본을 받아 같은 입력을 대조했다.
부모/자식 셀의 저장 높이와 padding은 유지됐지만, 자식 Table의 common.height가
2800→3280 HU, 그림 host LineSeg가 2800→3280 HU로 늘었다. 자식의 두 행 각각의
위 padding은 240 HU이며, 480 HU 차이와 대응한다. 같은 크기의 전체 셀 세로 경계를
내용 그림으로 옮겨 문단을 따라가게 하면 한컴의 셀 내용 요구 높이에 들어가는 문제다.
`hancom-metrics.log`, `solid-parent-hancom-resaved.hwp`, `restored-diagnosis.png`를 보존했다.

경계 그림을 축소하거나 셀 여백을 제거하지 않는다. 실제 변별로 입증된 연속 실선은
BorderFill로 발행하고 불연속 구간만 원본 paint로 보존한다. 같은 source primitive가
여러 셀/서로 다른 span의 변을 지나면 모든 해당 변의 근거가 함께 충족돼야 native로
소비한다. 불충분한 공유 변을 반복 제거하는 단조 closure와 명시적 work 예산을 사용한다.
실제 paint 분할도 셀별 source primitive 소유를 소비한다. 짧은 변의 native 선과 긴
이웃 변의 보존 paint가 중복되는 상황을 허용하지 않는다. 비표준 경계의 스타일 변경
일반화는 이번 연속 실선 구현의 완료 범위에 포함하지 않는다.

## 최종 소비 경로와 확인 범위

| 구현 주장 | 실제 적용 경로·독립 기대와 실행 증거 |
|---|---|
| 실선의 실제 셀 속성 | `structured_lowering/native_borders::plan`은 원본 MoveTo/LineTo·불투명 butt stroke의 출처와 변별 coverage·동일 색/두께를 요구한다. `structured_lowering::lower`는 partition 성공 후 해당 Cell의 BorderFill을 발행한다. 실제 틈·뒤에서 가리는 도형·dash/alpha/bitmap은 실선으로 채우지 않는다. 원래 Studio의 부모/새 행·열은 네 변 None, 보정 후 실제 Solid 속성이다(`before-studio.json`, `studio-result.json`). |
| 유일한 paint 소유 | `table_paint::partition`은 원본 조각을 기존 물리 셀에서 빼되, 그 셀의 입증된 native 변에 해당하는 조각만 drawing에서 제외한다. 다른 셀·내용 그림·background는 유지한다. 원본 0.5pt stroke extent는 native centerline과 보존 drawing의 합으로 동일하며 8개 중첩 family에서 독립 선언 길이와 일치한다(`normalized-area-sides.log`). |
| 내용 높이와 border 분리 | `table_paint::anchor`의 셀/문단 추종 속성과 실제 native 외곽선이 저장된다. 최종 한컴 HWP 재저장에서도 자식 높이는 원래 2800 HU이며, padding을 지워 맞추지 않았다(`hancom-fixed-metrics.log`). |
| 편집·측정·배치·저장 | 실제 `insert_table_row/column_native` → `table_ops` → template Cell의 BorderFill 상속 → `build_page_render_tree`의 cell border collector → 공통 paint backend를 소비한다. 삭제는 실제 inverse API다. 한컴 Print와 실제 Native/Studio API의 HWP/HWPX 저장·재열기에서 새 셀, 자식 표, 원문 순서·셀 내부 포함·표 뒤 본문을 대조한다. |
| 공유 edge 색상 | `collect_cell_borders`의 top/left 소유를 `merge_edge_slot` → `merge_border`의 같은 두께/같은 종류 tie에 전달한다. 다른 종류·두께 우선순위와 투명 선 guide는 유지한다. 독립 checkerboard Print와 같은 입력에서 이전 CLI의 가로·세로 공유 4개 edge가 반대 색으로 FAIL, 보정 후 모두 PASS다(`tie-before-after-svg.json`). 셀 벡터 역순·양 저장 재열기도 정식 검사에 포함한다. |
| 실제 용지 | PDF MediaBox → portrait 기준 PageDef + landscape → `PageArea`/page layout의 물리 폭·높이 → serializer를 잇는다. 원본·편집본의 한컴 출력 오른쪽 내용과 한 쪽 보존을 확인한다. |

opacity/complex path의 일반 BorderFill 대체, unequal/merged span의 closure 적용, 다중 쪽·
이어받기 표의 edge tie는 이번 독립 합성·편집 증거만으로 검증 완료라고 하지 않는다.
적용 예산을 초과하면 기존 paint 보존으로 돌아가며 무제한 추론하지 않는다.

### 추가 인쇄로 확인한 source paint 보존 경로의 미달

정식 fixture 최종 등록 전 periodic/occluded 부모의 무편집 source HWPX도 한컴 2020으로
인쇄했다. 입력은 이번 실제 Native가 생성한 원본 저장본이며 수동 줄 정보로 보정하지 않았다.
`negative-jobs.json`과 대응 source start/status/download에 원본·실행을 보존한다.

| 입력 × HWP/HWPX | Native / fresh WASM 최저 | 판정 |
|---|---:|---|
| periodic 부모 source Print | 95.04154% / 95.02341% | 직접 비교에 위치·글꼴 차이가 남음. 연속 실선 편집의 통과와 구분한다. |
| 가림 도형을 가진 occluded 부모 source Print | **55.77974% / 55.34269%** | 부모 선 누락·위치 차이 확인, 미충족. |

직접 PDF 기준의 219쪽 통과는 이 한컴 재소비의 통과가 아니다. 가림 때문에 root 경계를
그림으로 유지하는 보수적 모델 판단은 확인했지만, 그 그림의 한컴 anchor/내용 점유와
저장 배치는 완성되지 않았다. 이 경로를 임의 clamp·선 숨김으로 통과시키지 않는다.
이 두 진단 원본·저장본·Print·TSV는 로컬에 보존하고 새 정식 fixture/회귀 등록을 보류했다.
생성기는 계속 이 반례를 재현할 수 있다. 통과한 실선 부모와 틈이 있는 자식·실제 편집
3건만 정식 등록했으며 기존 회귀의 기대값을 삭제하거나 완화한 것은 아니다.
B16에서 원인·실제 소비를 이어 확인하고 최종 PR gate는 계속 보류한다.

## 최종 producer와 실행·시각 판정

작업 기준 commit은 `cd6c995d399b3a2460b483b6642ee5d38ed3840f`이며 아래 제품 source
지문으로 코드/빌드를 고정했다. 이후 정식 테스트·증거 문서만 추가/정리했고 제품 소스
2,445개가 같은 바이트인지 재확인했다. 문서 commit SHA와 제품 source 지문을 구분한다.

- source manifest: `source-runtime.json`, SHA-256
  `5181ae155956d5edded4328fe29b8da57a411d1676e902a5e5537bd821ae2289`.
- Native: `native-build-side-ownership.log` exit 0, `rhwp-candidate` SHA-256
  `dcccacc7f4c0d9aa77efa6614242616d2c79a56abb83197b1a7fe4d2ddcf8051`.
- fresh Docker WASM: `docker-wasm-side-ownership.log` exit 0 / 완료 후 복사,
  `wasm-build-completion.json`; 19,212,130 bytes, SHA-256
  `4a6fe031240d49d59efd0a64e1149e54a285e999b1fc2cf27ad77b83f8d37022`.
  실제 Studio 7793에서 로드한 응답의 바이트·해시도 같다(`studio-result.json`,
  `studio-cdp-row.json`, `studio-cdp-column.json`, `studio-cdp-ties.json`).
- 공유 target `/home/edward/mygithub/rhwp/target/pr-review`와 Docker target volume을
  재사용했다. 사용자 Studio 7700의 JS/WASM 해시는 작업 전과 동일하다.

증거 root는 저장소 밖
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-native-borders`이며,
ignored `output/pr-review/issue7998/p4-native-borders`가 이를 가리킨다. 명령은 각 results JSON의
`command`, 빌드/실제 API는 해당 로그에 보존했다. 임시 JSON·실행 로그·공급 글꼴은 커밋하지 않는다.

| 검증 범위 | Native / fresh WASM 결과 |
|---|---|
| 이번 원본·기존 대조군 62개 × PDF/HWP/HWPX | 각각 186건/219쪽, 최저 **97.45634% / 97.41574%**, 90% 미만·오류·쪽수 차이 0. 무편집 저장 재열기의 PNG는 직접 PDF 출력과 같다. `native/results.json`, `wasm/results.json`. |
| 직전 절편의 공통 기존 207쪽 | backend별 114쪽 PNG 동일 / 93쪽 변경, 0.01%p 이상 하락 0. `native-base-comparison.json`, `wasm-base-comparison.json`. 최신 upstream/devel 비교는 P5에서 수행한다. |
| 독립 한컴 Print: 양 부모 × 행·열·테두리 편집 및 행·열 삭제 복원, 양 저장 형식 | backend별 20쪽, 최저 **98.41387% / 97.76556%**, 전쪽 한 쪽 일치·90% 미만 0. `native-edited-results.json`, `wasm-edited-results.json`. WASM은 실제 Studio API가 생성한 저장본이다. |
| 독립 공유 색상 Print, 양 저장 형식 | 각각 2쪽, 최저 **98.41387% / 97.76556%**. `native-tie-results.json`, `wasm-tie-results.json`. |
| 새 정식 검사·관련 경계 | 새 3건 + 기존 composition 8건 + cellzone 3건·투명 merge 2건·width axis 2건·BorderFill inverse 4건 = **22건 통과**. `focused-results.json`. |

검토자는 같은 영역에서 새 셀의 연속 외곽, 중첩 자식의 원래 틈, AFTER CHILD 및 표 뒤
OUTSIDE AFTER, 가로·세로 공유 색상과 용지 오른쪽을 직접 대조했다.
2026-10-18 메인테이너는 아래 검토 화면과 실제 Studio를 확인한 뒤
**“시각·편집 구조 통과”**로 판정했다.

- 복사 가능한 비교: `/mnt/e/tmp/issue7998-semantic-table-review/p4-native-borders/index.html`.
  PNG·PDF·저장본·TSV가 상대 링크로 연결되며 외부 경로 참조가 없다.
- 실제 행 삽입: `http://localhost:7793/?review=issue7998-p4-native-row&lang=ko`.
- 실제 열 삽입: `http://localhost:7793/?review=issue7998-p4-native-column&lang=ko`.
- 공유 색상: `http://localhost:7793/?review=issue7998-p4-native-ties&lang=ko`.

## 정식 회귀와 남은 경계

새 `issue_7998_pdf_table_border_editing.rs` 3건은 source/실제 BorderFill, 실제 편집 API와
양 저장 형식·복원, 공유 edge 소유를 관계로 검사한다. source와 독립 인쇄 입력·Print는
[`native-border-sources.md`](../../pdf/issue7998/native-border-sources.md)에 연결한다.
`node scripts/rust-test-suite-manifest.mjs --prepare`로 자동 수집했고 workflow 호출은
추가하지 않았다. 새 테스트 전체를 과거 라이브러리에서 재컴파일한 것은 아니며, 수정 전
결함 검출은 실제 이전 CLI/WASM의 동일 속성·공유 색상 실행으로 연결했다.

기존 periodic 검사 1건의 Rectangle 면적만 세는 표현 assertion은 native 경계를 누락해
실패했다. 독립 source 선분 길이 기대값은 유지하고, 겹친 shared edge를 합집합으로 세어
native centerline × 원본 0.5pt + 남은 drawing으로 같은 extent를 검사한다. 8개 입력의
전체 페이지 source/Native/fresh WASM 및 직접 판정 뒤 변경했다. 기존 관계 검사와
내용 소속·순서·누락/중복 검사는 유지했으며 baseline 허용치를 완화하지 않았다.

이번 결과는 **연속 실선 경계의 실제 편집 계약**이다. 비표준 paint 경계의 일반 스타일
교체와 위 B16의 한컴 저장 미달, 병합/지역 격자·unequal span 근거, 본문 수평 메트릭과 B08의 원래 Helvetica
글꼴 대체 미달은 P4/P5에 남는다. 직전 B14의 원래 글꼴 88.50536% 문제를 이번 219쪽
대조군의 통과로 면제하지 않는다. 전체 Rust/lint/package·최신 devel·최종 PR 게이트도
P5에서 수행하며 `final_pr_ready=false`다. 원격 push·PR 생성·GitHub 댓글은 수행하지 않았다.

최종 소스·테스트의 rustfmt 및 PDF를 제외한 staged diff whitespace 검사는 통과했다.
두 독립 PDF의 xref 행 끝 공백은 PDF 선언 바이트의 일부라 `diff --check`가 보고하지만
원본·지문을 바꾸지 않는다. 생성기로 두 source의 동일 바이트 재현도 확인했다.
