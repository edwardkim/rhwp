# #7998 stage34 — 문자 없는 관측 간격의 표준 위치 탭

2026-10-20 다음 R2 작업 진행 승인으로 구현·집중 검증을 마쳤고, 작업지시자의
**“시각·편집 구조 통과”** 판정으로 이번 표준 위치 탭 범위를 수용했다. 검증한 제품과
같은 검사 소스를 정식 회귀에 등록하고 이 절편을 로컬 커밋으로 마감한다.
기준 HEAD는 `51430cf4dceb282cb783279883cbc252b5ee0b5e`다. 현재 제품은 원본 문자가 없는
큰 간격에 표준 TabDef와 `TAB_EXT_PLACEHOLDER`를 발행한다. 1차 고정 폭·중간의 넓은
줄 재구성 보정은 폐기한 완료안이며, 그 증거를 현재 결과와 구분해 보존했다.

아래 최종 증거의 루트는 `/mnt/e/tmp/issue7998-stage34-evidence/positioning-tab-final/`이다.
ignored `output/pr-review/issue7998/p4-horizontal-metrics`는 상위 stage34 증거를 가리킨다.

## 독립 근거·범위

공개 원본 `pdf/pr7704/6778-square-2020.pdf`(SHA-256 `d4c1fe7a8a3c025b2be3648ba6e2ad1123eff3dde008983edadcef7e8f7b6656`)의7쪽 셀에서 `룹을` 뒤→`구성하고` 앞에는
문자가 없고26.146pt 간격이 있다.9쪽 `3`→`책자` 앞도 문자 없는24.122pt다.
`reconstruct_line`의 source_character/source_utf16=None 관측으로 실제 원본 공백과 구분한다.
문자 속성의 합법 장평50–200%·자간-50–50%로 나타낼 수 없는 이 간격만 위치 탭으로 연결한다.
저작 당시 탭의 복구가 아니라 위치 추론이며 `authoredTab=unknown`을 기록한다.

실제 원본 공백(5·10쪽), 일반 문자 장평 한계(6쪽), 본문 자간 보류(1·4쪽)는 보존한다.
문서 ID 분기·비율 clamp·강제 개행·글줄별 글상자·그림 대체를 추가하지 않았다.
두 줄 대조군은 Base14 Helvetica와 명시적 PDF text-show 원점으로 독립 생성했다.
1쪽400×260pt에서10pt 글자의 ALPHA/BETA 원점은(20,210)/(70,210)pt,
다음 GAMMA DELTA는(20,196)pt, 뒤 SECOND는(20,160)pt다(PDF 좌하단 좌표계).
PDF의 노출된 content stream과 상위 `generate_gap_standard.py`에 생성 근거를 보존했다.
보고서7쪽 표/셀과 원문은 유지하고,9쪽은 `붙임3\t책자 표지`와 뒤 문단을 실제 문단으로
연결한다. 현재 셀 메트릭 보류는2줄→1줄, 메트릭으로 거절된 본문 제안은5개→4개다.
16문서의 쪽수·표 topology·셀 문단 내용 순서를 보존했다(`semantic-owner-comparison.json`).

## 규칙과 실제 소비 경로

추적 값은 **탭 뒤 첫 글자의 물리 원점**이다.

1. `reconstruct_line`의 원점·전진폭 → `observed_line_text`의 공통 문자 측정과 합법 속성 검사.
   원본 문자 없는 간격이 범위를 벗어날 때 다음 글자 원점을 owner-relative TabDef 정지점으로
   기록한다. 기존 resolver가 쓰는2배 HWPUNIT 단위를 따른다. 폭을 고정한 탭 캐시는 발행하지 않는다.
2. `finalize_text_axis`와 `wrap_width::ends`가 모두 HWP 스트림 축(탭8유닛)을 사용한다.
   후자의 문단 끝을 UTF-16 문자열 길이로 비교하던 불일치를 제거했다.
3. `text_measurement::compute_char_positions_walk`의 표준 무캐시 탭 경로가
   `find_next_tab_stop`을 사용한다. 측정·paint가 같은 위치 계약을 소비하므로 정지점 앞에
   X를 넣어도 뒤 단어는 같은 정지점에 놓인다. 원본 한컴 저장 탭의 수용 규칙은 바꾸지 않았다.
4. 기존 serializer가 미계산 탭의 새 줄 캐시를 생략한 저장본은 재열기에서 글줄을 구성해야 한다.
   `supports_cached_body_frame_controls`에 **실제 저장 줄이 없을 때만** 인증된 셀 테두리
   배경 그림을 흐름 중립으로 수용한다. 인증과 BehindText·비TAC·Column/Para 속성이 함께
   있어야 한다. 실제 캐시와 유효한 관측 줄은 종전 소유 경로를 유지한다.
5. 실제 셀 높이 측정 `height_measurer.rs:3072,4166`과 셀 배치 `table_layout.rs:9966`은
   `recompose_cell_lines_with_nested_tables`→비중첩 경로의
   `recompose_horizontal_cell_lines_for_width`→`recompose_cell_lines_in_frame`을 소비한다.
   이번 셀은 내부 중첩 표가 없으므로 자식 표 높이 투영 분기는 비해당이다. frame 결과의
   줄·기하를 높이와 배치가 함께 소비하며 별도의 측정 보정이나 최종 좌표 clamp를 추가하지 않았다.

7쪽 대상의352개 그림은 PDF·HWP·HWPX 모두 인증된 셀 테두리용 배경 그림이었다
(`cell-owner-diagnostic.log`). 페이지 컷·rowspan·continuation 규칙 변경은 비해당이다.
원본12쪽과 각 편집 저장본1쪽의 전체 쪽수·내용 보존 및 실제 glyph의 셀 내부 포함을 확인했다.

## 검출한 결함과 수정 전후 증거

최종 후보 `gap_contract.rs`의 **같은 소스**를 다음 동결 rlib와 현재 rlib에 링크했다.
source SHA와 명령·판정은 `gap-contract-before-after.json`에 고정했다.

| 경계 | 수정 전 실행 | 최종 실행 |
|---|---|---|
| 고정 간격 캐시 | `sealed-before-tab.log`: X 삽입이 탭 뒤 단어를 이동시켜 FAIL | 표준 정지점 유지 PASS |
| 무캐시 셀 재열기 | `sealed-before-frame.log`: 줄 재구성이 거절돼 셀의 줄/포함 관계 FAIL | 원래4줄·셀 내부 glyph 보존 PASS |
| 유효한 기존 줄 | `sealed-before-gate.log`: 중간의 넓은 수용 조건이 가운데 줄을 왼쪽으로 이동시켜 FAIL | 기존 관측 줄의 독립 PDF 원점 유지 PASS |

처음 고정 폭 보정은 자동90% 기준을 통과했지만 직접 확대에서 BETA 이동을 발견했다.
이어 셀 배경 수용을 넓힌 중간 보정은7쪽에-1.30970%p 하락을 만들었다. 이를 숨기거나
기준을 갱신하지 않았다. 실제 저장 줄이 없는 경우와 유효한 관측/저장 줄을 구분하는
소유 계약으로 잘못된 가정을 제거했다. 중간 코드·Native·부분 WASM·사진은
`before-no-cache-gate/`와 상위1차 증거에 보존했다. 후속 검증은 영향 페이지와 작은 경계
검사를 먼저 통과한 뒤 전체 범위로 진행했다(`native-direction-comparison.json`).

최종4개 후보 검사는 PDF/HWP/HWPX, 실제 삽입→paint→직접 삭제, 양 저장 재열기에서
원문·유일한 paint·순서·소유 셀 내부 포함·탭8유닛·글줄 컷·정지점을 검사한다.
독립 MuPDF의7쪽 네 가운데 셀 글자 원점도96dpi ±2px로 보호한다. 원본 x=71.961pt,
y=373.03233/452.74520/532.57797/697.15810pt는 `p7-independent-mupdf.xml`에서 얻었다.
이 source7쪽의 raw는 Native99.86930%·fresh WASM99.64065%이며 다른 문서의100% 쪽은
인증된 테두리 그림+기존 가운데 줄 경계를 대신하지 못한다. 좌표 검사와 관계 검사를 함께 유지한다.
사용자의 직접 판정 뒤 같은 소스를 `tests/cases/issue_7998_pdf_inferred_gap_tabs.rs`에
바이트 그대로 등록했다(SHA-256 `7cff6866e8b7a7fa3164e2ffc59007339ffaa99c677191188a8ba9b98294abc7`).
두 줄 fixture는 `pdf/issue7998/body-gap-tab.pdf`(730바이트,
SHA-256 `588282d30e688ca10e118083bd83d369d18a54a716c1ef13fc4124e4e1467cc9`)이며
독립 생성·Print에 사용한 PDF와 바이트가 같다. 기존 검사의 기대값을 변경하지 않았다.

기존 집중 검사는 #7998 159건+여백12건+들여쓰기22건+표준/저장 탭10건=**203 PASS**다.
같은 소스로 실행한 새4건 PASS를 정식 등록해 **집중 검사 합계207 PASS**를 연결했다.
등록 이후 제품·검사 실행 코드·입력이 같으므로 이미 통과한 실행을 반복하지 않았다.
이 합계는 전체 suite·lint·Worker/package·P5 완료를 뜻하지 않는다.

## 최종 빌드·독립 Print·전쪽 비교

최종 Native 명령은 `CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test
--features native-skia --lib --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`다.
2분57초·exit0이다. fresh WASM은 기존 고정 Docker 서비스/target의
`docker compose -p rhwp --env-file .env.docker run --rm --no-deps -e CARGO_BUILD_JOBS=14
-e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`으로7분36초·exit0이다.
Native SHA=`1ff5b15f1b2caeb7da69b283987c0d780d9dd1050c43ac25c5ba764599226dde`, WASM SHA=`26a3a8d57fd588b0f60f33b7ee3bd9ff6a1a8f1954754ea75b061247f4b3d2c7`다.
제품3파일·glue SHA와 public 일치는 `source-runtime.json`을 따른다.

독립 한컴2020 Print16작업+바이트가 같은2개 alias로 새 저장본18개를 검증했다.
모든 result의 engine2020·11.0.0.9136·method0·one_up을 확인했다. 마지막 캐시 소유 분기
보정 후18개 저장 bytes가 같음을 확인해 Print를 재호출하지 않았다
(`independent-print-contract.json`, `reused-print-after-no-cache.json`). 실제 Studio와
독립 한컴 단어 원점을 `positioning-stop-evidence.json`에서 대조한다.

| Backend | 입력/전체 쪽 | 최저 raw | 90% 미만/누락/쪽수 차이 |
|---|---|---|---|
| Native |91/102|91.61577%|0/0/0|
| fresh WASM |91/102|90.85213%|0/0/0|

범위는 기존70입력81쪽+새 원본3쪽+새 동일 편집 저장본18쪽이다. 전체 TSV는
`native/visual/*/silhouette.tsv`, `wasm/visual/*/silhouette.tsv`에 원문으로 보존한다.
기존81쪽 중 각 backend의79쪽은 PNG가 같고7·9쪽만 다르다. Native raw 변화는
+0.00108/+0.00444%p, fresh WASM은+0.00210/+0.00630%p로 **하락0**이다.
입력/기준 PDF SHA를 같게 대조했다(`native-previous-comparison.json`, `wasm-previous-comparison.json`).

원본7·9쪽의 독립 PDF·이전 Native·현재 Native·이전 WASM·현재 WASM을 같은 배율로
확대했고, 새 삽입/직접 삭제/저장본도 직접 대조했다. 큰 위치·줄바꿈·누락·겹침은 관측되지
않았으며 기존7쪽 fallback 모양,9쪽 WASM 농도 및 생성본 한컴 대체 글꼴 차이는 화면에 표시했다.
새 두 줄 삽입본은 Native96.16099%/WASM95.88949%로, 단어 위치 이동은 해소됐다.

실제 Studio9경로에서 삽입 paint·직접 삭제·Snapshot undo/redo·양 저장을 통과했다.
정지점은 삽입 전후7쪽362.5px/9쪽172.5px/두 줄93.3px로 각각 같다. PDF 경로의 실제
저장 bytes는 Native와 같다. fresh WASM SHA 로드·pageerror0을 확인했다(`studio-result.json`).

## 직접 판정과 남은 범위

비교: `E:\tmp\issue7998-horizontal-metrics-review\current\index.html`.
Studio7793에 `?review=issue7998-gap-final-cell`, `...-title`, `...-two-rows`,
`...-title-saved`, `...-restored` 다섯 새 탭을 열고 실제 파일·WASM 로드를 확인했다.
기존 탭과7700은 유지했다. 전체 TSV·대표 compare/standalone overlay/review·확대·원본·
양 저장본·동일 편집 독립 Print를 연결했고, 2026-10-20 작업지시자가
**“시각·편집 구조 통과”**로 판정했다.

실제 공백의 큰 전진폭·범위 밖 문자 장평/자간과 R1/B24·나머지 R2·P5는 남는다.
한 문단의 여러 글줄이 서로 다른 탭 정지점을 혼용할 때 source와 공통 next-stop이
일치하는지의 일반화 검증도 미검증으로 남긴다. 현재의 단일 위치 탭3입력 통과를 전체
PDF 문단/표 복원 완료로 확대하지 않는다. `parser_allocation_bounds_proven=false`를 유지한다.

처음 만든 비Base14 `/Arial` 대조군은 Widths/FontDescriptor 없이 추출기/독립 renderer의
메트릭이 달랐다. 그86.64238% 입력/출력은 상위 `source-inputs/gap-wrap.pdf`, `wrap-product*`,
`wrap-print/`에 보존하고 완료 범위에서 제외했다. 새 Base14 대조군 생성기와 수정 전후
결과를 구분했으며 입력 가정을 맞추려고 제품 규칙을 완화하지 않았다.

검증 후 제품3파일의 SHA는 `source-runtime.json`과 같다. 정식 등록 때 빌드·캡처·Print와
집중 검사를 반복하지 않았다. 자동 수집의 `--prepare`와 `--check`, 새 검사 파일의
`rustfmt --check` 및 `git diff --check`로 등록을 확인했다. 자동 수집은1567sources·
6977static attrs·28suites+20exceptions=48integration targets이며 CI 개별 호출은 추가하지 않았다.
새 모듈은 `regression_suite_007`에서 수집한다. manifest와 harness는 기존 정책의
ignored 생성물이며 별도의 수동 등록이나 Cargo target을 추가하지 않았다.
등록된 모듈·입력 지문·수용 판정과 커밋은 `registered-acceptance.json`에 연결한다.
전체 회귀·전체 lint·PR·원격 반영은 이번 절편에서 실행하지 않았다.
