# #7998 P4 아홉 번째 절편 — 필수 경계 paint의 실제 소유

- 기준 checkpoint: `99116a2fb943dcbb6d6bfa35a7fa5c1c22993eb4`.
- 2026-10-19 지시: 다음 절차 진행. stage9의 시각 통과와 51건 회귀를 보존한다.
- 이번 범위: B20의 새 지역 후보에서 필수 경계를 실제 셀 paint로 보존하고, 상단 정상 root와
  하단 후보의 문자·paint를 유일하게 소유한다. 일반 미확정 격자 전체나 P5 제출 게이트는 별도다.

## 구현 전 경로·독립 기대

공개 `pdf/issue7998/local-regions/independent-roots.pdf`는 정상 상단 18셀과 하단 19개
생성 label/주소의 대조군이다. 보고서 5쪽도 같은 하단 paint 소비에서 보류된다.
`sourceBoundarySegments/sourcePathIndices → positional ruleSourceControls →
table_paint::partition의 물리 frame/교집합·보존 검사 → 실제 Cell drawing/native border →
편집·양 저장·재열기`를 추적한다. source 경계를 무시하거나 목록에서 빼서 성공시키지 않는다.
소비되지 않은 필수 경계가 실제로 paint되지 않는지, 좌표·선/면적 판정이 잘못됐는지 먼저 구분한다.

같은 원본의 전쪽과 공개 반례를 비교한다. helper 계산을 기대값으로 복사하지 않고 PDF의
실제 선·생성한 label 주소·내용 보존과 실제 출력으로 판단한다. 적용 경계의 작은 Native
검증을 먼저 수행하고 방향을 확인한 뒤 Docker fresh WASM과 기존 대조군으로 넓힌다.
원시 증거는 ignored `output/pr-review/issue7998/p4-paint-ownership/`에 보존한다.


## 원인 판정

공개 대조군의 하단 요구 경계 2,500개를 실제 page control frame과 대조했다.
`path 2688` 하나만 셀 물리 범위와 교차하지 않는다. 표의 오른쪽은 533.95pt,
선의 실제 paint는 x=534.13pt부터 폭 0.36pt다. `boundary_contributor`의 0.5pt 추론
허용 범위 안에 있지만, 그 허용 범위를 실제 paint 이동이나 셀 확장 근거로 쓸 수는 없다.
나머지 2,499개는 실제 셀 소유의 물리 조각과 겹친다. 이 수치들은 해당 source의 진단이며
제품 분기 상수가 아니다(`control-frames.json`, `unintersected.json`).

`partition`은 원래 표 밖 잔여를 페이지에 보존한다. 그러나 마지막 검사는 required 전부가
`removed`에 있어야 한다고 요구하여, 정상적으로 페이지에 남긴 외곽 paint도 누락으로
판정했다. 셀에 소유된 조각과 교차가 전혀 없는 page paint를 구분한다. 원래 control이
unremoved source 목록에 남고, 실제 물리 교집합이 없을 때만 외부 소유로 인정한다.
셀 범위와 교차하는 미소비 경계는 계속 실패한다. 새 검사는 같은 work budget에 계상한다.
외부 source path ID를 상세 진단에 남기며 paint 좌표·크기·stroke·페이지 z-order를 바꾸지 않는다.
일반 외부 paint의 행열 상속/이동까지 검증한 주장이 아니며 source 보존과 셀 내부 소유를 구분한다.


## 첫 구조 실행

초기 producer에서 공개 두-root 입력은 실제 Table 1→2개(상단 18셀·하단 19셀),
독립 한컴 Print의 표에 오른쪽/아래 빨강 외부 띠를 더한 반례는 0→1개로 복원됐다.
경계를 가로지르는 빨강 띠와 표와 무관한 먼 장식 띠는 기존 1개 표를 유지한다.
외부 경계 ID만 새 진단에 남고, 교차 띠는 내부/잔여의 기존 물리 분할을 소비한다.
`initial-comparison.json`은 최초 실행이며 최종 Native/fresh WASM 시각 판정으로 쓰지 않는다.
물리 측도 0인 control은 외부 paint의 근거로 수용하지 않는 양수 측도 가드도 추가했다.
최종 제품 manifest SHA-256은 `a1a7303970220535a880a413bd7131665d84ca44eaa01001fcba31d8cbb61b94`다.


## 최종 제품·전쪽 검증

검증 제품은 위 2,448개 runtime 파일 manifest와 동일하다. 테스트·fixture·보고서 추가 뒤에도
제품 파일 SHA를 다시 대조했으며, 동일 제품의 Native와 fresh Docker WASM을 재사용했다.
Native 최종 빌드는 기본 feature(pdf-import/wasm-api 포함)+native-skia, release-test다.
첫 no-default-features 진단 명령의 wasm-api 누락은 빌드 설정 실패이며 결함 재현으로 세지 않는다.
초기 Native producer 뒤 양수 측도 가드가 추가되어 최종 producer를 한 번 재빌드했다.
검증과 직접 판정 이후 제품 코드를 더 바꾸지 않았다.

- Native: `native-build-guard.log`, 고정 `target/pr-review`, `CARGO_BUILD_JOBS=14`.
- fresh Docker: `docker-wasm.log`, `docker compose -p rhwp --env-file .env.docker`,
  jobs 10/codegen-units 16. `pkg`와 Studio `public`의 JS/WASM 신원이 같다.
- WASM 19,343,260 bytes, SHA-256
  `1f89908f7836ffd5b11efdb80155fdff436e71b213bb92d44b451aeb4d78c3f5`.
  실제 Studio의 `instantiateStreaming`에서 이 hash를 확인했다(`studio-result.json`).
- Native/fresh WASM 각각 **99입력 × 직접 PDF/HWP/HWPX = 297건, 330쪽**.
  누락·쪽수 차이·90% 미달·오류 0건. 전체 최저는 Native **97.47552%**,
  fresh WASM **97.65094%**다. 형식별 재열기 PNG는 각 backend의 직접 PDF와 같다.
- 새 물리 경계 4개와 공개 두-root 입력: 각 backend 15쪽 모두 **100%**.
- stage9와 공통인 318쪽 중 보고서 5쪽·공개 두-root 1쪽의 각 세 형식만 PNG가 바뀌었다.
  공개 두-root는 100%를 유지한다. 보고서 5쪽은 Native 99.90056→99.76761%
  (**−0.13295%p**), WASM 99.76806→99.60215%(**−0.16591%p**)다.
  하단의 positional 문자열을 실제 19셀의 공통 글자 메트릭으로 배치한 미세 차이를
  전후·독립 PDF와 같은 배율로 공개했다. 다른 공통 입력·쪽은 각 backend에서 byte 동일하다.
  이 국소 직접 수용은 P5의 최신 upstream/devel 무회귀 gate를 대신하지 않는다.
- `validate.py`/`aggregate.py`, `native/summary.json`/`wasm/summary.json`, 각 `results.json`의
  정확한 명령, `visual/<문서-형식>/silhouette.tsv`·manifest·PNG를 보존한다.
  원래 shard를 보존하고 aggregate는 symlink만 연결해 대용량 scratch를 중복 복사하지 않았다.
  이전 사진을 새 head 출력으로 표시하지 않는다.

실행 명령은 예를 들면 `python3 <증거>/validate.py native` 및 `... wasm`이며 실제 전쪽은
각 backend를 독립 shard로 나누어 실행했다(`native-shards.json`, `wasm-launch.log`).
모든 Visual Sweep은 `scripts/visual_sweep.py --silhouette-only --rasterizer png --compat 2022
--dpi 96 --embed-fonts full`과 각 입력의 기존 실제 font 경로를 사용한다.
새 입력·편집본은 공개 Arial 공급 경로이며, 기존 보고서의 Batang 조건을 바꾸지 않았다.

## 실제 편집·저장·한컴 확인

`api.rs`와 `verify_studio.mjs`가 원본 PDF의 실제 IR을 확인한다. 상단 18셀·하단 19셀의
37개 label/주소/span은 기존 공개 생성 근거를 따른다. 새 4셀의 원래 정상 생성 계약도
유지한다. 셀 문자가 유일하게 소속되고 셀 안에 포함되는지, 문자 X 삽입·삭제와 양 저장
재열기를 확인했다. 원래 control index를 저장 뒤 재사용하지 않고 현재 Table의 주소를 읽는다.

빨강 원본 `re f`의 총 물리 면적·조각 비중복을 실제 RenderTree에서 확인했다.
전부 표 밖인 source는 page owner, 교차 source는 cell 내부와 page 잔여를 함께 가진다.
테스트의 면적은 제품 partition helper가 아니라 PDF operator w×h를 기대값으로 사용한다.
허용치는 HWP 1/100pt 끝점 저장의 양자화 근거다. 내용 존재만으로 위치·paint 보존을 대신하지 않는다.
실제 HWP 출력은 `export_hwp_with_adapter`/Studio `exportHwp`다. 저수준 HWP writer를
HWPX 재열기 객체에 직접 호출하지 않았다.

메인테이너가 `independent-roots-edited`, `outside-right-edited`, `outside-bottom-edited`의
HWP/HWPX **6개 모두 정상 열림**을 확인했다. 파일 SHA는 전달 당시 `edited/inputs.json`과
현재 `native-api`가 동일하다. 메인테이너는 HWPX에서 PDF 3개를 직접 Print했다.
Creator는 `Hwp 2020 11.0.0.9166`, Producer는 `Hancom PDF 1.3.0.550`이다.
`reference/provenance.json`에 원본 대응·bytes·hash·pdfinfo를 보존한다.

- 같은 HWPX Print와 Native 편집 저장 6쪽: 최저 **99.66299%**.
- 실제 Studio fresh WASM 편집 저장 6쪽: 최저 **98.35379%**.
- HWP는 정상 열기 확인·같은 독립 Print 기준 재열기 대조다. HWP 자체에서 별도 Print한
  증거로 바꾸어 적지 않는다. 전쪽 로그/TSV는 `native-extra`, `wasm-extra`다.

`E:\tmp\issue7998-paint-ownership-review\index.html`의 원본/두 backend/독립 편집 Print,
경계 확대, 보고서 5쪽의 전후, 실제 셀 구조를 공개했다. 모든 링크는 상대 경로이며
이동 후 깨지지 않도록 파일 존재 검사를 통과했다. Studio 7793에는
`?review=issue7998-p4-paint-independent-roots`, `...-outside-right`, `...-outside-bottom` 탭을
열었다(`opened-review-tabs.json`). 2026-10-19 메인테이너의 이번 답변은
**“시각·편집 구조 통과”**다. 이 판정과 자동 점수는 구분한다.

## 정식 회귀와 전후 실패

새 `tests/cases/issue_7998_pdf_external_boundary_paint.rs`는 기존 자동 suite에 수집됐다.
fixture는 Native/fresh WASM 전쪽 100%와 직접 확대 확인 후 등록했다.
원본·실제 셀 소속/span·문자 순서/중복·셀 포함·paint 소유와 면적 보존, 양 저장·복원을
관계 검사로 보호한다. 절대 px/SVG hash pin·기존 기대값 완화는 없다.

`before_after.py`는 보존한 기준 CLI가 같은 PDF에서 생성한 HWPX를 현재 reader로 열어,
정식 회귀의 동일 관계 assertion을 대조했다. 기준 실행의 PDF import 자체는 성공한다.
오른쪽·아래 외부 띠는 Table 0개, 공개 두-root는 1개여서 **원래 결함인 실제 셀 누락으로
각각 FAIL**이고 최종 PDF import는 각각 1/1/2개로 PASS다. 교차·먼 띠 정상 대조군은
전후 PASS다. 이는 옛 CLI 산출물에 대한 같은 assertion이며 옛 라이브러리를 재빌드하여
전체 test case를 실행했다고 표시하지 않는다(`before-after-proof.json`, 개별 assertion 로그).

신규 정식 3건은 모두 PASS다(`new-case.log`). 기존 집중 11개 case의 51건도 모두 PASS이므로 **신규 포함 54건 PASS**다.
실제 명령·결과는 `focused-final-results.json`과 `focused-summary.json`으로 연결한다. 필수 원본·정상 검사를 삭제하거나 baseline을
늘리지 않았다. runtime 빌드·시각 캡처를 테스트 추가만으로 반복 생성하지 않는다.

## 범위와 남은 경계

이번 보정은 필수 source를 셀에 모두 넣어야 한다는 잘못된 가정을 제거한다.
상단과 하단을 독립적으로 생성하며, 표 밖에 실제로 존재하는 원래 경계의 page 소유와
셀 내부의 물리 조각을 함께 보존한다. paint를 지우거나 셀에 밀어넣는 예외가 아니다.
분할/이어받기·rowspan pagination 경로는 수정하지 않았다.

외부 paint의 일반 행열 이동/상속·비표준 스타일 교체는 B15/B16의 미완료 경계다.
B19 비사각 text clip, 일반 무외곽/미확정 격자 및 본문 메트릭 보류도 유지한다.
P4 전체 완료·작성 당시 표 복원의 보편적 정답·P5 최신 devel/전체 회귀/lint/package 게이트
통과를 주장하지 않는다. 이번 로컬 절편의 focused·전쪽 검증과 사용자 판정만 기록한다.
원격 push·PR 생성/상태 변경·GitHub 댓글은 하지 않았다.


## 실제 소비 코드 위치와 종료 확인

- `table_detection.rs::boundary_contributor`(371행)의 추론 근접도는 경계 후보 근거다.
  `structured_lowering.rs::lower`(1189행)의 `rule_controls`가 실제 positional control을 찾고,
  `table_paint.rs::partition`(319행·571행)의 source frame 교집합과 양수 물리 측도로
  셀 소유와 교차하지 않는 원래 page 소유를 구분한다. 근접 허용폭을 paint 이동에 재사용하지 않는다.
- 셀 조각은 `structured_lowering.rs`(1402행)의 실제 root/instance를 찾은 뒤
  `table_paint::anchor`(622행)의 cell-local Column/Para offset으로 연결되고,
  실제 `cell.paragraphs[0].controls`에 들어간다. 그 뒤
  `renderer/layout/table_cell_content.rs::layout_cell_shape_with_parent_path`(684행)의
  inner cell 원점/offset 분기가 최종 좌표를 소비한다. 원래 source를 지우기만 한 결과가 아니다.
- 완전히 외부인 필수 경계는 `removed`에 넣지 않는다.
  `structured_lowering.rs`(1487행)의 unremoved control 수집과 실제 owner paragraph 재할당까지
  그대로 추적했다. 새 외부 좌표 보정·셀 높이 가산은 없고 셀 anchor 경로도 타지 않는다.
  이번 변경은 측정/배치 높이를 별도로 조정하지 않는다. 배경 paint의 보임과 셀 내용의 흐름을
  구분하며 실제 셀/원문 포함·두 root/문자 편집 출력·외부 paint 양으로 결과를 확인했다.
- source/Native/fresh WASM·실제 Studio의 동일 제품 신원을 보존한다. 기본 Studio 7700은
  이번 작업의 산출물로 교체하지 않았다. 종료 검증의 hash와 로컬 checkpoint는 ignored
  `completion.json`에 보존한다. stage9에서 이미 수용한 다른 쪽은 같은 판정을 반복 요청하지 않았다.
