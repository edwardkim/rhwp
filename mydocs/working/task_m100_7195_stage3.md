---
kind: investigation
status: active
---

# #7195 3단계 — owner content box의 좌우 여백 반영

## 범위와 승인

- 작업지시: "코드를 구현해서 문제가 해결되는지 확인합니다. 시작하세요".
- [구현계획](../plans/task_m100_7195_impl.md), 이전 단계 `f0d6966b6`.
- 제품 기준 `8d45f242baa1a565357aaa38e9f459595b1e756c` + 현재 소스 diff.
- 별도 review worktree `rhwp-review-7195`, 공유 target `rhwp/target/pr-review`.
- #2279/#3798, 원격 push/PR/댓글/종료, 기준값 변경 없음. #2308의 ignore 유지
  여부는 바꾸지 않고 사유를 시각 판정 대기로 현행화했다.

## 구현 규칙

owner 파생 폭을 만들 때 `Cell::effective_padding(&owner_table.padding)`의 좌우를
cell width에서 뺀다. 원본 IR의 table/cell 선언 폭은 수정하지 않는다.
기존 파생 폭 scale을 높이 측정, cell unit 분할용 줄 구성, 열 폭 및 실제 layout이
사용하므로 paint 이후의 clip을 늘리거나 문자를 숨겨 통과시키지 않는다.

실물: 바깥 표 공통 좌우510HU(각 약1.80mm), host `aim=false`,
38245 − 510 − 510 = **37225HU = 496.333px**.
안쪽 표의 공통 좌우는0이고 inner cell `aim=false`이므로 안쪽 셀에 저장된510HU를
다시 차감하지 않는다. 바깥 여백과 안쪽 여백의 선택은 구분한다.

일반 중첩 표를 확대/축소하는 새 조건을 추가하지 않았다. 기존 short-owner
projection 선택 조건은 유지했다. 이 과거 호환 조건의 일반화나 다단계 중첩 전반의
설계 완료까지 주장하지 않는다.

## 수정 전 검출

새 `tests/cases/issue_7195_nested_owner_padding.rs`를 수정 전 제품으로 실행:

- run `0817e0dc-c1d3-477a-b8c6-a55f093d2c72`, exit100, **0 PASS / 2 FAIL**.
- 표 공통510/510에서 파생 폭38245, 기대37225로 실패.
- 실제81쪽 child bbox 폭509.933, 기대496.333으로 실패.
- 초기 진단은 field control을 고려하지 않고 child control index를0으로 가정하여
  projection lookup에서 실패했다. 실제 control 위치를 찾아 수정하고 위 음성 대조를 다시 실행했다.

## 집중 검증 중간 결과

`output/7195/stage3/run.mjs`, `focused.json/.stderr`에 명령·시간·결과 보존.
첫 수정본 run `41157352-e018-4298-bc4a-83eb5015db1b`: **13 PASS / 1 FAIL**.
신규 여백 precedence 5변형 및 두 페이지 line/clip 경계 검사는 통과했다.
기존 #2308 이어짐 assertion의 실패는 아래 실제 출력에서 검사 방식 문제로 분리했다.

검증 환경 정리:

- 주 작업 트리의 `cargo fmt --all -- --check`는 이전 #7090 generated suite가
  가리키는 원본 부재로 실패했다. unrelated 생성물을 수정하지 않고 정식 review worktree에서
  suite를 준비한 뒤 fmt를 실행하여 통과했다.
- 새 테스트 포맷 후 source weight가 바뀌어 manifest drift가 발생했다. 최종 검증 전
  review worktree에서 재생성/재검사한다. 생성물은 커밋 대상이 아니다.
- source-side tier 검사: 4205 tests / 298 modules, PASS.
- nextest0.9.137과 저장소 권고0.9.140 차이 및 JUnit 설정 경고 유지. 도구 업그레이드는 하지 않았다.

## 실제 출력과 이어짐 검사 정정

`cargo build --locked --profile release-test --target-dir .../rhwp/target/pr-review -p rhwp --bin rhwp`
성공(2분47초) 후 같은 바이너리로 PDF 비교·전체 SVG/render tree·선택쪽 debug SVG를 생성했다.
원본/PDF hash와 바이너리 hash, 제품 diff는 `output/7195/stage3/provenance.json`에 보존했다.

- 전후 **82쪽**. 전체82 SVG와82 render tree 비교에서 **81·82쪽만 변경**.
  34·66·70쪽을 포함한 앞80쪽은 byte-identical이다. 이는 새 한컴 시각 통과 선언이 아니다.
- 중첩 표/셀/첫 TextLine 폭 모두 **496.333px**, 시작x207.44px.
  오른쪽703.773px, 부모 grid 오른쪽710.573px와6.8px(오른쪽510HU) 간격을 보존한다.
- 81쪽 첫 줄 `○ 구내운반차 … 등의 사고`.
- 82쪽 첫 가시 줄 `를 예방함으로써 산업재해 감소*에 기여할 것으로 예상되나`.
  이전 줄은 continuation 내부의 위쪽 clip 밖에 있으므로 텍스트 추출만으로 중복 paint라고
  판단하지 않았다. 실제 PNG에서 이어짐/우측 글자 잘림 해소를 확인했다.

실패한 기존 검사에서 p82 첫 줄은 `를 ` / `예방함으로써 산업재해` / ` 감소` 등으로
분리되어 있었다. `contains_text`는 단일 TextRun만 검색한다. **기대 페이지·문구를 유지하고**
기존 p81 검사와 같은 `line_contains_text`로 바꾸었다. 문단/페이지 전체를 이어 검색하지 않는다.
stage2의 변경 전 진단에서는 line 단위로 이어도 해당 문구가 없었으므로, 기존 결함을 통과시키는
완화가 아니다. ignore 해제는 최종 시각 판정 이후 별도 절편에서 검토한다.

판정 자료:

- [81쪽 PDF/수정본 PNG](../../output/7195/stage3/after/cmp-p080.png)
- [82쪽 PDF/수정본 PNG](../../output/7195/stage3/after/cmp-p081.png)
- `output/7195/stage3/debug/76076_regulatory_analysis_081.svg`, `_082.svg`
- 변경 전: `output/7195/stage2/final/2308/cmp-p080.png`, `cmp-p081.png`
- 전후 기하·변경 쪽 원장: `output/7195/stage3/geometry.json`

## 최종 집중 검증과 남은 게이트

정정한 검사까지 **14 PASS / 0 FAIL** (run `622b2993-b3ea-484f-a347-c66790481e2c`),
fmt 및 suite manifest도 최종 통과했다. 1008 skipped는 이 실행의 필터 제외 수이며 ignored 수가 아니다.
명령·실행 ID·시각은 `output/7195/stage3/focused-final.json/.stderr`,
`fmt-final.json`, `manifest-final.json`에 보존했다. 검증한 review worktree의 제품·테스트
6파일이 작업 브랜치와 byte-identical임을 확인했다.
최종 CLI 빌드도 PASS. `seal.mjs`로 최종 binary hash가 출력 생성 때의
`419b56254ea8560d53229a735fbca069d95f4f05a8c2a90b304b18b893bb23db`와
같음을 확인했다. `closure.json`에 소스/테스트 hash와 최종 diff·검사 결과를 보존했다.
`git diff --check` PASS. 판정 PNG는 작업지시자에게 제시하며 수정은 아직 미커밋 상태다.
전체 회귀·native/WASM/workspace Clippy 묶음·Docker WASM·최종 시각 판정은 아직 실행/완료하지 않았다.

## 추가 시각 피드백 — 81쪽 위쪽 표 (조사, 미수정)

작업지시자는 아래쪽 `등의 사고` 개선을 확인했으나, 위쪽 계산식의 조기 줄바꿈과
끝말 `의견` 누락을 지적했다. bug-hunter 절차로 기존 PDF와 같은 페이지를 대조했다.
위쪽 `s0:pi=831 ci=0` 7×2 표 전체의 render tree는 stage2/stage3 사이 동일하다.
따라서 이번 owner 여백 차감이 새로 만든 회귀는 아니다. 해당 표는 81쪽에만 출력된다.

- 대상: 마지막 r6,c1 셀의 p0, ci1 중첩1×1 표.
- 부모 셀 선언 폭37962HU, 유효 좌우 여백510/510HU → 가용 폭36942HU(492.56px).
- 자식 표는 Absolute 폭36572HU(487.627px), 유효 좌우 여백0/0이다.
  기존 코드가 이 자식에서 부모 여백을 다시 차감하는 것은 관측되지 않았다.
- 계산식 원문에 수동 개행은 없다. 글자모양183은 한양신명조14pt, 장평100%,
  자간0, 문단 줄간격160%다. 저장 LineSeg 없이 Frame이 다시 줄을 구성한다.
- 원래36572HU에서 첫 줄 `…안전조치 비용(약 `, 다음 줄 `2만원) = 100,000,000원`.
  진단 복사본의 폭만36942HU로 바꾸면 첫 줄 `…비용(약 2만`, 다음 줄
  `원) = 100,000,000원`으로 PDF의 줄 경계와 일치한다. **이것만으로 Absolute
  자식 표를 부모 폭으로 확장하는 것이 올바른 규칙이라고 확정하지 않는다.**
- 마지막 p6 원문에는 `의견`이 있으며,36572/36942HU 모두 Frame 재구성 결과의
  네 번째 줄에 남는다. 실제 CLI의 CellUnit 진단도 p6 lines3..4를 포함한다.
  그러나 최종 render tree에는 p6의 세 줄만 있고 `의견`은 없으며, 다음82쪽으로
  이어지지도 않는다. 파싱 누락이 아닌 줄 구성 이후 배치/분할 단계의 소실이다.
- 진단 CellUnit 총 높이477.1px와 실제 자식 bbox 높이467.5px가 다르다.
  정확히 어느 cut/viewport 선택에서 마지막 줄을 버리는지는 아직 미확정이다.
  단순 폭 확장이나 clip 확대를 해결로 적용하지 않았다.

증적: `output/7195/stage3/upper-diagnostic.rs/.log`, `upper-trace.json/.log`,
`upper-trace/`의 실제 SVG. 진단은 기존 최종 librhwp.rlib를 rustc로 링크했으며,
제품 소스/테스트를 추가 변경하지 않았다. 공식 전체 회귀 실행이 아닌 국소 진단이다.
기준 PDF의 출력 경로/설치 폰트 미확인 한계는 stage2 provenance와 같다.

### 글꼴 이름/메트릭 가설 확인

작업지시자는 한컴의 표시가 `신명조14pt`라고 알렸다. 원본 HWP5의 CharShape183~185와
FACE_NAME raw bytes를 직접 확인했다. base_size=1400, 한글 font_id10/영문11/기타9의
원명 및 alt_name은 `한양신명조`이며 사용자 슬롯만 `명조`다. 파서는 FACE_NAME의
UTF-16 문자열을 읽어 보존한다(`src/parser/doc_info.rs::parse_face_name`). 따라서
`신명조` 원명을 파싱 도중 `한양신명조`로 바꿨다는 가설은 이 입력의 원본 레코드와
맞지 않는다. 한컴 UI 표시명과 저장 원명/실제 사용 글꼴의 관계는 미확정이다.

같은36572HU 폭·14pt·자간을 유지하고 진단용 해석 스타일의 이름만 `한양신명조`,
`신명조`, `HY신명조`, `HYSinMyeongJo-Medium`으로 바꿔 Frame을 실행했다.
이 두 문단의 줄 경계는 네 경우 모두 같았다. 실제 글꼴 파일 교체나 한컴 메트릭을
재현한 검사가 아니므로 글꼴 메트릭 문제 자체를 기각하지 않는다.

별도로 rhwp는 폭 계산용 내장 메트릭/보정과 SVG/CSS 표시용 폴백을 가진다.
`text_measurement.rs`에는 이 p81을 근거로 작성된 한양신명조 공백411/1024em 보정도
존재한다. 이 주석을 현재 원인 확정으로 사용하지 않으며 실제 Frame 적용 여부와
한컴의 글자·공백 advance를 더 대조해야 한다. 로컬 Fontconfig의 세 한글 이름 조회가
DejaVu Sans로 폴백한 사실 역시 Windows Studio의 실제 사용 face 증거는 아니다.

증적: `upper-font-diagnostic.rs/.log`. 원본·제품 소스는 추가 수정하지 않았다.

## Docker WASM 확인용 빌드

작업지시자의 WASM 판정 요청으로 2026-09-16에 현재 미커밋 수정본을 빌드했다.
`task_m100_7195`, HEAD `f0d6966b62fc587a4773b66046a85b76b39cbefd` + 이 단계의 diff다.
아래쪽 owner 여백 보정은 포함하지만 위쪽 표의 HFT 폭/`의견` 누락 수정은 포함하지 않는다.

- `docker compose --env-file .env.docker run --rm wasm`: exit0, **7분25초**, 최적화 포함.
- 산출물 `pkg/rhwp_bg.wasm`:11,199,085bytes,
  SHA256 `ed1d512e702a5d995fb97c4eccd6d47aee1b4015948dc11e62430ee779258416`.
- `node output/7195/stage3/verify-wasm.mjs`: 새 WASM 초기화 및 원본 HWP 열기 성공,82쪽.
- 기존 Studio `http://127.0.0.1:7700/` HTTP200. `/@fs/.../pkg/rhwp_bg.wasm`의
  HTTP200 응답 바이트가 새 산출물 SHA256과 일치한다. 서버 재시작/교체는 하지 않았다.
- 빌드 전후 제품 두 파일과 Cargo.lock 해시 불변. 소스/lock 변경은 없고 pkg만 갱신했다.
- 로그 `output/7195/stage3/wasm-docker.log`, 검사 `wasm-verification.json`.
  Node WASM 열기/HTTP 검증은 브라우저 시각 판정을 대체하지 않는다.81쪽 작업지시자 판정 대기.

## WASM 피드백: continuation 중복 줄 수정 착수

작업지시자가 p81의 `○ 구내운반차 … 등의 사고`가 p82 표 위에 다시 나타나는
스크린샷을 제공했다. 앞선 SVG 비교에서 clip 밖 재방출을 허용했던 검증은 이 문제를
잡지 못했다. HFT 폰트 차이는 지시에 따라 범위에서 제외하며, 중복 줄은 이번 절편에서
고친다. 실제 source cut/배치 원점이 일치하도록 처리하고 clip으로 숨기는 수정은 하지 않는다.

신규 `continuation_owns_only_remaining_source_lines_without_relying_on_clip` 계약은
수정 전 라이브러리에서 FAIL: p82에 이전 줄 y56.2867px, 다음 줄 y77.0867px가
함께 방출된다. 새 검사는 p81 소유 줄이 p82 tree에 다시 없고 잔여 줄은 원래 위치에
남는지 검사한다. 이는 글꼴 교체/줄바꿈 기대값 변경과 독립적인 분할 소유권 문제다.

### 원인 계층과 적용 규칙

- `mixed_nested_split_from_cut`: 부모 CellUnit의 `[lo,hi)`로 child flow 소비 높이와
  terminal 여부를 결정한다. 이번 변경은 cut의 수용 예산/flow 예약/표 높이를 바꾸지 않는다.
- `layout_table` → `layout_horizontal_cell_paragraphs`: scalar 1×1 continuation은
  물리 원점을 소비 offset만큼 올린 상태에서 child source cut을 복원한다.
- 관측 offset22.68px에 `cell_units_fitting_height`는 **1 unit /20.8px**를 반환한다.
  그런데 `replay_terminal_boundary_unit`이 이를0으로 되감아 이미 소비된 첫 줄을 재방출했다.
  `duplication-trace.log`의 임시 계측으로 확인했으며 계측 코드는 제거했다.
- 되감기 flag와 전달 코드를 제거했다. 물리 원점이 남아 있는 scalar continuation에서만
  같은 CellUnit prefix의 높이를 원점에 더해, prefix를 그리지 않고도 다음 줄 위치를 보존한다.
  새 원점을 받는 long terminal cursor와 recursive RowCut은 이 높이를 다시 더하지 않는다.
  부분 unit을 fully consumed로 승격하는 ceil/clamp나 문서명/문구 분기는 추가하지 않았다.
- 범위가 달라지지 않은 일반 terminal tail/nonterminal legacy 경로, multi-row/rowspan 재귀
  분할 알고리즘은 재설계하지 않았다. 그 경로 전체의 정합성 개선을 주장하지 않는다.

기존 WASM의 `setClipEnabled(false)` SVG를 Chrome으로 그리면 스크린샷과 동일하게
p82 표 위에 중복 줄이 나타난다(`duplication-before/p82-no-clip.png`). 사용자의 실제
브라우저 설정은 직접 읽지 않았으므로 짤림보기 설정을 단정하지 않는다. 두 Canvas 경로의
clip 처리와 관계없이 공통 render tree에 이전 줄이 없어야 한다는 계약으로 수정한다.

수정 후 standalone 집중 검사3건은 PASS. 별도 합성 진단에서 용지 아래 여백을 바꾼
가용 높이로 2줄 소비/1줄 소비/표 전체 이월을 만들었으며, 각각 잔여6줄/7줄/전체8줄이
방출된다. 이 숫자는 경로 도달 확인이며 한컴 시각 기대값이 아니다. 새 통합 계약은
원본 child 문단 문자열과 전체 조각의 텍스트를 순서대로 대조해 중복·누락을 검사한다.
여백 변형본의 다른 페이지 조판은 시각 검증하지 않았다.

### 최종 native 집중 검증과 WASM 재현

- nextest **16 PASS /0 FAIL**, run `7edcbc38-d6b6-4f80-9aee-33ca83503003`.
  972 skipped는 필터 제외다. `focused-duplication.json/.stderr`에 명령과 결과를 기록했다.
  fmt·suite manifest·최종 native CLI 빌드도 PASS. 작업/review의 제품·테스트6파일 동일.
- 원본82쪽 전체 재출력: 앞81쪽 SVG/render tree는 byte-identical. p82만 변경되며
  제거된 TextLine은 이전 쪽의 `○ 구내운반차 … 등의 사고` 1개다. 새 줄/좌표 변경은 없다.
  부모 표 p81 bbox `(75.6,913.9,635,119)`, p82 `(75.6,75.6,635,173.1)` 유지.
  `duplication-export.json`, `duplication-geometry.json` 참조.
- 클리핑을 적용한 native p81/p82 PNG는 각각 수정 전과 byte-identical이다.
  작업지시자의 **SVG/PNG에서는 보이지 않고 WASM에서만 보였다**는 관측과 구분하여
  기록한다. native PNG에 보이는 중복이 있었다고 주장하지 않는다.
- 수정 전 **실제 Chrome/152 Canvas**의 `HwpDocument.renderPageToCanvas`로도
  clip=true/false를 비교했다. true는 기존 정상 SVG처럼 보이고 false는 제공 이미지처럼
  중복된다. `duplication-canvas-before/manifest.json`은 이전 WASM
  `ed1d512e...` 해시와 실행 backend를 봉인한다. 사용자 Studio의 실제 설정/선택 backend는
  직접 읽지 않았으며, 왜 해당 환경에서 클리핑이 달랐는지는 미확정으로 남긴다.
- 새 Docker WASM 빌드 진행 중. 최종 사용자 WASM 시각 판정, 저장소 전체 회귀,
  PR 직전 Clippy 묶음, ignore 해제·커밋·push는 아직 완료하지 않았다.

### 새 WASM 빌드 및 실제 Canvas 확인 완료

위 빌드는 2026-09-16 완료: `docker compose --env-file .env.docker run --rm wasm`,
exit0, **7분28초**, `wasm-opt` 포함. `pkg/rhwp_bg.wasm` 11,199,221bytes,
SHA256 `48431e7ecb3503b04017edcf698db0588a74a57c27e1a71d0e82d965e1e6240f`.

- Chrome/152의 **실제 `renderPageToCanvas`**로 p81/p82를 clip=true/false 각각 실행했다.
  두 모드 모두 p82 render tree에서 이전 줄이 제거되고, 나머지 줄/표 bbox는 전부 유지된다.
  `duplication-canvas-check.mjs` PASS, `duplication-canvas-check.json` 참조.
- clip=false Canvas PNG를 직접 열어 p82 표 위의 중복 줄이 없고, 표 내부 첫 줄
  `를 예방함으로써…`부터 마지막 `기업의 자체 보상비 등)`까지 남음을 확인했다.
  clip=true p81/p82 PNG는 이전 WASM의 같은 모드 PNG와 byte-identical이다.
- [수정 전 Canvas, clip=false](../../output/7195/stage3/duplication-canvas-before/p82-no-clip.png)
  / [수정 후 Canvas, clip=false](../../output/7195/stage3/duplication-canvas-after/p82-no-clip.png).
  이 probe는 별도 Chrome 세션의 직접 Canvas 검사이며 실제 사용자 Studio 설정/편집 세션을
  재현했다고 주장하지 않는다. 글꼴 피델리티 전체 판정도 이번 범위가 아니다.
- 새 WASM 열기82쪽. Studio7700의 WASM HTTP200 응답 SHA가 위 새 파일과 일치한다.
  `wasm-duplication-verification.json`, `wasm-duplication-docker.log`,
  `duplication-canvas-after/manifest.json`에 봉인했다. 빌드 후 소스는 변경하지 않았다.

**남음:** 저장소 전체 회귀와 PR 직전 lint 게이트,
ignore 해제/단계 완료 절차. 이번 수정은 아직 미커밋이며 원격 작업은 하지 않았다.

### 작업지시자 WASM 판정 및 글자 크기 재설정 관측

2026-09-16 작업지시자가 **WASM의 다음 페이지 중복 출력 문제 해결**을 확인했다.
이 판정은 중복 수정에 대한 시각 통과이며, 전체 #7195 완료나 글꼴 피델리티 통과를 뜻하지 않는다.

추가 관측: 앞서 신명조 관련 누락을 보인 셀 안의 문단을 전체 선택하고 글자 크기를
14pt → 13pt → 14pt로 설정하면 전체 글자가 보인다. 이는 작업지시자의 실제 Studio
편집 관측이며, 에이전트가 같은 편집 세션의 전후 IR/레이아웃을 수집한 결과는 아니다.

- 최종 지정 크기가 다시 14pt인데 누락이 사라지므로, HFT 대체 글꼴의 폭 차이만으로
  증상을 설명하거나 원인을 확정할 수 없다.
- 최초 로딩의 저장 줄 정보·파생 레이아웃과 편집 후 재조판/캐시 무효화 경로의 차이가
  조사 후보다. 전체 선택 후 크기 재지정이 혼재한 글자모양을 정규화했을 가능성도 있으므로
  최종 표시 크기가 같다는 사실만으로 모든 서식·내부 상태가 동일하다고 단정하지 않는다.
- 후속 확인에서는 최초 열기/13pt 변경/14pt 복원 각각의 실제 CharShape run, 줄 구성,
  셀 높이와 분할 cut을 대조해 어느 단계에서 누락이 해소되는지 구분해야 한다.
- HFT 글꼴 자체의 피델리티 개선은 기존 지시대로 범위 밖이다. 이 피드백으로 제품 코드를
  추가 변경하거나 조사 범위·ignore 기준을 자동 확장하지 않았다.

### 별도 이슈 등록 및 남은 회귀 검토 순서

작업지시자 승인으로 크기 재설정 전후 텍스트 누락 차이를
[#7202](https://github.com/edwardkim/rhwp/issues/7202)로 분리 등록했다.
동일 샘플·증상 선행 검색 후 입력, 재현 단계, 관측/가설 구분, WASM 해시와 미검증 범위를
게시하고 API로 OPEN 상태와 본문을 재확인했다. 이 이슈의 제품 수정은 여기서 진행하지 않는다.

남은 두 계약은 [2단계 조사](task_m100_7195_stage2.md)의 #2279 → #3798 순서로 검토한다.

- #2279: `samples/86712_regulatory_analysis.hwp`. 기존 조사에서 rhwp 64쪽/참고 PDF 65쪽,
  근거설명은 rhwp p27/PDF p28이며 그 앞 산식 표부터 p26/p27 차이가 있다.
  우선 앞쪽 최초 배치 차이와 분할 규칙을 시각 검토한다. 페이지 번호 핀을 바로 바꾸지 않는다.
- #3798: `samples/issue3798/page_end_trailing_spill.hwpx`. 합성 문단의 글자 영역은 본문 안이고
  말미 간격만 넘어간다. 다음 쪽 이동을 요구하는 폐기 실험의 테스트 계약을 재검토한다.
  독립 한컴 기준은 아직 없어 제품 조판 실패로 확정하지 않는다.

위 수치·분류는 2단계 실행 결과이며 새 WASM/현재 수정본으로 두 계약을 재실행한 결과가 아니다.
이번 등록에서 ignore·기대값·제품 소스는 변경하지 않았으며 커밋/push/PR은 수행하지 않았다.

### #2279 재검토: 물리 5쪽 표의 본문 하단 초과

작업지시자는 차이가 5쪽에서 시작하며 표 하단이 편집영역을 넘는다고 지적했다.
bug-hunter 절차로 원본 열기 → 현재 native SVG/render tree → 참고 PDF p4–6 대조를
수행했다. 새 제품 수정은 하지 않았다. 기존 #2308 개선 코드와 동일한 native 바이너리/소스
해시를 `duplication-export.json`과 대조한 뒤 실행했다.

입력 `samples/86712_regulatory_analysis.hwp`, 참고 PDF
`pdf/86712_regulatory_analysis-hwp-2024.pdf`. PDF 메타는 Hwp 2024 0.0.0.0 /
Hancom PDF 1.3.0.550, 65쪽/A4다. 정확한 export UI와 당시 설치 폰트는 미확인이므로
참고 등급을 유지한다. 원본·PDF SHA는 아래 geometry.json에 기록했다.

대상은 **s0:pi=15 ci=0**, `TopAndBottom`, non-TAC, `RowBreak`, 제목행 반복인 **2×2 표**다.
현재 검토하는 5쪽 표에는 중첩 표가 없으며, 앞선 p27의 pi172 중첩 표와 다른 원인 계층이다.
body는 y75.6부터1046.9067px, 높이971.3067px이다(96dpi).

| 물리 쪽 | 열 소비 높이(px) | 최종 표 하단 y(px) | 본문 하단 초과(px) |
| --- | ---: | ---: | ---: |
| 4 | 977.2533 | 1051.2 | 약4.3 |
| 5 | 1010.5733 | 1086.4 | 약39.5 (10.45mm) |
| 6 | 1600.4400 | 1676.3 | 약629.4 |

표 bbox는 render-tree 소수1자리 출력이다. p5 비교 PNG를 직접 열어 표가 쪽번호 영역까지
내려오는 것을 확인했다. p6의 종이 밖 bbox 전체가 화면에 보인다는 뜻은 아니다.
4쪽에서도 geometry상 소폭 초과가 있어 '5쪽 이전에는 모든 경계가 정확했다'고 단정하지 않는다.
**p4–6 render tree는 2단계 출력과 byte-identical**: 이번 #2308 폭/중복 수정으로 새로
발생한 변화는 아니다. 최신 원격 devel이나 릴리즈의 재현 여부까지 새로 확인한 것은 아니다.

#### 원인 경로

1. `advance_row_cut_inner` (`src/renderer/layout/table_layout.rs:13111`)는 continuation에서
   시작 부분/말미의 `empty_spacer`를 높이 누적 없이 cut 인덱스만 전진시킨다
   (약13195–13212행). p5는 `[27,27] → [56,55]` 컷을 선택한다.
   기존 `RHWP_DIAG_SCAN` 실측은 budget935.1px, content 소비940.8px다.
2. 같은 선택 범위를 `row_cut_content_height` (약15693행)에서 다시 측정할 때는
   `units[su..eu]`의 원래 높이를 합산한다. 예산 계산에서 생략한 빈 줄 높이가 복귀한다.
   p5 왼쪽은29유닛 ×33.6px =974.4px, 오른쪽은28유닛 ×33.6px =940.8px다.
   여기서 큰 쪽974.4px에 셀 padding3.76px와 반복 header32.4133px가 붙어
   **1010.5733px**가 된다. source의 빈 문단도 14pt/180%, pitch2520HU=33.6px다.
   실제 p5 왼쪽 셀 tree에도 빈 TextLine이 남으며 마지막은 y1050.7px다.
3. typeset의 초과 검사 (`src/renderer/typeset.rs:24009` 부근)는
   `r > cursor_row || mixed_nested_owner_guard || native_split_continuation_row_tail`
   조건 안에 있다. 현재 행은 `r == cursor_row`, 일반2열/non-nested continuation이어서
   재-cut 검사에 들어가지 않고 초과 조각을 수용한다.
4. 일반 cut이 예산보다5.7px 더 소비한 부분에는 저장 reset/꼬리 흡수 등의 경로가
   추가로 관여할 수 있다. 현재 로그는 그 내부 분기별 발화까지 찍지 않았으므로
   특정 흡수 함수를 단독 원인으로 확정하지 않는다. 하지만 예산/최종 높이 불일치와
   최종 수용 검사 누락은 원본·cut·geometry·코드 대조로 확인된다.

따라서 단순 폰트 피델리티 차이나 총 페이지 수 assertion 문제가 아니다. **동일 조각의
분할 소비 높이와 실제 점유 높이가 일치하지 않고, 초과를 검증하는 경로도 제한되어 있다.**
빈 문단을 일괄 삭제하거나 외곽선을 본문 경계에서 자르는 수정은 해결이 아니다.
후속 구현에서는 선택된 줄/빈 문단의 점유 높이·padding·반복 제목행을 공통 결과로
계산하고, 일반 continuation에도 같은 실제 fit 규칙을 적용해야 한다. 진행 보장용
page-larger atom과 정상적인 분할 가능 텍스트를 구분해야 하며, 구현/전역 회귀는 미실행이다.
이것이 p27 및 총64/65쪽 차이 전부의 원인인지도 아직 확정하지 않았다.

증적: `output/7195/stage3/2279-p5/`의 `provenance.json`, `geometry.json`,
`pages.stdout`, `scan.stderr`, `raw.rs/raw.log`, `pdfinfo.txt`.
진단은 현재 라이브러리에 링크한 standalone reader와 기존 환경변수 계측이며 제품을 패치하지 않았다.

- [물리5쪽 비교 PNG](../../output/7195/stage3/2279-p5/compare/cmp-p004.png)
- [물리5쪽 디버깅 SVG](../../output/7195/stage3/2279-p5/debug/86712_regulatory_analysis_005.svg)
- 현재 WASM 사용자 관측과 native 재현을 연결한 조사다. 이번 턴에 WASM을 새로 빌드하거나
  동일 페이지 Canvas를 다시 캡처하지는 않았다. 기대값·ignore·baseline 변경은 없다.

### #2279 승인 구현 및 Docker WASM 검증

2026-09-16 작업지시자: “수정 구현 후 wasm 빌드합니다.” 기존 #2308 변경을 보존하고
위 continuation 원인만 수정했다. 기준 HEAD는 `f0d6966b62fc587a4773b66046a85b76b39cbefd`
이며 미커밋 변경을 포함한다. 정확한 제품/테스트 파일 SHA와 native 실행 파일 SHA는
`output/7195/stage3/2279-p5/after-provenance.json`에 고정했다.

#### 적용 규칙

- `advance_row_cut_inner`에서 continuation 시작/말미의 빈 문단을 높이 0으로 소비하던
  두 분기를 제거했다. 같은 `CellUnit.height`를 cut과 실제 조각 높이 계산에 사용한다.
- typeset에서 일반 continuation도 실제 조각 높이와 본문 예산을 비교한다.
- 첫 시도에서 새 테스트는 p5 초과가 약39.5px에서 약5.9px로 줄었으나 여전히 실패했다.
  재시도도 저장 프레임 초과 흡수(최대48px)를 다시 허용하는 경로가 남아 있었다.
  물리 초과 후 재시도는 `advance_row_cut_within_capacity`로 같은 walk를 호출하되
  저장 프레임 초과 흡수를 끈다. 줄 소유권·atomic control·진행 보장 규칙은 재사용한다.
- 문서 ID·페이지 수로 제품을 분기하지 않았고 원문/빈 문단을 삭제하거나 결과를 clamp하지 않았다.
  기존 첫 조각 저장 프레임 allowance와 다른 historical tolerance의 전면 개편은 하지 않았다.

#### 집중 검증

- 새 `tests/cases/issue_7195_continuation_height.rs`: 원본 및 하단 여백 +1200HU에서
  pi15의 **continuation** 본문 fit, 셀별 원문 텍스트 중복/누락 방지를 검증한다.
  첫 조각은 기존 저장 프레임 허용 규칙이 별도로 있으므로 이 계약에서 제외한다.
  총 페이지 수를 고정하거나 한컴 시각 일치로 간주하는 테스트가 아니다.
- 기존 내부 row-cut 테스트에 빈 문단 2개를 선행/말미로 둔 용량·실제 높이 일치 검사를 추가했다.
- 수정 전 바이너리에 새 계약을 링크한 red 검사: 처음 p4에서 실패했고 continuation 범위를
  명확히 한 뒤 p5 하단1086.4 > 본문1046.9로 실패했다(`red-test.log`, `red-continuation.log`).
- 첫 수정 집중 검사: **59 pass / 1 fail** (`focused.*`). 재시도 보완 후
  `node output/7195/stage3/2279-p5/validate.mjs focused-capacity`: **60/60 pass**.
  #2308/#5301/#3128/#7195 및 기존 row-cut 테스트를 포함한다. 상세 Cargo 명령은
  `focused-capacity.json`, 결과는 `focused-capacity.stderr`에 있다.
- review worktree의 fmt check, unit-tier check(4206개), suite manifest check 통과.
  root fmt는 과거 generated harness의 삭제된 source 참조 때문에 실패하여 기존 파일을
  임의 정리하지 않았다. 검증 전용 review worktree에서 정상 fmt를 수행했다.
  fmt로 바뀐 generated harness는 다시 `--prepare`한 뒤 `manifest-check-final.log`로 확인했다.
  generated 결과는 제출 대상이 아니다. 전체 회귀/Clippy 3종 PR 게이트는 이번에 미실행이다.

#### 실제 출력과 남은 경계

새 native 전체 SVG/render tree를 생성하고 참고 PDF p4–6과 표준 비교 PNG를 만들었다.
5쪽에서 표 외곽이 쪽번호 영역을 침범하지 않는 것을 직접 확인했다. 다만 참고 PDF는
한 줄을 더 수용하고 글꼴·줄폭/표 외곽선에도 차이가 있어 한컴 피델리티 통과를 선언하지 않는다.

| 물리 쪽 | 수정 전 표 하단 y(px) | 수정 후 표 하단 y(px) | 본문 하단 y(px) |
| --- | ---: | ---: | ---: |
| 4 | 1051.2 | 1051.2 | 1046.9 |
| 5 | 1086.4 | 1019.2 | 1046.9 |
| 6 | 1676.3 | 1019.2 | 1046.9 |

현재 pi15 조각은 p4–9에 배치된다. 원문 보존 검사는 모든 조각을 합쳐 검증했다.
4쪽 첫 조각의 약4.3px 초과는 기존 allowance이며 **미해결로 남긴다**.
native/WASM 모두 총64쪽이다. 참고 PDF65쪽과의 차이 및 p27 별도 중첩 표 원인이
전부 해결되었다는 뜻이 아니며 #2279 ignore/기대값은 유지했다.

- [5쪽 참고 PDF/native 비교](../../output/7195/stage3/2279-p5/after/cmp-p004.png)
- [5쪽 디버깅 SVG](../../output/7195/stage3/2279-p5/after-debug/86712_regulatory_analysis_005.svg)
- [5쪽 WASM Canvas, clip 해제](../../output/7195/stage3/2279-p5/canvas/p5-no-clip.png)
- [6쪽 WASM Canvas, clip 해제](../../output/7195/stage3/2279-p5/canvas/p6-no-clip.png)

#### WASM 준비

`docker compose --env-file .env.docker run --rm wasm` 성공(8m43s).
새 `pkg/rhwp_bg.wasm`: 11,199,057 bytes,
SHA256 `66b9a97acd3edb4443e350d30d4f037811e122ea986c74f1e5ba54a054aa5cc5`.
Studio `http://127.0.0.1:7700/`의 WASM HTTP200 응답과 빌드 해시가 일치한다.
76076도 새 WASM으로 열어82쪽임을 smoke 확인했다. 사용자 브라우저나 Studio 설정은 변경하지 않았다.

별도 Chrome152 세션에서 `HwpDocument.renderPageToCanvas`의 5·6쪽을 clip on/off로
각각 렌더했다. 두 모드 모두 표 하단1019.2px, 본문 하단1046.9px다. PNG도 직접 확인했다.
`wasm-verification.json`, `canvas/manifest.json`, `wasm-build.log`에 증적이 있다.
최종 작업지시자 시각 판정은 대기한다. 커밋/push/PR/원격 댓글은 수행하지 않았다.

### 작업지시자 판정: 5쪽 문제 해결, 21쪽 표 분할 차이 수용

위 판정 대기 이후 작업지시자가 새 WASM을 직접 확인하고 다음과 같이 판정했다.

- 이번에 수정한 표 하단의 본문 초과 문제는 **해결**이다.
- 다음 총 페이지 수 차이의 원인은 21쪽 마지막 표다. 한컴은 표를 나누어 조판하지만
  rhwp는 한 페이지에 담으며, 작업지시자는 **rhwp의 조판이 올바르다**고 판정했다.
- 이 21쪽 차이는 rhwp 조판 결함이 아니라 **메인테이너가 수용한 기준 출력과의 차이**로
  분류한다. 한컴의 분할이나 총65쪽을 맞추기 위한 제품 예외 처리는 추가하지 않는다.
  현재 rhwp64쪽/PDF65쪽이라는 수치만으로 이 사례를 회귀 실패로 판정하지 않는다.
- 오래전에 편집된 HWP를 최신 한컴에서 조판할 때 생긴 문제라는 설명은 작업지시자의
  관측·판단으로 기록한다. 에이전트가 한컴 버전별 동작을 추가 실험한 결과는 아니다.

판정 대상은 앞 절의 WASM SHA
`66b9a97acd3edb4443e350d30d4f037811e122ea986c74f1e5ba54a054aa5cc5`로 제공한 수정본이다.
이번 응답에서는 판정 기록만 추가했다. 코드·ignore·baseline·페이지 assertion은 변경하지 않았다.
후속 계약 정리에서는 21쪽의 수용된 차이를 실제 겹침/누락/본문 초과와 구분해야 한다.
앞서 기록한 4쪽 첫 조각의 소폭 초과 및 문서 내 다른 미검증 영역까지 자동으로
해결 판정한 것은 아니다. 남은 별도 검토 대상은 #3798 문단 말미 간격 계약이다.

### 작업지시자 판정: #3798 현재 WASM 조판 통과

작업지시자가 `samples/issue3798/page_end_trailing_spill.hwpx`를 현재 WASM에서 열어
한컴편집기와 동일하게 조판되는 것을 확인했다. **해당 샘플의 시각 판정은 통과**다.
판정은 직전 제공한 WASM에 대한 사용자 확인이며, 이번 응답에서 에이전트가 한컴을
실행하거나 PDF를 새로 확보한 것은 아니다. 한컴의 상세 버전/출력 증적은 추가 확보하지 않았다.

2단계의 '독립 비교 미확보·시각 판정 보류'는 당시 조사 상태다. 이번 사용자 직접 비교로
시각 판정은 갱신한다. 기존 ignored 테스트가 요구한 '경계 문단은 반드시 다음 쪽으로
이동'은 폐기 실험의 기대값이므로, 그 실패를 현재 샘플의 제품 조판 결함으로 취급하지 않는다.
제품 코드를 그 실험 기대값에 맞추기 위한 수정은 하지 않는다.

이번에는 판정만 기록했다. 테스트 실행 결과를 PASS로 바꾸어 기록하거나 ignore/기대값을
수정하지 않았다. 후속 검증 정리에서 폐기 실험의 보존 여부와 시각 확인된 동작을 보호할
계약을 구분하여 결정해야 한다. 전체 회귀 검증과 #2279 첫 조각의 잔여 경계 확인은 별도다.

### 세 잔여 회귀 계약 재실행 결과

작업지시자의 “회귀 테스트 통과되는지 진행해보고 보고” 요청으로 기존 assertion과 ignore를
변경하지 않은 채 세 이슈의 테스트를 재실행했다. `--run-ignored all`로 기존 ignored 실패도
실제로 실행했으며, 성공으로 보이기 위해 제외하지 않았다.

root 및 `rhwp-review-7195`의 관련 제품5파일/기존 테스트3파일 SHA256 일치를 확인했다.
HEAD `f0d6966b62fc587a4773b66046a85b76b39cbefd` + 현재 미커밋 수정본이다.
실행 스크립트 `output/7195/stage3/run-remaining-three.mjs`, 명령·파일 해시·시간·종료코드는
`output/7195/stage3/remaining-three/result.json`, 전체 로그는 같은 폴더의 `stderr.log`에 있다.

```sh
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -p rhwp \
  --test regression_suite_003 --test regression_suite_015 --test regression_suite_019 \
  --run-ignored all --test-threads 1 --no-fail-fast \
  --success-output immediate --failure-output immediate \
  -E 'test(issue_2279_layout_oracles::) | test(issue_2308_render_normalized_derived_state::) | test(issue_3798_page_end_trailing_spill::)'
```

빌드3m24s 성공, 테스트1.457s, nextest ID `7fef7e8f-a0a3-4277-b924-d8bbb964ea7e`,
종료코드100. **12건 중10 PASS / 2 FAIL**, 선택하지 않은601건은 필터 제외이며 이번 실행은
전체 회귀가 아니다. nextest0.9.137/권장0.9.140 경고와 미지원 설정 키 경고는 테스트 assertion
실패와 구분한다.

| 이슈 | 실제 결과 | 실패 의미와 후속 계약 |
| --- | --- | --- |
| #2308 | 6/6 PASS | 기존 중첩 표/줄 소유 테스트는 현재 구현으로 모두 통과 |
| #2279 | 3 PASS / 1 FAIL | `issue_2279_nested_cell_units_split_r27_not_r26`가 p27의 `편익 수혜자` 존재를 첫 assertion에서 거부. 앞서 수용한21쪽 분할 차이를 반영하지 않은 절대 쪽 배치 계약. 이후 assertion은 panic 때문에 이번 실행에서 평가되지 않았으므로, 일괄 -1쪽 치환으로 해결된다고 단정하지 않음 |
| #3798 | 1 PASS / 1 FAIL | `a_paragraph_whose_trailing_spacing_exceeds_the_spill_limit_moves_to_the_next_page`가 경계 문단의1쪽 배치를 거부. 작업지시자가 한컴과 동일하다고 확인한 동작과 폐기 실험 기대값이 충돌 |

따라서 **시각 판정 통과와 기존 자동 테스트 통과가 아직 일치하지 않는다**.
제품을 예전 쪽 배치/trim-cap 실험에 맞춰 되돌리지 않는다. 후속 작업은 판정 근거를 명시하고
#2279의 source 순서·중복/누락·본문 경계 보호를 보존하는 계약 및 #3798의 실제 쪽말 배치 계약을
정리한 뒤 다시 실행하는 것이다. 이번 요청에서는 실행·보고만 했으며 제품/테스트 기대값/ignore,
baseline, 커밋 또는 원격 상태를 변경하지 않았다.

### 승인된 조판을 보호하는 계약으로 전환·상시 검사 복귀

작업지시자가 계약 수정과 재검증을 승인했다. 이번에는 제품 소스나 샘플을 바꾸지 않고
다음 테스트3파일을 수정했다. 제품5파일은 앞선 `2279-p5/after-provenance.json`의 SHA와
동일함을 재확인했으므로 시각 판정한 WASM은 그대로 유효하다. 새 WASM 빌드는 하지 않았다.

| 파일 | 전환 내용 |
| --- | --- |
| `tests/issue_2279_layout_oracles.rs` | PDF 절대27~29쪽 대신 산식→근거설명→내부 표의 상대 페이지 소유, 주요 문자열의 쪽간/쪽내 중복·누락, 3×12/5×4 표의 순서·Body/owner clip 내 전체 외곽을 검사. 별도 테스트로 원본pi109 비용편익 표의 모든 셀 좌표·병합·원문이 한 조각에 보존되고 본문 안에 들어오는지도 보호 |
| `tests/issue_2308_render_normalized_derived_state.rs` | 이미 시각·집중 검사 통과한 이어지는 줄 계약의 ignore 해제. assertion은 이번에 추가 변경하지 않음 |
| `tests/issue_3798_page_end_trailing_spill.rs` | 폐기 trim-cap 가설 대신 원본의 경계 줄1쪽/뒤6문단2쪽, 모든 본문 원문의 순서·중복/누락, 실제 TextLine 본문 경계를 검사. 하단 여백+2400HU(32px) 합성 반례에서는 경계 줄도 다음 쪽으로 이동해야 함 |

기대값 근거는 원본 IR의 셀/문단 구조, 기존 oracle의 상대적인 내용 소유 보호 의도,
작업지시자의21쪽 배치 수용과 #3798 한컴 직접 비교 판정이다. 총64쪽이나 좌표 실측값을
복사하여 통과시키지 않았다. #3798 변형 입력은 계약 반례이며 한컴에서 확인한 샘플로
승격하지 않는다. 폐기 실험의 보고서·Git 이력은 보존한다.

세 기존 ignored 계약을 모두 상시 검사로 복귀했다. 최종 검증은 `--run-ignored all`을
사용하지 않은 **일반 실행**이다.

```sh
node output/7195/stage3/run-remaining-three.mjs contracts-sealed --normal
```

스크립트는 review worktree의 파생 suite 배정에서 세 모듈을 찾아 실제 Cargo 인자를 기록한다.
최종 suite는009/013/019, nextest ID `8494ad80-017c-4669-8329-5e83e8f8ea4d`,
빌드18.30s/테스트2.076s, **14 PASS / 0 FAIL**, exit0이다.

- #2279: **5/5 PASS** (기존4건 + 수용된 비용편익 표 직접 검사1건).
- #2308: **6/6 PASS**.
- #3798: **3/3 PASS** (기존2건을 재정의하고 실제 글자 초과 반례1건 추가).
- 599건은 선택된 binaries에서 필터 제외된 테스트이며, ignored 수나 전체 회귀 실행 수가 아니다.
- 변경한 테스트3파일 rustfmt check, `git diff --check`, review suite manifest check 통과.
  주석 정리 후 manifest의 자동 배정 drift가 검출되어 `--prepare`와 check를 다시 수행하고,
  그 최종 배정으로 위14건을 재실행했다. 생성 suite/manifest는 source 제출 대상이 아니다.
- 중간13건 통과 및14건 통과 기록도 `contracts/`, `contracts-final/`에 보존했다.
  최종 source SHA/명령/결과는 `output/7195/stage3/contracts-sealed/result.json`,
  전체 출력은 같은 폴더의 `stderr.log`, manifest 검사는 `contracts-manifest-final.log`다.

**잔여 진단을 통과로 덮지 않는다.** 문서 전체를 스캔하는 동안 물리4쪽(pi15)의
LAYOUT_OVERFLOW 약4.1px 및 물리25쪽(pi161)의 약6.8px 경고가 출력됐다.
이 경고는 이번에 보호한21쪽 표/27~28쪽 내용 소유 계약과 별도이고, 원인이 해결되었거나
문서 전체가 무초과라는 판정은 하지 않는다. 4쪽 bbox 실측 약4.3px와 로그 약4.1px는
배치 소비 높이/표 외곽의 계측 차이이며 기존 기록과 함께 보존한다.
전체 회귀와 PR용 Clippy3종 게이트는 미실행이다. 커밋/push/PR/원격 게시도 수행하지 않았다.

## 교체 입력의 4–7쪽 경계 재검증

이하 기록은 작업지시자가 교체한151040byte 입력
`ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88`에 대한 것이다.
위14건 통과와 기존 PDF의 판정은 이전 입력에 한정되며 새 입력의 통과로 재사용하지 않는다.

### 수정 전 원인과 음성 대조

`s0:pi15 ci0`은 반복 제목행이 있는2×2 RowBreak 표다. 본문 하단은1046.9067px이며
행1의 저장 줄은14pt(18.6667px), 줄간격1120HU(14.9333px), 줄 이송33.6px다.

| 물리 쪽 | 수정 전 표 하단(px, 외곽 stroke 포함) | 관측 |
| --- | ---: | --- |
| 4 | 1051.2 | 약4.3px 초과. 다음 행 컷 자체의 초과량을 허용량으로 삼는 예외로 수용 |
| 5 | 1019.2 | 약27.7px 잔여. 다음 한 줄의 실제 상자·아래 여백은 들어가지만 줄 이송 전체로 거부 |
| 6 | 985.6 | 약61.3px 잔여. 저장 경계 초과 흡수 후 strict 재-cut에서 예산을 다시 줄여 과도하게 후퇴 |
| 7 | 1019.2 | 약27.7px 잔여. 앞선 컷 오차 누적과 저장 줄 경계의 trailing 간격 처리 누락 |

기존 trailing 간격 공통 함수는 서로 다른 가시 문단 사이만 인정했다. 문단 내부의 reset과
옆 셀의 의도된 빈 문단이 갖는 source-frame reset을 제외하여, 동일 경계인데도 높이가 달랐다.
같은 문단의 줄 사이 reset은 다음 `vis_start`를 사용해야 하며 문단의 첫 segment를 다시 읽으면 안 된다.

- 수정 전 집중 실행: `2279-p5/source-reset-red.json/.stderr`, **62건 중59 PASS / 3 FAIL**.
  새 동일 문단 reset/빈 문단 reset 검사가0px 대8px로 실패했고, 첫 조각까지 검사한
  실물 경계 계약은4쪽 본문 초과로 실패했다.
- 별도 현재 테스트 소스를 수정 전 rlib에 직접 링크한 실행은 **0 PASS / 2 FAIL**.
  `replaced-86712-p4-p8-before/contract-tests.log`: 본문 초과와 source frame2의
  `ㆍ군수등은 서면동의서 기재사항의` 줄 누락(다음 쪽 조기 이동)을 각각 검출했다.
- source-side unit test 수 상한 검사가 새 `#[test]`3개 증가를 거부했다. 상한을 올리지 않고
  빈 문단 검사는 기존 spacer 계약, reset의 두 변형은 기존 reset 계약의 호출되는 assertion
  helper로 통합했다. 검사 내용은 유지하며 최종4205 tests/298 modules 정책 검사는 통과했다.

### 구현 범위

기존 공통 trailing 계산을 동일 문단·실제 높이가 있는 빈 문단의 저장 물리 reset까지 확장했다.
실제 줄 상자는 최소로 보존하고, 문단 뒤 간격은 문단 끝인 경우만 제외한다. 합성·편집 후 재조판
줄 및 control 소유 문단의 로컬 reset은 근거로 사용하지 않는다. 원본 IR은 변경하지 않는다.
첫 조각의 다음 행 초과를 스스로 허용하던 조건은 제거했다. 일반 strict 재-cut은 저장 경계 초과량을
가용 공간에서 중복 차감하지 않고, 논리 컷에 없는 paint 높이만 예약한다.
별도 mixed-nested owner-tail 재시도 계약 전체를 일반화했다고 주장하지 않는다.

변경 전4–8쪽 증적: `output/7195/stage3/replaced-86712-p4-p8-before/`.
수정 후 집중 검사·같은 입력 출력 대조 결과는 다음과 같다. 시각 통과는 아직 요청 전이다.

### 수정 후 집중 검사와 실제 출력

- `node output/7195/stage3/2279-p5/validate.mjs source-reset-green`:
  **60 PASS / 0 FAIL**, 실행4.940s. RowCut white-box, #7195, #2308, #3128, #5301 범위다.
  source reset 경계별 원문과 실제 조각의 원문이 일치하며, 하단 여백+1200HU 반례에서도
  모든 조각 본문 fit 및 각 셀 원문 한 번 출력을 통과했다.
- CLI 재빌드 성공2m37s. 동일 입력 전체64쪽(전후 동일)이며, 페이지 수를 맞추는 변경은 없다.
- `replaced-86712-p4-p8-after/manifest.json`, `seal.json`, `source.diff`에 입력/바이너리/제품·테스트
  SHA와 root/review 동일성, 실제 export 명령을 남겼다.
- 새 source-side tier, review suite manifest, 변경 Rust 파일 포맷, `git diff --check` 통과.

| 물리 쪽 | 실제 표 하단(px) | 마지막 TextLine 하단(px) | 본문 하단(px) |
| --- | ---: | ---: | ---: |
| 4 | 1036.3 | 1034.2 | 1046.9 |
| 5 | 1037.9 | 1035.8 | 1046.9 |
| 6 | 1037.9 | 1035.8 | 1046.9 |
| 7 | 1037.9 | 1035.8 | 1046.9 |
| 8 | 1037.9 | 1035.8 | 1046.9 |

표를 아래로 늘려 숨긴 것이 아니라 같은 컷의 trailing 간격을 공통 높이에서 제외했다.
첫 조각 끝[27,27], 이후[55,55], [83,83], [103,111], [103,139]로 저장 source 경계를
선택한다. 6·7쪽 debug PNG에서 새 마지막 줄·이어지는 줄과 표 외곽을 직접 확인했다.
4–8쪽 출력에서 기존4쪽 `LAYOUT_OVERFLOW`는 사라졌다. 전체 `dump-pages`는 모든 쪽을
실제 paint한 검사가 아니므로 로그 부재를 문서 전체 무초과로 해석하지 않는다.
아래 기존 테스트의 실제 render 경로에서는25쪽 pi161의18.8px 경고가 여전히 검출됐다.

- [수정 후6쪽 PNG](../../output/7195/stage3/replaced-86712-p4-p8-after/p6-debug.png)
- [수정 후7쪽 PNG](../../output/7195/stage3/replaced-86712-p4-p8-after/p7-debug.png)
- `output/7195/stage3/replaced-86712-p4-p8-after/debug/`에4–8쪽 debug SVG.

### 추가 발견 — #3931은 통과로 보고하지 않음

기존 trailing trim의 인접 계약인 `tests/issue_3931_declared_rowbreak.rs`를 새 native rlib에
직접 링크하여 실행했다(`source-reset-3931-tests.log`). **4 PASS / 1 FAIL**:
저장 geometry, 두 관심 표의 fragment 경계/본문 fit, HWPX 경로는 통과했지만
`issue_3931_keeps_pr4763_hwp_page_count_contract`가 기대384/실제385로 실패했다.
이 직접 실행은 Cargo/nextest 전체 게이트의 대체가 아니며, 정확한 수정 전 native와의
대조를 아직 하지 않았으므로 이번 변경의 회귀인지 기존 실패인지는 **미분류**다.
이 계약이나 baseline을 수정하지 않았다. 이전 WASM으로 보조 대조를 시도했지만
보관된 검증 hash와 현 `pkg/` hash가 달라 실행 전 거부했다. 이전 판정으로 추정하지 않는다.

### Docker WASM과 잔여 세 모듈 재검증

`docker compose --env-file .env.docker run --rm wasm` **성공7m11s**, 로그는
`output/7195/stage3/source-reset-wasm.log`다. 새11,199,519byte WASM SHA-256:
`f01375092ab7ce8c491d30a171395ffd3c7b9ab10eaca29fe63539376f62cbc0`.

`node output/7195/stage3/verify-source-reset-wasm.mjs` 성공:

- Studio7700의 WASM HTTP200 및 파일 hash 일치.
- 새86712 입력64쪽. WASM4–8쪽의 표/마지막 TextLine이 본문 하단 안에 있고, 각 조각의
  원문 소유가 위 native 출력과 일치한다. 76076은82쪽 열기 smoke 통과.
- `replaced-86712-p4-p8-after/wasm-verification.json`, `wasm-p4..p8.json/.svg`에 증적.
  실제 사용자 브라우저의 최종 시각 판정은 아직 대기다.

추가로 `run-remaining-three.mjs source-reset-remaining --normal`을 실행했다.
nextest ID `5851ccbf-cd1e-42c1-9776-0466c30422ef`, **14건 중12 PASS / 2 FAIL**.
#2308 6건, #3798 3건은 모두 통과했다. #2279는3건 통과하고 입력 교체 직후부터 실패했던
두 기존 계약(고정 창의 pitch 표본4개, p11의 잔여 행 문자열)이 동일하게 실패했다.
이 둘을 이번 수정으로 새로 검출한 회귀로 승격하거나 통과로 숨기지 않았다.
해당 결과 및25쪽 pi161의18.8px 경고는 `source-reset-remaining/stderr.log`에 보존했다.

현재 판정: 요청한4–7쪽 경계 수정과 WASM 준비 완료, 작업지시자 시각 검토 대기.
전체 회귀/PR Clippy 게이트는 미실행이며 #3931 쪽수 차이와 위 잔여 계약·25쪽 경고는
별도 정밀 분류가 남아 있다. 기준값/ignore 변경, 커밋, push, 원격 게시 없음.

### 후속 시각 판정과11쪽 조기 분할 조사 (2026-09-16)

작업지시자는 위 수정본의 시각 판정을 통과시켰다. 이후11쪽에도 가용 공간이 있는데
표를 일찍 분할한다고 보고했다. 이 판정은 앞선 수정 범위에 대한 것으로 문서 전체의
미해결 문제나 실패한 테스트 전체를 통과로 바꾸지 않는다.

현재 입력과 native를 다시 조사한 결과,11쪽 대상은 `s0:pi43 ci0`의1×1 RowBreak
TopAndBottom 표다. 기존 실패 계약 `issue_2279_rowbreak_float_splits_without_host_line_segs`는
`pi30` 내용의10/11쪽 존재를 검사한다. 실제11쪽 조기 분할과 그 테스트 실패는 다른 대상이다.
현재 입력의 `pi30`은10쪽에 전체 배치된다. 테스트는 원본 입력을 열며 host LineSeg를
시험 중 삭제하지 않는다. 오래된 주석의 “host 저장 LS 없음”은 현재 입력에 대한 재확인이 필요하다.

- 증적: `output/7195/stage3/replaced-86712-p11/`의 `manifest.json`, `raw.stdout`,
  `pages.stderr`, `tree/render_tree_011.json`, `tree/render_tree_012.json`, `p11-debug.png`.
- 본문 하단1046.9px, 실제 표 하단1020.7px로 약26.2px가 남는다.
- 원본 셀 p[8] 첫 줄은 vpos35572HU, 글줄 높이1500HU(20px), 줄간격752HU(10.0267px)다.
  바로 다음 줄은 vpos0으로 저장되어 있다. 따라서 원본 저장 정보도 이 첫 줄까지 현재
  물리 프레임에 두고, 두 번째 줄부터 다음 프레임에 두는 구성을 나타낸다.
- 실행 로그: 내용 예산502.3px,19유닛 후보504.3px → strict 재-cut에서18유닛으로 후퇴.
  이때 `ㅇ 주민대표단은 구역지정 이후 개별법에 따른 사업시행자 지정 이` 첫 줄이12쪽으로 간다.
- `native_multirow_saved_reset_trailing_trim`의 `table.row_count <= 1` 제외 조건 때문에
  앞서 구현한 저장 프레임 마지막 줄의 trailing 간격 제외가1행 표에는 적용되지 않는다.
  해당 줄간격10.0267px를 제외한 후보는 내용 약494.3px로 예산 안이다. 현 좌표에서
  예상 표 하단은 약1040.7px이며, 이는 계산상 수용 가능성이지 수정 후 실행 증거는 아니다.

판정:11쪽 조기 분할은 실제 높이 계산 결함으로 분류한다. 행 개수가 아닌 유효한 저장
줄 경계와 실제 점유 높이로 trailing 처리의 적용 범위를 정해야 한다. 원본 줄 정보의
재사용과 편집 후 재조판, control 소유 문단 및 물리 본문 경계 보호는 구분해 검증한다.
이번 조사에서는 제품 코드·테스트·baseline을 변경하지 않았다.

### 11쪽 1행 표 수정·집중 검증 (2026-09-16)

작업지시자가 위 원인에 대한 수정을 승인했다. `native_saved_reset_trailing_trim`으로
공통 함수명을 정정하고 행수 제외 조건을 제거했다. native 저장 LineSeg, RowBreak,
TopAndBottom, 비-TAC, control-free 인접 줄, 비합성·비reflow 등 기존 유효성 조건은 유지한다.
행 개수 대신 현재 컷이 유효한 저장 물리 프레임의 끝인지로 판단한다.

실제 호출 경로는 다음과 같다. 이번 수정은 `typeset.rs`에 새 예외를 추가하지 않는다.

| 단계 | 실제 소비 경로 | 이번 대상의 결과 |
| --- | --- | --- |
| 줄 선택/요구 높이 | `advance_row_cut_inner` → `native_saved_reset_trailing_trim` | 내용 예산502.3px에서19유닛의 trailing10.0267px 제외,494.3px 수용 |
| 누적 예약/예산 검증 | typeset 분할 스캔 → `row_cut_content_height` | 동일 컷+유효 상하 padding3.76px =498.1px, 가용506.1px 안 |
| 부족할 때 | `advance_row_cut_within_capacity` → 같은 inner walk | 본문 높이 감소 반례에서는 해당 줄을 다음 조각으로 이월 |
| 실제 표/셀 높이 | table_partial의 `row_cut_content_height`, `cell_cut_visible_height` | 표 하단1040.7px, 글줄 하단1038.8px, 본문 하단1046.9px |
| 이어받기 | 다음 조각 `start_cut=[19]` | p8의 두 번째 줄부터 시작, 문단 원문 한 번만 보존 |

일반 block의 높이 집계도 같은 helper를 소비하지만, 이번 대상은 rowspan 없는 단일 행의
row-cut 경로다. 이 수정으로 모든 rowspan 조합을 새로 검증했다고 주장하지 않는다.
원본 IR/CellUnit의 전체 이송 높이, 빈 줄의 글자 상자, 셀 패딩은 삭제하지 않았다.

- 수정 전: 새 `single_row_source_frame_keeps_its_last_line_without_overflow`를 직전 rlib에
  직접 링크해 **0 PASS / 1 FAIL**, 저장 프레임별 원문 분배가 `[빈 문자열, 문단 전체]`로
  잘못되는 것을 검출했다. `replaced-86712-p11/contract-red.log`에 원인별 실패 보존.
- 수정 후: 원본 저장 프레임의 첫 줄/나머지 줄 소유, 각 조각의 모든 TextLine 및 표의
  Body fit, 원문 중복·누락 방지, 하단 여백+1200HU의 공간 부족 반례 통과.
  직접 링크한 #7195 세 계약도 **3 PASS** (`contract-green.log`).
- 정식 집중 nextest: `2279-p5/validate.mjs single-row-green`, ID
  `63f9f260-4b91-4205-95a0-ab42bfa0dcfe`, **61 PASS / 0 FAIL**, 테스트5.580s.
  RowCut white-box, #7195, #2308, #3128, #5301을 포함한다. source-side1행 경계 반례를
  기존 테스트에 추가했으며 tier **4205 tests/298 modules**와 review suite 정책도 통과했다.
- 동일 입력 총64쪽(전후 동일). 4–8쪽 render tree는 직전 승인본과 byte-identical이다.
  새11·12쪽 debug PNG를 직접 확인하여 마지막 줄과 후속 두 번째 줄의 연결을 대조했다.
  `replaced-86712-p11-after/manifest.json`, `geometry.json`, `debug/`, `tree/`에 증적.
- #3931은 직전 보관된 실행 파일과 현재 rlib의 테스트를 각각 실행했다. 둘 다
  **4 PASS / 1 FAIL**, 쪽수 기대384/실제385로 동일하다. 이번1행 조건 제거로 새로 생긴
  쪽수 차이는 아니다. 이전 절편에서 처음 생겼는지는 여전히 별도 분류 대상이다.
  `replaced-86712-p11/3931-before.log`, `3931-after.log` 참조.

기존 #2279 두 실패 계약이나 baseline/ignore는 변경하지 않았다. 전체 회귀·PR lint 게이트는
이 집중 검증의 범위가 아니다. 새 Docker WASM과 HTTP/출력 대조는 아래 완료 기록으로 구분한다.

추가로 기존 #2279 테스트 소스를 현재 rlib에 직접 링크하여 재실행했다. **3 PASS / 2 FAIL**이며
고정 창 pitch 표본4개와 pi30의11쪽 문자열 부재로 이전과 동일하다(`2279-after.log`).
새 pi43 실물 계약의 성공을 이 두 실패의 해결로 보고하지 않는다.
독립 CLI `cargo build --locked --profile release-test ... -p rhwp --bin rhwp`도3m26s에
성공했다. 재내보내기 SVG의 동일성을 확인하여 먼저 만든 PNG와 실제 CLI 출력의 연결을 보존한다.

Docker WASM 완료: `docker compose --env-file .env.docker run --rm wasm`, **7m52s 성공**.
`replaced-86712-p11/single-row-wasm.log`에 로그를 보존했다.11,199,509byte,
SHA-256 `161810b14b2c421e0290fb89a6cd90551bb97bb5a58e297a7938f3c001d219f9`.
`verify-single-row-wasm.mjs`가 실제 WASM으로4–8·11·12쪽 render tree/SVG를 만들고,
각 관심 표/글줄의 본문 fit과 native 원문 소유 일치를 확인했다.11쪽 첫 줄도 존재한다.
Studio7700이 제공하는 WASM은 HTTP200이고 위 빌드 hash와 일치한다.
`replaced-86712-p11-after/wasm-verification.json`과 `wasm-p*.json/.svg` 참조.

11쪽 수정은 구현·집중 검증·WASM 준비 완료이며 새 시각 판정을 요청한다.
전체 회귀·PR Clippy 묶음은 미실행, 기존 #2279 두 계약 및 #3931 쪽수 차이는 미해결이다.
기준값 변경·커밋·push·원격 게시는 수행하지 않았다.

후속 작업지시자 판정: **“11, 12쪽 해결되었습니다.”** 위 Docker WASM 수정본의
11·12쪽 시각 판정을 통과로 기록한다. 이는 pi43의 조기 분할과 다음 쪽 이어받기에 대한
수용이며, 기존 #2279 두 실패 계약·#3931 쪽수 차이·전체 회귀 통과를 의미하지 않는다.

### 승인 후 남은 #2279 두 계약의 원인 분류

2026-09-16에 현재 교체 입력과 위 수정본으로 조사했다. 이번 절편에서는 제품 코드와
테스트 기대값을 변경하지 않았다. `output/7195/stage3/contract-triage/result.json`에
입력/CLI SHA, 실행 명령과 원문·좌표를 보존했다. 입력 SHA는
`ee82c7755617003cb972ba398da9cffadfed24ac0fa068eee1a5347da7658a88`,
CLI SHA는 `db10e40e87c2e06dfa712e379a54def6e3298dadcf67376ab4d2c2e33265288c`다.

1. `issue_2279_per_line_pitch_uses_line_max_font_size`: **검사의 표본 선택 결함**.
   테스트가 실제 글줄이 아니라 `x>=100px`, `140<=y<=660px`의 TextRun만 수집한다.
   현재 본문 줄은 주로 x≈95.6px에서 시작하며 글자모양으로 run이 나뉜 줄만 일부 선택된다.
   따라서 간격 표본이4개로 줄어 중앙값 검사에 도달하지 못한다. pi20·21·22의 실제
   TextLine18개를 문단별로 추적하면 인접 간격15개 중 첫 줄 다음3개는32px,
   나머지12개는 약29.8667px다. 원본의 첫 줄15pt/본문14pt와 줄간격160% 및 저장
   LineSeg의2400HU/2240HU 간격에 각각 대응한다. 이 실패는 글줄 간격이32px로
   잘못 통일되었다는 증거가 아니다. 좌표 JSON은0.1px 반올림됨을 구분했다.
2. `issue_2279_rowbreak_float_splits_without_host_line_segs`: **교체 입력과 과거 계약의
   전제 불일치**. 테스트는 pi30 host에 저장 LineSeg가 없고 표가10·11쪽에 걸친다고
   가정하지만, 현재 원본에는 `ts=0,vpos=58060,lh=400,spacing=200`인 LineSeg가 있다.
   테스트의 `core()`도 이 정보를 제거하지 않는다. 현재4×3 표의10개 셀과 원문은
   모두10쪽에 있으며, 표 하단1022.8px는 Body 하단1046.9px보다24.1px 위다.
   실패한 `준 준용` 부분도 누락이 아니라10쪽 규제대안2 셀에 있다. 새10쪽 debug PNG를
   직접 확인하여 표 외곽과 해당 셀 내용을 대조했다. 이는 기존 NO_LS 분할 경로가
   검증되었다는 의미도, 한컴과 문서 전체가 일치한다는 의미도 아니다.

두 검사의 기존 조건은 `0fd964974a`의 테스트에도 존재한다. HEAD의 과거 샘플 SHA
`32e2ed30e5d744ad747f04f090c022eca8270f9dd2d55e0613e2ad61058099e9`와 현재 사용자
교체 입력이 다르므로 같은 파일명만으로 과거 페이지/저장 정보 전제를 적용할 수 없다.

후속 계약 정비안은 (a) 문단·글줄 식별에 따라 인접 pitch를 원본 글자 크기/줄간격과
비교하고, (b) 현재 저장 정보가 있는 표의 원문 보존·본문 경계 검증과 원래 NO_LS
분할 회귀 보호를 별도 fixture 또는 전제가 명시된 합성 계약으로 분리하는 것이다.
단순히 표본 수를 낮추거나11쪽 문자열 기대를10쪽으로 바꾸는 것으로 대체하지 않는다.
기존 테스트 결과는 여전히 **3 PASS / 2 FAIL**이며 계약 수정·재실행 전까지 통과로
전환하지 않는다. #3931 쪽수 차이와 전체 회귀는 이번 분류의 범위 밖이다.

증적: `contract-triage/raw-pi20.txt`~`raw-pi22.txt`, `raw-pi30.txt`,
`tree/render_tree_010.json`, `p10-debug.png`, `result.json`.

### 승인 후 두 계약 정비와 집중 재검증

이번 절편은 테스트만 변경한다. root HEAD는 `f0d6966b62fc587a4773b66046a85b76b39cbefd`
위 WIP이며, review base는 `8d45f242baa1a565357aaa38e9f459595b1e756c`로 고정했다.
제품 `typeset.rs` SHA `7ce3afde7388484672a92cef13161d0357fea66dc88682e3176e035cfcf22079`,
`table_layout.rs` SHA `097685e73a9442ae023176f279b865ad4ad371137fc421d219829ea3885da0de`는
직전 시각 승인본과 동일하다. 사용자 교체 샘플도 변경하지 않았다.

- 기존 pitch 검사의 x/y 창, TextRun 간격 필터와 중앙값을 제거했다. 본문 pi20·21·22의
  TextLine을 식별하고 표/글상자 내부의 로컬 문단 번호는 제외한다. 저장 LineSeg와
  15pt/14pt·160%를 대조하여 모든 인접 간격을 검사하며, 세 문단 원문의 중복·누락도 검사한다.
- 현재 pi30 표는 저장 LS 존재를 먼저 확인한 뒤 전체 셀의 행/열/span/원문, 단일 조각
  소유, 표의 본문 fit 및 글줄의 셀 내부 표시를 검사한다. 기존 pi109 계약과 검증기를
  공유한다.11쪽에 특정 문자열을 강제하거나 총64쪽을 기대값으로 삼지 않는다.
- `tests/cases/issue_7195_no_ls_rowbreak_contract.rs`를 추가했다. 빈 문서의 스타일을
  재사용한 합성 IR이며 원본 스트림·봉인은 제거했다. 다른 문단의 LS 유무 × 본문
  240px/480px의 네 조합이다. host는 NO_LS이고 session-edited 우회가 아님을 확인한다.
  표는4행, 각 행에3개의10pt 문단이며 선언192px와 측정224px를 구분한다.
  작은 쪽은 첫 쪽 일부 수용과 이어받기, 큰 쪽은 전체 fit을 검사하고 표 조각의
  본문 경계·모든 원문 한 번 출력·앞뒤 텍스트 순서도 보호한다.

합성 실행 추적에서 `saved=None`, 선언192px, 가용240px가 확인됐다. 인접 LS가 없을 때
current92.7px/잔여147.3px, 있을 때84.7px/잔여155.3px였으며 둘 다 fragment scanner에
진입했다.480px 반례에서는 전체 fit했다. 이는 해당 규칙의 합성 검증이며 한컴 실물 시각
판정을 대체하지 않는다. 과거 `st.has_stored_line_segs` 조건을 되살린 제품 mutant의
실패 실행은 아직 하지 않았으므로 그 결함에 대한 새 red/green 증거로 주장하지 않는다.

검사 정비 전 동일 제품/입력의 기존 #2279는 **3 PASS / 2 FAIL**
(`replaced-86712-p11/2279-after.log`), 정비 후 직접 링크 검사는 **5 PASS / 0 FAIL**이다.
추가 합성1건(내부4조합)도 통과했다. 직접 실행 결과와 분기 로그는
`contract-triage/direct-validation.json`, `2279-updated.stdout`, `no-ls.stdout/.stderr`에 보존했다.
테스트 정비의 전후이지 제품 결함 수정의 red/green으로 해석하지 않는다.

root/review의 수정 테스트 두 파일이 byte-identical임을 확인했다. review에서 suite를
준비한 뒤 base 고정 manifest 검사는 **1343 sources / 5796 attrs / 48 targets 통과**다.
변경 파일 rustfmt와 `git diff --check`도 통과했다. 정식 관련 suite 실행 결과는 아래 기록한다.

정식 nextest 집중 검증은 **23 PASS / 0 FAIL**, 반복 실행도 **23 PASS / 0 FAIL**이다.
6개 suite에서 #2279 5건, #2308 7건, #3798 3건, #7195 8건을 실행했다.
ignore 우회 옵션은 사용하지 않았다.1215건은 집중 필터 밖이며 전체 회귀를 실행한 것은 아니다.

- 최초: `contract-triage/validate.mjs`, 빌드3m34s/실행4.059s,
  run ID `6aed0597-b40d-4867-ac22-65ef651c6096`.
- 반복: `contract-triage/validate.mjs focused-repeat`, 빌드0.15s/실행4.062s,
  run ID `ba41fbf4-1951-4b5c-ac41-7ecd8f471e6b`.
- 정확한 Cargo 명령, 입력/제품/테스트 SHA, 시작·종료 시각과 결과는
  `contract-triage/focused.json` 및 `focused-repeat.json`, 전체 로그는 각 `.stderr`에 있다.
- nextest0.9.137은 저장소 권장0.9.140보다 낮으며 `ci-duration-observation` 프로필의
  `junit.report-skipped` 키를 무시한다는 경고가 있었다. 이번 실행은 default 프로필이고
  테스트 자체는 위23건 실행·성공했다. 도구를 임의 업그레이드하지 않았다.
- review manifest의 base 고정 검사도 검증 후 다시 통과했다. 파생 suite는 review에만
  있으며 source PR 대상으로 stage하지 않았다.

따라서 이번에 정비한 두 #2279 계약과 대상 #2308/#3798 집중 검사는 통과다.
기존 #3931 기대384/실제385 차이, 전체 회귀, PR 직전 Clippy 묶음은 남아 있다.
제품 코드가 바뀌지 않았으므로 Docker WASM 재빌드는 하지 않았고, 기존11·12쪽 시각
승인본을 유지한다. 기준값·ignore 완화, 커밋·push·원격 게시도 하지 않았다.

### 추가 승인: #3931 쪽수 검사 1건 ignore

2026-09-16 작업지시자의 “이번 타스크에서는 이 실패는 ignore 처리” 승인에 따라
`issue_3931_keeps_pr4763_hwp_page_count_contract` 한 건만 명시적으로 제외했다.
기대384/실제385의 차이는 기존 [#7009](https://github.com/edwardkim/rhwp/issues/7009)에서
후속 추적한다. 같은 샘플에 대한 이슈임은 확인했으나 현재385쪽의 원인까지 동일하다고
확정한 것은 아니다. 기대값384는 유지하며 분류·수용 기준 확정 후 ignore를 해제한다.
표 분할·저장 줄 geometry·HWPX 경로 등 같은 파일의 나머지4건은 제외하지 않았다.

review에서 파생 suite를 다시 준비하고 base 고정 manifest 검사를 통과했다
(1343 sources / 5796 attrs / 48 targets). ignore에 따른 재배분으로 해당 파일이
suite005에서002로 이동했다. 이전005 대상 재실행은 0건/exit4였으므로 성공 증거에서
제외하고, 현재 모듈 위치를 자동 탐색하도록 검증 스크립트를 고친 뒤 다시 실행했다.

- 최종 집중 결과: **4 PASS / 해당 쪽수 검사 1 IGNORE**. 나머지 필터 밖 검사는 미실행.
- run ID: `10a89e59-3933-4086-bc63-3cd49ef8f83b`, 빌드7.26s/검사1.481s.
- 명령·제품/테스트 SHA·시각: `output/7195/stage3/contract-triage/3931-ignore.json`.
- 전체 로그: 같은 경로의 `3931-ignore.stderr`; 재실행 스크립트: `verify-3931-ignore.mjs`.
- nextest 버전/설정 경고는 앞서 기록한 것과 같으며 실행한4건은 모두 성공했다.

제품 코드·기대 페이지 수·다른 래칫은 변경하지 않았다. 전체 회귀 및 PR 직전 lint
검증은 이번 집중 검사의 결과로 대체하지 않는다. 커밋·push·원격 게시도 수행하지 않았다.

### 종료 범위 조정: 후속 이슈로 분리

작업지시자가 작업 장기화를 피하기 위해 잔여 실패를 별건 이슈로 나누어 처리하도록 승인했다.
제품 코드와 테스트를 더 변경하지 않고 다음과 같이 범위를 확정했다.

| 잔여 대상 | 추적 | 현재 처리 |
| --- | --- | --- |
| #3931 HWP 쪽수384/385 | [#7009](https://github.com/edwardkim/rhwp/issues/7009) OPEN 재확인 | 기존 승인대로1건ignore, 나머지4건 유지 |
| 76076 크기14→13→14pt 재설정 전후 누락 차이 | [#7202](https://github.com/edwardkim/rhwp/issues/7202) OPEN 재확인 | 기존 별건 유지, 중복 등록 없음 |
| 교체86712 물리25쪽 pi161 본문 하단+18.8px 경고 | [#7204](https://github.com/edwardkim/rhwp/issues/7204) 신규 등록 | 별건 조사. 테스트 실패가 아닌 진단 경고이며 가시 초과/회귀 여부 미분류 |

신규 등록 전 `86712`, `pi161`, `18.8`로 검색했다. 닫힌 #2237/#5705는 주로pi172의
관련 이력이라 동일 원인으로 묶지 않았다. #7204에는 교체 입력과 제품 파일 해시,
미커밋 코드 포함 사실, 로그·물리 쪽 식별, 재현 및 완료 조건을 남겼다.
정확한 게시 본문은 `output/7195/stage3/contract-triage/followup-p25-issue.md`다.

초기 통과2건과 정비 후 통과3건은 상시 검사를 유지한다. 이전부터 범위 밖인 ignored 검사는
그대로 두며 전체 제외 수는 실제 최종 목록 없이 계산해 보고하지 않는다. 이번 절편에서
새ignore, baseline변경, 전체 회귀 재실행, 커밋/push/PR/close는 하지 않았다.
최종 전체 검증에서 추가 실패가 나오면 개별 원인 분류와 후속 이슈를 기록하고,
현재 작업의 구현 범위를 계속 확장하지 않는다. 미실행/보류를PASS로 기록하지 않는다.
