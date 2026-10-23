# #7998 stage37 — 범위 종료·최신 devel 통합과 최종 검증

2026-10-20 작업지시자가 남은 구현을 후속 이슈로 분리하고 P5로 마무리하도록 지시했다.
완료 범위는 현재까지 구현한 의미 표/문단·편집/저장 계약이다. 모든 PDF 작성 구조의 복원이나
후속 #8106–#8110의 해결을 주장하지 않는다. 이 문서는 진행 중 기록이다.

## 고정 기준과 범위

- 직전 로컬 구현: `77f46b19e` (stage36, 동일 바이트 후보 정식 등록 포함).
- fetch한 원격 devel: `5e2b9bfbf`.
- root devel은 사용자 untracked 피드백 두 파일을 보존한 채 해당 remote로 fast-forward했다.
- 남은 B24 내용 성장 격자 → #8106, 보고서6쪽 glyph 폭 → #8107, 본문1/4쪽 자간 → #8108.
- stage36의158입력169쪽/각 backend, Print32저장본·Studio15경로는 병합 전 증거다.
  병합 후 제품이 바뀐 경로의 검증을 대신하지 않는다. 같은 입력/PDF는 재사용한다.

## 병합 보정

16개 충돌의 양쪽 구현을 대조했다. 제품 경로는 PDF source stroke의 path 투영과 새 metafile
image payload를 함께 유지하고, 명시적 physical dash와 새 HWP dash 표현을 함께 소비한다.
글자 위치는 최신 devel의 언어별 percent 속성으로 통일해 pixel offset과 이중 적용하지 않으며,
paint bounds/fast-lane도 같은 위치를 고려한다. 공통 TAC 셀 projection·source row 소유를 보존한다.
독립 Base14 AFM과 새 한컴 font overlays를 함께 유지한다. CFB의 border/text-clip 보존 stream과
HWP3 재정렬 stream은 공존한다. Layer minor27 충돌은 두 계약을 합친28로 구분한다.

최신 devel의 PDFium 기준 rasterizer를 채택한다. 과거 Poppler TSV는 그대로 보존하고 새 최종
시각 검증과 혼동하지 않는다. stroke 상태가 같은 fill을 바꾸지 않는 독립 계약은 유지한다.
상세 충돌 목록은 ignored `output/pr-review/issue7998/final-integration/merge-resolution.json`이다.

디스크 여유7GiB로 필수 빌드가 어려워, 실행 중 Cargo/rustc/wasm 도구가 없음을 확인한 뒤
24시간 이상 미사용한 debug incremental12개만 정리해12.36GiB를 확보했다. 공유 target의
실행 파일·deps·fingerprint·다른 증거는 유지했다. 경로/시간/전후 여유는 같은 폴더
`incremental-cleanup.json`에 기록했다.

## 최종 게이트

진행 순서는 병합 제품의 컴파일·집중/시각 방향 확인 → 필수 전체 Rust/lint/Native Skia·
Worker/package·실제 Studio와 전쪽 fresh WASM → 기록/본문 준비다. 후속 구현을 시작하지 않는다.
아직 이 단계의 통과·PR 준비 완료를 선언하지 않는다. 원격 push·PR 생성·merge는 별도 절차다.

## 첫 통합본의 완료 범위와 실패

첫 통합 제품 `aa73b27d7`의 Native/fresh Docker WASM을 동결했다. Native158입력169쪽은
글꼴 미달6쪽(최저89.54211%)을 제외하고90% 이상, WASM은169쪽 모두90% 이상
(최저90.70673%)이다. 모두 누락·쪽수 차이는 없다. 이 결과를 뒤의 폭 격자 보정 제품의
증거로 재사용하지 않는다. 원본·기준 PDF와 실제 코드/입력이 같은 증거만 재사용한다.

실물A는 현재 IR의15표161셀·5324문자, 실물B는1표60셀·267문자다. 양 저장 형식에서
표/셀의 중첩 순서와 보이는 원문 순서·쪽수12/1이 유지됐다. 이것은 현재 IR 왕복 검사이며
작성 당시 표·셀 수의 정답을 입증한 것은 아니다. 실제 Studio15경로의 삽입·직접 삭제·
Snapshot undo/redo·양 저장은 최종 Native와 바이트가 같다. Worker7개123 assertion도
통과했다. 첫 실행의 Chrome 경로 누락은 환경 실패로 구별해 올바른 경로로 실행했다.

동일 편집 저장본30개 중24개는 stage36에서 인쇄한 입력과 바이트가 같아 그대로 재사용했다.
긴 문단의 새6저장본만 새 한컴2020 Print4작업+2동일 입력 alias로 검증했다. 같은169쪽
집합에서 해당6개만 교체하고152개는 실행 파일/입력/PDF의 지문 cache로 재사용했다.

Helvetica 목록6건은 입력에 프로그램이 없는 Helvetica가 선언돼 있고, 실제 Native 관측은
시스템 Noto Sans KR, 한컴 Print의 내장 face는 Haansoft Batang이다. Arial/HANBatang/Batang
공급 디렉터리의 실제 파일을 확인했다. 이전/현재의 문단·글줄·글자 bbox/소속은 모두 같았다.
정식 일반 PNG 모드의 `font_mismatch_exception`6건을 확보했다. raw미달을 그대로 유지하며
일반통과나 자동승인으로 바꾸지 않는다. B08 후속은 #8109, 실물B의 가는 선/작은 글자
농도 차이는 #8110으로 분리했다. 후자는 원인 확정이 아닌 후속 조사이며 devel 대비 하락0건을
주장하지 않는다. 원본·식별 가능한 이미지는 로컬에만 유지한다.

Rust release 빌드는 성공했다(19분30초). lib test 생성자3곳의 새 모델 필드 누락은
`#[cfg(test)]` 영역에서 false/None으로 보완했다. 제품은 바뀌지 않아 앞선 빌드·시각 결과를
반복하지 않았다. 이후 release lib는3800 PASS/2 FAIL/12 ignored였다. 실패는 편집 때
`content_for_style`이 새 devel의4 HWPUNIT 줄 폭 격자를 누락한 것이다.

## 폭 격자 보정 — 병합 회귀만 수정

원본 소유자 폭5002 HU와 원시 칸 폭4998 HU를 구분하는 합성 입력이다. 저장 줄 폭은
독립 한컴 줄 메트릭의4 HU 규칙(#7412:56문서/19837행)에 따라 각각5000/4996이 된다.
수동 합성 입력의 가정을 실제 한컴 저장본과 혼동하지 않는다. 기대값을 느슨하게 하지 않고
`ParagraphBox::content_for_style`의 전체 내용 폭을 먼저 스냅한 뒤 원래 문단 여백을 뺀다.
원점/양 끝을 각각 스냅하거나 출력을 clamp하지 않는다.

실제 소비는 `commands/text_editing.rs`의 flat/nested 편집과 `commands/document.rs`의 초기/
on-demand 재조판 → `ParagraphBox::effective/frame` → `composer::reflow_line_segs`의
column_start/segment_width 발행이다. paint의 `layout/paragraph_layout.rs`·`shape_layout.rs`는
같은4 HU 기준을 여백보다 먼저 적용하는 `body_for_style`의 frame을 소비한다. 행/쪽 분할,
이어받기 높이·glyph 크기·장평/자간은 바꾸지 않는다.

첫 focused편집2건은 통과했다. 빠른 전체 lib에서 초기/on-demand2건이 반대 기대값5002±1을
고정하고 있음을 확인했다. 소유자 폭5002 검사는 유지하고, 두 저장 줄 기대값만 같은 독립
4 HU 규칙의5000으로 엄격하게 맞춘다. 원시 칸 폭·중복 패딩을 허용하는 범위 완화가 아니다.
보정 제품의 focused/전체 lib와 영향 시각 검증을 먼저 확인한 뒤 최종 전체 게이트를 마친다.
이 기록은 아직 최종 통과 판정이 아니다. B24·glyph/자간·글꼴/농도 후속 구현을 시작하지 않는다.

폭 격자 일관화 후 빠른 전체 lib는 root 3802 PASS/12 ignored, 내부 crate 182 PASS로
실패가 없다. 물리 소유자 폭과 저장 줄 폭을 구분해 초기/편집 양쪽을 확인했다.
최종 Native/fresh WASM은 이 제품으로 다시 동결하고, 같은 입력·독립 PDF를 재사용한다.

최종 격자 보정 code head는 `bd707a7af`다. 새 Native158입력169쪽은 첫 통합본과
모든 페이지 점수가 같고 누락·쪽수 차이가 없다. 30개의 실제 편집 저장 bytes도 첫 통합본과
같아 독립 Print를 다시 요청하지 않았다. Native raw미달6건은 별도의 정식 글꼴 예외 판정으로
연결하며 silhouette-only의 예상 exit1을 일반 실행 오류나 통과로 바꾸지 않는다.

Native169쪽은 첫 통합본과 점수뿐 아니라 PNG bytes도 모두 같다. 같은 source·독립 PDF의
확대 판독은 보존하고 새 runtime의 실제 출력을 확인했다. 새 code의 fmt, 정식 suite manifest,
unit-tier 정책도 통과했다(1633 sources/48 integration targets; 4101 unit declarations).
변경 없는 TypeScript·Studio Node·package·Visual Sweep 계약은 source 해시 대조 뒤 앞선
성공 결과를 재사용한다. Rust 폭 계산을 소비하는 Native/WASM·Studio/Worker는 새로 실행한다.

메인테이너의 새 WASM 직접 테스트 요청에 따라 Windows Chrome CDP19222에 새3탭을
준비했다. 검증 서버7793의 `issue7998-maintainer-final-{operators,quotation,cell}`이며
보고서5쪽·견적서1쪽·중첩 셀 대조군이다. 실제 로드 WASM SHA는
`ee3f48c58710ddb0c4206934befa062733a55ec08c9270b4b3393dc4fa5acdc3`로 동결 제품과 같다.
각 원본 PDF와 실제 이 WASM에서 내보낸 양 저장본은 로컬
`E:\tmp\issue7998-maintainer-review\`에 보존했다. 이3탭은 직접 판정 준비이며
자동15경로에 합산하거나 사용자의 시각·한컴 열림 통과로 기록하지 않는다.

## 메인테이너 불수용 — 실사용 입력 우선 재현

2026-10-20 메인테이너는 이전 devel 대비 종합 개선이 없고 `reousrce_limit.pdf`가
사용할 수 없을 정도로 느리며 표 인식도 모두 실패했다고 판정했다. 기존169쪽·합성 대조군과
저장 왕복 수치를 실사용 목표 달성으로 승격하지 않는다. 완료·PR 준비를 보류하고 전체 검증
controller를 중단했다(release build만PASS, release lib 컴파일 중단, 나머지 미실행).
원본162쪽·1,920,186bytes·SHA256 `b5dd80e561c055c028b5297c7b5ad04c1c1de6de6b246282635b5915680e11e7`을
동일 producer 옵션의 devel/후보에서 비교한다. 새 기능 확장 전에 실제 느린 단계와 Table/Cell
생성/보류 사유를 확인하며, 이 문제를 승인 없이 후속 백로그로 넘겨 마감하지 않는다.

같은 조건의 Native CLI `import-pdf <원본> <별도.hwpx> --json` 전체162쪽 비교는
devel5.73478초, 후보76.40665초(13.323배)였다. 저장 XML의 실제 Table/Cell은 양쪽 모두0/0이다.
후보의157쪽은 `open outer boundary`로 검출에서 거절했고, 나머지5쪽(16·20·21·144·162)은
`recognition_deferred`로 lowering되지 않았다. 161쪽은 clip 관측512개 제한에도 도달한다.
1쪽 원본을 직접 보면 긴 rowspan 셀이 페이지 끝에서 계속되어 외곽 일부가 닫히지 않는다.
이를 완전한 닫힌 사각형만 유효한 표라는 가정으로 거절한다. 외곽을 임의로 그려 결과를
맞추거나 이 사례를 별도 예외로 분류하는 방식으로 해결하지 않는다.

추가 별도 GDB 관측3회는 ① `observed_line_text → fit_spacing → estimate_text_width_exact`,
② `body::lower → wrap_width::fit → resolve_styles_for_document → resolve_styles_with_variant`의
전체 스타일 Vec 복사, ③ 같은 임시 ResolvedStyleSet 해제에서 멈췄다. 실제 호출 경로의 증거이며
세 표본만으로 시간 비중을 확정하지 않는다. `wrap_width::fit`이 문단마다 계속 커지는 문서
전체 스타일을 새로 만든다는 코드와 일치한다. 원본 스타일 수는 devel75305/후보89921이다.
새 문단 메트릭 경로는 CharShapeRef가 u32임에도 u16 한도를 검사해, 실제 본문 후보446개가
`PDF character style budget exceeded`로 보류된다. 이는 단순 출력 점수 차이가 아니다.

실행·프로필 원본은 ignored `output/pr-review/issue7998/resource-limit-recheck/`의
`results.json`, `*-time.txt`, `candidate-page-summary.json`, `style-pool-counts.json`,
`candidate-p1-detail.json`, `profile-gdb.log`에 보존했다. 첫 통계는 디버거 없는 전체 실행이고
GDB 실행은 별도 진단으로 구분했다. parse-only helper는 공용 cache의 dependency metadata
불일치(E0460)로 실행하지 못했으며 그 실패를 제품 결함/성능 측정으로 세지 않는다.

제출 assets의 stage는 취소하고 파일은 보존했다. 기존 시각 자료·백로그 이슈도 삭제하지 않지만
현재 후보의 최종 수용 증거로 쓰지 않는다. 재개 조건은 실제 이 원본에서 페이지 경계의 표/셀
소유를 보존하는 IR, 실제 셀 편집·양 저장, 사용 가능한 Native/fresh WASM 열기 시간이다.
시간 단축만으로 표 인식 실패를, 픽셀 일치로 의미 IR 실패를 면제하지 않는다.

2026-10-20 재개 승인 뒤 같은 입력의 실패 경계를 먼저 보정한다. `wrap_width::fit`은
문단이 참조하는 CharShape와 문단·탭 속성만 지역 ID로 투영해 **기존 공통 reflow**에
전달하고, 결과를 문서 ID로 복원한다. 기본 스타일의 글자 단위 여백 기준도 보존한다.
공통 조판 계산을 근사식으로 대체하지 않는다. 잘못된 u16 CharShape 한도는 u32로 맞춘다.

독립 pypdf content-stream 진단에서 1쪽의 실제 text-show에 활성인 클립은 246종이며
빈 영역에도 출력 호출이 있다. 모든 텍스트를 무조건 제거하지 않고 공백의 유효한 클립도
관측한다. 열린 외곽은 완전하고 경쟁하지 않는 활성 클립 tiling·실제 괘선과 모순 없음이
입증될 때만 셀로 채택하며, 빈 타일의 바깥 변은 실제 paint coverage를 별도로 확인한다.
없는 선을 추가하거나 검출 예산을 늘리는 수정이 아니다. 이 단계는 구현 중이며 아직 통과가 아니다.

공용 target은 다른 작업 `pr7930-integration`이 사용 중이어서 대기했다. 첫 컴파일에서
다른 worktree의 backend 산출물 재사용(clip API 누락)과 새 projection의 default Style 참조
형식(u16)을 확인했다. 형식은 바로잡고 이 worktree의 vendor lib 진입점 mtime만 갱신해
정확한 소스로 다시 빌드한다. 공용 캐시 삭제·다른 작업 중단·별도 target 생성은 하지 않았다.
이 빌드 실패는 실제 PDF 처리 시간이나 구조 검증 결과로 계산하지 않는다.

첫 실행에서 1쪽은 Table1/Cell274를 생성했고 가시 문자533개는 이전 위치형 출력과
누락·추가가 없었다. 전체 실행은 새로 검출한 셀 때문에 기존 고정16384개 한도에 걸렸다.
검출 예산을 무제한으로 푸는 대신 요청 entry 정책과 실제 IR u16 문단 자원 영역의
최솟값을 상한으로 쓰고, 쪽별 tile/work 한도·저장 자원 preflight는 유지한다.
이 변경 이후의 중간 실행은28.49초·162쪽/표7개로, 아직 실사용 실패다.

남은 빈 영역은 text-show뿐 아니라 빈 **clip scope**가 남아 있었다. 16쪽의 빈 BT/ET
가설은 독립 pypdf 관측0건으로 기각하고 관련 backend 변경은 제거했다. 원래 clip 관측을
사용하되 기존 축에 맞고 실제 내용이 소유하지 않은 구역만 보완한다. 새 축·임의 병합·
paint 생성은 금지하고, 실제 괘선을 가로지르거나 빈 구획끼리 경쟁하면 보류한다.
이 선택은 원 작성 셀 구조의 복원 보증이 아니며 `empty_grid_region_clip_scope` 근거를
진단에 따로 남긴다. 공백 전진폭의 클립 밖 부분은 잉크가 아니므로 내용 자체를 자르지 않고
원래 clip 교집합으로 소유만 정한다. 잘못된 좌표를 clamp하는 처리가 아니다.

별도 기존 WASM 진단에서 1쪽274개 셀 중앙 중6곳이 셀의 background/border 사각형을
개체로 선택했다. 셀 소유 paint 표식을 렌더 트리에 전달해 개체 선택 query에서만 제외한다.
paint·문서 주소·그리기 정렬은 보존하며 일반 내용 도형은 선택 가능하게 유지한다.
페이지별 glyph fitting도 문서 전체 대신 해당 쪽의 source CharShape만 공통 resolver에
전달하도록 바꿨다. 최종 Native/fresh WASM 검증 전이며 이 기록은 완료 판정이 아니다.

### 실제 162쪽 복구 중간 결과와 저장 경계

runtime10(`0af139cf59fb8cb93dd76211f9e2e842e74d7866a6d30715d16e3f4fcf216dce`;
dirty source patch `b6aed54090575e65a599bd015cc49845ad7c03870483abe04426884be62260a7`)은
전체 가져오기·HWPX 저장 24.27589초, 실제 Table162/Cell54,938, 섹션162,
lowering 보류0을 기록했다. 원본 위치형 저장본과 가시 문자89,386개의 multiset은 같았다.
이는 순서·소속·시각 일치를 대신하지 않는다. 최고 RSS470,272KiB다. 최종 성능·수용 판정이 아니다.

77쪽의 빈 첫 열만 다른 행 축을 갖는 경우에는 양쪽의 실제 괘선·기존 내용과 비경쟁하는
빈 clip scope를 근거로 지역 축을 보완했다. `boundary_contributor`의 교집합이 비었는데
음수 구간을 틈으로 처리하던 오류도 보정했다. 표준 테두리가 실제 pen의 양쪽 절반을 칠하는
범위를 먼저 소비하고, 교차 pen이 정확히 덮는 같은 색의 짧은 끝부분만 인접 셀 속성에서
제외했다. 독립 짧은 선·다른 색·미지원 그림은 이 조건으로 제거하지 않는다.
16쪽의 글자·bitmap 구두점이 interleave된 셀은 원래 그리기 순서를 유지한 편집 가능한
개체로 실제 셀 안에 보존한다. 해당 셀의 통합 Paragraph 복원은 미검증으로 명시한다.

runtime10 저장본의 Native 전쪽 raw 최저는92.71875%로 모두90% 이상이지만, 기준 devel의
최저98.88854%와 비교하면22쪽이 하락한다. 뒤쪽22쪽의 주요 차이는 글자 속성이
74,968개로 HWPX reader의 resource ID65,535를 넘었기 때문이다.
실제 `header.rs::set_indexed`에서 초과 속성이 건너뛰어져 기본 글자 속성으로 표시된다.
u32 IR 허용만으로 저장 계약을 입증할 수 없었다. 이 상태를 통과로 보고하지 않는다.

이 후속 보정은 각 쪽 lowering 직후 모든 CharShape 필드와 raw provenance가 정확히 같은
레코드를 문서 공통 pool에서 공유하고, 새 쪽의 Table/Cell/Shape 문단 참조를 재매핑한다.
서로 다른 속성은 합치지 않으며 실제 unique resource 한도는 유지한다. 기존 쪽의 ID·기본
스타일·줄 메트릭은 바꾸지 않는다. 새 Native/fresh WASM·양 저장 및 실제 Studio 검증은
이 보정 뒤 다시 진행한다. 기존 WASM 빌드와 runtime10 수치는 중간 진단으로만 보존한다.

### 재시험용 복구본 — 메인테이너 수용 대기

최종 Native runtime은 `8270ba96e655a94af4e49ddcafbae8379e3b6a8dd37a810fad18833d040b099e`,
fresh Docker WASM은 `0aa9c38c94c46d9ca1f1981b935c975169a9637bb95a69f41f0fa5772ca064df`다.
제품은 아직 uncommitted recovery이며, 코드 HEAD `bd707a7af`만으로 식별하지 않는다.
`/mnt/e/tmp/issue7998-resource-recovery-20261020/final-identity.json`에 tracked patch와
추가 `char_pool.rs`의 해시를 별도로 보존했다. 최종 CLI는 같은 폴더의
`rhwp-recovery-final`이며 옛 중간본 이름 `rhwp-final`의 대체 사실도 identity에 명시한다.

같은 원본 전체의 Native 가져오기·HWPX 저장은24.13030초, Table162/Cell54,938,
보류0, 출력 섹션162다. CharShape은161개로 줄었다. 원본 위치형 저장본과 **각 쪽별**
가시 문자 multiset이 모두 같고 전체89,386개다. 이 수치를 읽기 순서의 증거로 대체하지 않는다.
devel5.73478초보다 여전히 느리다. 옛 후보76.40665초와의 개선만으로 수용 완료를 선언하지 않는다.

기존 수용본과 같은 실제 글꼴 디렉터리(HANBatang/Arial/Batang)를 공급한 전쪽 비교는
Native162쪽 최저99.56743%, fresh WASM162쪽 최저99.56279%, 쪽수 일치·미달0이다.
Native는 같은 글꼴의 devel 대비 하락0쪽이다. 초기 font supply none 비교는 별도 진단으로
보존하며 이를 fonted 수치와 혼용하지 않는다. 중첩 cell 대조군은 같은 글꼴에서 이전 수용본과
PNG가 같다. 1·16·77·95·151·152·153·161·162쪽 HWP 재열기 Native도 확인했지만,
이 새 162쪽 HWP/HWPX의 한컴 열기·인쇄는 미검증이다. 헤더 글꼴 굵기·선 농도의 기존 차이는
직접 비교 화면에 남긴다. 전쪽 TSV와 PNG는 `final-native-fonted/`, `final-wasm-visual/`이다.

Windows Chrome155의 실제 새 WASM 원본 로드는 첫 표시25.128초·162쪽이었다.
처음 CDP 클릭/휠 시도는 기존 문서 복구 대화상자와 화면 밖 좌표 때문에 무효로 분리했다.
복구본은 삭제하지 않고 ‘나중에’로 미뤘으며, 화면 안 실제 셀 클릭은60.93ms·Table CellPath
진입, 휠은694px 이동·최대 프레임 간격52.2ms였다. 수용 판정은 아직 받지 않았다.

독립 headless Studio의 최종 검증에서는 실제 Table의 controlIdx로 대상을 제한했다.
CellPath가 본문 글상자에도 사용되므로 첫 진단의 ‘■’ 글상자 편집은 셀 편집 증거에서 제외했다.
실제 첫 셀 ‘부서명’의 API 삽입42.1ms·삭제37.7ms, 양 저장2.436초,
HWP/HWPX 재열기는8.442/8.459초·각162쪽·첫 쪽274셀로 확인했다.
마우스로 실제 첫 셀에 진입한 뒤 키보드 입력은414ms(입력~두 RAF 포함),
`부서명 → 부X서명 → undo → 부서명`이 유지됐다. 휠560px·최대 프레임 간격22.2ms다.
1쪽274개 셀 중앙에서 장식 개체 선택0·셀 캐럿274를 확인했다. 16쪽의 개체 hit2곳은
실제 bitmap 구두점 그림이며, 이 셀의 위치형 편집 개체 보존 한계는 유지한다.

집중 회귀는 기존 자동 수집 suite008/022에서 총47건 통과했다. 새 ClipPath enum을 누락한
기존 bitmap test visitor의 컴파일 오류는 자식 순회 추가로 바로잡았다. 다른 실패는 같은
글자 속성 공유로 A/B가 한 TextRun이 된 것을 별도 run이어야 한다고 가정한 검사였다.
공개 charX의 글자 위치·원래 ±2px·소유 셀 포함·순서·내용 보존 조건으로 바꿨으며 기대 위치를
완화하지 않았다. 관련5개 원본의 Native/fresh WASM 각5쪽은 모두 raw100%다.
시험 코드만 갱신했으므로 같은 제품의 WASM을 다시 빌드하지 않았다. 전체 P5 회귀는 미완료다.

메인테이너용 자료는 `E:\tmp\issue7998-resource-recovery-20261020\review\index.html`,
원본 `original.pdf`, 최종 Native 저장 `recovered-final.hwp`·`recovered-final.hwpx`,
실제 WASM 저장 `studio-saves\recovered.hwp`·`recovered.hwpx`다.
검증 Studio7793의 pkg/public과 동결 WASM 해시는 같고, root7700 산출물은 변경하지 않았다.
최종 확인 때 CDP review 탭은 닫힌 상태이므로 열어 두었다고 안내하지 않는다.
완료·PR 제출·전체 회귀 통과를 선언하지 않으며 메인테이너의 실제 재시험 판정을 기다린다.

### 2026-10-21 표 인식 수용·가져오기 성능 보정

메인테이너는 “표인식 해결되었지만 속도는 개선해야 할 지점”으로 판정했다.
표 인식 복구는 수용됐으며 같은 실제 입력의 열기 지연을 계속 보정한다. 기존 표/셀 소유와
공통 메트릭의 정확성을 성능과 교환하지 않는다. 전체 P5·제출 보류와 승인된 후속 범위는 유지한다.

복구본 Native를 동결한 채 별도 GDB 실행에서 0.65초 간격32개 스택을 수집했다.
15개가 `observed_line_text → fit_spacing → estimate_text_width_exact` 안에 있었다.
이 표본 비율을 정확한 시간 비중으로 보고하지 않는다. 실제 코드에서 공백의 합법 장평
151개와 각 자간 후보를 검색하며 동일 스타일·문자 조합도 공통 measurer로 반복 계산한다.
원인·출력 계약은 기존 stage3/35/36의 문자 전진폭 복원이며, 탐색 범위·순서·비교 오차를
변경하는 대신 쪽별 불변 SourceStyles에 정확한 측정 결과의 제한된 memo를 둔다.

키는 원본 CharShape ID·언어 슬롯·문자·장평/자간이며, 원래 스타일과 조정 스타일을 구별한다.
원래 스타일은 해당 쪽에서 불변이고 fitting은 장평·자간 두 필드와 ASCII 구두점 슬롯의
같은 두 필드만 변경하므로 다른 상태가 누락되지 않는다. 쪽 종료 시 폐기하며 최대16,384개만
보관한다. 가득 차면 같은 measurer를 다시 실행하며 입력 수용/예산은 바꾸지 않는다.
측정값은 기존 자간 후보 선택 → `CharShape.ratios/spacings`와 typography evidence →
Paragraph char refs/LineSeg → 공통 reflow·paint·양 저장의 동일 경로로 소비된다.
별도 근사 폭·반올림·좌표/쪽 분할 보정은 추가하지 않는다.

실행 원본은 `output/pr-review/issue7998/resource-limit-recheck/perf-20261021/`이다.
첫 GDB 실행은 `start`의 인자 초기화로 CLI 사용법 오류가 났으며 측정에서 제외했다.
수정한 GDB 실행은 원본162쪽을 정상 종료했다. 디버거 없는 전후 비교·구조/저장 동일성 및
새 Native/fresh WASM 검증을 마친 뒤 성능 결과를 기록한다. 현재는 구현·검증 중이다.

반복 메트릭 memo만 적용한 중간본은20.66166초였고, HWPX의178 ZIP entry와 전체 파일
SHA256이 이전 복구본과 같았다. 이후 인식 결과 JSON을 다시 직렬화해 복사하던 두 report
조립 지점은 이미 소유한 Value 배열을 이동하고, ExtractedPage가 중복 보관하던 recognition
report는 원래 소유자에서 직접 읽도록 했다. detection은 기존 snapshot 형태로 이동한다.
초기 내부 field rename의 누락으로 컴파일1회 실패했으며, 기존 positional 진단의 의미를
바꾸지 않도록 snapshot 이름을 유지해 바로잡았다. 실행 성능/구조 결과에 합산하지 않는다.

최종 Native `66cab1a817dc43ec555496d94e937525c5591bbca9f3befb89f8985fee34e120`은
19.90085/18.45132초였다. 같은 실행안의 수정 전 복구본 추가 측정은25.07800초이며
이전24.13030초도 원본으로 유지한다. 24~25초 → 18~20초의 제한된 실제 입력 개선이다.
측정 중 Cargo/브라우저 검증을 겹치지 않았다. debugger 시간을 섞지 않는다.
최고 RSS는 전후약326,000KiB로 감소를 주장하지 않는다. 3개 측정의 표162/셀54,938,
출력162쪽·보류0과 HWPX 전체 바이트가 같고, 출력 목적지 경로만 제외한 JSON 진단도 같다.
처음 검증 스크립트가 목적지 경로 차이까지 불일치로 판정한 것은 비교 오류이며,
동일한 첫 실행 결과를 재사용해 진단 전체를 대조했다. 제품/측정은 다시 실행하지 않았다.
새 Native 전쪽 출력과 focused 회귀·fresh Docker WASM 검증은 진행 중이다.

최종 fresh WASM은 `b0d52b582d96dc23e7bab19819a5224e832c14549cfa830fc5a929c2816ebb98`다.
검증용 Studio7793의 pkg/public과 실제 Chrome 로드 해시가 같았다. 동일 headless Chrome152의
첫 표시 측정은 이전28.2723초 → 현재17.8191초다(각 제품1회, Windows25.1초와 혼용하지 않는다).
실제 첫 셀의 `부서명 → 부X서명 → undo → 부서명`을 확인했고, API 삽입40.2ms·삭제35.0ms,
키보드 입력~두 RAF397ms, 양 저장2.348초, HWP/HWPX 재열기8.321/7.988초였다.
각162쪽/첫274셀을 유지했다. 휠560px 이동·최대 프레임 간격23.6ms이며 브라우저 오류0이다.
16쪽의 bitmap 구두점2개 외에 표 장식 개체가 셀 클릭을 가로채지 않는 것도 유지됐다.
두 실제 WASM 저장 파일은 이전 복구본의 같은 API 편집/복원 저장과 각각 바이트가 같다.

집중 suite008/022는34+13=47건 통과했다. 새 Native/fresh WASM은 각각180쪽에서
90% 미만·누락·쪽수 차이0이다. 실제 원본162쪽 최저는99.56743%/99.56279%이며
양쪽162개 PNG가 각각 이전 복구본과 같다. 추가 원본 보고서12쪽·중첩 셀1쪽·공백5쪽도
통과했고, 보고서/셀의 Native HWPX는 이전 복구본과 바이트가 같다. 공백5쪽은 양쪽100%다.
같은 원본/PDF에서 새로 생성한 출력의 전체 TSV를 확인했다. 새77쪽의 Native/독립 PDF를
직접 대조해 기존 헤더 굵기·가는 선 농도 차이를 구분했고, 이미 수용된 별도 범위를 다시
완료로 바꾸지 않았다. 전체 P5와 이162쪽 저장본의 한컴 열기·인쇄는 여전히 미완료다.

메인테이너 재시험은 `http://localhost:7793/?review=issue7998-performance-20261021&lang=ko`,
원본 `/mnt/e/tmp/reousrce_limit.pdf`를 사용한다. `E:\tmp\issue7998-perf-20261021\`에
원본·저장본·전쪽 비교 화면·TSV·시간 비교·source/runtime 지문을 보존했다.
현재 제품은 uncommitted이며 이전 복구 산출물을 덮어쓰지 않았다. 성능 보정의 로컬 검증과
재시험 준비를 마쳤지만 전체 #7998 완료·PR 제출·사용자의 새 속도 수용으로 보고하지 않는다.

### 2026-10-21 속도 수용과 추가 font metric 입력

작업지시자는 “이제 기다릴만하네요”로 위 성능을 수용했고,
`/mnt/e/tmp/font_metric.pdf`의 `PDF_RESOURCE_LIMIT: font metric entries`를 보고했다.
새 입력은4쪽·312,237bytes·SHA256
`da368c37dd117d78cd220f8e96b37f322fbaa68244e9a3b0ee7df0c95fae1ab2`다.
앞선 속도·표 인식 수용을 취소하거나 다른 미완료 구현을 확대하지 않고 이 실패를 보정한다.

현재 동결 CLI에서 같은 오류를 재현했다. 독립 pypdf/fontTools 관측은 맑은 고딕2개 face와
바탕1개 face이며 TrueType 매핑27,133/27,133/20,609개와 별도의 CID 폭을 갖는다.
특히 바탕의 `/W [0 34794 1000]`은34,795개 CID 폭이다. 글꼴 데이터·언어별 폭을 버려
수용하지 않는다. 실제 실패는 첫 쪽의 text 단계에서 font counter99,997+4 >100,000이다.
아직 content parse0회여서 여러 쪽을 반복 해석한 실패로 분류하지 않는다.
명시적125,000/250,000 policy의 진단 가져오기는4쪽 모두 성공(각0.418/0.372초)했고
같은 누적 decoded 작업99,284,682bytes였다. 한도 없이 실행한 결과가 아니다.

선행 계약은 #7911 → PR#7913/`4662c069e`의 stage2 3.22 font 예약·cache 격리와
정식111건 자원 검사다. 후속 PR#7970/`515bd72dc`는 제품의 선택 쪽수·누적 decoded/
연산/entry 정책을 조정했으나 font_entries는 raw decoder 기본100,000을 그대로 받았다.
두 PR의 본문·리뷰/댓글과 실제 코드·기록을 대조했다. raw decoder 한도나 선예약·sticky
실패는 변경하지 않고 제품 `PdfImportPolicy`의 유한 상한만1,000,000으로 조정한다.
항목당128bytes decoded 선계상, 선택 쪽수에 따른 decoded 상한(4쪽은256MiB),
CMap scan·객체·font cache 한도 및 호출자가 명시한 작은 font 한도는 유지한다.
글꼴 자원은 한 쪽에도 여러 face가 공유되므로 쪽수만으로 축소하지 않는다.

새 정식 반례는3 CID face×40,000개 폭의 **paint 없는 자원 계약**이다.
기존 자동 수집 suite001의 검사 소스에서 명시100,000/119,999 거절과120,000 정확 경계
수용을 함께 보호한다. 처음 빈 stream은 font loading을 생략해 적절한 반례가 아니었다.
paint 없는 비어 있지 않은 BT/Tf/ET로 수정했고, 제품 변경 전 동결 rlib에 같은 검사 함수와
입력 helper를 직접 연결해 기본 요청80,000+40,000 >100,000의 실제 FAIL을 확인했다.
미수정 제품의 Cargo 검사 FAIL로 바꿔 보고하지 않으며 직접 연결 명령과 로그를 보존했다.
첫 단축명+exact 실행0건은 통과로 세지 않고 전체 module 이름으로 실제 FAIL을 확보했다.
제품 수정 전 진단 policy로 생성한 HWPX의 Native4쪽은 독립 PDF 대비 최저99.88105%였다.
이는 제품 기본 요청의 수정 후·fresh WASM 검증을 대신하지 않으며 새 빌드에서 재확인한다.
실행·선행 자료·원본 관측은 `output/pr-review/issue7998/font-metric-20261021/`에 보존한다.
제품 보정 뒤 자동 수집 suite001의 전체 자원124건은 PASS했다. 원본에는 개인 정보가 있어
PDF·PNG·실제 저장본은 ignored output 및 로컬 폴더에만 유지하며 공개 회귀는 합성 자원
입력을 사용한다. 전체 P5·PR 제출 조건은 아직 마감하지 않는다.

추가 입력의 최종 Native runtime은
`73e2a65eab3f74ca4e8119f4f8fd1c0f94de64cf2e249ab18a7ebb7849650310`이며
로컬 `E:\tmp\issue7998-font-metric-20261021\identity.json`에 source 지문도 함께 기록했다.
최종 fresh WASM은 `afc5e0e5a7753b05cd2495e2e00e1c1f94c13905ea3ae08138e05f15ccf81f87`이며
pkg/public과 실제 Studio 로드 해시가 같다. 새 기본 요청 Native/fresh WASM은4쪽 모두
누락·쪽수 차이·90% 미만0이며 최저99.88105%/99.80979%다.
실제 Studio에서는2.146초에4쪽을 열고, HWP/HWPX 양 저장·재열기와 쪽별 표시 문자
digest가 유지됐다. 기존162쪽도17.707초·첫274셀로 열렸다. Native의 해당162쪽 HWPX는
직전 속도 수용본과 전체 바이트가 같다. 이 실행을 전후 반복 성능 통계로 확대하지 않는다.

재시험은 `http://localhost:7793/?review=issue7998-font-metric&lang=ko`에서 원본을 연다.
`E:\tmp\issue7998-font-metric-20261021\review\index.html`에4쪽 전부 비교와 원본·
실제 WASM 저장본을 준비했다. 해당 개인 정보 문서의 이미지·PDF·저장본은 로컬에만 두고
정식 resource 반례에 실제 내용을 넣지 않았다. 기존180쪽 시각 증거는 직전 성능 제품에
연결하며 이번 font 정책 보정의 새 캡처와 구분한다. 전체 #7998 마감·원격 반영은 미완료다.


### 2026-10-21 font metric 수용·추가 recognition 회귀

작업지시자는 “처리 확인했습니다”로 font metric 입력의 보정을 확인했다.
새 입력 `/mnt/e/tmp/tb_ownership_inf.pdf`는34쪽·1,572,698bytes·SHA256
`378db14c4196dfea4e279ecf0adaf96d706256d40c4ac7fc72f26b910c2827f1`다.
직전 수용 CLI는 원본5쪽(source_page4) recognition에서 실패하며, 고정 devel CLI는34쪽을
정상 가져온다. 이번 작업의 회귀로 조사하며 새 입력을 버리거나 오류를 성공으로 바꾸지 않는다.
실패 원본과 devel 출력·진단은 `output/pr-review/issue7998/tb-ownership-20261021/`에 둔다.
기존 extract의 map_err가 세부 원인을 버리므로 공개 오류 문자열은 유지하고 opt-in
진단에 실패 규칙과 source page를 보존한다. 선행 계약은 stage1의 단일 소유·보류 보존,
P3 `b923b179a`의 본문 구역 분할과 stage9 `99116a2fb`의 셀 지역 소유다.
진단 빌드·실패 조건 확인 중이며 아직 수정·검증 완료로 판정하지 않는다.


원인은 본문 partition과 recognition이 서로 다른 span 공간 범위를 사용한 것이다.
원본5쪽 span48·155는 클립 밖의 공백 전진폭이며, 소유 선택 helper는 유효한 원본 clip을
사용한다. 그러나 body_regions는 raw advance bbox로 구역을 나눠 위 구역을269.91pt에서
종료했다. 소유 clip은222.36–474.35pt에 걸쳐 있어 후속 contains 검사와 모순됐다.
spans/text-show의 원래 좌표·폭·클립을 변경하거나 공백을 삭제하는 문제가 아니다.

`body_regions::partition`이 active span clip을 받고 detector/recognition과 같은
`span_ownership_bounds`를 소비하도록 했다. 이 결과는 본문 구역의 소속·구역 bbox →
recognize_region/make_lines의 유효성 확인 → 원래 glyph 메트릭의 문단/positional lowering →
공통 IR·renderer·양 저장으로 이어진다. glyph 측정/paint 좌표와 선 높이·쪽 분할은 바꾸지 않는다.
소유 contains 검사, 모호한 제안의 보류, 원문 보존과 유한 자원 한도도 유지한다.
공개 오류 문구는 유지하며 opt-in 진단에 recognition 규칙과 원본 쪽을 남긴다.

작은 독립 PDF는 넓은 제목·두 단·clip 밖의 blank advance를 갖는다. 초기 진단본은 제목이
용지 끝을 넘는 별도 입력 오류가 있었으므로 결함 재현 증거에서 제외하고 제목만 올바르게
재작성했다. 최종 세 입력 중 clip 공백만 수정 전 recognition FAIL이며, clip 공백을 뺀 대조군과
clip 없는 대조군은 수정 전에도 PASS다. 두 정상 대조군의 HWPX는 수정 후와 바이트가 같다.
같은 정식 검사 함수/입력 helper를 동결한 수정 전 rlib에 직접 연결하면 원래
PDF_RECOGNITION_FAILED로 FAIL, 수정 후 rlib에 연결하면 PASS다. Cargo 수정 전 실행으로
바꿔 주장하지 않는다. 최초 검사 초안의 모든 drawing control 순서=읽기 순서 가정은 제거했다.
최종 검사는 단일 소유·문자 누락/중복 없음·단별 줄 순서·원래 공백 좌표/전진폭·실제 Paragraph
그룹과 HWP/HWPX 재열기를 검사하며, 새로운 읽기 순서 복원 완료를 주장하지 않는다.

Native/fresh Docker WASM 각37쪽은 누락·90% 미만·쪽수 차이0이다. 실물34쪽 최저는
92.89699%/92.38421%이며 세 합성 입력은 양쪽 모두100%다. 시각 선행 조건을 확인한 뒤
기존 자동 수집 `issue_7998_pdf_table_inference.rs`에 관계형 회귀1건을 추가했다.
고정 devel Native34쪽 대비 raw 하락0쪽이고, 실패했던5쪽 PNG는 devel과 바이트가 같다.
2쪽의 기존 글꼴 굵기·점선 폭, 5쪽의 배경·세 단·원문 보존을 직접 대조했다. 기존 글꼴 차이가
해결됐다고 보고하지 않는다. 이 문서는 의미 표0/보류24이며, 이번 오류 해소를 원 작성 표의
복원 완료로 승격하지 않는다. 불확정 표 후보는 기존처럼 원문 위치 표현을 보존한다.

최종 동결 Native는 `35afb87ba26a9f9fdaffd68d1a88f4a3dc8a55e4a911de953c6e52c38b646347`,
fresh WASM은 `32724c8f424594e6d049dc550383aaf2d3c468d18f15f084a0e6dad8c6de74f7`다.
Docker 빌드는8분14초에 완료했으며 pkg/public·실제 Studio 로드 해시가 같다.
빌드 뒤 extract.rs의 rustfmt 들여쓰기만 바뀌었고 규칙/표현식 변경은 없다.
컴파일 source patch와 서식 보정 후 patch를 각각 보존하고 동일 실행 산출물을 재사용했다.
`E:\tmp\issue7998-ownership-20261021\identity.json`에 두 patch와 추가 char_pool 해시를 연결했다.

실제 Studio에서는 원본34쪽이2.497초에 열리고 HWP/HWPX 재열기는1.645/1.561초였다.
각34쪽·쪽별 표시 문자 digest가 유지됐으며 브라우저 오류0이다. font_metric4쪽과 기존162쪽도
같은 WASM에서 정상 열렸다(162쪽17.610초·첫274셀). Native의 두 기존 입력 HWPX는 이전
수용본과 각각 전체 바이트가 같다. 이1회 실행을 통계적 성능 개선으로 확대하지 않는다.

로컬 비교 자료는 `E:\tmp\issue7998-ownership-20261021\review\index.html`,
원본·실제 WASM 저장본은 같은 상위 폴더의 original.pdf/studio-saved.hwp/studio-saved.hwpx다.
검증용 Studio는 `http://localhost:7793/?review=issue7998-ownership&lang=ko`이며 root7700은
변경하지 않았다. 이 저장본의 한컴 열기·인쇄와 전체 P5·원격 제출은 이번 완료 범위가 아니다.
정식 집중 검사 실행 중이며 source·검사·로그는 같은 ignored 증적 폴더에 연결한다.


정식 자동 수집 suite008/012에서 table_inference28건·body_paragraphs4건, 총32건이
모두 PASS했다. 새 클립 공백 회귀도 포함하며 다른 모듈은 실행한 것으로 세지 않는다.
최종 fmt와 git diff --check도 PASS다. 이 로컬 보정·재시험 준비를 마치며 사용자의 새
시각 판정과 전체 P5 완료를 자동 선언하지 않는다. 승인된 기존 백로그와 원격 보류는 유지한다.


### 2026-10-21 소유권 수정 수용·ASCII85 마스크 입력

작업지시자는 앞선 소유권 보정에 “해결 확인했습니다”로 회신했고,
`/mnt/e/tmp/bounded_import_path-02.pdf`의 PDF_IMAGE_UNSUPPORTED를 보고했다.
새 입력은5쪽·228,901bytes·SHA256
`0ba1f7d5b72b8bd92e19b1198017dc0efb4bbd68ec0bdde14dfb94ef3e0523b5`다.
ReportLab 생성 PDF의3·4쪽 이미지4개와 DeviceGray8-bit SMask가 모두
`/Filter [/ASCII85Decode /FlateDecode]`를 사용한다. 현재/고정 devel 모두3쪽에서
`limited image mask filter`로 거절하므로 앞선 본문 소유권 수정의 회귀가 아니다.

#7950 → PR#7953/`b7a538803`의 StreamMask, #7974 → PR#7980/`f9d2ab17a`의
색상 키/stencil 샘플 경로를 이슈 전체·PR 본문/리뷰/댓글·실제 커밋과 stage 보고서로 추적했다.
StreamMask만 Filter없음/단일Flate로 제한하고 있었으며 색상 키/stencil은 같은 요청 예산의
ASCII85/Hex/Flate/LZW/RunLength byte filter pipeline과 정확한 packed-row 길이 검사를 사용한다.
독립 근거는 PDF Reference1.6 §3.3 및 PDF Association의 ISO32000-2 §7.4 예시다:
https://pdf-issues.pdfa.org/32000-2-2020/clause07.html#74-filters

보정은 StreamMask도 기존 source-sample helper를 소비하도록 연결하는 것이다.
실제 호출은 handle.decode → StreamMask.read → 공통 samples →
decode_stream_with_encryption → 요청 stream_parameters_limited → bounded filter 각 단계 →
정확한 행 길이 확인 → 원래 Decode/Matte/alpha 합성 → PNG/BinData → 공통 IR/paint다.
픽셀 codec(DCT/JPX/JBIG2/CCITT)의 원시 샘플 오독은 계속 거절하며 DecodeParms 지원 범위,
filter depth·중간 출력/누적 decoded 예약·행 크기·취소·sticky 실패 한도를 유지한다.
새 decoder 구현·자원 상한 변경·마스크 생략은 추가하지 않는다.
실행/선행 자료는 `output/pr-review/issue7998/image-codec-02-20261021/`에 보존하며
현재는 구현·집중/시각 검증 중이다. 전체 P5와 원격 절차 완료로 보고하지 않는다.


새 정식 검사는 합성 **decoded-resource 계약**이며 렌더링 golden을 추가하지 않는다.
같은 입력을 사용하는 동결 수정 전 backend 연결 검사에서 `limited image mask filter` FAIL을
확보했고, 수정 후 자동 수집 suite001의 자원125건과 suite021의 기존 마스크8건,
총133건이 모두 PASS했다. 새 검사는 두 mask 유형의 lossless byte filter7경로, 실제 RGBA,
정확한 누적 decoded 경계와 filter 깊이의 sticky 실패·미지원 pixel codec·손상 ASCII85·
packed-row 길이 오류·기존 DecodeParms 거절을 보호한다. 검사 초안의 빌드/실행 실패와
실제 제품 결함을 구별하며 제품 수정 전 CLI도 같은3쪽 오류를 재현했다.

새 Native에서 원본5쪽을 열었고 독립 pypdf로 해석한 원본 RGB/8-bit alpha plane을 합성한
4개의 전체 RGBA SHA가 저장 HWPX의 PNG 크기/픽셀과 정확히 같다. 이를 원본 PDF의
paint 전체 일치나 작성 당시 표 구조의 정답으로 확대하지 않는다. 기존 explicit Mask·고해상도
SMask·Matte·색상 키·stencil5입력의 HWPX는 직전 수용 제품과 각각 바이트가 같다.
새 원본의 마스크 없는1·2·5쪽을 같은 선택(`0-1,4`)으로 가져온 HWPX도 수정 전후 같다.

Native 전쪽 비교는 원본5쪽 최저90.25971%, 기존 대조5쪽은 모두100%다.
새 지원 차트3·4쪽은99.24530%/97.05735%다. 입력이 사용하는 NanumBarunGothic/굵은 face의
실제 파일을 확인한 `/usr/share/fonts/truetype/nanum`을 공급했다.5쪽 등의 기존 글꼴 굵기·
자간 차이는 직접 원본/Native를 대조해 남겼으며 마스크 지원으로 해결됐다고 주장하지 않는다.
동결 Native SHA는 `effdae3a0dfd2b4f0bde4f810233f44f2b3ee95df840b5509fa5f20b889d46ea`다.
실행 source 지문은 `E:\tmp\issue7998-image-codec02-20261021\identity.json`에 기록했다.
fresh Docker WASM·실제 Studio·양 저장/재열기 검증을 이어가며 아직 완료로 판정하지 않는다.


최종 fresh Docker WASM은 `f6e4b12877759ec3c7ed19ce5cb25a4785622502849c6555810623e7f2667f1f`이며
9분03초에 완료했다. pkg/public과 실제 Studio 초기화 응답 해시가 같다. 제품 source는 Native
검증 뒤 바뀌지 않아 같은 지문을 사용했다. Native/fresh WASM 각10쪽은 누락·쪽수 차이·
90% 미만0이며 실물5쪽 최저90.25971%/90.46384%, 기존 대조5쪽은 양쪽 모두100%다.
새 지원 차트3·4쪽의 WASM은99.24798%/97.21453%다.3쪽 원본·Native와 실제 Studio 화면에서
차트·투명 배경·주변 문구를 직접 대조했다.5쪽의 기존 굵기·자간 차이는 남으며 glyph 개선
완료로 보고하지 않는다. source의 masked3·4쪽만 검사한 것이 아니라 전체5쪽을 비교했다.

실제 Studio에서1.564초·5쪽을 열었다. HWP/HWPX 저장·재열기는 각5쪽·쪽별 표시 문자
digest와 의미 표2개(20셀/77셀)를 유지했다. 원래 작성 구조의 완전한 독립 정답으로 수량을
사용하지 않는다. WASM 저장 HWPX와 HWP/HWPX 재열기 뒤 다시 소비한 PNG도 모두
독립 source RGBA4개와 크기·픽셀이 정확히 같다. 별도 UI paint 검증에서 실제3·4쪽으로
이동해 각각 image2개가 있는 Canvas2D 페이지를 표시·캡처했고 browser error0을 확인했다.
이는 앞선 저장 API 검증과 다른 실제 차트 표시 경계이며 전체 API 검사를 반복하지 않았다.

같은 새 WASM의 소유권34쪽·font_metric4쪽도 정상 열렸고, 기존162쪽은17.526초·첫274셀을
유지했다. 이1회 실행을 통계적 성능 개선으로 주장하지 않는다.133건의 정식 집중·fmt 및
vendor rustfmt·최종 diff 검사는 PASS다. 긴 P5 전체 회귀/한컴 인쇄·원격 등록은 미완료다.

재시험 URL은 `http://localhost:7793/?review=issue7998-image-codec02&lang=ko`이며,
`E:\tmp\issue7998-image-codec02-20261021\review\index.html`에 전쪽 비교와 원본·
실제 WASM 저장본·전쪽 TSV를 준비했다. Studio3·4쪽 캡처는 같은 상위 폴더의
studio-page-3.png/studio-page-4.png다. 로컬 보정과 재시험 준비를 완료하며 새 메인테이너
시각 판정을 자동 선언하지 않는다. root7700과 원격 브랜치는 변경하지 않았다.

### 2026-10-21 마스크 수정 수용·반복 stencil XObject 입력

작업지시자는 앞선 ASCII85 마스크 보정을 확인했고 `/mnt/e/tmp/xo_visits.pdf`의
`PDF_RESOURCE_LIMIT: XObject visits`를 보고했다. 새 입력은 18쪽·1,307,972 bytes,
SHA256 `2c7cf86272ff1a8535e4a8c5db0a20b685268fa43199bef627593c25d49c7f97`다.
고정 devel과 직전 수용 Native 모두 같은 오류이며 현재 실패는 4쪽 text 경로의 누적
10,000 + 1회 예약이다. 앞선 마스크 보정이 만든 회귀로 분류하지 않는다.

#7911 → PR #7913/`4662c069e`의 요청별 방문량 도입, #7957–#7971 → PR #7970/
`515bd72dc`의 요청 정책·실제 이미지 배치·원본 paint 순서 계약을 실제 반영 diff와
PR 본문/논의, stage2/stage8 기록으로 대조했다. 텍스트의 두 해석과 path·image-handle
해석은 같은 요청 방문 예산을 소비한다. cache hit도 작업이며 반복 paint를 삭제하지 않는다.

독립 pypdf content walk에서 원본의 XObject 호출은 201,834회다. 4·6·8·9·11·13·15·17쪽은
각각 8×8, 1-bit ImageMask 하나를 25,228회 타일로 그리고 JPEG 한 개를 별도로 그린다.
고유 그림이 25,229개인 것이 아니다. 위치는 0.001pt 단위의 약간 다른 간격이므로 정수 격자로
좌표를 반올림해 복사하지 않는다. 해석 한도만 올리고 모든 배치를 IR 개체로 내보내는 방식은
편집·표 추론 비용과 저장 자원 ID 한도를 악화시킨다.

보정 범위는 선택 쪽수/유한 operator 규모에 맞춘 제품 visit/reference 정책과 반복 stencil의
배경 그림 투영이다. 원본 방문·배치·source pixel은 그대로 계상하고 명시한 host cap·raw
backend 기본값·depth·decoded·pixel 한도는 유지한다. source page traversal 상한도 기존
100,000쪽을 유지한다. 본문·path 사이에 paint barrier를 두고 동일 자원/색/alpha의 축 정렬
stencil 연속 구간만 한 번 decode한다. 실제 CTM·사각 clip을 각 출력 pixel 중심에서 적용해
source-over로 합성하며 글자·표는 rasterize하지 않는다. 다른 자원·색·비사각 clip·방향·
보간 방식과 중간 글자/선은 개별 경로를 사용한다. 원래 작성 프로그램의 그림 객체를 복원한
추론이 아니라 반복 배경 paint의 bounded 저장 투영이다.

수정 전 실물/독립 3,072회 stencil 대조 입력의 같은 오류와 관측량을 ignored
`output/pr-review/issue7998/xobject-visits-20261021/`에 보존했다. 현재 Native 집중 검사와
실물 전쪽 검증을 진행하며 전체 P5·시각 판정·원격 등록 완료로 보고하지 않는다.

최종 Native 자원128건·기존 text paint 순서2건·마스크8건·이미지 배치3건, 총141건이
PASS했다. 새 3,072회 입력은 정식 검사와 같은 생성 함수의 바이트를 동결 수정 전 제품에
전달해 10,000회 경계에서 FAIL하고 수정 후 12,288회 계상으로 PASS했다. 낮은 명시적
visit/placement/reference cap도 거절하며 다음 정상 요청은 성공한다. 중간 문자·색상 변경은
투영 구간을 나누고 alpha 0은 빈 배경 그림을 만들지 않는다. 새 검사들은 decoded 샘플·
작업 예약 계약이며 렌더링 좌표/golden을 추가하지 않았다.

검토 중 투영 거절 뒤 매 suffix를 재탐색할 가능성을 찾아 선형 fallback 구간을 유지하도록
보완했다. 일반 packed 1-bit gray 512회와 reference cap 1,600의 같은 입력은 보완 전
image_decode에서 cap FAIL, 보완 후 원래 512개 그림을 보존해 PASS했다. source stencil
샘플에 이 작은 보완은 영향을 주지 않아 18쪽 저장 HWPX는 보완 전후 바이트가 같다.
이 수정 때문에 중간 WASM 빌드를 중단하고 최종 source로 다시 빌드하며, 이전 WASM을
최종 증거로 재사용하지 않는다. 원본·중간 진단·보완 전/후 실행은 같은 ignored 폴더에 남겼다.

전체 18쪽 Native TSV의 최저는 2쪽 88.80969%이며 기존 devel의 같은 선택 PNG와 정확히
같다. 직전 수용본/현재 후보의 선택 HWPX도 같다. 독립 PDF와 직접 비교하면 NanumSquareB
글꼴 굵기 차이가 남는다. 실제 source face 파일을 공급했고 폰트 파일 부재로 판정하지 않는다.
4쪽 등 반복 배경은 raw 100%인 경우에도 미세 무늬가 달라 source-over 투영의 샘플링 차이를
직접 확대에 남겼다. 이 차이는 해결하거나 승인받기 전 시각 완료로 표시하지 않는다.
작업지시자에게 구체적인 비교 화면과 함께 범위 판정을 요청했으며 아직 예외를 수용했다고
기록하지 않는다. Native의 작은 자원 대조 1쪽은 100%다.

실제 소비 경로를 대조하다가 선택 후 traversal 축소가 metadata에만 반영되는 초안 결함을
발견해 실행 경로를 보정했다. ImportBudget에 단조 감소하는 reference/xobject 실효 상한을 두고,
`for_selected_pages → restrict_traversal_limits → charge_reference_visit / xobject_visit`를
연결한다. 페이지 트리에서 이미 소비한 양은 초기화하지 않으며 cache hit를 포함한 모든
호출이 같은 실효 상한을 소비하고 첫 실패는 sticky 상태로 유지한다.

70,000개의 같은 CTM `/Im Do`와 VISIBLE 문구로 만든 **자원 계약**에서 초안은 선택 한도
250,000으로 보고하면서 실제 280,000회 해석을 허용했다. 동결 Native CLI와 동일 입력의
JSON에 보존했다. 최종 정식 검사는 같은 입력을 실제 250,000 + 1 경계에서 거절한다.
별도 직접 budget 검사는 상한을 다시 크게 요청해도 증가하지 않음, 첫 실패 유지, 이미
소비한 reference보다 작게 줄이면 즉시 실패함을 확인한다. 이 보완 뒤 최종 집중은
자원130 + text 순서2 + 마스크8 + 배치3 = 143 PASS다. 중간 E0460 링크 실패는 결함 재현으로
세지 않았고, 실제 보정 전 CLI/보정 후 정식 검사와 공개 CLI 거절을 사용했다.

최종 Native의 원본 18쪽 가져오기는 단일 실행 약 2.97초였다. 807,336 XObject visits,
404,355 reference visits, 실제 parse 18회/cache hit 54회와 누적 decoded 작업
823,899,677 bytes를 관측했다. 이는 live heap/RSS 상한이 아니다. 작은 실행 값의 범위를
넘는 보편적 성능 주장을 하지 않는다. 한도 연결 보완 전후 실물 HWPX는 바이트가 같고,
최종 Native에서 전쪽을 다시 출력해 원본18 + 자원대조1쪽의 TSV를 갱신했다. 2쪽 미달과
배경 직접 판독 보류는 그대로다. 마스크5쪽·font4쪽·소유권34쪽의 Native HWPX도 이전
수용본과 각각 바이트가 같다. 최종 source 지문은 새 identity.json에 연결하며 fresh WASM을
기다린다. 이전 중단 빌드나 해당 뒤의 pkg/public 불일치 오류는 최종 WASM 검증이 아니다.

최종 Docker WASM은 574초에 완료했고 SHA256은
`0d1b44ce52a56d2eae728f0b871a9ae0747c337edbb7ea318d957f056737b68b`다.
pkg/public의 JS·WASM 바이트와 실제 Studio 초기화 응답 지문을 확인했다. Native는
`2886a124de31332e2879acef2785d1f51ebc9500e799637d4fc490686bdd1add`이며 source patch/new-file
지문은 `E:\tmp\issue7998-xobject-20261021\identity.json`에 기록했다. 마지막 Native 뒤에
제품 source를 바꾸지 않았고 기존 중단 빌드와 구분했다. 최종 fmt/vendor rustfmt/diff는 PASS다.

실제 새 Studio에서 원본은 약 4.680초·18쪽으로 열렸다. HWP/HWPX 양 저장과 재열기는
각18쪽·쪽별 표시 문자 digest·표 셀 수를 유지했다(작성 당시 구조의 독립 정답으로 세지 않음).
2·4·17쪽의 실제 Canvas2D 표시와 browser error 0도 확인했다. 기존 마스크5쪽·소유권34쪽·
font4쪽은 정상 열렸고, 마스크의 PNG4개는 독립 source RGBA 전체 픽셀과 같았다. 기존162쪽은
약17.283초·첫 표274셀을 유지했다. 이 단일 실행들을 일반적인 속도 보장으로 확대하지 않는다.
한컴의 이 새 HWP/HWPX 열기·인쇄 호환성은 이번 확인 범위가 아니다.

Native/fresh WASM 각19쪽(원본18 + 자원대조1)의 전체 TSV·PNG를 보존했고 누락/쪽수 차이는
0이다. 실물 최저88.80969%/88.39467%는 양쪽 모두2쪽이며 자원 대조는 양쪽100%다.
2쪽 글꼴과4쪽 등 배경의 실제 무늬 차이를 review/overlay·글자 없는 같은 영역의 4배 확대에
명시했다. 자동 점수와 별개로 직접 판독했으며 시각 PASS로 바꾸지 않는다. 범위 예외/추가
보정의 작업지시자 답변은 아직 대기 중이므로 `re_review_required`와 전체 P5·원격 보류를
유지한다. 준비 완료는 이 **로컬 열기 후보와 재시험 환경**에 한정한다.

Windows CDP의 기존 탭은 보존하고 `http://localhost:7793/?review=issue7998-xobject&lang=ko`
새 탭에 실제 원본18쪽을 열었다. 전쪽 비교·원본·실제 양 저장본·TSV는
`E:\tmp\issue7998-xobject-20261021\review\index.html`에 연결했다. source marker는
bd707a7af + 위 uncommitted 지문이며 root7700·커밋·원격 브랜치·PR은 변경하지 않았다.

### 2026-10-21 작업지시자 범위 판정 — 열기 개선 수용·출력 차이 후속 분리

작업지시자가 **“열기 개선으로 한정·출력 차이는 백로그”**로 답변했다. 위의 판정 대기는
이 답변으로 종료하며, `xo_visits.pdf`의 18쪽 열기·반복 처리 개선을 이번 로컬 범위로
수용한다. 2쪽 글꼴 굵기와 4·6·8·9·11·13·15·17쪽 반복 배경의 미세 무늬는
[승인된 후속 백로그](../plans/task_m100_7998_backlog.md#추가-입력-xo_visitspdf의-출력-차이--승인된-후속-백로그-2026-10-21)에
원인 추적·검증 조건과 함께 남겼다. 새 GitHub 이슈를 등록한 상태는 아니다.

Native/fresh WASM 최저88.80969%/88.39467%, 자동 `re_review_required` 및 직접 판독의
출력 차이는 수정하거나 통과로 바꾸지 않는다. 이는 작업지시자가 정한 열기 범위의
수용이며 전체 시각 피델리티·한컴 저장 호환·#7998 P5·PR 완료 판정은 아니다.

제품·검사 소스와 Native/WASM 산출물이 최종 identity의 지문과 같음을 확인했다.
기존 집중143건·Native/fresh WASM 각19쪽 TSV·실제 Studio18쪽 열기 및 양 저장·재열기
결과를 재사용하며 같은 빌드·검사를 반복하지 않는다. 이번 갱신은 계획·백로그·보고·로컬
비교 화면의 범위 판정에 한정하고, 측정 원본과 커밋·원격·PR 상태는 변경하지 않았다.

### 2026-10-21 추가 extraction-002.pdf — 회색 Mask 해석 경계

사용자 입력은1쪽·102,959 bytes·SHA256
`6e63f7295fecf18a013ad221ea43ec9c524f1ed249e37128b211851312a0f0f7`다.
직전 최종 Native에서도 `PDF_PARSE_FAILED`이며 opt-in 진단은 `image_decode`,
`explicit Mask must be a 1-bit image mask`다. 컨테이너·글자·path 추출은 성공했고
512×256 JPEG의 `/Mask`가 `/ImageMask` 없는 DeviceGray8비트 그림을 가리킨다.
이 원본의 마스크 샘플은 모두255다. 원본은 보존하고 진단용 SMask/Mask 제거본과
원본 Poppler 출력만 ignored 증거에 분리했다. 진단본을 원본 검증으로 바꾸지 않는다.

독립 근거는 PDFium `CPDF_DIB::StartLoadMask → StartLoadMaskDIB`의 실제 일반 그림
마스크 경로다. [공식 소스](https://pdfium.googlesource.com/pdfium/+/refs/heads/main/core/fpdfapi/page/cpdf_dib.cpp)의
blob `60224e2a4b73645f1970c488435b47979743dde1`은 Mask stream도 grayscale alpha로
읽으며 Matte의 unblend는 SMask 분기에만 적용한다. 표준 명시적 stencil의 역극성과
구별한 호환 해석을 적용하고, 회색1/2/4/8비트·Decode·SMask 우선권을 같은 제한 디코더로
연결한다. 색 공간/깊이/표본 길이·중첩 거절·취소·예약 상한은 유지한다.

소비 경로는 `StreamMask::read의 분류 → 공통 샘플 decode/예약 → alpha → apply의 RGBA
→ PdfImageHandle::decode → PDF picture IR`이다. 그림 CTM/clip/원점·문자·표 추론 규칙은
변경하지 않는다. 수정 전/후 decoded-resource 검사와 정상 stencil/SMask 대조를 먼저
실행하고 실물1쪽 Native/fresh WASM·Studio 열기/양 저장·재열기를 대조한다. 이 원본의
시각 완료와 기존 P5 완료는 아직 판정하지 않는다.

기반 이슈 #7950과 통합 PR #7953·실제 merge `b7a538803`의 원래1비트 Mask/SMask
범위를 대조했다. 현재 실패의 strict 분기는 그 최초 지원부터 존재하며 직전 XObject
보정이 만든 회귀로 분류하지 않는다. 당시9개 SMask/Matte 샘플의 증거와 이번 일반
DeviceGray Mask를 구분했다. 원 논의·현재 원본 진단은 같은 ignored 폴더에 보존했다.

동결한 직전 라이브러리로 새 decoded-resource 검사 자체를 실행하면 동일한
`explicit Mask must be a 1-bit image mask`로 FAIL한다. 최종 Native 정식 경로에서는
자원132건 + 기존 #7974 마스크8건 =140 PASS다. 새6경계(1/2/4/8비트·역 Decode·
Mask의 Matte 무시)의 RGBA는 독립 PDFium과 전 픽셀이 같다. 잘못된 sample/색 공간/
flag/깊이/중첩과 codec cap255의 실패·다음 요청 복구도 검사했다. 기존 검사·fixture·
CI 호출은 완화하거나 추가하지 않았다.

원본 Native는 단일 실행 약0.266초·1쪽으로 열리며, 같은 입력의 서명은512×256·alpha255를
유지한다. 독립 PDFium의 원본 JPEG 샘플과 채널 차이는 최대1, 평균0.00112다. 이 비교는
원본의 Mask 샘플이 전부255임을 별도로 확인한 경우이며 일반 마스크를 제거하는 계약이
아니다. 전쪽 Native raw99.45899%, 여섯 자원 대조는100%다. 기존 마스크5·반복배경18·
font4·소유권34쪽의 Native HWPX는 직전 수용본과 각각 바이트가 같다.

직접 판독에서 제목의 face 선택과 가는 표선 coverage 차이는 남아 있다. 이는 새
마스크를 버려도 된다는 근거가 아니며 전체 시각 완료·새 예외 승인으로 보고하지 않는다.
기존 XObject 입력에만 승인된 범위 예외를 이번 새 원본에 자동 적용하지 않는다.
현재 Native·비교 원본은 `E:\tmp\issue7998-extraction002-20261021\`에 보존했다.

Docker 최초 실행은 새 project network의 address pool 소진으로 빌드 전에 실패했다.
네트워크를 정리하거나 설정을 바꾸지 않고 기존 `rhwp` project/network와 공용
`target/pr-review`를 사용해 재실행했다. 새 최적화 WASM과 실제 Studio 증거는 완료 후
추가한다. 위 Native 결과를 fresh WASM 검증으로 대신하지 않는다.

최종 Docker WASM은9분34초에 완료했고 SHA256은
`b1a74176c4a42acf5ee527343e6729c0e82e8bb27105944e9de2f9a60bdcbd51`다.
pkg/public과 실제 Studio가 초기화한 WASM 응답의 지문이 같다. Native 실행 파일 지문은
`bc3e0a9753e8a3c7856bc9a9a38efc9838c698e250e7d160b6bfd3b0b88cc5ff`이며 제품
patch/new-file 지문을 같은 폴더 identity.json에 연결했다. 빌드 뒤 제품 코드는 바꾸지 않았다.

실제 Studio7793에서 새 원본은 약1.726초·1쪽으로 열렸다. 실제 Canvas2D 표시, 브라우저
오류0, HWP/HWPX 양 저장·재열기의1쪽/표시 문자917개 보존을 확인했다. 서명 RGBA는
Native·WASM 저장 및 두 재열기에서 정확히 같다. 기존 마스크5쪽·반복배경18쪽도 열렸다.
이 재열기는 rhwp 경로이며 한컴의 이번 저장본 열기·Print 호환성은 미검증이다.

원본1쪽 + 독립 grayscale 자원 대조6쪽의 전쪽 Native/fresh WASM TSV를 각각 확보했다.
원본 raw99.45899%/99.46414%, 자원 대조는 모두100%, 누락/쪽수 차이는0이다. 실제
서명의 크기·가림·위치를 같은 영역으로 확대 대조했다. 제목 face와 가는 선 농도 차이는
남고, 이 원본의 Table IR은0개다(실제 양 저장/재소비 XML도 tbl0개). 구조 보존 검사에서
표 수0을 유지했다는 결과를 표 인식 성공으로 보고하지 않는다. 현재 완료는 마스크 오류를
해소한 **PDF 열기·그림 보존 확인**이며 전체 시각·의미 표 복원·P5 완료가 아니다.

비교 화면 `E:\tmp\issue7998-extraction002-20261021\review\index.html`에 원본·양 저장본·
Native/fresh WASM 전쪽 TSV·overlay·서명/제목/표선 확대를 연결했다. 메인테이너 새 탭
`http://localhost:7793/?review=issue7998-extraction002&lang=ko`에 실제 원본1쪽을 열었다.
기존 탭·root7700·커밋·원격·PR은 변경하지 않았고 남은 출력 차이에 대한 새 예외나
시각 통과를 임의로 기록하지 않는다.

### 2026-10-21 추가 coll_entries.pdf — collection 항목 예산

원본69쪽·988,629 bytes·SHA256
`917142de2704ae037b0132f5a344ce5331b1fd96cc19dd574666aaacfbf0268c`의 직전 Native도
`collection entries`로 실패한다. 기존 진단 경로에서51쪽 text 추출 시138,000+1개를
관측했다. 독립 pypdf가 센 본문 operand 배열 항목만142,722개이며, lexical operator
249,011개·text-show16,020회다. 다른 xref/객체 배열과 paint 순서/clip 기록도 같은
collection 예산을 소비하므로 쪽수×2,000의138,000 한도는 이 구조를 수용하지 못한다.

원 PR #7970·merge `515bd72dc`와 stage5의 원래 정책/검증을 대조했다. 제품의 전역
collection 상한은400,000이며 실제 parser 배열/사전·xref·text paint 순서·clip의
예약 지점에 같은 request counter가 연결되어 있다. 중복 예약 결함으로 단정하거나
계상을 삭제하지 않는다. 전역 collection 한도와 명시적 host 상한을 유지하고, 선택
쪽수만으로 이 문서 공통 구조 예산을 다시 축소하는 휴리스틱을 제거한다. 선택 연산·
span·decoded·traversal 한도와 raw decoder entries100,000은 그대로다.

소비 경로는 `PdfImportPolicy.backend.entries → ImportBudget 생성 → 선택 후
restrict_structure_limits → parser/xref/paint의 charge_entries`다. 이미 소비한 값을
초기화하지 않으며 낮은 host cap·sticky 실패도 검증한다. 새 검사는 큰 TJ 배열의
실제 예약/내용 보존과 예산 경계를 다루는 decoded-resource 계약으로, 페이지 골든을
추가하지 않는다. 실제69쪽 열기·전체 Native/fresh WASM TSV·양 저장 재열기·정상
대조를 확인하며 전체 P5·시각 완료는 이 단계에서 선언하지 않는다.

collection 보정의 최종 Native는 단일 실행2.637초·69쪽·실제180,625항목이다. parsed
objects4,869·operators249,011·본문 parse69회/cache hit207회로 계상을 삭제하거나
중복 parse를 숨기지 않았다. codec 요청85,994,893·누적 decoded162,212,111 bytes는
실제 resident/live heap의 측정값이 아니다. 자원134건은 최종 정식 suite에서 PASS했다.
새 구조 cap 검사와 원래 마스크/방문·font·취소·sticky 검사를 함께 실행했다. 최초 검증기의
Document 전체 Serialize 사용은 E0277 컴파일 실패였으며 문단 단위로 바로잡았다. 이를
제품 결함의 수정 전 FAIL 증거로 세지 않는다. 동결 직전 라이브러리의 큰 TJ 배열 입력은
실제 collection100,000+1에서 실패하며 최종 같은 입력은 정상 처리·낮은 host cap에서
정확히 거절된다.

기존 extraction1·마스크5·반복배경18쪽의 Native HWPX는 직전 수용본과 각각 같다. 같은
원본의 선택1·51·69쪽 HWPX도 보정 전후 바이트가 같다. 51쪽의 보정 전 선택 Native
PNG와 현재 전체 출력의 해당 쪽 PNG도 픽셀까지 같다. Native69쪽 TSV의 미달은45쪽·
최저17쪽49.46762%다. 문서의 한글은 저장 IR에 남지만 일부 face의 화면 표시와 간격이
다르며, 이 증상을 source 손상이나 파일 부재로 단정하지 않는다.

작업지시자는 **“열기 개선으로 한정·출력은 백로그”**로 답변했다. collection 예산 보정과
새69쪽 열기를 로컬 범위로 수용하고 위 출력 차이는 [승인된 후속 백로그](../plans/task_m100_7998_backlog.md)에
분리한다. Native 수치·미달/직접 판독·자동 `re_review_required`는 그대로 보존하며
전체 시각·P5/PR 통과로 바꾸지 않는다. 이 사례의 새 GitHub 이슈는 등록하지 않았다.
새 WASM 빌드와 실제 Studio/저장 검증은 계속 진행하며 최종 증거를 아래에 추가한다.


collection의 fresh Docker WASM 빌드를 완료했다(compile2m34s·wasm-pack 전체8m45s,
14jobs/codegen-units16). 실제 Studio7793의 instantiateStreaming으로 로드한 SHA256은
`777ef4943dee9997408645ea25ab6f79beef04e767634f4bd01a58cabb662876`이며 pkg/public/검증 pkg를 같은 바이트로 고정했다.
실제 event bus 열기 단일 계측은4.626초·69쪽이며 HWP/HWPX 저장 뒤 재열기는 각각
2.338/2.159초·69쪽이다. 전쪽 문자 digest와 table cell 배열은 원본 열기와 양 재열기에서
동일하고 browser pageerror0이다. Canvas2D1·69쪽 표시와 기존 마스크5·반복배경18쪽
열기도 확인했다. 이 저장 확인은 rhwp 내부 재소비이며 한컴 열기·Print 호환의 증거가
아니다. 결과는 ignored `collection-entries-20261021/studio-results.json`, 직접 저장본은
`E:\tmp\issue7998-collection-20261021\studio-saved.hwp`·`studio-saved.hwpx`에 보존했다.


collection 최종 전쪽 결과: Native/fresh WASM
각69쪽에서 누락·독립 PDF 쪽수 불일치0이다. Native 최저17쪽49.46762%·미달45쪽,
fresh WASM 최저1쪽94.55309%·미달0이다. Native 자동 gate는 `re_review_required`,
WASM 보조값 gate는 `not_evaluated`이며 전쪽/직접 시각 승인을 대신하지 않는다.
동일96DPI의1·17·51쪽 비교와17쪽 개별 PNG를 직접 판독했다. WASM17·51쪽에는 Native에서
빠진 본문 한글이 표시된다. 양쪽1쪽 제목의 face/장평·간격,17쪽 굵은 영문 표제와 표선
농도 차이는 남으며 승인된 출력 백로그에 포함한다. 이를 이번 예산 보정이 해결했다고
보고하지 않는다.

최종 비교는 `E:\tmp\issue7998-collection-20261021\review\index.html`이며 각69행
Native/WASM TSV 원문·대표 compare/overlay PNG를 연결했다. ignored 증거는
`output/pr-review/issue7998/collection-entries-20261021/`의 `focused-final.log`,
`after.json`, `studio-results.json`, `visual-summary.json`, `final-verification.json`,
각 backend의 `source/silhouette_manifest.json`을 따른다. artifact `identity.json`의
code HEAD·제품 patch·untracked 제품2파일·test source·Native·WASM·입력 SHA를 대조했다.
최종 제품 소스 변경 없이 pkg/public/검증 pkg가 일치하고 fmt·diff check도 PASS다.
WASM 빌드나 기존 집중 검사를 문서 갱신 때문에 다시 실행하지 않았다.

메인테이너 테스트 탭 `http://localhost:7793/?review=issue7998-collection&lang=ko`에
새 WASM과 원본69쪽을 실제로 열어 두었다. 기존 탭과 root7700은 변경하지 않았다.
작업지시자의 열기 전용 수용에 따라 이 사례의 로컬 보정/검증을 마무리하며, 출력
결함은 백로그·전체 P5/PR·새 저장본의 한컴 호환은 미완료로 유지한다. 새 commit·원격
push·PR/이슈/comment 변경은 수행하지 않았다.


## 추가 입력 text_not_supported.pdf — 회전 글자 (2026-10-21)

사용자 입력7쪽·15,285 bytes·SHA256
`edb9988a597d8fdc4323131bfd6b2cfeffc6f768ffc9e5e98b93b99ef8d250f8`를
직전 collection Native로 열면 PDF_TEXT_UNSUPPORTED로 실패한다. pypdf의 원시
CTM/Tm 독립 대조에서4쪽 차트 연도30개의45도 회전이 확인됐고 나머지6쪽에는 회전이
없다. 입력은 ReportLab 생성 PDF이며 HWP 원본이나 한컴 Print로 출처를 바꾸지 않는다.
원본 PDF 자체의 PDFium raster를 가져오기 결과의 독립 기준으로 사용한다.

최초 제품 PR #7913/실제 반영4662c069e의 extract/lower는 회전·vertical·mirror를
일괄 거절했다. 현재도같은 거절이고 회전 지원의 회귀라고 주장하지 않는다. 독립 규칙은
ISO32000-1 §9.4.4의 text rendering matrix와 glyph displacement다:
https://opensource.adobe.com/dc-acrobat-sdk-docs/standards/pdfstandards/pdf/PDF32000_2008.pdf

이번 로컬 범위는 수평 쓰기의 순수 회전+균일 scale을 편집 가능한 글상자로 보존하는
것이다. 반전/실제 vertical writing·shear·비균일 scale, 표준 정수각으로 표현할 때
반HU보다 큰 endpoint 오차가 생기는 회전은 명시적으로 거절한다. 문서/숫자 상수로
특정 차트만 고르는 분기는 사용하지 않는다. 회전 글자는 수평 표/본문 문단 추론에
잘못 투입하지 않고 positional TextBox에 남긴다. 그림 대체·문자 삭제는 하지 않는다.

소비 경로는 backend의 실제 text-show CTM×Tm witness/paint 순서 → import의
geometry 검증 → local Paragraph/LineSeg와 shape rotation/AABB → shape_layout의
current local extent/rotation → Native/WASM 공통 변환이다. 원래 baseline origin을
고정하도록 center를 역산하고 clip은 같은 inverse frame으로 투영한다. HWP/HWPX
회전 속성·raw matrix도 함께 기록하며 실제 재열기로 대조한다. 신규 회귀 등록에 앞서
회전/무회전/clip 반례의 전쪽 Native/fresh WASM 출력과 독립 PDF를 먼저 확인한다.
직전 불변 제품·입력/출력은 지문으로 재사용하며 전체 P5와원격 작업은 포함하지 않는다.


첫 Native 실행에서 clip이 없는 회전 글자의 빈 경로를 `valid([])`로 검사하는 결함이
검출됐다. None을 유지하도록 바로잡았다. 이 후보의 실제 실패 로그는
`text-transform-20261021/native-verify.log`에 보존한다. 그 뒤 독립 진단 입력은 원본에
동일한 전체 쪽 clip만 추가했다(`full-page-clip-diagnostic.pdf`). 이 입력의 통과를
원본 통과로 대체하지 않는다. 출력에서30개 label이 수평으로 남아 공통 paint 계층이
부모 rectangle의 회전을 자기 외곽에만 적용하고 TextBox 자식에는 적용하지 않는 것을
확인했다. SVG와 달랐으며 helper 호출만으로 실제 공유라고 판정하지 않았다.

공통 Layer group의 optional `paint_transform`([a,b,c,d,e,f])에 부모/자식 전체 변환을
발행하도록 보정한다. 원 primitive transform은 지워 이중 적용을 막고 subtree visual
bounds도 같은 행렬로 투영한다. Native Skia·WASM Canvas2D·CanvasKit이 동일 group
행렬을 소비하며 클립도 그 내부에서 적용된다. 일반 무변환 경로는 그대로다. Layer
schema minor29로 구별한다. SVG의 원래 parent transform은 같은 의미로 유지한다.

별도로 기본 PDFium 참조 raster가 이 원본의 비내장 HYGothic-Medium 한글을 누락함을
확인했다. 기본 출력도 보존하며, FPDF_LIBRARY_CONFIG.m_pUserFontPaths에 실제로 존재하는
H2GTRM.TTF 등 검증 글꼴 경로를 공급하니 한글이 표시됐다. 입력 PDF는 변경하지 않았으며
글꼴 파일의 family name HYGothic-Medium을 fontTools로 확인했다. 올바른 참조는
`E:\tmp\issue7998-text-transform-20261021\reference-pdfium-fonts\manifest.json`에
PDFium/입력/글꼴 SHA·96DPI를 기록한다. 기본 참조 미표시를 제품 오류나 허용 예외로
바꾸지 않으며, 두 TSV를 구분해 제출한다.


작업지시자는 **“열기·회전 보정으로 한정·기존 출력은 백로그”**로 판정했다. 원본7쪽
열기·4쪽 회전 및 실제 저장 재소비를 현재 범위로 유지하고,7쪽의 기존 font/표선 차이는
[출력 백로그](../plans/task_m100_7998_backlog.md)에 남겼다. 기본 PDFium과 올바른 글꼴
공급 기준을 구분하며 raw 미달·자동 gate를 통과로 바꾸지 않는다. 새 WASM·최종 Native
검증은 계속 진행한다. 제품이 같으면 확인된 결과를 재사용하고 전체 P5/PR은 미완료다.


### 회전 보정 최종 로컬 검증

제품 code HEAD `bd707a7af055ebc047e8d360f079f215af5a6a09` + 미커밋 제품 patch
SHA256 `22d6a8dc6cef96e6f2629c10cd0937042a91d5eda5880761ee01679f883ba129`에
새 제품 3파일 지문을 더해 고정했다. 최종 Native SHA256은
`0b8b4e591bf3638055e0022103ad3f139e531eaa00734462c1b737e844158476`, fresh Docker
WASM은 `be051150d440cc042db2b06599bec639322a3f115aa3c095957e0d0adeeffe90`다.
검증 pkg·worktree pkg/public의 WASM이 같으며 실제 Studio가 이 SHA를 로드했다.
frontend CanvasKit source·입력·신규 제품 파일·저장본 지문도 artifact `identity.json`과
`final-verification.json`에 있다. 최종 Native 빌드는 release-test/native-skia,
Docker WASM은 shared target·14 jobs·codegen-units16·wasm-opt 종료 성공을 확인했다.

독립 기준은 원본을 바꾸지 않은 PDFium + 실제 HYGothic-Medium 공급 출력이다.
96DPI 전쪽 raw TSV는 아래와 같다. PNG-pair 모드의 쪽수 메타데이터를 전체 쪽수 확인의
대용으로 쓰지 않았다. 원본 PDF·Native 가져오기·실제 WASM·양 저장 재열기가 각각 7쪽이며
각 TSV도 7행이고 누락이 없다. 원본 A4 raster 높이 1123px와 제품 1122px의 반올림 차이는
review panel에서만 아래쪽 흰 여백으로 맞췄고 크기나 좌표를 재조정하지 않았다.

| 범위 | Native raw | fresh WASM raw | 판정 |
| --- | ---: | ---: | --- |
| 원본 4쪽, 회전 연도 30개 | 97.14196% | 97.06169% | 각도·기준점·클리핑 직접 확대 대조 |
| 원본 전 7쪽 최저 | 80.62888% (7쪽) | 80.69685% (7쪽) | 승인된 기존 출력 백로그, 자동 보류 유지 |
| 독립 회전 대조군 9개 최저 | 97.21048% | 95.70248% | 전쪽 선행 검증 뒤 신규 회귀 등록 |

기본 PDFium 설정의 한글 누락 결과는 `native/source`, `wasm/source`의 진단 TSV에
보존했다. 올바른 글꼴 기준은 각 backend의 `source-fonts/silhouette.tsv`다. 대조군은
0·2·45·90·180·-45도, 사각 clip, Form, baseline이 페이지 밖에 있으나 글자가 안에 있는
경계다. 2도는 backend reading-order의 5도 snap과 구분된다. 음수각 파일명의 도구 slug
충돌은 검증 key를 `rotation-minus-45`로 구분했다. 원본/회전/clip 대조군의 세 backend
열은 같은 크기·좌표로 직접 판독했다. 원본 7쪽의 기존 글꼴·표선·음수 기호 차이는 수용
백로그이며 최종 Native는 수정 전 선택 출력과 실제 픽셀이 같다. 나머지 원본 6쪽과
기존 collection 69쪽·extraction 1쪽의 HWPX는 직전 수용본과 바이트가 같다.

실제 Studio eventBus로 PDF를 열어 7쪽 표시를 확인했다. 단일 측정 첫 열기 2,317ms이며
보편적 속도 개선 수치로 주장하지 않는다. 실제 HWP/HWPX 저장·재열기에서 전쪽 문자
digest·표 셀 구조와 4쪽의 변환 group 30개를 보존했다. 회전 대조군의 실제 TextBox API로
X 삽입·직접 삭제 후 원문을 복원했고 두 형식 재열기에서도 편집 문단과 회전 group이
유지됐다. 화면 4·7쪽 paint 확인과 browser pageerror 0건을 기록했다.
초기 검증 스크립트의 portable Layer/export 직접 호출은 Canvas2D metric 보호 오류가
났으므로 `withPortableMetrics` 동기 거래로 바꿨다. 보호를 끄거나 제품을 바꾸지 않았다.

신규 검사는 `tests/cases/issue_7998_pdf_table_inference.rs`의 회전 보존·지원 밖 변환
거절 2건이며 자동 수집 경로를 사용한다. fixture는 사용자 원본이 아닌 독립 matrix와
ArialMT 폭으로 만든 `pdf/issue7998/text-transform/`의 작은 PDF 12개다. 지문과 생성
스크립트는 `fixture-manifest.json`에 연결했다. 수정 전 frozen CLI는 45도/clip 입력에서
의도한 PDF_TEXT_UNSUPPORTED로 실패했고 outside-origin은 성공을 반환하지만 글자를
누락했다(`before-controls.json`). 최종 검사에서는 그 글자를 정확히 한 개의 편집
TextBox와 변환 ancestor 안의 문자로 보존하며 양 저장을 확인했다. 구버전 소스에서 전체
신규 Rust suite를 재빌드한 결과로 바꾸어 주장하지 않는다.

정식 자원 검사 134건 PASS, 실제 표/문단 suite 30건 PASS 범위를 확인했다. 표/문단은
처음 29건 통과와 새 회전 1건의 잘못된 검사 가정을 분리했다. 테스트가 클리핑된 회색
기준선 그림까지 문자 rasterization으로 오인했으므로 Picture 0개라는 가정을 제거하고,
유일한 편집 문단과 실제 변환 subtree의 원문·순서·중복 없는 보존을 검사했다. 이름별
예외나 제품 변경을 추가하지 않았다. 해당 1건만 다시 실행해 PASS했고 나머지 29건의
변경 없는 성공 결과를 재사용했다. wrapper가 derived suite007로 안내한 0건 실행은
통과로 세지 않았다. 실제 checked-in collector suite008을 따라 실행했다.

명령·로그는 ignored `output/pr-review/issue7998/text-transform-20261021/`의
`native-build-hull-final.log`, `docker-wasm.log`, `native-final-verify.log`,
`wasm-final-verify.log`, `resource-tests.log`, `semantic-tests-final.log`,
`rotation-test-completed.log`, `studio-results.json`, `visual-summary.json`,
`final-verification.json`에 보존했다. Rust fmt·diff check와 변경된 frontend의 기존
최종 TypeScript 검사도 통과했다. 제품이 바뀌지 않아 빌드·전쪽 캡처를 반복하지 않았다.

CanvasKit 요청의 별도 실행은 기존 resource preparation 실패로 Canvas2D로 fallback했다
(`canvaskit-results.json`). 따라서 새 CanvasKit group 분기의 실제 replay는 미검증이다.
Native Skia와 fresh WASM Canvas2D의 실제 출력은 확인했다. 회전 글자의 마우스 hit/caret
전체 경계, 이번 신규 저장본의 한컴 열기·Print 호환, 실제 vertical writing·mirror·shear·
정밀도 밖 회전은 이번 완료 주장에 넣지 않는다. clip 저장의 한컴 기본 표현 한계도 기존
손실 보고에 남으며 rhwp 왕복 보존과 구분한다. 전체 #7998/P5/PR은 미완료다.

메인테이너 탭 `http://localhost:7793/?review=issue7998-text-rotation&lang=ko`에 새 WASM으로
원본을 열고 4쪽으로 이동해 두었다. 비교 화면은
`E:\tmp\issue7998-text-transform-20261021\review\index.html`, 실제 저장본은 같은
상위 폴더의 `studio-saved.hwp`·`studio-saved.hwpx`다. 기존 탭과 root7700은 보존했다.
새 commit·원격 push·GitHub PR/이슈/comment 변경은 하지 않았다.

## 추가 피드백 — PDF 글상자 더블클릭 캐럿 (2026-10-21)

작업지시자가 `coll_entries.pdf` 15쪽 boolean과 `text_not_supported.pdf` 4쪽 회전
글상자를 지목했다. 실제 마우스 이벤트에서 boolean은 같은 owner의 offset4 hit를
얻지만 dblclick이 flat offset0으로 덮어쓴다. 회전 2010 label은 section3/para0/control80
hit 뒤 기존 ppi0 가드가 진입을 막아 페이지 첫 본문 캐럿이 남는다. 최초 진단의
비유효 좌표/단일 clickCount 호출은 재현으로 세지 않았고, 현재 Puppeteer의 count2에서
실제 mousedown detail1/2와 dblclick을 확인했다.

기존 #686 본문·댓글 → 해결 PR #812와 리뷰 → 실제 커밋 79291f37f7f/eab22908695,
release #970 통합 경로를 읽었다. 원래 보호한 것은 반복 바탕쪽을 누른 페이지와
편집 target의 실제 페이지가 다를 때 첫 쪽으로 점프하는 결함이다. paraIndex0을
바탕쪽 출처로 간주하는 가정은 일반 PDF 구역 첫 문단 글상자에 적용할 수 없다.

수정 경계는 pointer 진입에서 클릭한 owner/path/offset과 실제 target page를 보존하는
것이다. 반복 바탕쪽의 다른 페이지 target은 계속 거절한다. 회전 Rectangle paint의
동일 parent matrix를 hit 역투영과 두 cursor query(Flat/Path)의 최종 점·방향에 공유하고
DOM caret도 그 방향을 소비한다. PDF 문자를 다시 나누거나 IR/원문·배치를 바꾸지 않는다.
일반/회전 원문, 키보드 이동/편집·저장 재열기, 기존 바탕쪽 반례를 검증한다. 승인된
7쪽/collection 기존 출력 백로그와 P5 보류는 유지한다.

### 캐럿 보정 구현·로컬 검증

원인 경로는 `onDblClick → enterTextboxEditing(..., offset0)`의 포인터 위치 덮어쓰기와
`ppi===0`을 반복 바탕쪽으로 간주한 진입 차단이다. 수정은
`input-handler-mouse.ts::onDblClick`에서 hit의 section/parent/control을 선택 대상과
대조하고 `getCursorRectByPathNear`의 실제 target page가 클릭한 page와 같은지 확인한 뒤
`CursorState.moveToHit`로 전체 path·offset·cursorRect를 보존한다. target이 다른 쪽이거나
다른 owner면 진입하지 않는다. 종전 가드의 출처인 #686/#812를 읽은 근거는 위와 같다.

회전 좌표는 `ShapeTransform::matrix_about`의 부모 matrix를 `paint/builder.rs`와
`queries/transformed_text.rs`가 공유한다. 전자는 Rectangle의 자식 text/clip을 묶는
paint group에 적용하고, 후자는 실제 composed TextRun의 owner·replay glyph advance와
같은 frame을 사용한다. `hit_test_native`는 포인터를 로컬 glyph box로 역투영하고,
flat TextBox cursor 및 full-path cursor query는 같은 frame으로 최종 caret 원점·방향을
계산한다. `caret-renderer.ts`의 일반 caret·IME overlay가 그 방향을 소비한다. 이후
별도 offset0 진입은 없으며 일반 표 fast path는 그대로다. 새 문단/줄 추론이나
PDF lowering·원문 조정으로 포인터를 맞추지 않았다. 실물 검증은 수평·순수 회전 TextBox
범위이며 mirror/shear·중첩 clip의 hit 우선순위 전체·IME 실기 입력은 완료 주장에 넣지 않는다.

실제 회전 원본의 작은 선택 핸들이 문자와 겹치는 추가 경계도 확인했다. 이동 없는
mousedown/up만으로 `finishPictureResizeDrag`가 선택 bbox를 저장 크기로 다시 환산해
원점을 바꿨으며, 동일 좌표의 첫 hit offset2가 두 번째에는 offset1로 달라졌다.
`input-handler-picture.ts`의 update/finish에서 실제 포인터 이동이 한 번도 없으면
크기·위치·undo 기록을 변경하지 않게 했다. 기존 resize 식을 바꾸거나 좌표를 clamp하지
않았고 실제 drag는 기존 실행·undo 기록을 유지한다. 최종 실제 이벤트는
mousedown1/up1/mousedown2/up2/dblclick 모두 offset2이며 포인터 진입 후 회전 caret도 유지한다.

| 검증 범위 | 수정 전 → 후 관측 | 증거 |
| --- | --- | --- |
| boolean, 구역14/문단0/control275 | 클릭 offset4 → 종전 offset0 / 보정 offset4 | `before-result.json`, `boolean-before-assert.log`, `studio-results.json` |
| 회전 2010, 구역3/문단0/control80 | 종전 본문 para0의 offset0 / 보정 해당 TextBox offset2·-45도 | `rotated-before-result.json`, `rotated-before-assert.log`, `studio-results.json` |
| 움직임 없는 핸들 클릭 | 종전 속성 setter·변경 이벤트 발생 / 보정 0회, 실제 drag는 동작 | `handle-before.log`, `handle-before-results.json`, Node 집중 검사 |
| 실제 두 원본·양 저장 재열기 | 실제 dblclick·X 삽입·Backspace 복원 6경로 PASS, page/owner/offset 보존 | `studio-results.json` |
| 반복 바탕쪽 원본 #686 | 실제 16쪽·선택 상태의 더블클릭에도 1쪽 점프 없음 | `master-selected-results.json` |
| 다른 쪽 target/다른 owner | 실제 진입 handler에서 편집 차단 | Node 집중 검사 2경계 |
| 공통 frame API | 이전 frozen WASM은 회전 방향/좌표 assertion FAIL / fresh WASM의 flat/path 일치·가시 glyph hit PASS | `frame-before.log`, `frame-results.json` |

위 파일명은 ignored `output/pr-review/issue7998/textbox-caret-20261021/` 기준이다.
회전 선택의 사전 상태는 공개 개체 선택 API로 만들고, 진입·편집·삭제는 실제 마우스·키보드를
사용했다. 바탕쪽도 선택 상태를 명시한 별도 실행으로 변경한 branch를 확인했다.
원래 native query의 outside-origin 진단은 페이지 밖 caret까지 반환하지만, hit의 입력
검사는 실제 페이지 안의 가시 점에 한정한다. 없는 hit 실행을 통과로 세지 않는다.

최종 제품 Rust patch 지문은 `87cef304dc127e45f8d3d99fe6cf72eb681d97be78a9660b3adf13006bf62432`,
source HEAD는 `bd707a7af055ebc047e8d360f079f215af5a6a09`이며 새 제품 파일 지문도
`E:\tmp\issue7998-textbox-caret-20261021\identity.json`에 별도로 고정했다. frozen Native
SHA-256은 `5a1009aa67ce634a7b0d2e2017341603772a1e85dc38b41d317b1860e2840595`, Docker의
fresh WASM은 `6fb49735c980c532ea1c82901fc46f4a16ff3bb4e8ebd07675681d71b546a10e`이다.
빌드는 공유 target/pr-review·14 jobs·WASM codegen-units16으로 완료했고 실제 Studio와
독립 browser 모두 로드한 WASM SHA를 확인했다. 뒤의 핸들 보정은 TS 입력 처리만
변경했으므로 Rust/WASM을 다시 빌드하거나 변하지 않은 문서 캡처를 반복하지 않았다.
최종 frontend 파일 지문도 같은 identity에 연결했다.

두 원본 76쪽과 독립 회전 대조군 9쪽, 총 85쪽을 Native/fresh WASM 각각 다시 출력했다.
85쪽 모두 각 backend의 직전 수용 출력과 픽셀이 같다. 원본 4쪽 raw는
Native97.14196%/WASM97.06169%, 회전 대조군 전쪽 최저는 Native97.21048%/WASM95.70248%다.
기존 text 7쪽의 Native80.62888%/WASM80.69685% 및 collection Native49.46762%/미달45쪽은
승인된 출력 백로그로 유지한다. collection WASM69쪽 최저94.55309%·미달0이다. 전쪽 TSV는
원본 전체 쪽수(7·69)와 행 수를 대조했고 `native/`, `wasm/` 아래 각각 보존했다.
기존 글꼴·출력 차이를 이번 caret 완료나 전체 PR gate 통과로 바꾸지 않았다.

정식 회귀 `rotated_text_hit_and_cursor_follow_the_painted_frame_through_both_saves`는
이미 전쪽 Native/fresh WASM90% 이상을 확인한 독립 PDF8개를 사용한다. 순수 회전의
방향·원문 owner·glyph advance와 역 hit를 원본·HWP·HWPX에서 확인한다. ±2px 좌표는
양쪽99% 이상인 45도 완료 페이지의 PDF(110,130)pt top-down baseline에만 적용했다.
다른 쪽은 관계 검사이며 페이지 밖 점을 hit 검증으로 세지 않는다. 전체 새 Rust 회귀를
수정 전 코드에서 다시 빌드한 것으로 주장하지 않고 동일 compiled WASM API의 수정 전
실패를 별도 연결한다. 실제 collector suite008의 표/문단31건 PASS, Node2350건 중
2348PASS·2SKIP·실패0, 새 집중6건 PASS, tsc·Rust fmt·diff check PASS다. 마지막 Rust
fmt는 테스트만 공백 정리했으므로 제품·검사 의미가 같은 기존 실행을 재사용했다.
더 큰 전체 Rust/P5·PR 준비 검증을 이번 집중 결과로 대체하지 않는다.

확인 화면과 원본·양 저장본·전쪽 TSV는
`E:\tmp\issue7998-textbox-caret-20261021\index.html`에 모았다. 검증용 Chrome의 새 탭
`http://localhost:7793/?review=issue7998-textbox-caret-boolean&lang=ko`에는15쪽 boolean,
`http://localhost:7793/?review=issue7998-textbox-caret-rotated&lang=ko`에는4쪽 회전 2010을
선택해 두었다. 2026-10-21 작업지시자가 두 탭의 직접 확인 결과를 **“편집 동작 통과”**로
판정했다. 이번 일반·회전 글상자의 더블클릭 진입·캐럿·문자 편집 보정은 수용 완료다.
기존 탭·root7700·원문/WIP는 보존했고 commit·remote push·GitHub 변경은 하지 않았다.
