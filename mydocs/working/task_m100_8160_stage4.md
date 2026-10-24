# #8160 단계 4 — 전 페이지 회귀와 기존 기대값 정정

## 착수 조건과 권한

- 브랜치: `task/8160-sample16-raw100`. 선행 증거 commit `d311cb14b`.
- source `803046519`의 Native/fresh 표준 WASM 384/384쪽 raw 100%, 양방향 정수 미대응 0을
  먼저 확정했다. 증거는 `mydocs/pr/assets/issue_8160/candidate33-all-backends.json`이다.
- 구현·로컬 단계별 커밋·필수 검증은 계속한다. 원격 push·PR 생성은 결과를 정리해 사용자
  승인을 받은 뒤 진행한다.

## 기존 검사 감사와 정정 순서

1. BCP: 기존 `issue_7642_sample16_2022_paragraph_relations`의 한 줄 기대값은 원본 두
   LineSeg(0/64)·새 Hancom 2020 Print 두 줄과 다르다. 예외는 이미 `f0f434b8f`에서 제거했다.
   기존 함수를 두 줄의 내용·문단 소유·후속 문단·비겹침으로 정정한다. 별도 중복 검사를 만들지 않는다.
2. 글자 폭: 기존 세 단위 검사는 `fixed_glyph_advances=false`인 default를 쓰면서 명시적인
   잉크 폭 맞춤을 기대한다. 명시적인 true 사례의 기존 음수 자간·배분 간격 보호를 유지하고,
   요청하지 않은 일반 잉크 보존은 별도 의미 검사로 확인한다.
3. 전 페이지: 새 독립 Print에서 내용과 기준선 좌표를 직접 추출한다. 실제 원본의 문단·셀
   경로는 주소로 연결하며 현재 출력의 bbox를 독립 기대값으로 복사하지 않는다.
   각 문서 64쪽 모두를 내용·소유·순서·중복·좌표의 실제 실행 대상으로 등록한다.
4. `verified_stored_table_picture_rows`의 HWPX 18쪽 보호는 전 페이지 검사와 병합해
   원본 표·그림·표 세 컨트롤의 소유·순서·셀 주소 계약을 그대로 보존한다.
5. `verified_sample16_2024_print_boundaries` 및 단 첫 여백 검사는 입력이 구형
   `hwp3-sample16-hwp5.hwp`다. 이번 별도 `-2024.hwp`와 바이트가 달라 중복으로 삭제하지 않는다.
   parser/save-version 계약과 일반 corpus baseline도 다른 의미를 보호하므로 유지한다.

## 검증 상태

기존 잘못된 기대값의 실제 실패와 정정 뒤 통과, 새 검사별 독립 기대값·수정 전 결함 검출,
최종 lint·전체 nextest·Native Skia·브라우저 검증 결과를 실행 후 기록한다.

## BCP 기대값 정정의 실제 실행

동일한 동결 Native rlib `803046519`에 기존/정정 후 정식 case source를 각각 `rustc --test`로
링크했다. 기존 소스는 BCP `len=2`에 기대값 1로 실패(exit 101, 다른 표 검사는 PASS)했다.
정정 후에는 두 검사 모두 PASS(exit 0, 2.59초)다. 실행 환경은 Linux이며 같은 원본·Print와
전쪽 384개 통과 뒤에 검사만 정정했다. 이 focused 진단은 이후 정식 generated suite·nextest
검증을 대신하지 않는다.

BCP 함수는 원본 LineSeg 문자 경계 `[0, 64]`, 두 줄의 전체 내용 순서·누락·중복, 두 번째 줄
`립`, 각 런의 본문 소유와 후속 문단의 내용·비겹침을 보호한다. 2022 Print 모드도 명시한다.
기존 잘못된 한 줄 함수를 교체해 같은 증상을 검사하는 중복 함수를 추가하지 않았다.

## 전 페이지 관계 검사와 시각 회귀 통합

세 문서 192쪽의 독립 문구 572곳을 새 Print에서 선정했다. 실제 PDF 기준선 ±2px,
쪽의 단일 원문 소유·내용·누락·중복, 원본 문자 범위와 순서·가로 포함을 보호한다.
글상자와 표의 로컬 문단 번호를 본문 주소로 혼동하지 않으며 원본 컨테이너 경로를 다시
해소한다. 같은 주소·문자 시작을 다른 쪽에 중복 배치해도 실패한다. 첫 실행은 동결
Native rlib로 세 검사 모두 PASS, 192쪽, 2.04초다. 정식 nextest 실행은 별도로 남아 있다.

기존 `verified_stored_table_picture_rows`를 공유 helper로 옮겨 새 세 검사에서 실행한다.
기존 소유·내용·셀 주소·앞뒤 비겹침 assertion은 보존하며 HWPX 외 두 HWP에도 적용한다.
구형 HWP를 사용하는 기존 2024 경계·단 첫 여백 검사와 parser/save-version/corpus 보호는
다른 입력 또는 의미 계약이므로 유지한다.

전쪽 실제 paint는 새 `verify_sample16_raw100.py`가 384개 PNG를 정수 비교한다.
384/384 PASS(exit 0)를 다시 확인했으며, 수정 전 candidate31 WASM은 2022 18쪽 15/6,
35쪽 0/67을 검출해 382/384 FAIL(exit 1)했다. 같은 Native33은 대조군이다.
이 결과는 실제 구버전 출력의 결함 검출이며 환경·컴파일 실패를 결함 검출로 세지 않는다.

기존 lib test 빌드에서는 이번 스타일 확장의 `isolated_jamo_font` 초기화가 여섯 곳에
누락돼 컴파일 실패했다. 해당 단위 입력은 선택 글꼴이 없는 대조군이므로 None을 명시해
보완했다. 이 실패를 글자 폭 기대값의 실제 FAIL로 보고하지 않는다. 기존 세 기대값의
런타임 재실행을 진행 중이며 이후 명시 요청/일반 잉크 계약을 정정한다.

## 글자 폭 기대값 정정과 source-side 개수 정책

초기화 누락을 보완한 실제 기존 lib 검사에서는 3개 중 1 PASS·2 FAIL을 확인했다.
실패는 default `fixed_glyph_advances=false`의 결과 None과 과거 기대값 Some(0.5)/Some(1.0)의
불일치다. 세 기존 검사에 명시적인 폭 맞춤 요청 true를 넣어 음수 자간 보호와 양수 배분
간격 제외·음수 간격 보존의 기존 의미를 유지했다.

일반 잉크 보존의 추가 4조건×ASCII pin 2종도 실제 lib harness에서 PASS했다.
중간 후보의 4개 검사 모두 PASS(exit 0, 빌드 9분 31초, 실행 0.053초)다.
source-side 테스트 총량 증가 금지 정책에 따라 새 `#[test]`는 남기지 않고 해당 내용을
기존 폭 맞춤 검사가 호출하는 helper로 통합했다. 정책 재실행은 4,101개·296개 모듈로
exit 0이다. 최종 3개 함수 구성의 정식 실행은 최신 devel 반영 뒤 전체 nextest에서 확인한다.
중간 4개 harness 결과를 최종 3개 구성의 실행 결과로 표기하지 않는다.

CanvasKit 준비 보정까지 포함한 Studio 전체 Node 재실행은 exit 0, 2,363개 중
2,361 PASS·0 FAIL·2 SKIP, 92.369초다. SKIP은 기존 Node 전용 pkg-node 부재의 두 왕복
검사이며 실제 fresh web WASM 전쪽 출력과 구분한다.

## 최신 devel #8181 반영

추가 base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`의 패키징 PR #8181을 반영한다.
기존 단계별 commit과 증거 SHA를 보존해 merge하며, Cargo workspace exclude 충돌은
이 작업의 JPEG encoder fork와 upstream의 직접 SVG fork를 모두 제외하는 합집합으로 해소한다.
두 의존성·lock의 자동 병합을 보존한다. 실제 기능 보존과 최종 전쪽 검증은 merge 뒤 실행한다.
#8181의 표준 Rust Error, CLI 직접 core 로드, dev-tools generator와 package source 사전 검사를
현재 후보에서 확인하며 패키징 기능을 삭제하거나 이전 root patch를 복원하지 않는다.

### 패키징 기능 보존에서 검출한 누락

merge `b0f745578`의 locked metadata는 통과했으나 실제 source preflight는
`local dependency outside candidate graph: .: vendor/rhwp-jpeg-encoder`로 실패했다.
원본 #8181의 후보 그래프에 이번 인코더 fork가 추가되지 않은 것이 원인이다.
제품 버전·실제 게시 권한을 바꾸지 않고 fork 이름·정확 버전·비게시·include/라이선스와
9개 source 후보 그래프를 맞춘다. 원본 코드·저작권과 upstream checksum은 그대로 보존한다.
Rust import alias와 인코딩 구현은 바꾸지 않는다. 보정 후 locked metadata·source 사전 검사와
필수 기능 검증을 다시 실행한다.

locked offline metadata는 exit 0이며 lock 변경은 JPEG encoder의 이름·버전과 root 참조만
10줄이다. source preflight의 단위 검사 18개도 PASS(exit 0, 2.599초)다. commit 고정 뒤 실제
9개 archive 사전 검사를 실행한다.

### 사용자 지시에 따른 리베이스

작업지시자의 최신 지시는 `upstream/devel` 리베이스 후 진행이다. 위 merge 실행 기록은
당시 사실로 보존하고, 현재 패키징 보완을 일반 commit으로 고정한 뒤 최신 devel 위로
작업 브랜치를 리베이스한다. 원격 push는 아직 없으며 rebase 전후 SHA·충돌 해소·기능 보존과
최종 전쪽 검증을 새 head 기준으로 기록한다. 기존 source/산출물 해시를 새 SHA로 덮어쓰지 않는다.

## 리베이스 완료와 최종 검증 기준 고정

사용자 지시대로 최신 `upstream/devel` `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` 위로 리베이스했다.
전 head `67c27cb3b97404d421c8b7431257104b11b4f4dd`, 후 head `2cd7f5dde2ebbd2e6dd1cc7d9dc28663bf0cded5`다.
전체 파일 tree hash는 양쪽 `b08f7a24e4db6ab5da77738d5ba8450f8a7e7543`로 완전히 같아 구현·입력·증적·계획 누락이 없다.
Cargo exclude와 단계 기록의 충돌은 기존 두 fork 및 모든 기록을 보존해 해소했다.
`rebase-d9fd85ed5.json`에 21개 commit의 이전/새 SHA와 로컬 보존 branch를 연결했다.
작업 트리는 clean이며 이 상태에서 final 검증을 다시 시작한다.

패키징 보완 전 실패를 실제 보완한 뒤 commit `67c27cb3b`에서 9개 source archive의
preflight가 exit 0이었다. 비게시·미래 제품 버전 blocker는 기존 정책의 의도된 상태이며
실제 registry 게시를 승인하거나 수행한 결과로 표현하지 않는다. 새 head에서도 재확인한다.

두 family·정역순 글자 정상 대조군은 실제 원본 TTC index 1·3을 web 등록 API로 공급한
fresh 표준 WASM33에서도 두 쪽 글자 영역 양방향 미대응 0이다. Native API의 같은 결과와
함께 기록하며 기존 삼각형 stroke 차이가 있는 전체 두 쪽을 raw 100%로 주장하지 않는다.
원격 push·PR 생성은 최종 결과를 정리하고 승인받은 뒤 진행한다.

### generated suite 준비에서 검출한 target 예산 보정

리베이스 후 review worktree에서 실제 `--prepare`는 49 > 48 target 예산으로 실패했다.
새 전쪽 case의 `#[path] mod`가 별도 예외 target으로 분류된 것이 원인이다.
공유 helper를 전쪽 case 내부의 일반 함수로 합쳐 자동 suite 통합을 사용한다.
기존 18쪽 assertion·세 문서 실행은 유지하며 target 예산이나 정책을 높이지 않는다.

통합 후 같은 동결 실제 엔진으로 세 전쪽 검사를 다시 실행해 모두 PASS(exit 0, 1.50초)다.
전쪽 case는 343줄이며 18쪽 보호의 모든 기존 assertion을 내부 helper로 보존했다.
정식 suite 준비·실행은 새 commit을 review worktree에 반영한 뒤 다시 확인한다.

## 중요 영역의 px 좌표 보강

사용자 지시에 따라 전 192쪽의 첫 독립 내용 앵커에 가로 원점 ±2px를 추가했다.
기존 572개 문구·기준선·원본 주소는 그대로 보존했다. 새 Print의 72 DPI 좌표를
96 DPI로 변환하며, 렌더 결과나 현재 bbox를 기대 좌표로 복사하지 않았다.

6쪽 평가표, 18쪽 제목 표·WMF·다음 제목 표, 35쪽 구성 표·WMF, 38쪽 운영 현황표,
58쪽 인력 서식은 세 저장본 합계 셀 624개·그림 6개의 네 변을 각각 ±2px로 검사한다.
같은 원본 표·셀·그림의 단일 소유, 셀 행·열·병합 범위도 확인한다. PDF의 실제 cell/WMF
외곽 clip과 원본 작성 순서를 대응했으며 반복 인쇄의 동일 clip은 한 번만 사용했다.
38쪽 꼬리말은 내용·단일 소유·가로 원점·기준선도 보호한다.

6쪽 두 산식은 원본 제어·스크립트와 첫 가시 글자의 가로 원점, 분수선 높이·길이를
±2px로 확인한다. 원본 수식 글꼴을 공급하지 않는 일반 Rust 환경에서 내부 글자 폭에 따른
분수선 가로 끝점은 Print와 최대 약 2.5px 다르므로 그 값을 일반 환경의 고정 좌표로
등록하지 않았다. 실제 원본 글꼴을 공급한 Native/fresh WASM의 모든 픽셀은 기존 전쪽
raw100 검사로 확인한다. 이 구분을 가로 끝점까지 검사했다는 주장으로 확대하지 않는다.

BCP는 기존 정정한 검사 안에서 두 줄의 독립 가로 원점·기준선·기준선 간격 ±2px를 추가했다.
같은 의미의 새 `#[test]`를 만들지 않았다. 직접 동결 Native33 rlib로 보강한 전쪽 검사
3/3 PASS(exit 0, 1.43초), BCP와 기존 표 관계 2/2 PASS(exit 0, 0.89초)를 확인했다.
최종 commit의 generated suite·lint·nextest 실행은 별도로 진행한다.

`tests/fixtures/issue8160/{hwpx,hwp2024,hwp2022}.json`은 `include_str!`로 직접 읽는
정식 회귀 기대값이며 검사 코드와 함께 유지한다. 좌표 추출용 임시 Python 도구는
`output/pr-review/issue8160-20261022/font-diagnostics/`로 옮겼고 커밋하지 않는다.
직접 실행 로그·중간 PDF clip 목록·진단 소스도 같은 ignored 작업 영역에만 둔다.

### 독립 Skia 도구의 lock 보존

#8181의 독립 workspace인 `tools/skia-surface-probe`에서 실제 locked offline metadata가
exit 101이었다. root에 추가한 JPEG fork가 도구의 별도 lock에 빠진 것이 원인이다.
Cargo로 fork 한 개와 root 참조만 추가했고, 다른 의존성 버전은 바뀌지 않았다(5줄 추가).
보정 후 같은 locked offline metadata는 exit 0이다.

`VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json`, 공용 `target/pr-review`,
`CARGO_BUILD_JOBS=2`로 실제 `cargo run --locked --manifest-path tools/skia-surface-probe/Cargo.toml`을
실행해 exit 0을 확인했다. 빌드는 5분 7초이며 llvmpipe Vulkan, scale 1.25에서 내부색·host
state 복원·잘못된 인자의 pixel 보존 세 계약이 모두 PASS다. 실제 GPU 가속의 증거로
확대하지 않는다(`hardwareAccelerationVerified=false`). 로그는 ignored
`output/pr-review/issue8160-20261022/logs/skia-probe-host.log`에 있다.


## 전체 실행에서 확인한 기존 Poppy 기대값의 정정

최종 source `5f7e7e7f00827a2cf3fc5dffbc4e8be89166b51e`의 전체 nextest에서
`hci_poppy_unmeasured_ascii_and_unverified_styles_keep_the_display_metric`가 실패했다.
20pt 굵은 `O/O`의 공개 문자 경계는 원본 경로 `[0, 22.2, 30.0, 52.2]`,
표시용 Palatino 경로 `[0, 22.2, 30.1, 52.4]`인데 기존 검사는 두 경로의 완전 동일을 요구했다.

이 검사는 도입 commit `a7c518ace2888f414571cb17329aaeb82010ffd6`에서 추가됐으며,
PR #8064의 실제 merge `9b01d5d1447220b3a7345703e79ad42cf3f502f4`에 포함됐다.
GitHub commit 연결과 merged 상태, 도입 → merge → 현재 base `d9fd85ed5`의 ancestry를
확인했다. 당시의 미확인 굵은체 fallback 계약은 유효했지만 이번 독립 증거로 적용 범위가 바뀌었다.

이번 작업의 `poppy-original-widths.json`은 원본 HFT em=512의 일반체 `O=402`,
`!=142`, 굵은체 `O=426`, `/=151`, `!=142`를 보존한다.
새 MCP의 별도 ASCII 표본(job `5ea451f8-4b75-4816-84ac-b51474acaf75`)에서
12·16·20·32·52pt의 실제 전진과 교차 확인한 근거다. Type3 윤곽 Widths를 굵은체
실제 전진으로 혼동하지 않는다. 원본 HFT와 독립 Print의 해시는 기존 자산에 고정돼 있다.

사용자의 잘못된 회귀 검사 수정 지시에 따라 기존 함수 안에서 일반체·굵은체의 12·20·52pt
문자 경계를 위 독립 전진으로 확인하도록 정정한다. 허용 정밀도는 공개 charX의 0.1px
직렬화와 원본 1800dpi 격자를 합한 0.11px다. 미확인 일반/굵은 기울임체의 기존 표시 face
동일성은 유지한다. test 수를 늘리거나 제품 메트릭을 대체 face로 되돌리지 않는다.
수정한 검사의 실제 재실행과 최종 전쪽 Native/fresh WASM 검증은 이어서 수행한다.

사용자의 최신 원격 지시는 세 검토 문서의 전쪽 TSV를 다시 검증해 모두 100%이면
PR 및 후속 처리까지 완료하는 것이다. 이 조건과 정본의 필수 검증을 충족하면 이미 받은
승인 범위에서 push·Open PR·정확한 head CI·문서 trailing·병합·후속 처리를 진행한다.
앞 절의 별도 승인 대기 기록은 당시의 지시를 보존한 것이며 최신 실행 조건은 이 절을 따른다.

### 전체 실행의 8개 실패와 직렬화·출력 계약 정정

`5f7e7e7f0`의 전체 nextest(run `8383ccdc-8184-4c77-8dff-5ba47126f320`)는
11,365개 실행 중 11,357 PASS, 8 FAIL, 45 skipped로 끝났다. 컴파일 11분 50초,
실제 테스트 2,051.773초이며 종료 코드는 100이다. 실패를 성공으로 재분류하지 않는다.

- Poppy 1개는 위 절의 미확인 굵은체 가정이다.
- `issue_1116` 1개, `issue_5843_tab_dot_leader_pitch` 2개,
  `issue_7292_tab_leader_head_gap` 1개는 dash `<line>`만 찾으므로 개별 `<circle>`을
  출력한 실제 점선 리더를 0개로 해석했다. pitch/diameter 두 검사는 빈 목록으로
  통과하는 문제도 있어 실제 리더 존재와 같은 줄의 균일 간격을 함께 확인한다.
- 예약 팔레트 2개와 visible-crop 팔레트 1개는 배치별 JPEG의 원본 RGBA 완전 동일성,
  또는 배치별 payload에도 BinData source key가 항상 있다는 가정이 잘못됐다.

리더의 선행 해결은 `f341567397c60064c9e0c0586cce68c9569f44a9` → PR #5853의
merge `0f9ceeb19a7f36efebe77f03a4c7d39fb3f8c83f`, 앞 간격은
`263ec361330e9888d0cb914be6c17e2eb614c5d8` → PR #7310의
merge `4c4a3c69f1db5dba8fb95fd30529e97025f99886`다. 실제 GitHub merged 상태와
commit 연결을 확인했다. 공통 SVG 판독기를 세 검사에 공유하며 개별 원과 기존 dash 선을
함께 읽는다. 정책 예외나 target을 늘리지 않고 절대 manifest 경로의 `include!`로 재사용한다.

KTX 2쪽의 첫 점은 독립 PDF의 T3 Encoding code00 → HFT1, CharProc xref24에서
확인했다. 원점 164.28pt, 크기 15pt, glyph 윤곽의 x 중심 192/1000em이므로
실제 점 중심은 222.88px이다. 기존 219.07px는 glyph advance 원점이며 실제 원의
왼쪽/중심과 같은 값이 아니다. 현재 실제 SVG 원 중심 222.96px를 같은 물리 좌표로 비교한다.

예약 팔레트 도입 `87af62e58ae3c68ca28747bb745c773f3b76d654`와 visible-crop 도입
`eef3c95eba09bff5574ceb257eb9ee37ed5d2529`는 PR #8064의 실제 merge
`9b01d5d1447220b3a7345703e79ad42cf3f502f4`에 포함됐다. 이후 #8120
`80968497c2c178eeecb3b2e1d04d086ad27a0dc7`의 정식 test 경로 이동도 보존한다.

예약 팔레트의 기존 독립 입력 2개를 새 MCP의 실제 Hancom 2020·2024 Print 0으로
각각 출력했다. `pdf/issue8160/bitmap-{white-palette,indexed-color}-controls-{2020,2024}.pdf`
네 파일은 임시 PDF가 아닌 정식 회귀 기준이다. 입력/PDF SHA·job·실제 제품 버전·인쇄 경로는
`palette-reference-refresh.json`에 보존한다. 한 쪽씩이며 흰색 팔레트는 그림 6개,
indexed/RGB는 12개다. 모든 그림은 독립 PDF에서 JPEG로 저장되어 있었다.

수정한 검사는 독립 PDF를 직접 읽어 paint 순서의 전체 JPEG 픽셀을 출력 payload와 비교한다.
과거 계조 두 값만 고르는 대신 실제 전체 색상과 알파·해상도·소유·순서·내용을 확인한다.
Screen의 원본 RGBA와 일반 source-key 조회 보호는 유지한다. 배치별 OutputDevice 바이트는
원본 키로 생략하지 않고 inline/omit 요청의 실제 바이트가 동일하게 보존되는지 검사한다.

동일 제품 코드의 동결 source `803046519` Native/표준 WASM 대조군 12쪽을 측정했다.
예약 팔레트 8개 결과와 기존 기본 HWP5 목차 2개 결과는 raw100이다. KTX 목차는
Native 99.70954%, WASM 99.68345%다. 이는 최종 제출 빌드의 결과로 바꾸어 적지 않는다.
최종 코드의 검사 실행과 fresh 출력 대조는 이어서 완료한다.

### 정정한 검사의 실제 실행

여섯 case의 기존 함수 17개를 모두 선택해 실제 generated suite에서 재실행했다.
`--prepare`로 재수집한 뒤 case별 예상 개수(2·6·4·1·2·2)와 실제 개수가 모두 같았으며
17/17 PASS, exit 0이다. 컴파일 포함 97.207초, 실제 실행 1.465초다.
중간의 미완성 타입 오류와 갱신 전 manifest로 10개만 선택한 실행은 최종 PASS에서 제외했다.
테스트 내용 해시·실행 명령·수정 전 실패와 실제 시각 대조군은
`regression-expectation-corrections.json`에 기록한다.

KTX 목차의 점 간격·첫 점·숫자와 앞뒤 내용 및 indexed/RGB 팔레트의 실제 Native/PDF/WASM
동일 배율 비교 PNG도 열어 직접 판독했다. 점선이나 그림의 누락·겹침 없이 순서와 색이 보존됐다.
이 단계는 제품 코드 변경 없이 잘못된 기존 회귀 계약을 정정한 결과다.
최종 commit의 전체 Rust 검증·Skia·최적화 fresh WASM·전체 384쪽 재산출과 PR은 이어서 수행한다.
