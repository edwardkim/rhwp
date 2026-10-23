# #7445 리베이스 보정 68단계: 저장 CellBreak 첫 조각의 행 소유 — 진행 중

## 범위와 독립 근거

기존 고정 목록 32번 `samples/issue5941/1490000-201600081_roadmap_research.hwp`만 다룬다.
독립 `pdf/issue7382-regression-review/1490000-201600081_roadmap_research-2020.pdf`는 302쪽이다.
새 보정 대상·PDF·기대값을 추가하거나 변경하지 않는다. parent는
`41c06540d992e0cb15c9650829ec133ad6a85b26`, base는 `7bb143879dbde80a02fb4a63692556ac28fe46ac`다.

원 #7445 완료 문서의 원문 셀 소유 검사 `bee3c0088d`와 기존 검증 code head
`0ecf268fc4df5921f0a5102385c22e3b6afcaf1d`의 생산·소비 코드를 다시 읽었다.
0ecf의 prepare에는 선언 상자를 온전한 실제 행 높이 합과 대조하는
`declared_cellbreak_prefix_end`와 그 결과를 `source_cellbreak_row_end`로 넘기는 경로가 있다.
현재 통합된 소스에는 후속 문단으로 증명하는 기존 경로만 있고 이 생산·연결이 없다.
과거 전체 통과 기록을 현재 후보 통과로 재인용하지 않고 같은 원문·독립 PDF로 다시 검증한다.

## 보정 전 확인

67단계 봉인 Native/fresh WASM의 생산 소스를 커밋에 연결하고 295~302쪽을 새로 출력했다.
이는 전 302쪽 수용이 아닌 집중 진단이다. 두 출력의 전체 쪽수는 302로 같다.
297쪽 raw는 Native **46.18187%**, WASM **46.16818%**다. Native 296쪽 review·overlay와
297쪽 review를 직접 대조해 Finland 행이 독립 PDF의 297쪽 대신 296쪽 아래에 먼저 놓임을 확인했다.
296쪽은 raw 94%대지만 내용의 쪽 소속 오류가 있으므로 통과로 면제하지 않는다.

현재 기존 검사 3개를 그대로 실행해 **2 PASS / 1 FAIL**, 2.21초다.
Finland 행 소속만 실패하고, 295쪽 마지막 저장 주석 상자와 300~302쪽 Germany·출처 소유는 통과한다.
진단 Native CLI는 layout을 두 번 계산해 같은 `pi=1305` 로그가 두 번 나타난다. 처음의 단일 로그
가정 assertion은 관측 오류였으며 실제 CLI exit 0·두 계산의 일치 여부를 다시 확인했다.

## 원인과 후보

문단 1305의 표는 CellBreak이며 원본 선언 높이가 첫 조각의 온전한 행 경계를 기록한다.
현재는 본문 잔여 843.3px 안에서 측정 행 0~9의 842.9px를 수용해 Finland까지 먼저 둔다.
독립 PDF와 과거 연결은 첫 조각을 행 0~8로 닫고 뒤 행들은 다음 조각이 그대로 소유한다.

후보는 같은 `resolve_row_heights_trusting_declared`의 실제 행 높이를 사용해 선언 상자를 닫는
온전한 prefix를 증명하고, 기존 `source_cellbreak_row_end`로 넘긴다. 편집·재조판·무효 저장 줄과
중첩 행의 다른 물리 소유는 제외한다. 새 수치 허용치 대신 같은 HU→px 합산의 f64 roundoff만
구분한다. 실제 현재 호스트·캡션·바깥 여백이 본문 안에 들어가는지도 확인한다.

생산 결과는 기존 첫 조각 `scan_row_count`를 제한하고, 그 스캔이 선택한 동일 행 범위를 실제 부분
배치가 소비한다. 저장 원점을 다시 가져오거나 전체 표 높이를 줄이지 않는다. RowBreak/CELL의
글줄 분할 정책과 새 수용 원장은 변경하지 않는다. 문서·쪽 ID 분기도 없다.

67단계의 불변 빌드로 보정 전 Native/fresh WASM 전 302쪽을 별도 산출하고 있으며,
후보의 새 Native 빌드와 기존 검사를 먼저 확인한다. 실행·중간 JSON·로그는 ignored
`output/pr-review/7445-completed-20261009/rebase-8155-20261022/stage68-roadmap-fragment-ownership/`에
보존한다. 이 단계는 아직 검증 중이고 전체 PR 준비 완료가 아니다.


## 전쪽 진단과 후속 후보 — 아직 미수용

보정 전 Native/fresh WASM을 각각 전 302쪽으로 완성했다. 기존 8쪽 묶음은 PNG 생산에서도
문서를 매번 초기화하므로 자기 실행만 종료하고, 완료 Native 264쪽의 원본 hash를 보존했다.
현재 정본 PNG exporter의 8작업자 단일 호출로 잔여 쪽과 fresh WASM 전쪽을 산출했다.
`capture-batching-recovery.json`과 `whole-before-fast-state.json`에 소유·종료·재개 근거가 있다.
최저 raw는 Native 7.45335%, WASM 7.45345%이며 양쪽 21쪽이 미달이다.

첫 후보 V1은 기존 로드맵 3검사가 모두 통과하고 297쪽 소속을 복구했다. Native 전쪽에서
296·297쪽 PNG만 바뀌고 나머지 300쪽은 보정 전과 같았지만 20쪽 미달이 남았다.
확대 정상 대조군은 132 실행, 127 PASS / 기존 5 FAIL, 4,976 skip이다. 전체 PR 통과가 아니다.
67단계 전 봉인 Native로 117·128·131·142·153·252쪽을 다시 산출해 현재 미달이 그 단계에서
새로 생긴 것이 아님을 같은 raw 수치로 확인했다.

V2는 셀 안의 ColumnDef(1단)가 잉크·흐름을 소유하지 않는 기존 공통 판별을 저장 plain
줄·문단·컷 재시작 조회 세 곳에서 공유한다. 표 1645의 행 9, 열 2는 두 줄 뒤에 실제 vpos reset이
있지만 메타데이터 하나 때문에 그 경계가 거절됐다. 본문 예산을 넓히지 않고 같은 저장 종료 줄의
바깥 advance를 다음 조각에 남겼다. 기존 로드맵 3 PASS, Native 전 302쪽의 미달은 17쪽으로
줄었고 117~119쪽은 복구됐다. fresh WASM 전쪽 수용은 아직 실행하지 않았다.

V3는 이미 인증된 본문 reset 앞 조각에 새 각주 참조가 없을 때 현재 선택 각주의 실제 paint
상자를 예약한다. 추정 안전 여백을 다시 차감하거나 이미 환급한 여백을 중복 환급하지 않는다.
그 결과 127~139쪽의 새 Native 집중 raw는 모두 98% 이상이지만 전체는 301쪽이 되어 기존
302쪽 검사 3개가 실패했다. 이 결과를 완료로 채택하지 않았다.

독립 PDF·V3 전체 page-item 원장·127~144쪽 review를 대조하니 원래의 여분 128쪽과 별개로,
구역 2의 서로 다른 그림이 한 쪽에 겹친 결함이 드러났다. 141쪽의 그룹 개체는 캡션을 포함한
완전 상자 뒤에 저장된 빈 호스트 줄을 갖지만, 공통 자리차지 밴드가 캡션을 제외하고 그 줄의
양수 vpos를 쪽 원점으로 사용했다. 뒤 본문이 약 52.4px 올라가고 문단 6의 다음 쪽 reset이
인증되지 않아 뒤 그림도 같은 쪽에 그려졌다. 독립 PDF의 141쪽은 앞 그룹과 본문 3줄,
142쪽은 뒤 그룹과 마지막 본문 줄을 각각 소유한다.

V4는 자리차지 밴드가 기존 공통 캡션 높이를 포함하게 하고, 그 완전 상자가 첫 저장 호스트 줄을
정확히 닫을 때만 쪽 축 0과 실제 줄 원점을 수용한다. 실제 fit이 같은 쪽의 전체 줄을 수용한 뒤
기존 ParagraphOrigin으로 paint에 원점·소비 끝을 전달한다. 빈 호스트를 삭제하거나 그림을
문서·쪽 번호로 이동하지 않는다. V4 빌드·기존 검사는 현재 실행 중이며 결과 확인 전 미검증이다.

모든 중간 실패와 원본 PNG·TSV·로그는 위 output 경로의 v1/v2/v3/v4 구분으로 보존한다.
새 실물 회귀·px 핀은 Native/fresh WASM 전쪽 선행 조건을 충족하기 전 추가하지 않는다.


## V6b 중간 검증 결과 — 제출 보류 유지

V4는 측정에서 302쪽·기존 검사 3 PASS를 복구했지만 paint의 첫 ParagraphOrigin 수용 조건이
양수 시작을 제외해 141쪽 본문을 다시 위로 올렸다. V5는 측정과 같은 자리차지 상자·저장 줄의
닫힘으로만 첫 양수 원점을 인증하고, 실제로 수용한 같은 원점에서 paint를 시작하도록 연결했다.
Native 전쪽 미달은 153·252쪽 두 곳으로 줄었다. 141쪽의 앞 그림·본문과 142쪽의 뒤 그림을
각각 직접 판독했다. 단순한 helper 공유가 아니라 최종 원점 선택까지 대조한 보정이다.

252쪽의 표 861은 행 선언 합만으로 실제 저장 줄의 유효성을 판단할 수 없었다. 원본에서 각 행의
`max(셀 선언, 저장 잉크 끝+유효 안 여백)`은 2231·6170·15746·9362HU이고 합은 표 선언 높이
33509HU와 정확히 같다. 정상 저장 줄과 전체 표 프레임의 독립 닫힘이다. 그런데 더 짧은 셀 선언만
보고 캐시를 무효화하고 마커를 재구성해 Spain 문단의 마지막 줄을 합쳤다.

V6b는 온전한 원본 격자의 이 닫힘을 공유하고, 원 HWP 셀의 clean geometry miss를 소유하지 않는
지역 프레임에서는 마커 존재만으로 저장 줄을 버리지 않는다. 실제 stale·편집·재정렬과 기존
pre-2020 condense 경로는 유지한다. 입력·기대값·완료 페이지 핀을 바꾸지 않았다.
V6 첫 컴파일은 signed cell_spacing을 u64로 바로 변환한 제 오류로 실패했다. checked 변환으로
고친 V6b의 실제 빌드와 검증만 후보 증거다. 이 컴파일 오류를 제품 회귀 FAIL→PASS로 인용하지 않는다.

Native 빌드는 116초대에 exit 0이고 기존 로드맵 3검사도 3 PASS다. 확대 nextest는 151 실행,
**147 PASS / 기존 4 FAIL / 5734 skip**, 검사 시간 12.902초다. 새 실패는 없다.
기존 로드맵의 text_overlap partition 7도 통과했다. #7696의 저장/재조판 밴드, #6718의 저장
reset, #7702/#8105의 condense·번호와 정상 저장 줄, #7993 soft-hyphen, #7514 목록 caret,
앞 단계의 표 분할·각주·그림 대조군을 포함한다. 남은 4건은 전체 PR 완료와 구분한다.

- `rowbreak_rowspan_opening_row_cut::sample_task1725_block_that_fits_the_page_end_stays_whole`
- `text_overlap_baseline::text_overlaps_do_not_grow_partition_12`
- `issue_7379_rowbreak_table_footnote_reservation::terminal_caption_withholds_the_last_rowspan_unit_together`
- `issue_7470_rowbreak_source_frame_ownership::original_page_count_and_cut_labels_keep_their_owners`

fresh WASM은 locked wrapper의 web/pkg/no-opt 로컬 대체 빌드로 219.640초, exit 0이다.
Docker 최적화 빌드가 아니다. root pkg·Studio public·봉인 JS/WASM은 hash가 같다.
실제 Studio 초기화 응답은 아직 검증하지 않았다.

| 출력 | 전쪽 수 | raw 최저 | 90% 미달 | 153쪽 제외 최저 | 252쪽 |
| --- | --- | --- | --- | --- | --- |
| native | 302 | 83.55427% | 153쪽 한 곳 | 94.50025% | 99.19253% |
| wasm | 302 | 83.55427% | 153쪽 한 곳 | 94.49952% | 99.20590% |

전쪽 TSV의 쪽 목록은 1~302로 연속하며 누락·중복이 없다. Native/fresh WASM의 127·141·252쪽
review를 직접 열어 각주 위 본문, 앞뒤 그림의 분리, 원본 표의 마지막 저장 줄과 뒤 본문 보존을
확인했다. 296·297쪽은 양쪽 raw 100%다. 153쪽 미달이 남으므로 새 실물 회귀·px 핀을 추가하지
않았으며 기존 검사도 제거·완화하지 않았다.

## 153쪽 잔여의 독립 진단

153쪽은 글꼴 결함으로 분류하지 않는다. 원본 `BinData/BIN0036.bmp`는 24bpp 비압축
1000×514 비트맵이고 독립 Print에는 같은 크기의 JPEG XObject가 있다. JPEG 압축만의 문제라는
가설은 진단용 PDF의 동일 위치에 원본 BMP를 넣어도 raw 82.82953%였으므로 입증되지 않았다.
이 수동 진단 PDF·샘플링 PNG는 독립 기준이나 실제 엔진 통과 결과가 아니다.

동일 원본을 한컴 2020 engine, 실제 11.0.0.9136, Print method 0,
`hancom2020_pdf_driver_one_up`로 다시 인쇄했다. timeout 1800초, job
`e96ddd80-20a5-469d-a2f4-78ea8bf595d9`, 87초 succeeded, 전 302쪽이다.
기존 PDF의 153쪽 content stream과 JPEG 바이트는 새 독립 인쇄와 동일하다. 전 문서에서는
301쪽 content stream이 동일하다. 기준 PDF를 교체하거나 점수를 재정의하지 않았다.

Print MediaBox는 595×841pt이며, 153쪽 bitmap의 PDF 사각형은
`91.031,109.920~510.086,432.487pt`다. Native tree의 같은 bitmap은 약
`121.3,146.7,559.0,430.5px`다. Print 장치 좌표 변환과 이미지 축소 필터가 옅은 격자·점에
미치는 영향을 추적 중이다. 단순 압축·글꼴 탓이나 특정 쪽 보정으로 결론내리지 않는다.
원본/독립 기준/90%와 devel 대비 2%p 기준은 유지한다.

## 보존 증거와 다음 검증

[전후 전쪽 TSV·대표 review/overlay·생산 hash 원장](../pr/assets/issue7445/completed-20261009/rebase-7bb143879-20261023/roadmap-fragment-ownership/README.md)을
중간 증거로 보존한다. 이것은 PR 제출 완료가 아니다. 실행 로그·진단 JSON·수동 재구성 파일은
ignored output에만 있다. 현재 구현과 이 중간 증거는 로컬 커밋으로 정리하고, 153쪽 이미지 출력과
남은 기존 4검사·최신 devel 전쪽 비교·전체 회귀·필수 게이트·실제 소비 경로 검증을 계속한다.


## 고정 devel의 원본 출력 확인

공유 target에 base를 다시 빌드하지 않고, 봉인 605ccfd9와 현재 base 7bb143879의 src·assets·ttfs·
Cargo·crates·vendor·build.rs Git tree 일치를 다시 확인했다. 봉인 Native로 같은 원문·독립 PDF·
96 DPI·Print·같은 호스트 글꼴 공급에서 전쪽을 실제 출력했으나 base는 299쪽, PDF는 302쪽이었다.
정본 도구의 쪽수 gate가 실패했으므로 전 302쪽의 엄격한 devel raw 비교 통과로 인용하지 않는다.
실제 생성한 299쪽 PNG·tree와 PDF 전쪽 PNG 및 실패 로그는 모두 보존했다.

원본의 고유 산포도 캡션으로 확인한 구역 3 문단 54는 base 실제 152쪽, 후보·PDF 153쪽에 있다.
내용 소속을 맞춘 진단에서 base 전체 페이지 raw는 74.06488%, 후보는 83.55427%다.
그림 부분 진단도 67.17334%→79.38395%지만 이 내용 대응·crop 진단은 엄격한 같은 페이지 번호의
전쪽 비교나 후보 90% 통과를 대신하지 않는다. 원본 PDF의 쪽수·내용 소속을 맞춘 후보의 실제
전쪽 수용과 남은 bitmap 출력 보정을 계속한다.
