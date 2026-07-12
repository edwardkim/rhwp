# Changelog

이 프로젝트의 주요 변경 사항을 기록합니다.

## [Unreleased]

## [0.7.18] — 2026-07-11

> v0.7.17 후속 patch — 렌더링 정합 대규모 보정(부동/전면 개체 페이지네이션, RowBreak 표,
> 미주 흐름), 초대형 표 성능, 편집기 캐럿·undo·OLE 정합, 관용 파싱·HWPX 보존 확대,
> WMF 도형 재작성. 내부 리팩토링 21라운드(행동 회귀 0) 동반. 공개 API 하위 호환 유지 — PATCH.

### 렌더링 정합
- 부동/전면 개체 페이지네이션 일치: 전면 이미지·tac 스택 쪽당-1장 분할(#1995/#2004/#2006),
  용지-앵커 글뒤로 표 절대배치(#1994), 부동 폼 별지 서식 과분할 81→18쪽(#2019),
  1×1 중첩셀 콘텐츠 분할 6→15쪽=한글(#2007).
- RowBreak 표: 블록컷 sliver 흡수(#1921), 각주 다수 과분할(#1937), 부재-LINE_SEG 셀
  라인높이 em 교정 213→159쪽(#1842), 선언-fit 확대(#2097), vert_offset 이중계상(#2015).
- 쪽 하단 saved-bounds 신뢰 확대(#2093), 하단 고정 틀 vpos=0 리셋 보존(#1920/#2158),
  vert=쪽 valign=Bottom 표 하단 밀착(#1858), Square-wrap 비-TAC 그림 프레임 배치(#1230),
  TAC 표 내용 높이 확장(#1835), tac 그림 lazy vpos 이중 가산 수정(#1898).
- 함초롬 계열 라틴/구두점 폭을 Haansoft Batang 메트릭으로 대체 — 한글 폰트 대체 동작
  모사(#2156).

### 성능
- 거대 셀 렌더 O(pages×cell) 재계산 제거 — cell_units 메모이즈(#1949), 초대형 표
  O(n²) 셀 측정 제거로 52,694셀 문서 렌더 타임아웃 해소(#2063), Rc→Arc 로
  DocumentCore: Send 복원(#2087).

### 편집기 (rhwp-studio)
- 후반 페이지 셀 캐럿 계산 3,064ms→33ms — 캐럿 페이지 힌트 탐색(#2021).
- undo 스냅샷 라우팅 일원화: 찾아 바꾸기(#2037), 그림/수식 속성 다이얼로그(#2028/#2077).
- OLE 개체 선택/캡션/붙여넣기 한컴 호환(#2069), 표 셀 Enter 캐럿 정정(#2164),
  표 셀 장문 입력 캐럿·IME 경계 보정(#1951), 반복 셀 Enter 좌표 재계산(#2164).

### 파서·직렬화
- 관용 파싱: UTF-16/UTF-8 디코딩 폴백으로 부분 손상 실문서 로드(#1932), 미등록
  styleIDRef 기본 스타일 강등(#1933), 암호화 HWPX 감지·분류(#1946), DRM/빈 파일
  분류(#1982), MAX_XML_SIZE 256MB(#1917).
- HWPX 보존 확대: 각주/미주 모양(#1984), breakLatinWord(#1986), secPr 2필드(#1987),
  curSz=0 sentinel(#2017), BinData 빈 확장자(#1981). HWP5: page_def 소실(#1915),
  CommonObjAttr 왕복(#1916). HWP3→HWP5 탭 단위 통일(#1950).

### WMF/도형
- write_line 완전 재작성 — connectLine 변질·컴포넌트 소실 수정(#1943), legacy 공용
  도형 drawText 미방출 수정(#1944), 도형 restrictInPage 하단 클램프(#2075),
  셀 앵커 그림 셀 vertical_align 반영(#2071).

### CLI·도구
- export-png hide_fill 검은 페이지 수정(#2083), ir-diff 출력 가드/요약 유지 재구조,
  코드 품질 대시보드 총량 지표·시계열 발행(mydocs/metrics/, #2130/#2132).

### 내부 품질
- 복잡도 리팩토링 21라운드: 최대 CC 288→117(−59%), 행동 회귀 통산 0 (#1904/#2131 결산,
  산식 v2.1·감소 잠재량 스캐너 도입 #2130). 프론트 Phase 0 기준선 진행(#2124).

### 기여자

이번 사이클(v0.7.17 이후)에 머지된 기여자 PR 268건 (GitHub 핸들, 알파벳순):

- @humdrum00001010 — HWPX 패키지 그래프 직렬화 복원(#1675), 미주 배치·구분선·번호 한컴 정합(#1875) 외 5건
- @jangster77 (Taesup Jang) — 표/편집 정합·외부 PR 통합·검토 130건: 행정업무 편람 페이지 정합(#1691), OLE 개체 선택/캡션/붙여넣기(#2103), 표 셀 장문 입력 캐럿·IME(#2172) 등
- @johndoekim — 방점 렌더 정합(#1738), OOXML 차트 C1c/C1d(#1890/#1954/#2140) 외 6건
- @kkyu8925 — 빈-host co-anchored float 표 orphan control(#1767) 외 4건 *(첫 기여 환영!)*
- @lpaiu-cs — undo 스냅샷 라우팅 3부작(#2028/#2039/#2078), restrictInPage 클램프(#2033/#2076), BinData 채번(#2040), 셀 앵커 그림 valign(#2074) *(첫 기여 환영!)*
- @oleg-sung — TAC 도형 뒤 가운데 정렬 셀 vpos 수정(#1674) *(첫 기여 환영!)*
- @physwkim — cell_units Rc→Arc Send 복원(#2087), 새 문단 서식 상속(#2095)
- @planet6897 (Jaeook Ryu) — 렌더링 정합 연작 74건: RowBreak 표·부동/전면 개체 페이지네이션·초대형 표 성능(#2063)·10k 표본 검증 등
- @postmelee (Taegyu Lee) — CI release-test 전환(#1739), 프론트 리팩터링 계획 v2(#2080)·Phase 0 기준선(#2174 진행) 등 28건
- @seo-rii — CanvasKit 개체 gap 진단 정렬(#1806) 외 3건
- @snvtac — 미주 reset rewind 열 배치 수정(#1495) *(첫 기여 환영!)*

의존성: dependabot 6건 (quick-xml 0.41 보안 릴리스 포함).


## [0.7.17] — 2026-06-23

> v0.7.16 후속 patch — OOXML 차트 렌더 정합 첫 작업, legacy 도형 shapeComment 직렬화,
> WASM options object API, rhwp-studio 표/그림/커서 편집 정합 다수, 렌더 권위 보강,
> 의존성 일괄 업데이트. 공개 API 하위 호환 유지(positional API 유지) — PATCH. 브라우저 확장 0.2.6 동반.

### API
- WASM public API 의 고인자(7+) 함수 26개에 options object 변형 `*Ex(options_json[, image_data])`
  를 추가했다 (#1413). 기존 positional API 는 그대로 유지되며(하위 호환), `*Ex` 는 JSON
  options 로 같은 동작을 한다. downstream 은 중간 삽입형 시그니처 변경에 덜 취약하다.
  - 대상: insertPicture(하이브리드: image_data 별도 인자), insertClickHereFieldInCell,
    splitTableCellsInRange, splitTableCellInto, moveVertical, setPageHide,
    setCharShapeIdInCell, insertClickHereFieldByPath, getSelectionRectsInCell,
    exportSelectionInCellHtml, deleteRangeInCell, copySelectionInCell, applyCharFormatInCell,
    setNoteEquationProperties, setFormValueInCell, setActiveFieldInCell, removeFieldAtInCell,
    pasteHtmlInCell, moveLineEndpoint, mergeTableCells, insertTextInCell, insertClickHereField,
    getTextInCell, getFieldInfoAtInCell, evaluateTableFormula, deleteTextInCell.
  - 설계 관행·breaking change 표기 규약: `mydocs/manual/wasm_api_options_convention.md`.
  - 소비자(@rhwp/core) README 에 `*Ex` 안내 + 소비자 편집 API 매뉴얼 추가 (#1445).
  - **권고**: 고인자 API 는 `*Ex` 사용을 권장한다. positional 시그니처 변경 시 CHANGELOG
    `### API` 에 인자 index 변경을 명시한다.

### 렌더링 (차트)
- OOXML 차트 27종 중 데이터가 추출되던 7종(3D막대4·3D원형1·ofPie2)을 요소명 라우팅으로 2D 근사
  렌더 전환 — "차트 (미지원)" placeholder 제거 (#1453, C1a / #1431 Track C).
- 막대 차트 `c:grouping`(stacked/percentStacked) 반영 — 누적/백분율 막대 6종 정합 (#1453).

### HWPX 저장 계약 (serializer fidelity)
- `render_common_shape_xml` 경유 legacy 도형(ellipse/arc/polygon/curve/chart/ole)의
  `hp:shapeComment` 직렬화 누락 정정 — round-trip 보존 (#1451).
- `ir-diff` tab_extended 예약 필드[3,4,5]의 거짓 차이 보고 제외 (#1473).

### 렌더링
- Text IR v2 폰트 fallback 시 권위 gap 유지, CanvasKit replay 계약 가드 확장 (#1429/#1447/#1469).
- 표 셀 TAC 그림과 텍스트 세로 정렬 보정 (#1352). 글자처럼 해제 그림 재흐름 보정 (#1459).

### rhwp-studio
- 미저장 문서 자동 백업 + 복구 UI (#1448). 로컬 글꼴 감지 동의(opt-in) 흐름 (#1328).
- 쪽 테두리 미리보기 버튼 토글 복구 (#1426). 그림 삽입·인라인 커서 정합 (#1452).
- 표 줄/칸 입력·지우기 회귀 보정(생성 직후 칸/줄 추가 시 표 높이 보존) + 한컴식 통합 대화상자·단축키 (#1481).
- 표 셀 드래그 선택·한컴 호환 표 편집 (#1443), 셀 보호 속성 보존·입력 차단 UX (#493).
- 크기 고정 개체 조작 차단 (#1436), 온새미로 그림 어울림·문단 테두리 정합 (#1440).
- 스타일 적용·표 캡션/모양복사 보정 (#1470), 플랫폼별 메뉴 단축키 표시 보정 (#1476).

### 브라우저 확장 (0.2.6)
- viewer 인라인 스크립트 CSP 위반 정정(theme-init.js 분리) + 다크 아이콘 자산 누락 복구 (#1444).
- Chrome 다운로드 `onDeterminingFilename` 리스너의 전역 부작용 제거 — 다른 확장의
  `download({filename})` 하위폴더 저장을 방해하지 않도록 `onCreated`/`onChanged` 관찰자로 전환 (#1471).

### 인프라
- `Cargo.lock` git 추적 — 재현 가능 빌드 + CI 캐시 키 안정화 (#1423, macOS FFI 영역 제외).
- 의존성 일괄 업데이트: zip 8.6.0, serde_json 1.0.150, snafu 0.9.1, subsetter 0.2.6,
  skia-safe 0.99.0, unicode-segmentation 1.13.3, wasm-bindgen-test 0.3.75, @types/chrome 0.2.0
  (#1461~#1468).

### 기여자

이번 사이클(v0.7.16 이후)에 머지된 기여자 PR (GitHub 핸들, 가나다·알파벳순):

- @jangster77 (Taesup Jang) — 표 줄/칸 입력·지우기 회귀 보정(#1481), 표 셀 드래그 선택·편집(#1443)·셀 보호(#493), TAC 그림 정렬·크기 고정·온새미로(#1352/#1436/#1440), 자동 백업 복구(#1448), 그림/커서 정합(#1452/#1459), 스타일·캡션·단축키(#1470/#1476)
- @johndoekim — OOXML 차트 C1a 라우팅 + 막대 누적 (#1453)
- @oksure (Hyunwoo Park) — ir-diff tab_extended 예약 필드 거짓 차이 제외 (#1473)
- @postmelee — 로컬 글꼴 동의 흐름(#1328), 쪽 테두리 토글(#1426), Chrome 다운로드 interceptor 부작용 수정(#1471), PR 리뷰 워크플로 문서(#1425)
- @seo-rii — Text IR v2 폰트 권위 유지(#1429), CanvasKit replay 가드 확장(#1447/#1469)

## [0.7.16] — 2026-06-19

> v0.7.15 후속 patch — HWPX 저장 계약(serializer fidelity) 정밀화, 누름틀 안내문 한컴 호환,
> rhwp-studio 드래그&드롭 보안 게이트, 렌더·표·그림 정합과 외부 기여자 PR 다수 반영.
> 공개 API 하위 호환 유지 — PATCH.

### HWPX 저장 계약 (serializer fidelity)
- 셀·글상자 subList 내부 컨트롤 보존, lineseg 원본 보존, 표/그림/묶음 캡션 직렬화 (#1379/#1380/#1387/#1403).
- secPr 페이지 여백·본문 단(colPr) 정의를 IR 값으로 치환 — 템플릿 하드코딩 제거 (#1388/#1407).
- 그림 크기 요소(curSz/imgRect/imgDim), MEMO 필드 parameters, shapeComment, borderFill/numbering 등록 축, 표 pageBreak 보존 (#1389/#1391/#1392/#1384/#1409/#1393).
- 파서 autoNum 폭 일관화, newNum 슬롯 위치 정정 (#1382/#1407). 열거 속성 표면 표기 정합 검사 추가 (#1402).
- DocInfo·numbering paraHead·cellzoneList·useKerning·useFontSpace 등 무손실 라운드트립 보강 (#1405/#1350), hp:tc 셀 필드 이름 파싱 (#1401).

### 한컴 호환
- 누름틀(ClickHere) 안내문(Direction) command 포맷을 한컴 정답지 동형으로 정정 — 한컴 에디터에서 안내문이 바인딩되지 않던 문제 해소 (#1434).

### rhwp-studio 보안·UX
- 드래그&드롭 로컬 파일 로딩을 기본 동작에서 제외하고, 드롭 시 모달 확인 대화상자로 명시적 동의(opt-in) 후에만 로딩 (#1439). 확장/웹 공통.
- 누름틀 양식 모드·경계 편집, 편집 커서/포커스, 개체 속성 비율 유지·크기 고정, 표 셀 TAC 그림 세로 정렬 (#1419/#1428/#1430/#1436/#1352/#258).
- 다크테마 지원 + 잔여 UI 대비 정리 (#1420/#1422). replaceAll 저장 유실 정정, 이미지 Shift 크기 조절, 표 생성 직후 F5 처리 (#1398/#1400/#1404).

### 렌더링
- native PDF export API(DocumentCore) + report-only PDF visual diff (#1359). Text IR v2 폰트 증명 게이트, exact font replay proof, glyph orientation/transform authority 정밀화 (#1421/#1429/#1312).
- 미주 높이 모델 측정 SSOT·게이트 재보정, 공식 미주 모양 모델 정규화, 적분기호 글리프 (#1363/#1370/#1410/#1314/#1377). 쪽 영역 제한 표 셀/회전 셀 그림 배치 정합 (#1282).

### 기타
- 차트 샘플 코퍼스 27종(OOXML+레거시) 검증 fixture 추가 (#1431 P-1). 미주 덤프·sweep 검증 인프라 분리 (#1395).
- 인쇄 시 혼합 용지 크기 보존 (#1383). 온새미로 그림 어울림·문단 테두리 정합 (#1441).

### 기여자

이번 사이클(v0.7.15 이후)에 머지된 외부 기여자 PR (GitHub 핸들, 가나다·알파벳순):

- @Martinel2 — useFontSpace IR 필드 + HWP5/HWPX 파서·직렬화 (#1350)
- @Mireutale — HWPX 표 셀 탭/줄바꿈 인라인 직렬화, 그림 effects/shadow roundtrip 보존 (#1360/#1349)
- @jangster77 (Taesup Jang) — 미주 모양 모델 정규화, 누름틀 양식·편집, 다크테마, 표 셀 그림·크기 고정, 온새미로 정합 (#1410/#1419/#1420/#1427/#1430/#1435/#1437/#1441), 검증 인프라 (#1395)
- @johndoekim — 차트 샘플 코퍼스 27종 (#1431/PR #1432)
- @msjang (Minseok Jang) — 인쇄 시 혼합 용지 크기 보존 (#1383)
- @mrshinds — TAC 표 host-line spacing (#1376)
- @oksure (Hyunwoo Park) — replaceAll 저장 유실, createEmpty 기본 구역, 이미지 크기 조절, hp:tc 셀 이름, 표 생성 후 F5, 캡션 파싱·직렬화 (#1398/#1399/#1400/#1401/#1404/#1406)
- @physwkim (Sang Woo Kim) — HWPX 무손실 라운드트립(DocInfo·cellzoneList·useKerning 등) (#1405)
- @planet6897 (Jaeuk Ryu) — 미주 높이 SSOT·게이트, 적분기호 글리프, 미주 발산 진단·종결 (#1314/#1371/#1374/#1377/PR #1390)
- @postmelee — rhwp-studio 다크모드 잔여 UI 대비 (#1422/PR #1424)
- @seo-rii (Seohyun Lee) — 렌더러 baseline sweep, native PDF export API, Text IR v2 폰트 증명 게이트 (#1312/#1359/#1421/#1429)

## [0.7.15] — 2026-06-06

> v0.7.14 후속 security patch — 브라우저 확장 service worker fetch 경로 보안 강화,
> 수식 TAC 흐름·커서 이동 보정, HWPX 저장 계약 후속 보강. 공개 API 하위 호환 유지 — PATCH.

### 보안
- Chrome/Firefox 확장 service worker의 문서 fetch 경로를 강화했다 (#1307).
  - message sender 검증을 추가해 extension viewer와 content script 호출 경계를 분리.
  - localhost, loopback, link-local, private network, 내부 호스트명 URL을 차단.
  - redirect 이후 최종 URL을 같은 정책으로 재검증.
  - extension-side fetch에 `credentials: "omit"` 적용.
  - 자동 thumbnail 데이터가 page DOM에 직접 노출되지 않도록 hover card 내부 처리를 보강.
- Chrome/Edge/Firefox 확장 `0.2.4` 배포 준비: 새 권한 또는 새 외부 네트워크 endpoint 없음.

### 수식·미주 흐름
- 수식 TAC-only 라인의 자동 줄넘김과 문단 들여쓰기 적용을 보강하고, 미주 영역 커서 이동 회귀를 정정 (#1310).
- 강제 줄넘김 뒤 TAC 수식 커서 이동과 문단 간 이동에서 중복 정지/스킵 현상을 줄임 (#1308/#1310).
- 미주 수식 script 렌더링, continuation spacing, superscript alignment 후속 보정 (#1301/#1303/#1306).

### HWPX 저장 계약
- HWPX 그림 직렬화에서 flip/rotation 하드코딩 및 `isEmbeded` 누락을 정정 (#1309).
- HWPX 대각선 셀 테두리 `hh:slash` / `hh:backSlash` type 보존 (#1311).
- zero-length HWPX field ordering 보존 (#1299).

### rhwp-studio·문서
- 문단 정보 대화상자에서 왼쪽 여백과 내어쓰기 바인딩을 분리 (#1307 후속 작업 중 발견).
- visual sweep contributor guide와 rsvg/font 준비 문서를 보강 (#1292).
- CLI 분석·디버깅 명령 가이드 보강.

### 기여자
보안 제보와 재검증에 도움을 준 Dangel, 그리고 본 패치 사이클에 기여한 모든 외부 기여자와
Dependabot에 감사드립니다.

## [0.7.14] — 2026-06-05

> v0.7.13 후속 patch 사이클 (5/26~6/5) — 미주(해설) 흐름·간격 정합 집중, 수식 렌더링/배치
> 정밀화, 표 셀 안 그림 편집(삽입·복사·hit-test) 한컴 정합, HWPX 저장 계약 확장, 외부 기여자
> PR 다수 흡수. 공개 API 하위 호환 유지 — PATCH.

### 미주(Endnote) 흐름·간격
- compact 미주의 문제 제목 사이 간격(between-notes 7mm), 다줄 문단 줄간격, 연속 인라인 수식
  다행 병합, 구분선 아래 여백을 한컴 기준으로 정합 (#1240, #1241, #1247, #1255, #1259, #1262,
  task #1245/#1248/#1256/#1257/#1258)
- 미주 다단(EACH_COLUMN) 흐름에서 단 끝 줄 위치·오버플로우 보정

### 수식(Equation) 렌더링
- 스크립트 토큰 처리: root/sqrt·관계연산자 glued-split, rm+bar(overline) leak, prime/cdots 연접 (#1208)
- LEFT-RIGHT 구분기호 그룹 뒤 첨자 결합(`|x|^3`) (#1226)
- 큰 연산자(Σ/∏/∫) 피연산자 간격, 수식 포함 줄 본문 한글 압축·겹침 해소 (#1235, #1223)
- HWPX 미주/각주 prefixChar 마커 접두문자('문') 복원 (#1202)

### 표 셀 안 그림(Picture) 편집 — 한컴 정합
- 표 + picture 삽입/토글/시각/클릭 정합 (#1177), 셀 내부 도형 '개체 속성' 다이얼로그 (#1150)
- 중첩 표 셀 picture 복사(Ctrl+C) + 떠있는 개체 paste cascade (#1228)
- 사각형 글상자 안 picture 클릭 hit-test/속성/삽입 (#1254), 중첩 표 셀 붙여넣기 경로 보존 (#1207)
- HWP5 wrap=Square 호스트 본문 커서 전진(답안↔문제 겹침), 수식-only 셀 z-표 행 압축 (#1220, #1225)

### 레이아웃·렌더링
- HWPX curve 도형 `<hp:seg>` 외곽선 렌더링 (#1203), textFlow 속성 roundtrip 보존 (#1213)
- BehindText/InFrontOfText z-order 합성, 용지 기준 BehindText 그림/표 z-order, 바탕쪽 글상자 번호 (#1163, #1252)
- 회전 90°/270° 이미지 bbox 이중회전 정정 (#1102), RawSvg(OLE/차트) 첫 로드 백지 렌더 (#1182)
- 폰트 충실도: 한컴 돋움 폴백을 Noto Sans KR ExtraLight로 (#1234)
- 한컴오피스식 격자 보기·쪽 테두리, 잔상 통합 fix (#1137, #1164)

### HWPX 저장 계약
- Bookmark/Field dispatcher 연결, OLE chart, 회전 그림, 맞쪽 편집 여백 교대, masterpage idRef (#1289, #1242)
- 본문·표 셀 문단 id 전역 유니크 (#1222), external image reference/bytes injection contract (#1142/#1143)

### rhwp-studio
- 입력 편집 재렌더 비용 축소(narrow invalidation) (#1212), 모달 대화상자 드래그 공통화
- mac 창 리사이즈 가운데 정렬, 찾기/이동 대화상자 Enter 처리, hit-test caret snapping (#1193, #1281, #1291)

### 인프라·문서
- Dependabot 결합 의존성 그룹화, vite/puppeteer-core dev-dep bump (#1214, #1216)
- macOS headless Skia font lookup hang 방지 (task #823), Rust 테스트 경고 정리 (#1180)
- ClickHere 필드 값 설정 시 파일 손상 정정 (#1076)

### 기여자
@planet6897, @postmelee, @jangster77, @johndoekim, @Martinel2, @Mireutale, @chkwon, @oksure,
@seo-rii, @xogh3198, @twoLoop-40, @lidge-jun, @humdrum00001010, @HaimLee-4869, @wonbbnote
및 Dependabot. 감사합니다.

## [0.7.13] — 2026-05-26

> v0.7.12 후속 patch 사이클 (5/18~26) — HWPX 렌더링/저장 호환성 집중 정정, 시험지·공공기관 문서군 회귀 해소, 외부 기여자 PR 다수 cherry-pick.

### 핵심 변경

- **HWPX → HWP 저장 호환성 대폭 보강**
  - 표/셀 axis contract, cell LIST_HEADER materialization, gradient `BORDER_FILL`, 셀 안쪽 여백, 셀 배경 이미지 채우기 유형 저장 정합.
  - 메모 컨트롤 직렬화, 목차 필드 마커/페이지 표기 출력, 페이지 번호 감추기/새 페이지 번호 시작 등 문단 컨트롤 저장 보강.
  - `hwpx-h-01/02/03`, `mel-001`, `aift`, `exam_kor`, `exam_social` 계열 파일손상/중단 케이스 다수 해소.
- **HWPX 렌더링 정합**
  - 바탕쪽(짝수/홀수/마지막), 머리말/꼬리말, 문단번호, 글상자 위치·그라데이션·곡률, 문단 테두리/지문 박스 렌더링 보강.
  - `exam_kor.hwpx`, `exam_social.hwpx`, `hwp3-sample16-hwp5.hwpx` 등 한컴 변환본과의 SVG/웹 캔버스 시각 정합 개선.
- **페이지네이션·조판 정정**
  - HWPX `treat_as_char` 표 LINE_SEG lh over-inflation, 중첩 표 페이지 분할, 그림 pushdown/vpos 이중 계상, 다단 미주 vpos absolute, 본문 하단 overflow 측정 통일 보강.
  - HWP3/HWP5 변환본 sample16 계열 page break 및 문단 간격 분석 인프라 보강.
- **rhwp-studio / 확장 UX**
  - TAC 도형 커서 이동 및 연속 공백 이동 경험 개선.
  - Chrome 확장에서 로컬 `file://` HWP/HWPX 열기 시 파일 URL 접근 권한 안내 및 중복 다운로드 억제 (#1131/#1132).
- **인프라와 PR 처리**
  - CI runner 디스크 부족 완화 step 추가 (#1109).
  - 외부 PR #1077/#1078/#1080/#1081/#1117/#1120/#1125/#1132 등 검토·cherry-pick 반영.
  - CanvasKit glyph payload gate 및 COLRv1 glyph gradient replay 단계 보강.

### 잔존

- GitHub Actions 장애로 v0.7.13 준비 시점의 원격 CI 자동 실행이 지연될 수 있음. 로컬 build/test/wasm 검증으로 보완한다.
- `exam_social` HWPX → HWP 저장의 3페이지 홀수 머리말 글상자 높이 문제는 후속 이슈로 분리.

## [0.7.12] — 2026-05-18

> v0.7.11 후속 patch 사이클 (5/12~18) — 외부 기여자 다수 PR 19건 머지 + 본 사이클 @jangster77 PR 시리즈 7건 (#956~#968). 416 files / +64383 / -3323.

### 핵심 변경

- **원 Issue #952 (1 통합 → 5 분리 결함) 완결** — @jangster77 진단 방법론 (부분 해결 + 명확한 분리, archive/task936 "9회 시도 + 5회 revert" 대조 교훈):
  - Issue 1 (#956): 쪽 테두리 paper-based outline 강제 — `#920` 비트 해석 회귀 정정 (5+ samples 한컴 viewer 실측 정합)
  - Issue 2 (#958, #957): sample16 page 18 빈 caption phantom advance 정정 (RHWP_DEBUG_TAC_CURSOR)
  - Issue 3 (#961, #959): 시험지 page 1 문9 — horz_rel_to=Column picture column 외부 emit advance skip
  - Issue 4 (#963, #960): 시험지 page 2 cases formula off-by-one — has_line_break line 마지막 run end-position TAC 포함
  - Issue 5 (#964, #962): 시험지 page 2 보기 textbox inline equation duplicate emit 차단
- **WMF SetTextAlign vertical bits 정정** (#966, #965): `mode & VTA_TOP(=0)` 항상-true 버그 → WMF [MS-WMF] 2.1.2.18 spec 정합 (PR #918 거대 PR Stage 33-A root cause ~60 lines 단독 포팅)
- **HWP3 sample18 페이지 수 +2 inflate 정정** (#968, #967): 빈 paragraph + [쪽나누기] + overflow case 단독 page 차단 (v2 정밀화 — aift.hwp snapshot 회귀 해소)
- **release 빌드 LTO + codegen-units=1 + strip** (#818, #790): rhwp CLI -28% (14→10 MB) / WASM -6.5% (4.6→4.3 MB)
- **rhwp-studio 신규 기능** (5/12~18): F5 본문 블록 선택 + F3 영역 확장 (#811/#220) + 메뉴 hotkey 인프라 (#810/#792) + 쪽 새 번호로 시작 (#809/#791) + searchAllText API + rhwpDev.goto (#814/#692) + Task #571 문서 비교·이력 분리 PR 1/3 (#799/#571)
- **HWP3/WMF/EMF 렌더링 정정** (5/12~18): EMF/WMF image 콘텐츠 렌더 (#860/#864) + HWP3 ch=9 탭 spec §10.5 (#934/#929) + 다수 외부 PR cherry-pick (#933/#939/#941/#947/#953/#954 등)

### 외부 PR (19 머지 + @jangster77 시리즈 7)

5/12~18 누적 외부 기여자 PR 19건 cherry-pick + 본 세션 @jangster77 7 PR (#956~#968) — 각 PR cargo test 1288 + 광범위 sweep 169 페이지 회귀 0 + 작업지시자 시각 판정 일관 검증.

### 잔존

- HWPX sample18-hwp5.hwpx +7 inflate (별도 task)
- `samples/hwp3-sample18.hwp` fixture 별도 추가 권장 (#968 회귀 가드)

## [0.7.11] — 2026-05-11

> v0.7.10 후속 patch 사이클 (5/10 + 5/11) — 외부 기여자 다수 PR 30+ 머지. (CHANGELOG.md 소급 보강 — v0.7.11 릴리즈 시 누락분)

- **Skia native raster 단계적 진전** (Issue #536): P8 (#761) Layer IR contract hardening + P9 (#769) text replay parity + P11 (#797) Text IR v2 compatibility contract
- **HWP3 native 렌더링** (#753): hwp3-sample10.hwp Oracle 763 페이지 8 단계 정정 + Git LFS pdf-large/ 격리
- **rhwp-studio 인터랙션** (#781/#786~#818): scrollbar drag + chord 키 Ctrl+N→Ctrl+M (Chrome reserved shortcut 회피) + 한글 IME chord e.code 판별 + 표 셀 pattern_type 가드 + Alt/Option+Arrow 단어 이동 (#794) + 표 셀 드래그 셀 컨텍스트 (#795) + 줄 끝/문서 끝 커서 (#807/#808)
- **rhwp-studio editor 신규 기능**: 표 편집 Undo/Redo + 표 크기 조절 SnapshotCommand + 셀 편집 다수 + 다단/새 번호 dialog + Ctrl/Cmd+Arrow / Ctrl+E 단축키

## [0.7.10] — 2026-05-06

> v0.7.9 후속 patch 사이클 — 외부 기여자 7명 흡수 (PR 13건 cherry-pick) + AI 파이프라인 / VLM 연동 도입 + CLI 바이너리 릴리즈 파이프라인 (Issue #608/#612).

### 신규 기능

- **CLI 바이너리 릴리즈** (Issue #608/#612, [@almet](https://github.com/almet) 의 요청)
  - 4 플랫폼 GitHub Release 자산 첨부 (Linux x86_64 / macOS x86_64+aarch64 / Windows x86_64)
  - SHA-256 체크섬 동봉
  - `.github/workflows/release-binary.yml` 신규
- **PNG raster backend** (PR #599, [@seo-rii](https://github.com/seo-rii)) — render P4 단계
  - native Skia 기반 `PageLayerTree` → PNG export
  - `native-skia` feature gate (기본 빌드 영향 0, opt-in)
  - `DocumentCore::render_page_png_native(page)` API
  - **AI 파이프라인 + VLM (Vision-Language Model) 연동 도입** (메인테이너 후속 정정)
    - `--vlm-target claude` (1568 longest edge / 1.15 MP, Claude Vision 정합)
    - `--scale <배율>` / `--max-dimension <픽셀>` (자동 scale 계산)
    - `export-png` CLI 명령 + 매뉴얼 (한글 + 영문 dual)
    - 한글 폰트 fallback chain + char 단위 fallback (공백 두부 정정) + `--font-path` 동적 폰트 로딩

### 외부 PR cherry-pick (13 PR / 7 컨트리뷰터)

#### [@planet6897](https://github.com/planet6897) / Jaeook Ryu — 8 PR (협업 컨트리뷰터)

- **PR #587** — HWP 5.0 스펙 0x18/0x1E swap (하이픈 ↔ 묶음 빈칸)
- **PR #589** (Task #511 v2 + #554) — HWP3 Square wrap 보완6+8 (페이지네이션 안전) + HWP3 변환본 식별 휴리스틱 (HWP 3.0 직렬화 round-trip 정합)
- **PR #561** (Task #548) — 셀 inline TAC Shape margin + indent 정정
- **PR #564** (Task #521) — TAC 표 outer_margin_bottom 누락 정정
- **PR #570** (Task #568) — 인라인 표+수식 단락 우측 편위 정정
- **PR #575** (Task #573) — 보기 셀 분수 단락 인라인 표 셀 paragraph 라우팅 정정 (Issue #572 인접 효과 자동 정정)
- **PR #580** (Task #577) — 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋 정정 (HWP IR anchor 시점 정합)
- **PR #584** (Task #574) — HY견명조 heavy display 오분류 정정 (TDD Stage 2/3 RED→GREEN)
- **PR #592** (Task #588) — exam_eng.hwp p7 #40 글상자 사이 화살표 누락 (PUA U+F003B → ↓ 매핑 + PDF 글리프 외곽 직접 분석)
- **PR #593** (Task #590) — Square wrap 표 horz_rel_to=단 속성 정합 (단일 줄 분기 가드)
- **PR #567** (Task #565) — 인라인 수식 미렌더 정정

#### [@oksure](https://github.com/oksure) (Hyunwoo Park) — PUA SVG 출력

- **PR #600** (closes #513) — Supplementary PUA-A (U+F02B1~F02C4) SVG 출력 정정 — Task #509 후속 매핑 우선순위 정정

#### [@jangster77](https://github.com/jangster77) (Taesup Jang) — HWP3 본질

- **PR #589** (commit author, @planet6897 PR 등록 협업 흐름)

#### [@seo-rii](https://github.com/seo-rii) — render P4

- **PR #599** (refs #536) — native Skia PNG raster backend

### 메인테이너 정정

- **Skia 폰트 영역 5개 정정** (PR #599 후속, `876d820`):
  - 한글 폰트 fallback chain 추가 (Noto Sans KR / Nanum 등)
  - `--font-path` 동적 폰트 로딩 (`with_font_paths` API, ttfs 디렉토리)
  - char 단위 fallback (NBSP / U+2007 / U+200B 두부 정정)
  - VLM 옵션 (`PngExportOptions` + `VlmTarget::Claude`)
  - `export-png` CLI 명령 + 매뉴얼 dual

### 인프라

- **CI 빌드 안정성 향상** (`Cargo.toml` `[[example]] required-features`)
- **광범위 페이지네이션 회귀 sweep 도구** — 164 fixture (158 hwp + 6 hwpx) / 1,614 페이지 자동 검증 (PR #564 도입, 본 사이클 모든 PR 처리에 적용)

### 후속 이슈

- [#613](https://github.com/edwardkim/rhwp/issues/613) — VLM 프리셋 확장 (GPT-4V / Gemini / Qwen-VL / LLaVA)
- [#614](https://github.com/edwardkim/rhwp/issues/614) — DPI 메타데이터 옵션 (`--dpi` PNG pHYs chunk)
- [#615](https://github.com/edwardkim/rhwp/issues/615) — `pua_oldhangul.rs` U+F53A 매핑 한컴 정합 영역
- [#598](https://github.com/edwardkim/rhwp/issues/598) — rhwp-studio 각주 삭제 기능 (한컴 정합 UX, 외부 컨트리뷰터 공개)

### 잔여 PR (v0.7.11 후속 patch 영역)

- PR #601, #602 (@oksure)
- PR #607 (@dicebattle)
- PR #609 (@jangster77, Task #604 Document IR 표준 정합화)
- PR #611 (@kihyunnn)

---

## [0.7.9] — 2026-05-01

> v0.7.8 후속 사이클 — Task #501 (cell.padding 한컴 방어 로직) + PR #428/#494/#478/#498 cherry-pick + 외부 기여자 4명 흡수

### 회귀 정정 (메인테이너)

- **Task #501 — mel-001.hwp 2쪽 표 셀 높이 처리 회귀** (closes #501)
  - 본질: HWP 셀 IR 의 `cell.padding.top + bottom > cell.height` 인 비정상 케이스 (mel-001 셀[21] r=2 c=2 "현 원": pad=(141,141,1700,1700), cell.h=1280 HU). HWPX `hasMargin="0"` 명시 정합.
  - 회귀 origin: Task #347 의 `prefer_cell_axis` 가드가 비정상 padding 도 cell 우선 적용 → row_heights 거대 → TAC 표 비례 축소 (scale 0.45) → 행 12~20px 축소 + 셀 진입 결함.
  - 정정: `resolve_cell_padding` 끝에 **한컴 자체 방어 로직 모방** 가드 추가 — pad_top + pad_bottom > cell.height 면 cell.height 의 절반까지 비례 축소. `measure_table_impl` 1-b단계도 동일 안전망.
  - 작업지시자 통찰: *"이런 경우 한컴은 자체 방어로직으로 처리한다면?"* — Task #347 가드 (KTX 목차 R=1417 HU 정합) 보존 + 한컴 동작 모방 가드 추가
  - 트러블슈팅 + 위키 ([HWP 셀 Padding 방어 로직](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81)) 작성

### 외부 PR cherry-pick (3 건 / 17 commits)

- **PR #428 그룹 내 그림(Picture) 직렬화 구현** (by [@oksure](https://github.com/oksure))
  - `serialize_group_child` 의 `ShapeObject::Picture` 분기 (빈 TODO) 구현 — 그룹 내 그림이 포함된 HWP 저장 시 그림 데이터 유실 결함 정정
  - SHAPE_COMPONENT + SHAPE_COMPONENT_PICTURE 레코드 추가 (Chart/OLE 자식과 동일 패턴) + 매직 상수 정리 (`tags::SHAPE_PICTURE_ID`)

- **PR #494 Paragraph::utf16_pos_to_char_idx 외부 노출** (#484, by [@DanMeon](https://github.com/DanMeon))
  - PR #405 와 같은 결의 외부 binding 작업 — `helpers::utf16_pos_to_char_idx` (pub(crate)) 의 알고리즘을 `Paragraph::utf16_pos_to_char_idx(&self, utf16_pos: u32) -> usize` (pub) 으로 캡슐화
  - 단위 테스트 6건 추가, semver MINOR 영역 (+1 method, 알고리즘 변경 없음)

- **PR #478 Layout 정합 + 수식 정정 합본** (by [@planet6897](https://github.com/planet6897))
  - 9 Task / 97 commits 누적 PR 중 페이지 레이아웃 무관 영역 **7 Task / 10 commits** 분리 cherry-pick (5 단계 머지)
  - **#488** (수식 토크나이저 폰트 스타일 prefix 분리 + svg/canvas 렌더러 italic honor) — 단위 테스트 14건 추가
  - **#490** (빈 텍스트 + TAC 수식 셀 alignment 적용) — exam_science p1 셀 7/11 28/36 수식 중앙 정렬
  - **#483** (각주 multi-paragraph line_spacing 정합 + trailing line_spacing follow-up)
  - **#489** (Picture+Square wrap 호스트 paragraph 텍스트 LINE_SEG cs/sw 적용)
  - **#495** (셀 paragraph 인라인 Shape 분기 가드 — 부분 정정, 잔존 [이슈 #502](https://github.com/edwardkim/rhwp/issues/502) 분리)
  - **#480** (wrap=Square 표 paragraph margin x 좌표 반영)
  - **#476** (PartialParagraph 인라인 Shape 페이지 라우팅, +881/-4)
  - 미흡수: #479 (paragraph trailing line_spacing/HWP vpos) — 한컴 2020 정답지 시각 판정 필수, [이슈 #503](https://github.com/edwardkim/rhwp/issues/503) 분리

### 회귀 검증 인프라 (외부 기여)

- **PR #498 Canvas visual diff 파이프라인** (relates #364, by [@seo-rii](https://github.com/seo-rii))
  - PR #456 (P2 PageLayerTree replay 전환) 후속 P3 검증 레이어 — rhwp-studio E2E 에 legacy Canvas vs PageLayerTree replay Canvas **픽셀 diff 자동 검증** + GitHub Actions Render Diff workflow 추가
  - 7 commits 분리 (test + diagnostics + docs + CI runner + 보안 hardening 3건)
  - 변경 영역: JS E2E + CI workflow + 문서 + Vite 설정 (Rust 변경 0)
  - 기본 fixture 3개 (KTX/biz_plan/tac-case-001) 모두 0 diff 확인

### 분리된 후속 이슈

- [#502](https://github.com/edwardkim/rhwp/issues/502) — 문단 내 글상자 TextRun 처리 (Task #495 잔존 결함)
- [#503](https://github.com/edwardkim/rhwp/issues/503) — Task #479 본질 정정 흡수 (한컴 2020 시각 판정 필수)

### 위키 정합

- [HWP 셀 Padding 방어 로직](https://github.com/edwardkim/rhwp/wiki/HWP-%EC%85%80-Padding-%EB%B0%A9%EC%96%B4-%EB%A1%9C%EC%A7%81) 신규
- [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/%ED%95%9C%EC%BB%B4-PDF-%ED%99%98%EA%B2%BD-%EC%9D%98%EC%A1%B4%EC%84%B1) 정황 IV 추가 (21_언어_기출 한컴 2020 = rhwp 정답)

### 검증

- cargo test --lib 1086 → **1102 passed**
- cargo test --test svg_snapshot 6/6, issue_418 1/1, issue_501 PASS (신규 통합 테스트)
- cargo clippy --lib -- -D warnings 0건
- WASM 4,206,487 bytes (Task #501) → 4,202,430 bytes (PR #478 5차 후) → 4,211,280 bytes (4차 #480 후)

## [0.7.8] — 2026-04-29

> v0.7.7 후속 사이클 — 외부 컨트리뷰터 다수 + 메인테이너 회귀 정정 + 위키/README 정비

### 외부 PR cherry-pick (15 건)

라이브러리 본질 정정 (조판 / 페이지네이션 / 직렬화):

- **PR #391 다단 섹션 누적 공식 회귀 정정** (#391, by [@planet6897](https://github.com/planet6897))
  - `src/renderer/typeset.rs` 누적 공식을 `col_count` 로 분기 — 단단 → `total_height`, 다단 → `height_for_fit` (trailing_ls 인플레이션 차단)
  - exam_eng (2단): 11 → **8 페이지**, 단독 1-item 단 (p3/p5/p7) 해소

- **PR #396 수식 렌더링 개선** (#174, #175, by [@oksure](https://github.com/oksure))
  - 인라인 수식 높이를 `eq.common.height` (HWP 권위값) 기준으로 설정 + X/Y 스케일링 동시 적용
  - 수식 내 CJK 문자 이탤릭 / 너비 보정 비적용 — CASES 한글 행 겹침 정정
  - 메인테이너 후속 정정 3건 (Canvas 분수선 y / Equation 스케일 / Limit fi=fs)

- **PR #395 그림 밝기/대비 효과 SVG 반영** (#150, by [@oksure](https://github.com/oksure))

- **PR #397 수식 ATOP 파싱 및 렌더링 보정** (by [@cskwork](https://github.com/cskwork))
  - **본 저장소 첫 외부 컨트리뷰터** — `EqNode::Atop` AST 파싱 + 분수선 없는 위/아래 배치 (HWP 의 ATOP / OVER 의미 분리)

- **PR #400 HWPX 수식 직렬화 보존** (#286, by [@cskwork](https://github.com/cskwork))
  - `render_paragraph_parts` 의 controls 무시 정황 정정 + parser XML entity 복원
  - 한컴 한글 2020 정상 열람 + PDF 일치 확인 (한컴 origin hwp 라운드트립 회귀 commit `ecd7d9a` 추가)

- **PR #401 v2 표 페이지 분할 rowspan>1 셀 분할 단위** (#398, by [@planet6897](https://github.com/planet6897))
  - `BLOCK_UNIT_MAX_ROWS=3` 임계 — 작은 블록 (≤3 행) 만 보호, 큰 rowspan (≥4 행) 행 단위 분할 허용 (HanCom 호환)
  - synam-001.hwp 5페이지 회귀 정정 (35→37→**35** 페이지)

- **PR #406 동일 문단 inline TAC 그림 페이지네이션 정정** (#402, by [@planet6897](https://github.com/planet6897))
  - 같은 paragraph 의 두 번째 inline 그림이 첫 번째와 같은 y 좌표에 그려져 겹침/오버플로 발생하던 문제 정정
  - 27→30 페이지 (분할 정상화)

- **PR #408 heading-orphan vpos 기반 보정** (#404, by [@planet6897](https://github.com/planet6897))
  - vpos 기반 5 조건 AND trigger (current fit + vpos overflow + next substantial + next 못 fit + single column non-wrap) — vpos overflow 41건 중 1건만 진짜 orphan
  - 9쪽 pi=83 헤딩 → 10쪽으로 push, 후속 표와 함께 배치

- **PR #410 TopAndBottom Picture vert=Para chart 정정 + atomic TAC top-fit** (#409, by [@planet6897](https://github.com/planet6897))
  - v1: `prev_has_overlay_shape` 가드 확장 (Picture + TopAndBottom + vert=Para)
  - v2: `typeset_section` controls 루프 chart 높이 누적
  - v3: `typeset_paragraph` atomic TAC top-fit 시멘틱 (60px tolerance)

- **PR #415 Task #352 dash 시퀀스 Justify 폭 부풀림 정정** (#352, by [@planet6897](https://github.com/planet6897))
  - dash leader elastic Justify 분배 (PDF 모방), exam_eng Q32 dash advance 12.11 → 7.06 px

- **PR #424 다단 우측 단 단행 문단 줄간격 누락 정정 (vpos 보정 anchor)** (#412, by [@planet6897](https://github.com/planet6897))
  - layout.rs vpos 보정 공식 정정 — `col_anchor_y` 도입 (body_wide_reserved 푸시 직후 anchor 보존), `curr_first_vpos` 우선 사용, page_path/lazy_path 분리
  - exam_eng p1 우측 단 item 7 ①~⑤ 15.33→**22.55px 균일**, 좌측 단 item 1 catch-up 28.56→21.89

- **PR #427 SvgRenderer defs 중복 방지 HashSet 통합** (#423, by [@oksure](https://github.com/oksure))
  - `arrow_marker_ids: HashSet<String>` → 범용 `defs_ids: HashSet<String>` 통합, O(n)→O(1)

- **PR #434 그림 자동 크롭 (FitToSize+crop) 공식 교정 + 테두리 inner padding** (#430, by [@planet6897](https://github.com/planet6897))
  - svg.rs / web_canvas.rs 의 crop 스케일 공식 정정 (`cr/img_w` → `original_size_hu/img_size_px`) + 헬퍼 `compute_image_crop_src` 추출 (SVG/Canvas 단일 진실 원천)
  - 별도 fix: 테두리 문단 inner padding (텍스트가 테두리에 붙는 문제)

API 추가 / 도구:

- **PR #405 `Paragraph::control_text_positions` 추가** (#390, by [@DanMeon](https://github.com/DanMeon))
  - 외부 binding 노출용 API 리팩토링

- **PR #411 `editor.exportHwp()` API 추가** (by [@ggoban](https://github.com/ggoban))
  - 신규 컨트리뷰터 첫 PR — iframe wrapper `@rhwp/editor` 에 exportHwp() 노출

- **PR #413 rhwp-studio PWA support** (#383, by [@dyjung150605](https://github.com/dyjung150605))
  - 신규 컨트리뷰터 첫 PR — vite-plugin-pwa, manifest scope `/rhwp/`, icon 192/512/maskable, registerType=autoUpdate, WASM precache

- **PR #419 PageLayerTree generation API 도입** (#364, by [@seo-rii](https://github.com/seo-rii))
  - `paint` 모듈 신규 (2,376 lines, builder/json/layer_tree/paint_op) — PageRenderTree → PageLayerTree 변환
  - opt-in transition adapter (`svg_layer.rs`, `RHWP_RENDER_PATH=layer-svg`)
  - 기존 5 렌더러 파일 변경 0 라인, 광범위 309 페이지 SVG 100% byte 동일 (피델리티 분석 보고서)

### 메인테이너 작업 (3 건)

- **Task #394 셀 진입 시 투명선 자동 ON 로직 비활성화** (#394)
  - input-handler.ts 5 영역 주석 처리 — 한컴 출력 정합

- **Task #416 `find_bin_data` 가드 결함 정정** (#416)
  - `c.id == bin_data_id` 가드 제거 — `c.id` 는 storage_id, bin_data_id 는 인덱스. sparse id 범위 분기 (HWPX 차트 60000+N 보존). 단위 테스트 7건 추가

- **Task #418 `hwpspec.hwp` p20 빈 문단 + TAC Picture 이중 emit 정정** (#418)
  - Task #376 정정 commit 이 devel 미머지 (close 됐지만 임시 브랜치에만 존재) → 동일 결함 재발
  - paragraph_layout 의 set_inline_shape_position + layout.rs::layout_shape_item 의 already_registered 가드 추가
  - 신규 메모리 (close 시 commit devel 머지 검증) + 트러블슈팅 신설

### 정비 / 문서

- **위키 페이지 [한컴 PDF 환경 의존성](https://github.com/edwardkim/rhwp/wiki/한컴-PDF-환경-의존성) 보강**
  - "발견 정황 II (PR #434 / 이슈 #430)" 섹션 추가 — 한컴 2010 ↔ 2020 ↔ 한컴독스 가 같은 hwp 를 다르게 조판하는 정황. 단일 한컴 정답지 가정의 한계 재확인.
  - rhwp 의 현재 출력이 시험지 조판자 의도에 더 부합 가능성 (원본 JPEG "(A 형)" 잔재 보존)

- **README.md / README_EN.md 보강**
  - Contributing 섹션에 "한컴 PDF 는 정답지가 아닙니다" 항목 추가
  - 신규 "위키 자료 (Wiki Resources)" 서브섹션 (위키 9개 페이지 링크)

- **samples 정답지 자료 추가** — 모든 컨트리뷰터와 fork 사용자 공유
  - `samples/2010-exam_kor.pdf` (한컴 2010, 4.57 MB)
  - `samples/2020-exam_kor.pdf` (한컴 2020, 4.57 MB)
  - `samples/hancomdocs-exam_kor.pdf` (한컴독스, 6.05 MB)
  - `samples/복학원서.pdf` (이슈 #421 한컴 정답지)
  - `samples/synam-001.hwp` (PR #401 회귀 검증)
  - `samples/atop-equation-01.hwp` (PR #397 시각 판정)

### 검증

- `cargo test --lib`: **1066 passed** (1008 → +58, 회귀 0건)
- `cargo test --test svg_snapshot`: 6/6 passed
- `cargo test --test issue_418`: 1/1 passed (Task #418 회귀 보존)
- `cargo clippy --lib -- -D warnings`: 0건
- WASM 빌드: 4,182,395 bytes (변동 +47 KB)
- 광범위 byte 비교: 10 샘플 / 309 페이지 SVG 회귀 검증 (PR 별 검증 게이트)
- 작업지시자 SVG + Canvas 양 경로 시각 판정 (PR #401 v2 / #406 / #408 / #410 / #415 / #424 / #434)

### 외부 기여자 감사

본 사이클 외부 기여자 (가나다순):
[@cskwork](https://github.com/cskwork), [@DanMeon](https://github.com/DanMeon), [@dyjung150605](https://github.com/dyjung150605), [@ggoban](https://github.com/ggoban), [@oksure](https://github.com/oksure), [@planet6897](https://github.com/planet6897), [@seo-rii](https://github.com/seo-rii)

특히 [@cskwork](https://github.com/cskwork) 님은 **본 저장소 첫 외부 컨트리뷰터** 로 PR #397 / #400 두 건을 머지하셨고, [@planet6897](https://github.com/planet6897) 님은 본 사이클 외부 PR 의 다수 (8 건) 를 진단 + 정정해주셨습니다.

## [0.7.7] — 2026-04-27

> v0.7.6 회귀 정정 사이클 (TypesetEngine default 전환 후 누락된 시멘틱 복원)

### 수정 — TypesetEngine 회귀 정정

- **페이지네이션 fit 누적 drift 수정** (#359)
  - typeset 의 fit 판정과 누적을 분리: fit 은 `height_for_fit` (trailing_ls 제외), 누적은 `total_height` (full)
  - 단독 항목 페이지 차단 가드 추가 — 다음 pi 의 vpos-reset 가드가 발동될 예정인 경우 빈 paragraph skip / 안전마진 1회 비활성화
  - **k-water-rfp**: LAYOUT_OVERFLOW 73 → 0 (drift 311px 정정)
  - **kps-ai**: 60 → 4

- **TypesetEngine page_num + PartialTable fit 안전마진** (#361)
  - `finalize_pages` 의 NewNumber 적용 조건을 Paginator 시멘틱과 동일하게 정정 (`prev_page_last_para` 추적, 한 페이지에서 한 번만 적용)
  - PartialTable 직후 fit 안전마진 (10px) 비활성화 — PartialTable 의 cur_h 는 row 단위로 정확
  - **k-water-rfp**: 28 → 27 페이지 (page_num 정상 갱신)
  - **kps-ai**: page_num 1, 2, 1, 1, 2~8 정상 (NewNumber 컨트롤 처리)

- **kps-ai PartialTable + Square wrap 처리** (#362, 8 항목 누적)
  - **wrap-around 메커니즘 (Square wrap) 이식** ★ — Paginator engine.rs:288-372 의 wrap zone 매칭 + 활성화 시멘틱을 TypesetEngine 에 이식. 외부 표 옆 paragraph 들이 height 소비 없이 흡수됨
  - 외부 셀 vpos 가드 — nested table 셀에서 LineSeg.vertical_pos 적용 제외 (p56 클립 차단)
  - PartialTable nested 분할 허용 — 한 페이지보다 큰 nested table atomic 미루기 대신 분할 표시 (p67 빈 페이지 차단)
  - PartialTable 잔여 height 정확 계산 — `calc_visible_content_height_from_ranges_with_offset` 신설
  - nested table 셀 capping 강화 (외부 행 높이로 cap)
  - hide_empty_line TypesetEngine 추가 (페이지 시작 빈 줄 최대 2개 height=0)
  - vpos-reset 가드 wrap zone 안에서 무시 (오발동 차단)
  - 빈 paragraph skip 가드 강화 — 표/도형 컨트롤 보유 paragraph 는 skip 안 함 (pi=778 표 누락 차단)
  - **kps-ai**: 88 → 79 페이지 (Paginator 78 와 일치, LAYOUT_OVERFLOW 60→5)

### 보안

- **rhwp-firefox/build.mjs CodeQL Alert #17 해소** (#354)
  - `execSync` shell 사용 → `execFileSync` (`shell: false`) 로 전환

### 검증

- `cargo test --lib`: 1008 passed, 0 failed
- `cargo test --test svg_snapshot`: 6/6
- `cargo test --test issue_301`: 1/1
- WASM 빌드 통과
- 작업지시자 시각 판정 통과 (kps-ai p56, p67-70, p72-73, k-water-rfp 전체)

## [0.7.6] — 2026-04-26

> 외부 기여자 다수 + 조판 정밀화 사이클

### 추가
- **`replaceOne(query, newText, caseSensitive)` WASM API** (#268)
  — 분석·구현 by [@oksure](https://github.com/oksure) (신규 기여자)
  - `replaceText` 의 위치 기반 시그니처 vs 검색어 기반 호출 mismatch crash 해결
  - 새 API 추가로 하위 호환성 100% 보존
  - 5 unit tests (한국어 multi-byte 경계 포함)

- **SVG/HTML draw_image base64 임베딩** (#335)
  — 분석·구현 by [@oksure](https://github.com/oksure)
  - 기존 placeholder (`<rect>`/`<div>`) → 실제 이미지 base64 data URI 임베딩
  - `render_picture` / `web_canvas` 와의 backend 정합

### 수정
- **목차 리더 도트 + 페이지번호 우측 탭 정렬** (#279)
  — 분석·구현 by [@seanshin](https://github.com/seanshin)
  - `fill_type=3` 점선 리더를 round cap 원형 점으로 표현 (한컴 동등)
  - `find_next_tab_stop` RIGHT 탭 클램핑 제외 — 들여쓰기 문단의 페이지번호 정렬 보정
  - 메인테이너 추가 보강: 셀 padding 인지 leader 시멘틱, 페이지번호 폭별 leader 길이 차등화, 공백 only run carry-over

- **form-002 인너 표 페이지 분할 결함** (#324)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - `compute_cell_line_ranges` 잔량 추적 → 누적위치 (`cum`) 기반 재작성
  - `layout_partial_table` 의 `content_y_accum` 갱신 + split-start row 통일된 계산
  - 작성자 자체 v1 → v2 → v3 보강

- **typeset 경로 PageHide / Shape / 중복 emit 결함** (#340)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - 세 결함을 typeset.rs 의 누락이라는 공통 원인으로 통합 진단
  - `engine.rs` 와의 정합 (PageHide 수집 + `pre_text_exists` 가드 + Shape 인라인 등록)

- **Firefox AMO 워닝 해결 (rhwp-firefox 0.2.1 → 0.2.2)** (#338)
  — 분석·구현 by [@postmelee](https://github.com/postmelee)
  - manifest `strict_min_version` 142 상향 (`data_collection_permissions` 호환)
  - `viewer-*.js` 의 unsafe `innerHTML` / `Function` / `document.write` sanitize
  - rhwp-studio 28 파일 DOM/SVG API 교체 + Reviewer Notes 한/영

- **Task #321~#332 누적 정리 + vpos/cell padding 회귀 해소** (#342)
  — 분석·구현 by [@planet6897](https://github.com/planet6897)
  - vpos correction 양방향 가드 + cell padding aim 명시값 우선 정책
  - typeset/layout drift 정합화 + 메인테이너 검토 응답으로 KTX TOC 결과 (#279) 복원

### 기타
- **신규 기여자 환영 안내** — README.md / README_EN.md Contributing 섹션에 PR base=devel 명시 (#330 close 후 후속 개선)

## [0.6.0] — 2026-04-04

> 조판 품질 개선 + 비기능성 기반 구축 — "알을 깨고 세상으로"

### 추가
- **GitHub Actions CI**: 빌드 + 테스트 + Clippy 엄격 모드 (#46, #47)
- **GitHub Pages 데모**: https://edwardkim.github.io/rhwp/ (#48)
- **GitHub Sponsors**: 후원 버튼 활성화
- **그림 자르기(crop)**: SVG viewBox / Canvas drawImage로 이미지 crop 렌더링 (#43)
- **이미지 테두리선**: Picture border_attr 파싱 + 외곽선 렌더링 (#43)
- **머리말/꼬리말 Picture**: non-TAC 그림 절대 위치 배치, TAC 그림 인라인 배치 (#42)
- **로고 에셋 관리**: assets/logo/ 기준 원본 관리, favicon 생성
- **비기능성 작업 계획서**: 6개 영역 13개 항목 3단계 마일스톤 (#45)

### 수정
- **같은 문단 TAC+블록 표**: 중간 TAC vpos gap 음수 역행 방지 (#41)
- **분할 표 셀 세로 정렬**: 분할 행에서 Top 강제, 중첩 표 높이 반영 (#44)
- **TAC 표 trailing ls**: 경계 조건 순환 오류 해결 (#40)
- **통화 기호 렌더링**: ₩€£¥ Canvas 맑은고딕 폴백, SVG 폰트 체인 (#39)
- **반각/전각 폭 정밀화**: Bold 폴백 보정 제거, 스마트 따옴표/가운뎃점 반각 (#38)
- **폰트 이름 JSON 이스케이프**: 백슬래시 포함 폰트명 로드 실패 수정 (#37)
- **머리말 표 셀 이미지**: bin_data_content 전달 경로 수정 (#36)
- **Clippy 경고 제거**: unnecessary_unwrap, identity_op 등 6건 수정 (#47)

## [0.5.0] — 2026-03-29

> 뼈대 완성 — 역공학 기반 HWP 파서/렌더러

### 핵심 기능
- **HWP 5.0 / HWPX 파서**: OLE2 바이너리 + Open XML 포맷 지원
- **렌더링 엔진**: 문단, 표, 수식, 이미지, 차트, 머리말/꼬리말/바탕쪽/각주
- **페이지네이션**: 다단 분할, 표 행 단위 분할, shape_reserved 처리
- **SVG 내보내기**: CLI (`rhwp export-svg`)
- **Canvas 렌더링**: WASM/Web 기반
- **웹 에디터**: rhwp-studio (텍스트 편집, 서식, 표 생성)
- **hwpctl 호환 API**: 30 Actions, Field API (한컴 웹기안기 호환)
- **VS Code 확장**: HWP/HWPX 뷰어 (v0.5.0~v0.5.4)
- **755+ 테스트**

### 조판 엔진
- 줄간격 (고정값/비율/글자에따라), 문단 여백, 탭 정지
- 표 셀 병합, 테두리 스타일, 셀 수식 계산
- 다단 레이아웃, 문단 번호/글머리표
- 세로쓰기, 개체 배치 (자리차지/글자처럼/글앞/글뒤)
- 인라인 TAC 표/그림/수식 렌더링

### 수식 엔진
- 분수(OVER), 제곱근(SQRT/ROOT), 첨자
- 행렬: MATRIX, PMATRIX, BMATRIX, DMATRIX
- 경우(CASES), 정렬(EQALIGN), 적분/합/곱 연산자
- 15종 텍스트 장식, 그리스 문자, 100+ 수학 기호
