# #7998 stage35 — 여러 글줄이 공유하는 위치 탭

2026-10-20 다음 작업 진행 승인으로 구현·집중 검증을 마쳤고, 작업지시자가 **“시각·편집
구조 통과”**로 판정했다. 기준은 stage34 수용 커밋 `f2aa3e10b9e1a8132024f2f8399e6d79acbce294`다.
검증한 후보와 같은 바이트의 정식 회귀4건·PDF fixture4개를 등록해 로컬 절편을 마감한다.
전체 R2/P5 완료가 아니다.
증거 루트는 `/mnt/e/tmp/issue7998-stage35-evidence/`다.

## 독립 근거와 수정 범위

Base14 Helvetica10pt·400×260pt PDF의 명시적 text-show 원점으로 입력을 독립 생성했다
(`generate_multistop.py`). 첫 줄 ALPHA(20pt) 뒤 BETA(70pt), 둘째 줄 GAMMA(20pt) 뒤
EPSILON(80pt)을 관측한다. 원본 문자가 없는 간격이며 작성 당시 탭이나 작성 폭은 unknown이다.
현재 기준 코드에서는 둘째 줄 EPSILON도70pt에 놓인다. source80pt를96dpi로 환산한
106.667px 대신93.333px이므로10pt 왼쪽으로 이동한다(`before-cross-stops/gap-results.json`).
줄 수·줄 왼쪽 원점만으로는 검출하지 못하는 결함이다.

같은 줄의70/120pt와 다른 줄의80pt, 긴 첫 단어가70pt를 이미 지나140pt를 선택하는
정상 사례, recognition이 불확정 구역으로 보류해야 하는 분리 runs를 구분했다.
거절 입력을 맞추기 위해 recognition 규칙을 바꾸지 않았다. 실제 source 문자·공백은
보존하며 source 없는 간격만 추가 표준 탭으로 표현한다. 캐시 폭 고정·의미 문자 치환·
강제 개행·개별 글줄 글상자·그림 대체·합법 메트릭 범위 확대는 하지 않았다.

| 독립 입력 | 바이트 | SHA-256 |
|---|---:|---|
| `cross-stops.pdf` | 783 | `554b0edd5266fc6c2345a30206c5e26c3c896a033786013accd587f8cc180809` |
| `same-stops.pdf` | 831 | `bf53dfae787765d9809a784522282b8268b892a3eb4532c41d49ba1887c13eca` |
| `compatible-wrap.pdf` | 798 | `f009131d68202979d15bbb9a9dcf67c61b11d398c64a05d1d594ad6e0b6b03d5` |
| `multi-stop-cross-row.pdf` | 770 | `f055488732be9cffc49c1097c4eeaa8d6404b26cbc0824940fa63417815349a5` |

source PDF의 노출 content stream과 생성기를 보존했다. 원본 source 네 쪽의 Native/fresh
WASM raw는 각각100%다. 같은 입력·DPI96·독립 text-show 좌표에서 ±2px source 검사를
후보에 두고 저장/편집은 그 source와의 원점 관계·소유·순서·포함을 검사한다. 사용자 직접
통과 판정 뒤 같은 검사 소스를 정식 회귀에 등록했다. 한컴 Print 저장본의96%대 점수를
그 저장본의 새 절대 px 핀을 추가하는 근거로 사용하지 않는다.

## 공통 결과와 실제 소비 경로

추적 값은 **각 source 간격이 방문해야 할 문단 정지점의 순서**다.

1. `observed_line_text`는 다음 글자 원점·앞 글자의 실제 측정 끝·UTF-16 위치를 같은 typed
   `PositioningTab` 결과로 발행한다. 원본 문자 없는 간격만 대상이며 실제 공백은 보존한다.
2. `paragraph`는 모든 줄의 TabDef를 정렬·중복 제거하고 `materialize_positioning_tabs`에서
   공통 `find_next_tab_stop`으로 다음 정지점을 확인한다. 다른 줄의 중간 정지점이 있으면
   목적지까지 표준 탭을 추가한다. 긴 글자가 이미 지난 정지점에는 불필요한 탭을 넣지 않는다.
3. 문자열·tab_extended·CharShapeRef·LineSeg를 같은 내용 축으로 옮기고 기존
   `finalize_text_axis`로 HWP 스트림 축(각 탭8유닛)을 발행한다. 중간 탭도 표준 미계산 폭
   마커를 사용하므로 이후 삽입 시 기존 간격 폭을 고정하지 않는다. 추가 출력은 기존
   `MAX_GLYPHS` 한도 내로 제한하며 실패하면 기존 source 보존 트랜잭션을 사용한다.
4. `wrap_width::Trial`의 공통 `reflow_line_segs`가 실제 줄 컷·기준선·물리 원점을 재현한
   결과만 발행한다. 탭이 있는 줄 끝 벡터는 폭 증가에 따라 단조 증가하지 않는다. 기존 이분
   탐색의 성공 결과는 재사용하고, 실패할 때 양 구간을64회 한도로 탐색해 실제 재현 폭
   하나를 증명한다. 단일 `[w,w]` 기록은 witness이며 원래 작성 폭이나 최대 유효 구간을
   복구했다는 뜻이 아니다. 예산 내 실패도 모든 폭에서의 불가능을 입증하지 않는다.
5. 그 같은 FlowItem 문단/줄·`paragraph_flow_end`를 본문과 셀의 측정·paint·저장/재열기가
   소비한다. 공통 측정기·renderer의 정상 저장 LineSeg 수용 규칙을 변경하지 않았다.
   페이지 컷·rowspan·표 이어받기 알고리즘 변경은 비해당이며 전체 쪽수·뒤 문단·원문 소유를
   실제 출력으로 확인한다. 이번 수정은 PDF adapter의 공통 결과를 보정한 것이다.

중간 탭 보정만으로 source 위치는 맞았지만, 기존 폭 탐색이 실제 가능한 두 줄 폭을
놓쳐 삽입 때 한 줄로 합쳐졌다. `width-scan.json`의 실제 편집 경로와 `trial-probe.json`의
진단을 대조했고 공통 reflow 결과를 벗어난 별도 줄 나눔 구현을 추가하지 않았다.
private symbol을 호출한 `trial_probe.rs`는 원인 관측용이며 독립 기대값·정식 회귀가 아니다.
최종 후보는 공개 DocumentCore 편집·실제 paint·양 저장의 결과로 검사한다.

## 수정 전 FAIL / 수정 후 PASS와 편집 구조

동일 후보 `bcd8ffcd42e4b21fe7cfdddad8d80b26d1580e2fee4800262491436c57c05e8a`의 정확한 코드로 두 동결 rlib와 현재를
대조했다(`multistop-before-after.json`).

| 경계 | 수정 전 | 최종 |
|---|---|---|
| 다른 줄 정지점 | `before-contract-final.log`: EPSILON source 원점 FAIL | 원본70/80pt·같은 줄120pt PASS |
| 편집 후 두 줄 | `tabs-only/before-wrap-contract-final.log`: X 삽입 후1줄 FAIL | 공통 wrap witness의2줄·뒤 문단 PASS |
| 정상 긴 글자·불확정 구역 | source 소유·기존 정지점 계약 | 보존 PASS |

새 후보4건은 PDF/HWP/HWPX 각3경로에서 삽입→실제 paint→직접 삭제, 양 저장 재열기의
원문·글줄·유일한 paint·순서·소유 셀 내부 포함·탭8유닛·정지점 관계를 검사한다. hard break를
넣지 않으며 미래 모든 편집의 줄 수를2줄로 고정하지 않는다. 기존 집중은159+stage34의4+
여백12+들여쓰기22+표준/저장 탭10=**207 PASS**, 새 후보4건까지**211 PASS**다.
직접 통과 판정 뒤 `tests/cases/issue_7998_pdf_shared_tab_stops.rs`에 새4건을 바이트 그대로
등록했다. 독립 입력4개도 `pdf/issue7998/shared-tabs/`에 그대로 복사했다. 같은 입력의
기존 기대값·fixture를 바꾸지 않았다. 따라서211건은 정식 등록된 검사들의 실행 결과이며,
등록만을 위해 동일한 실행을 반복하지 않았다.

기존16문서의 실제 표·셀·문단 소유와 원문 순서는 같다(`semantic-owner-comparison.json`).
추가로 탐색된 stage34 fixture는 기존 검증 입력과 같은 PDF다. 실제 Studio9경로에서도
삽입 paint·직접 삭제·Snapshot undo/redo·양 저장을 통과했고 PDF 경로 저장 bytes는
Native와 같다. 새 WASM SHA의 로드와 pageerror0을 확인했다(`studio-result.json`).

## 독립 Print·전쪽 Visual Sweep

Native: `CARGO_BUILD_JOBS=14 cargo build --locked --profile release-test --features native-skia
--lib --bin rhwp --target-dir /home/edward/mygithub/rhwp/target/pr-review`, 최종2분44초·exit0.
fresh WASM: 고정 Docker 서비스/같은 target에서 `docker compose -p rhwp --env-file .env.docker
run --rm --no-deps -e CARGO_BUILD_JOBS=14 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm`,
7분46초·exit0. source2파일·Native·WASM·glue/public 지문은 `source-runtime.json`에 고정했다.
Native SHA=`ac6068694539f2959fb5fc769cdcf3fa8f29829d674d779b1c232154b900eb5a`, WASM SHA=`f171497069b501fed2e975366588e2129858b92de0011440eff84fcbdda56e94`다.

실제 동일 편집 저장본18개를 한컴 Print12작업+바이트가 같은6개 alias로 검증했다.
각 결과의 engine2020·11.0.0.9136·method0·`hancom2020_pdf_driver_one_up`을 확인했다
(`independent-print-contract.json`).18개 Print의 단어 정지점은 독립 MuPDF text extraction으로
source 원점0.5pt 이내임을 확인했다(`independent-stop-origins.json`). 고정된 source 범위와
단어 paint는 별개로 직접 대조한다.

| Backend | 입력/전체 쪽 | 최저 raw | 90% 미만/누락/쪽수 차이 |
|---|---|---|---|
| Native |113/124|91.61577%|0/0/0|
| fresh WASM |113/124|90.85213%|0/0/0|

기존91입력102쪽+새 원본4쪽+새 동일 편집 저장본18쪽이다. 각 backend의 기존102쪽은
**모두 이전 수용본과 PNG가 같다**(`native-previous-comparison.json`, `wasm-previous-comparison.json`).
새 제품 코드의 전쪽 재출력과 이전 캡처 재사용을 혼동하지 않았다. 같은 새 입력/기준 PDF/
runtime hash가 중복되는 현재 실행 내 alias만 재사용했다. TSV 원문은
`native/visual/*/silhouette.tsv`, `wasm/visual/*/silhouette.tsv`와 비교 화면의 `tsv/`에 있다.

변경 영역을 같은 배율로 직접 확대했다. source 위치·두 줄·뒤 문단의 큰 이동/겹침/누락은
관측되지 않았다. 한컴은 Helvetica를 Haansoft Batang으로 대체하므로 outline 차이는 남아
화면에 표시했다. 단어가 누락됐다는 축소 미리보기의 초기 관측은 원본96dpi8배 확대·
300dpi·Poppler/MuPDF에서 모두 표시되는 것으로 해소했다(`print-word-rasterizer-diagnosis.png`).
이 관측을 맞추기 위한 제품 변경은 없었다. 점수와 직접 판독은 별도로 기록한다.

## 직접 판정·남은 범위

비교: `E:\tmp\issue7998-multistop-review\index.html`.
Studio7793의 `?review=issue7998-multistop-cross`, `...-same`, `...-long`, `...-saved`,
`...-restored` 새 탭에 실제 source/편집 저장본·새 Docker WASM을 열었다.7700과 기존 탭은
유지했다. 2026-10-20 작업지시자가 **“시각·편집 구조 통과”**로 판정했다.

정식 등록 시 제품2파일 SHA와 실행 후보 SHA를 `source-runtime.json`에 대조해 변경이 없음을
확인했다. `rust-test-suite-manifest.mjs --prepare`/`--check`로 자동 수집을 확인하고 새 검사
파일의 `rustfmt --check`와 `git diff --check`를 수행했다. 새 모듈은
`regression_suite_016`에 자동 수집됐으며 총1568소스·6981정적 test 속성·48/48타깃이다.
PDF xref의 유효한 끝 공백이 source 공백 검사로 잡혀 기존 PDF와 같은 `-text -diff`
속성을 새 fixture 경로에도 적용했다. PDF 내용·SHA는 바꾸지 않았다. 등록 파일·fixture의 바이트 동일성,
자동 수집 경로·로컬 절편 커밋과 깨끗한 worktree 상태는 `registered-acceptance.json`에
연결한다. 동일 제품의 Native/fresh WASM 빌드·124쪽 캡처·독립 Print·집중211건의 실행은
재사용했으며 이번 등록을 새 실행으로 보고하지 않는다.

실제 source 공백의 큰 advance(보고서5·10쪽), 일반 glyph 장평/자간(6쪽·본문1/4쪽),
R1/B24·나머지 R2·P5는 남는다. 탭 폭 탐색의 미검증/예산 초과를 의미 구조 없음이나
정상 복원으로 보고하지 않는다. `parser_allocation_bounds_proven=false`를 유지한다.
전체 suite·전체 lint·Worker/package·PR·remote push를 이번 절편에서 실행하지 않았다.

시작 시 Linux 여유 약0.7GiB에서 종료된 Chromium 임시 후보부터 확인했다. 실행 중
Cargo/rustc/wasm-pack/wasm-opt와 정확한 경로의 fd·cwd·maps 참조가 없음을 확인한 뒤
72시간 이상 지난 debug 증분 캐시 하위3개만 제거해 약6.25GiB를 확보했다
(`cleanup-candidates.json`, `cleanup-result.json`). executable·deps·release-test·나머지 공유
캐시는 보존했다. 소스·원본·검증 증거 삭제는 없다.
