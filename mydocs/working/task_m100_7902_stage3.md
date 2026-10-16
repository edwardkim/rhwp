# #7902 3단계 — #7923 머지 후 리베이스와 통합 검증

Issue: #7902. 원래 #7877의 19문서·835쪽 범위를 유지한다.
이전 구현·검증의 고정 SHA와 증거는 [2단계 기록](task_m100_7902_stage2.md)에 있다.

## 기준과 현재 상태

- #7923은 2026-10-15 22:27:06 KST에 merge됐다.
- 새 기준: `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`.
- 이전 head: `2eaf83c1239badf4e27e86e70c5602b9bee03b82`, 이전 기준: `463c48cc2`.
- 리베이스 후 구현 head: `043581ffc462b1cad0b06b5ddc39f7bc73b06aa1`.
- 11개 커밋을 적용했다. 충돌은 아래 두 파일이며, 그 외 패치도 새 호출 경로에서 검증한다.
- 생성 Studio JS는 SHA-256 `fd499c9e…47a0a3`인 기존 바이트를 복원했다.
  작업 전용 stash만 제거했고 이전 stash 3개는 그대로다. 이 JS를 새 WASM 검증으로 보고하지 않는다.
- 공용 target은 `/home/tsjang/rhwp/target/pr-review`다. Cargo는 순차 실행하며 기존 사용자 지정 jobs8을
  시작값으로 유지한다. 새 CPU−2 지침은 메모리·동시 작업에 따라 조절할 수 있는 권장값이다.

## 충돌 해소 근거

| 경로 | 해소와 검증할 계약 |
| --- | --- |
| `scripts/export-wasm-png-for-sweep.mjs` | 양쪽 모두 FontFace family를 원문으로 등록한다. load·document.fonts.add를 공유하는 기존 helper를 유지하고 실제 브라우저 계약 검사를 실행한다. |
| `src/renderer/skia/text_replay.rs` | #7923의 전체 텍스트 subpixel 보존을 유지한다. 생성 HFT 번호의 baseline_snap(false)는 별도로 보존한다. Native 소수 위치·번호·글꼴 진단을 함께 확인한다. |

`rebase-7923-state.json`에 이전/현재 SHA, stash 보존, 생성 JS 해시와 해소 근거를 기록했다.
`range-diff`의 차이는 위 두 해소와 새 base의 문맥 변경이다. 자동 병합 자체를 조판 통과로 보지 않는다.

## 새 지침 적용

- 완료 페이지를 고정할 때 같은 핵심 동작을 확인할 수 있는 후보 중 Native/fresh WASM 점수의
  최솟값을 기준으로 100% → 99%대 → 98%대 순으로 선정한다. 해당 변경의 표 시작·이어받기·종료·
  후속 본문·수식·각주 경계와 기존 내용/소유 관계 검사를 함께 보호한다.
- 새로운 source·글꼴에서는 기존 페이지의 98% 증거를 그대로 적용하지 않는다. 후보 페이지별 양쪽
  점수와 선정 이유를 새 결과에 연결한다. 90%·0.01%p·전체 쪽수/내용·확대 판독 기준도 유지한다.
- 50MiB 이상 PDF의 새 PR 포함을 금지하고 로컬 안정 경로와 출처·해시·쪽수·명령을 기록한다.
  기존 원본/Print의 바이트와 검증 범위를 바꾸지 않는다.

## 검증 순서와 미검증 범위

1. 새 호출 경로와 기존 보정의 생산→측정→실제 배치 소비를 대조하고 Native CLI를 새 head로 빌드한다.
2. 먼저 화학제품/6761-control 29쪽, nested 두 형식 104쪽, overfill 끝 조각과 rowbreak HWPX15쪽을
   독립 Print와 직접 PNG로 확인한다. 같은 새 기준의 producer를 별도 보존해 기준/후보를 비교한다.
3. 경계 출력을 먼저 보정한 뒤 원래 19문서 전쪽 Native/fresh WASM을 새 source·같은 입력/Print로
   검증한다. SVG 전쪽 생성은 하지 않으며 필요한 최종 glyph 계약 검사만 기존 명시 경로를 쓴다.
4. 실제 새 결과를 근거로 회귀·필수 lint·정책 검사와 필요한 전체 회귀를 수행한다. 이전 10310개
   통과를 이번 통합 head의 전체 회귀 결과로 바꾸어 기록하지 않는다.

Native CLI 빌드와 fresh WASM 빌드, 실제 Canvas 글꼴 등록/실패 정리 브라우저 검사2개는 통과했다.
첫 새 Native/WASM 전쪽 실행에서는 overfill2·5쪽이 미달이었다. 아래 원본 인증 보정 뒤에는
**Native19문서835쪽·fresh WASM19문서835쪽 모두90% 이상·누락0·Print 쪽수 일치**다.
새 기준에서도 남은 WASM104쪽0.06227%p 하락을 해소됐다고 주장하지 않는다.
이전 전체 회귀 통과를 현재 source의 결과로 바꾸어 기록하지 않으며, 필수 검증·하락 보정 전에는
PR 생성·승인·merge와 옵션2 기록 PR을 진행하지 않는다.

## 기존 회귀 검사 재검토

사용자의 문서별 px 보호 요청에 따라 전쪽90% 통과 문서마다 양쪽98% 이상인 완료 페이지의
중요 위치를 독립 Print 좌표±2px로 보호한다. 기존7원본(화학제품·6761세 변형·overfill·nested두
형식)은 기존 검사 재검증과 경계 보완으로, 나머지12원본은 새 완료 페이지 기하 검사로 연결한다.
blank/쪽번호만 있는100% 쪽으로 실제 표·그림의 경계를 대체하지 않는다. 같은 동작의 후보는
양쪽 최솟값이 높은 쪽부터 고르며, 유일한 결함 경계는 그 쪽의 증거를 유지한다.
19원본 모두 새 전쪽 Native/fresh WASM835쪽에서90% 이상이며, 독립 Print와 쪽수도 일치한다.
이는 WASM104쪽의 devel 대비0.06227%p 하락을 면제하거나 PR 준비 완료를 뜻하지 않는다.

- 원본19문서의 직접 경로·파일명 참조64파일(그중Rust56파일)과 공용 corpus 참조를 추적한다.
  상세 상태는 ignored `rebase-7923-existing-regression-inventory.json`이다.
- 번호: HWP/6761-control·shifted·later29쪽, HWP/control·shifted100쪽은 양쪽100%다.
  독립 Print rawdict의 글리프 원점을96dpi로 환산하고6배 확대 판독했다.
  기준선만 보던 기존2검사에 변형과 한/두 자리 번호의 가로 원점·내용·누락/중복 검사를 보완한다.
  글리프의1px 안팎 raster 차이는 남으며100%를 엄격 픽셀 동일로 보고하지 않는다.
- 이미지 스택: HWP/HWPX 전8쪽 최저Native95.64519%, WASM96.39187%.
  3~8쪽 review를 직접 읽었고 첫/마지막 이미지·명단·원본 그림 순서를 대조했다.
  기존 쪽수2검사 안에 원본 ID·단일 쪽/셀 소유·순서·포함·본문 보존을 보완한다.
  별도 issue_4771 검사는 이미 같은 관계를 보호하고 있어 유지한다.
- vpos5쪽의 헤더/제목 클릭점4개를 원본 셀 경로→커서 사각형→같은 셀 진입 관계로 바꾼다.
  대상 헤더/제목을 Native/WASM/Print로 대조했다. Native의 하단10~12번 마커 paint 차이는
  이 클릭 검사 통과로 해결됐다고 보고하지 않으며 이 쪽 전체의 px 고정 근거로 삼지 않는다.
- 기존 저장 LineSeg·여백·글리프 전진폭의 독립 계약과 이미 관계형인 소유/순서/포함 검사는 유지한다.
  변경된 검사도 실행 검증 전 완료로 판정하지 않는다.

## overfill 저장 경계의 통합 회귀 진단

- 새 Native 전48쪽에서2쪽26.97603%,5쪽24.56113%; fresh WASM도 같은 내용 이동을 보였다.
  원본/Print의 쪽수는48로 일치하지만, 원본 첫 프레임의 마지막 문단이2쪽으로 이동한다.
- 이전 고정 producer와 새 producer의 유닛 높이·첫 조각 예산979.333px는 같다.
  첫 컷은26→25유닛으로 바뀌었고, 다음 문단22의 양수 원점1500HU 재시작이
  `hard_break_before=true`에서false가 됐다.5쪽의 문단88도 같은 원점 계약을 확인한다.
- 생산→소비: 원본 셀의 공유 LineSeg 좌표/문단 위 간격 인증→`cell_units_uncached`의
  저장 프레임 플래그→`advance_row_cut_inner`의 강제 컷 인증→row scan의 수용 높이/컷→
  `layout_partial_table`의 실제 내용/후속 원점이다. 양쪽 소비자가 같은 인증을 사용해야 한다.
- 수정 근거: 원본 RowBreak 표의 선언 row 합이 object 첫 프레임보다 크고, 변경되지 않은
  제어 없는 문단들이 공유 저장 좌표계를 가지면 HWPX도 양수 위 간격 원점의 저장 컷을 소유한다.
  문단 지역0 원점만 있거나, 선언된 원본 조각이 없거나, 편집/재조판된 셀은 이 인증이 아니다.
  전체 본문 높이를 현재 셀의 첫 조각 예산으로 대신하지 않는다. 문서 ID·숫자 조건은 넣지 않는다.
- overfill의 기존 기대값·baseline 변경과 새 회귀 추가는 전쪽90% 복원·직접 판독까지 보류한다.
  현재 실패를 기존 이슈 이관이나 허용치 변경으로 제거하지 않는다.

새 기준2ef의 실제 출력 쪽수도 확인했다: overfill은 **Native48 / Print48**이다
(`rebase-7923-base-boundary/native/overfill-hwpx/run_manifest.json`). 이전463c 기준의
49→48 불일치를 새 기준의 보류 사유로 재사용하지 않는다. 새 기준의 전쪽 TSV와
후보의 페이지별0.01%p 무회귀·독립 Print·직접 판독으로 다시 판정한다.

원본 선언 프레임 인증은 작은 `stored_text_frame_owner.rs`로 분리했다. 기존 Native
판정은 동일하게 유지하고, 공유 원본 좌표를 쓰는 HWPX 텍스트 셀의 생산/소비 두 곳이
같은 인증을 사용하도록 연결했다. 편집된 표·TAC·원본 조각이 아닌 선언·문단 지역0만 있는
줄은 이 계약을 얻지 않는다. 새 빌드·전쪽 검증 전에는 해결로 보고하지 않는다.

### 첫 인증안의 실패와 원본 첫 조각 인증

선언 행 합만 확장한 첫 안은 실제 overfill에 발동하지 않았다. 원본 셀 선언 높이는
1566/1566/1566/566/566HU로 완전한 내용 높이가 아니다. 이 안을 해결로 채택하지 않았고,
실효 없는 안의 WASM 빌드는 해당 작업 process group만 종료했다(컴파일 실패가 아님).

원본 첫 조각은 선행 행의 원본 글줄·패딩8684HU + 마지막 줄63000+1200HU + 셀 패딩566HU
= **73450HU**로 선언 object73450HU를 정확히 닫는다.
`rebase-7923-original-opening-frame-witness.json`에 원본 XML 관측값을 보존했다.
새 인증은 이 첫 경계의 독립 원본 등식을 사용한다. 첫 경계 전에 개체·합성 줄·편집·rowspan의
알 수 없는 행 높이가 있으면 측정값으로 채워 인증하지 않는다. 뒤 조각의 개체는 앞 조각의
인증을 무효화하지 않으며, 개별 후속 컷은 변경되지 않은 글줄과 문단 위 간격을 재확인한다.

- 첫 안 Native 빌드10분53초, 실효 없음. 새 원본 인증 Native 빌드는3분30초·exit0.
- 새 producer `rebase-7923-opening-frame-native-dev-rhwp`, SHA-256
  `26f11eb5f39725df9580f031d07bf0b8c59b15655e43b613ffe50641ead6fd88`.
  source는 HEAD778b54b8e + 기록된 runtime patch이며, commit HEAD와 동일하다고 표기하지 않는다.
- Native 선택1/2/5/43/44/45/46쪽은 모두99%대이며, 이전 미달2쪽은99.67072%,5쪽99.67504%다.
  전체48쪽과 독립 Print48쪽은 일치한다. 이후 완료한 전쪽 결과는 아래에 연결한다.
- 새 경계 PNG는 현재 producer를 직접 실행했다. 해시·96dpi·독립 PDF가 같은 **기준 PDF 래스터만**
  재사용해 반복 PDF 변환을 줄였다. rhwp PNG/렌더 트리를 이전 출력으로 대체하지 않는다.
  `capture-boundary-with-reference.py`와 각 `capture_manifest.json`에 명령·원본/producer/reference
  해시·전체 쪽수를 남겼다. 이 선택 쪽 실행의 not_evaluated는 전쪽 승인 판정이 아니다.

## 문서별 완료 페이지의 px 회귀

증거 루트는 ignored `output/pr-review/issue7902-render-repair-20261014/`다.
`rebase-7923-opening-frame-full/{native,wasm}/<key>/silhouette.tsv`와 `run_manifest.json`은
각 원본/Print·글꼴·compat·96dpi·producer 해시를 고정한다. Native producer는 위26f11e…이고,
fresh dev WASM은 `06aafcaf2dcb2f20acbaef77c1b51c134e2a0e11ff28e6e163984a5f8d6b1e11`이다.
둘 다 HEAD778b54b8e에 `rebase-7923-opening-frame-source.json`의 runtime2파일을 적용한 source다.
dev/no wasm-opt 여부를 숨기지 않으며, 재생성 Studio JS를 포함한 사용자 변경은 이 검증에 섞지 않는다.

아래 점수는 새 전쪽 TSV의 반올림 전 원값이다. Native/WASM의 순서이며, 선택 쪽 중 한 쪽이라도
98% 미만이면 그 쪽의 px 고정은 허용하지 않는다. 전쪽 최저값과 페이지별 목록은 위 원본 TSV에 있다.

| 원본 또는 key | 완료 쪽 | Native / fresh WASM (%) | 중요한 위치와 관계 |
| --- | --- | --- | --- |
| `chemical-original` (`1480000-201900042-chemical-product-labeling-study.hwp`) | 29·100 | 각각100 / 100 | 한/두 자리 HFT 번호의 가로 원점·기준선·내용·스타일·중복 방지. 기존57·77쪽의 정확한 표/그림 핀도 재검증한다. |
| `6761-control.hwpx` | 29·100 | 각각100 / 100 | 같은 번호 계약과 원본 변형의 독립 쪽 소유 |
| `6761-shifted.hwpx` | 29·100 | 각각100 / 100 | 같은 번호 계약, 이동된 본문과 번호의 분리 |
| `6761-later.hwpx` | 29 | 100 / 100 | 한 자리 번호 계약.100쪽은 다른 본문이므로 같은 번호로 고정하지 않는다. |
| `156757920-animal-welfare-husbandry-guidelines.hwp` | 1 | 98.65081 / 99.12658 | Square 인증마크 네 모서리·옆 첫 본문 원점/기준선·원본 레인 소유 |
| `nested_table_fragment_cut.hwp` | 4 | 99.38505 / 99.44873 | 걸친10×2 중첩 표 네 모서리·물리 셀 포함·원문 단일 소유 |
| `stored_frame_page_larger_rowbreak.hwpx` | 1 | 99.52513 / 99.49009 | 신청서 마지막 괘선·제목/서명 기준선·같은 원본 표 소유 |
| `table_scattered_header_rowbreak.hwp` | 2 / 3 | 100 / 100; 99.92916 / 99.98932 | 반복 머리글, 첫 표 종료·후속 표 시작, 각 표 네 모서리·첫 시험 항목 원점/기준선·중복 방지 |
| `rowbreak-problem-pages.hwp` | 17 | 99.99898 / 99.99923 | 이어받은 예시 프레임/diagram 네 모서리·뒤 과업 표의 순서 |
| `rowbreak-problem-pages.hwpx` | 17 | 99.99898 / 99.99923 | HWP와 별도 원본·독립 Print에서 같은 핵심 경계 |
| `table_giant_cell_overfill.hwpx` | 1·2 / 43·44·45 | 최소99.67072 / 99.61593 | 첫 원본 프레임의 마지막 정의/다음 제목 기준선, 종료 셀·가운데 정렬·내용 컷 |
| `issue2004_cell_image_stack.hwp` | 5 | 100 / 100 | 그림 네 모서리·단일 셀 소유; 기존 마지막 프레임 괘선 핀도 유지 |
| `issue2004_cell_image_stack.hwpx` | 5 | 100 / 100 | 별도 HWPX 그림 네 모서리·셀 포함; 기존4~8쪽의 정확한 x 핀도 유지 |
| `table-vpos-01.hwp` | 1 | 99.17078 / 99.50987 | 표 안 제목 기준선·제목/본문 순서·원본 표 소유 |
| `table-vpos-01.hwpx` | 1 | 99.17121 / 99.51014 | HWPX 원본의 같은 중요 경계. HWP 검사만으로 대체하지 않는다. |
| `pic-in-table-with-toggle.hwp` | 1 | 99.93139 / 99.93088 | 셀 안 그림·표 뒤 띠·후속 그림의 네 모서리/순서·셀 포함 |
| `issue1949_giant_cell_nested_tables_perf.hwp` | 104 | 99.69595 / 99.87312 | 이어받기14×2 표 위끝·후속 그림 위끝·단일 소유·앞뒤 순서 |
| `issue1949_giant_cell_nested_tables_perf.hwpx` | 104 | 99.69595 / 99.87312 | HWPX의 같은 원본 경계.98% 보호와 별개로 devel 대비 하락은 계속 보류다. |
| `issue5543_carried_anchor_ladder.hwpx` | 1 | 98.95533 / 99.70580 | 첫 저장 프레임의 마지막 본문/주석 기준선·간격/순서 |

선정 근거:

- 번호29·100, 이미지 스택5, scattered2는 같은 동작을 검사할 수 있는 양쪽100% 쪽이다.
  100%는2px 관용 실루엣 값이며 엄격 픽셀 동일을 뜻하지 않는다.
- Square1·nested4·nested104·overfill의 첫/종료 조각·rowbreak17은 해당 소유/이어받기/종료가
  실제 발생하는 경계다. 무관한 높은 점수의 다른 내용 쪽으로 대체하지 않는다.
- stored-frame의2~5쪽100%는 빈 쪽이므로 실제 신청서의 마지막 괘선/서명 핀을 대신하지 않는다.
- vpos5쪽은 점수는 더 높지만 Native 하단 마커 paint 차이가 남는다. 새 px 핀은 직접 판독한1쪽의
  표 제목에 두며5쪽의 셀 진입·중첩 경로 관계는 별도로 유지한다.
- carried-anchor는 양쪽98% 이상인1쪽(최솟값98.95533)과3쪽(98.93301) 중1쪽을 우선한다.
  98% 미만인2쪽 로고/4쪽 후속 문단은 저장 메트릭·쪽 소유 관계로 보호하며 px 핀을 추가하지 않는다.

독립 기대값은 `rebase-7923-independent-geometry-candidates.json`(PDF image bbox/벡터),
`rebase-7923-independent-pdf-table-geometry.json`(실제 괘선),
`rebase-7923-independent-pdf-text-baselines.json`과
`rebase-7923-extra-opening-and-number-print-origins.json`,
`rebase-7923-independent-lane-and-row-baselines.json`(rawdict 글자 원점)에 기록했다.
좌표는 원 PDF72pt→96dpi 환산이며 구현의 계산값을 기대값으로 복사하지 않았다.

직접 판독은 `rebase-7923-opening-frame-full-reviews/<backend>/<key>/review/`와
`rebase-7923-geometry-direct-reading/`의 Native/WASM/Print 같은 영역 패널에 연결한다.
패널은 측정 뒤 만든 이동/맞춤 없는 동일 crop이며 TSV의 영역·관용 거리·DPI를 바꾸지 않는다.
그림/괘선/줄·기호의 순서·누락·겹침을 대조했다. 글꼴 획과 그림 보간의 작은 차이는 남아 있으며
숫자100%를 엄격 픽셀 동일로 설명하지 않는다. 기존 번호/표/스택 핀의42개 새 래스터는 앞선
직접 판독의 래스터와 바이트 동일하다(`rebase-7923-existing-pin-fresh-raster-identity.json`).

## 실행과 남은 gate

- 기존2개 번호 검사와2개 쪽수 검사 안에 원본 변형·가로 원점·원본 그림 ID/순서/포함을 보완했다.
- rowbreak13쪽572/995px 하한과 글상자 폭/높이 선택을 원본 빈 줄 간격·내용/포함 관계로 바꾸고,
  18쪽의 임의100px 허용은 끝난 표의 부재와 후속 본문 소유로 강화했다.
- vpos 클릭4개는 셀 경로의 커서 사각형을 사용한다. 중첩 표1쪽 허용은2.5→2px로 강화했다.
  기존7095 그림 프레임은635px 폭 선택 대신 바깥1×1 소유로 찾고 원본 그림 포함을 검사한다.
- 새 완료 페이지 기하13검사는 [정식 case](../../tests/cases/issue_7902_completed_page_geometry.rs)에 있다.
  나머지7원본의 기존 검사는 [원본 경계 case](../../tests/cases/issue_7902_original_frame_and_content.rs)에서
  보호한다. 합성 좌표·전체 SVG 해시로 대신하지 않는다.
- 보정한 overfill을 과거 이관만으로 제외하지 않도록 off-canvas/text-overlap 공용 수집 경로를
  복원한다. 독립2024 Print와 검증한 compat2024를 사용하고 baseline 건수를 새로 완화하지 않는다.
  45쪽3.933px 오탐을 정상 baseline으로 등록하지 않는다.
- 첫 focused41검사와 추가 geometry12+7095의13검사, 이후 묶음55검사는 통과했다.
  진단 오탐 수정 후57검사와 Native/WASM Clippy·workspace build·all-target Clippy·fmt·manifest·
  unit-tier·문서 링크가 모두 통과했다(`rebase-7923-regression-policy-and-lint-result.json`).
  본문/첫 행의 독립 x·기준선 핀까지 보완한 최종 묶음은70/70 통과(실행51.633초),
  fmt·manifest·all-target Clippy·문서 링크도 exit0이다. 정확한 명령/시간은
  `rebase-7923-important-interior-points-final-result.json`에 기록했다.
- 새 원본 첫 내용 컷 검사는 고정 devel producer와 보정 전778 producer에서 각각 exit101로 실패했다.
  원인은1쪽의 마지막 문장이2쪽으로 옮겨진 내용 차이다. HFT 번호 검사도 devel의1060.48px가
  독립1062.719971px 기준선 범위를 벗어나 실패했다. 현재 source의 대응 검사는 통과했다.
  나머지 정확한 배치의 추가 핀을 모두 과거 결함 검출 증거로 주장하지 않는다.
- `audit-rebased-full-sweep.py`는 입력/Print·글꼴·DPI·compat·측정 script와 TSV SHA·전쪽 범위를
  대조하고 Decimal로 devel−후보>=0.01을 판정한다. 양쪽 각각19문서835쪽 비교 완료이며,
  유일한 하락은 HWPX nested104쪽99.93539→99.87312(0.06227%p)다.
  새98% px 검사 통과로 이 하락을 면제하지 않는다. 전체 release-test 회귀·최종 PR gate는 아직 남아 있다.

### 공용 off-canvas 검사를 복원하며 발견한 진단 오탐

추가한 전쪽0건 검사는 수정 전 실패했다. 원인은 출력/조회 순서나 compat 차이가 아니었다.
진단의0-기반44는 **실제45쪽**이며, 표 자체 높이는1020.98667px로 쪽 안에 있다.
그 뒤 빈 TextLine/빈 TextRun의1113.12+13.33333px 점유 상자를 심층 세로 합집합에 넣으면서
조상 표의 진단 상자가1048.98667px로 부풀어 용지 아래3.93333px 신호를 만들었다.
선택/전쪽 CLI와 PNG의 실제45쪽 표 상자는 같다. 잘못 추정한 조회 순서 원인은 채택하지 않았다.

독립 Print와 양쪽45쪽 review에서 실제 글자/괘선은 보존되며 같은 쪽 점수는
Native99.98393%, WASM99.98383%다. 레이아웃의 빈 줄 높이·내용 컷·클립은 바꾸지 않고
`deep_vertical_union_bbox`에서 기존 `is_empty_text_carrier` 판정을 재사용한다.
이 판정은 문자열/대체 표시가 모두 비고, 배경·테두리·탭 리더·개체 자식이 없는 줄에만 해당한다.
공백 문자열·표시 문자·장식·개체가 있는 반례는 계속 검사한다. 순수 진단 계약2개와
정식 원본48쪽 검사를 함께 실행하며, 실제 조판을 고친 것처럼 보고하지 않는다.

### 최신 devel 확인과 검증 체크포인트

2026-10-16 사용자의 PR 전 head 확인 지시에 따라 `git fetch upstream devel`을 실행했다.
원격 head는 기존 기준 `2efefc6a1f9812a188e32c1cb283e9ef7962ea3e`와 같아
리베이스하지 않았다. 최종 전체 회귀와 WASM104쪽 하락 해소는 아직 남아 있다.
진단 수정 후 별도 Native CLI로 overfill48쪽을 확인한 결과 off-canvas0·text-overlap0이며,
별개 overflow1건은 유지한다(`rebase-7923-overfill-anomaly-after-fix-result.json`).
렌더링 두 파일의 SHA는 전쪽 Visual Sweep producer와 동일하며 진단 수정은 출력에 영향을 주지 않는다.
