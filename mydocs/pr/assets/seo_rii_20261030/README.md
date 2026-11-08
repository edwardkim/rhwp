# seo-rii Native PR 통합 검증

## 검증 상태

로컬 필수 검증 완료(2026-10-31). 기준 `96ccd557592a711f215e7ad28822f4c7a3f00c15`, 통합 제품 소스
`c3f5cbd6d7beecdaa231cd3a40080f2e7f3c8c1e`. 이후 테스트 첫머리 주석은 입력·계약과
시각 증거의 관계를 설명하며 제품 코드나 assertion은 바꾸지 않는다.

[입력·기준 PDF](inputs.json)는 기존 저장소 자원을 그대로 사용한다. PDF의 해당 한컴 세대는
본문·글꼴 대조군 2020→compat 2022, 이미지 crop 2022, 편람 2024다. 임시 실행·전체 PNG는
`output/pr-review/seo-rii-20261030/`에서 만들고 최종 근거만 이 경로에 보존한다.
개인 글꼴 프로그램은 커밋하지 않는다.

## 원인과 실제 소비 경로

| PR | 생산 → 실제 소비 | 검토한 계약 |
| --- | --- | --- |
| #8270 | `NativeDocumentSession::render_region` → `render_region_cached` → `PreparedSkiaPage::plan_region/render_region` → RGBA frame | revision/font generation/page/scale 키, 최대 4개 LRU, 전체 retained byte 한도와 요청 peak를 할당 전에 확인. 편집·undo·IME·font 교체 무효화 |
| #8275 | classifier와 trusted CI workflow → `run-rust-test.mjs` → Native cfg suite | region cache/planner/host surface가 release와 release-test에서 실행되며 최신 devel의 glyph·이미지 검사를 유지 |
| #8276 | `SkiaTilePlan`/공통 `SkiaRegionPlan::for_tiles` → 고정 512px 격자와 2px guard → `render_prepared_raster` → 실제 겹침 영역 copy | viewport와 무관한 tile origin/clip, 한 번에 최대 516²×4 working bitmap. full-page exact crop와 별도 raster 계약 |
| #8279 | `renderer::arrow::{line_heads,connector_heads}` → `validate_direct_pdf_tree` → 기존 Skia Line/Path replay → PDF canvas | 요청한 화살촉 수, 유한 scalar/outline, 몸통·endpoint metadata·cubic tangent 검사. geometry 생산을 PDF에서 재구현하지 않음 |

문단·줄 소속, 측정/배치, pagination·rowspan·이어받기·저장 LineSeg는 변경하지 않는다.
네이티브 모듈은 wasm32에서 제외되며 PDF admission도 native-skia+non-wasm cfg다.
기준 devel의 #7697/#7688 공통 화살촉, #7964의 기존 Direct PDF 제한, #8389의 편람 글꼴·이미지,
#8393의 caption 통합을 확인했다. 새 CI 호출은 Native cfg가 default suite에서 실행되지 않는 범위를
보완하며 기존 호출을 중복시키지 않는다.

## 합성 계약과 실물 증거 구분

- `issue_536_native_region_cache`: 합성 도형·빈 문서의 exact crop, 타일 overlap, LRU/예산/무효화.
- `skia_region_plan`: 합성 페이지 크기와 요청의 격자·stride·바이트 예산, 실제 버퍼와의 일치.
- `issue_536_native_session`: new_blank 저장·재열기, 소유 수명·revision·feature-off 오류.
- `render_p37_direct_pdf_export`: re-03-latin 실제 입력의 API/PDF 구조와 합성 이미지·화살촉
  vector stream 계약. 합성 vector 좌표의 기대값은 #7688 독립 Print의 화살촉 크기·끝점과 PDF CTM에서
  정하며 현재 arrow helper의 반환값을 기대값으로 부르지 않는다.

위 합성 assertion에는 다른 문서의 raw 수치를 붙이지 않는다. 실물 Visual Sweep은 아래 별도 검증이다.
`consumer_probe.rs`는 네 실물 문서를 scale 0.75/1/1.25로 실행하는 검토 도구이며 자동 suite는 아니다.
두 페이지 cache 왕복은 각 실물 입력의 section을 메모리에서 복제한 진단 입력이다. 정상 한컴의
실제 두 페이지 문서로 오인하지 않는다. 편람은 원본 383쪽을 그대로 사용한다.

## 현재 확보한 결과

- 집중 Native 81건, Native lib 4,040건(ignored 12), placeholder 2건 통과.
- Node CI 계약 95건, Python CI 계약 95건, actionlint·workflow shell 구문 검사 통과.
- 실제 네 문서의 prepared/host는 full-page 픽셀과 동일. 후보 왕복 0→1→0은 replay 2/hit 1.
- 후보 Native의 다섯 문서 전쪽 최저 99.66%, 편람 383쪽 최저 99.96%.
- 편람 63쪽·289쪽 Direct PDF 생성과 기준 Print 실루엣 100.00%.
- baseline 대조·fresh WASM·전체 lint·workspace build·전체 회귀 완료. 원격 게시·CI·merge는 별도 승인 전이다.

## 전쪽 실측과 전후 비교

2026-10-30, 96 dpi, canonical `visual_sweep.py --silhouette-only`의 색상 경계 보정 전
2px 실루엣 지표다. Print 세대·글꼴·입력은 base/candidate에서 고정했다. Native CLI와
최적화 WASM의 실제 Chromium Canvas 출력을 사용했다. 모든 원본/PDF의 쪽수가 일치한다.

| 원본 | 전체 쪽 | base Native → 후보 | base WASM → fresh 후보 |
| --- | ---: | ---: | ---: |
| body-list-typed | 1 | 100.00 → 100.00 | 100.00 → 100.00 |
| helvetica-styles | 1 | 99.66 → 99.66 | 99.64 → 99.64 |
| pic-crop-01 | 1 | 100.00 → 100.00 | 94.65 → 94.65 |
| shape-group-02 | 1 | 100.00 → 100.00 | 100.00 → 100.00 |
| re-03-latin | 1 | 100.00 → 100.00 | 100.00 → 100.00 |
| 편람 HWP | 383 | 99.96 → 99.96 | 99.96 → 99.96 |

[Native 전쪽 비교](native-page-comparison.tsv)와 [WASM 전쪽 비교](wasm-page-comparison.tsv)는
각각 388쪽이며 하락·90% 미만·누락은 0쪽이다. [Native PNG 비교](native-pixel-comparison.json)와
[WASM PNG 비교](wasm-pixel-comparison.json)에서도 각각 388쪽이 byte-identical이다.
`base/`, `candidate/` 아래 원 TSV를 보존했다. 모든 수치가 완전한 픽셀 일치나 한컴과의
전체 피델리티 100%를 뜻하지는 않는다.

새 타일 API는 기존 CLI를 바꾸지 않으므로 `consumer_probe.rs`로 별도 실행했다.
네 실물 문서를 각각 세 배율로 실행했고 viewport가 512px 경계를 가로질러 움직일 때의
겹치는 픽셀이 일치한다. 타일 출력의 독립 Print 비교는 scale 1(96 dpi)에서 수행했고 최저값은 99.66%다.
다른 두 배율은 raw crop·겹침 계약을 검사했으며 독립 Print 측정과 구분한다. 기존 full/prepared/host
12개 실물 PNG는 base와 byte-identical이다([소비 경로 비교](consumer-comparison.json)).
0→1→0 왕복은 base replay 3/hit 0 → 후보 replay 2/hit 1이며 각 frame은 해당 페이지의
full-page crop와 같다. timing JSON은 로컬 진단값이고 빌드와 CPU를 공유했으므로 시스템 전체의
속도 개선률로 일반화하지 않는다.

## 실제 Direct PDF 대조와 확대 판독

[수정 전 거부](pdf-negative-control.json): source p63(`--page 62`)은 connector line style,
p289(`--page 288`)는 arrow style로 실패했다. 후보는 각각 51,044/128,882 bytes의 1쪽 PDF를
생성했고, 같은 source 쪽의 독립 Print와 실루엣 100.00%다. 이 두 쪽을 편람 전체 383쪽의
Direct PDF 지원 완료로 확대하지 않는다. 생성 PDF에서 선택 문서는 p1이지만 원본 쪽 대응은
각각 63/289로 기록했다.

직접 판독한 대표 근거:

- [타일 글꼴 스타일](candidate/review-tiled/helvetica-styles/review_001.png): bold/italic,
  줄 순서·간격과 장평을 확인했다. 목록 들여쓰기·다음 항목의 위치도 별도 body-list panel에서 확인했다.
- [이미지 crop](candidate/review-wasm/pic-crop-01/review_001.png)와
  [그룹 도형](candidate/review-wasm/shape-group-02/review_001.png): 잘린 이미지 내용·비율,
  도형의 gradient·그림자 위치를 확인했다. crop의 테두리와 래스터 획 굵기에는 Print와 잔차가 있으나
  해당 출력은 base와 동일하며 이 Native API 통합으로 새 차이가 생기지 않았다.
- [63쪽 PDF](candidate/review-pdf63/review_001.png),
  [289쪽 PDF](candidate/review-pdf289/review_001.png): 연결선 끝점, 화살촉 방향·수량,
  본문과 상자 간격을 직접 대조했다.
- [289쪽 Native/Direct/Print 2배 확대](candidate/review-pdf289/zoom-native-direct-print.png),
  [WASM base/후보/Print 2배 확대](candidate/review-wasm-handbook/p289/zoom-base-candidate-print.png):
  가로·세로·대각 화살표가 해당 직위 사이에 남아 있고 끝점·몸통·머리가 누락되지 않았다.
  Print의 더 진한 획과 AA 차이는 기존 Native/WASM에서도 관찰되며 PDF admission이 geometry를
  새로 생성하거나 기존 PNG를 바꾸지는 않는다.
- [전쪽 최저값 130쪽](candidate/review-wasm-handbook/p130/review_130.png): 본문 줄바꿈과
  화면 이미지의 위치·크기를 확인했다. 글꼴·이미지의 잔차는 전후 동일하다.

변경 영역에는 수식이 없으며 수식 조판을 수정하거나 검증했다고 주장하지 않는다.
실물의 절대 좌표·전체 SVG hash를 새로 회귀 기대값으로 추가하지 않았다.

## 실행 환경과 산출물

- 기본 작업공간 공용 target: `/home/edward/mygithub/rhwp/target/pr-review`.
  Cargo/rustc/wasm-pack/wasm-opt는 순차 실행했다. 31 GiB host에서 Cargo jobs 6,
  전체 nextest 8 threads로 브라우저 작업과 메모리를 나누었다.
- candidate Native executable SHA-256:
  `fcc6fedb9fcc622394e70be90b49752d999a5f3732f5ed394eca2c8831894ce0`.
- base Native executable은 고정 base worktree에서 새로 빌드했다. SHA-256:
  `d263e0da3435a396104c424b00ca68831671d0463c8a4cba20b5696ae8c9b1c5`.
- candidate fresh WASM: Docker Compose, Rust 1.93.1, wasm-pack 0.15.0, locked wrapper,
  기본 release+wasm-opt, exit 0(17m21s). 호스트와 같은 물리 target을 `/review-target`에 마운트하고
  쓰기·hard-link를 확인했다. [산출물 해시](candidate-wasm-provenance.json)의 JS·WASM은
  `rhwp-studio/public/`과 일치한다. 브라우저는 Chromium 155.0.8059.79다.
- base WASM은 이전 검증의 `d189352a1b` 산출물
  `ce942e2d6e3d37dc4fd593cae55423b5b9303f510405d868a91c57bf39196f8d`를 보존해 이번에 실제 실행했다.
  `d189352a1b`→고정 base의 src/crates/vendor/Cargo diff가 비었음을 확인했다.
  [이전 provenance](../davindev_20261030/rebase-cb9c45/provenance.json)는 build 출처이고,
  위 388쪽 수치는 이번 재실행 결과다. 후보 바이너리 해시의 차이는 출력 변화로 추정하지 않고
  실제 byte-identical PNG를 확인했다.
- [글꼴 출처·hash](font-provenance.json). 편람용 HBATANG 부분집합은 원 프로그램의 해당
  951개 codepoint에 대해 분해한 outline과 advance가 같은지 확인했다. 글꼴 파일은 저장소에 넣지 않았다.

## 최종 로컬 게이트

[실행 결과와 명령](validation-results.json): 전체 nextest **12,198 PASS / 0 FAIL / 45 skipped**,
실행 517.660초, 장시간 검사 1건. 테스트용 최적화 컴파일 10m54s는 실행 시간과 구분한다.
Native 집중 81건, Native lib 4,040건(ignored 12), placeholder 2건, CI Node/Python 각 95건도
통과했다. 이 숫자들은 각 실행의 결과이며 서로 겹치는 검사를 합쳐 고유 테스트 수로 표시하지 않는다.
Native feature-off session 3건은 전체 nextest에서 실제 실행됐다.

fmt, Native/WASM32/workspace all-target/Native Skia all-target Clippy(`-D warnings`), workspace
build, suite manifest, actionlint, workflow shell syntax와 diff check를 통과했다.
테스트 첫머리 주석이 source byte weight를 바꿔 파생 harness가 달라졌고, `--prepare`로 재생성한
후 정책 check와 전체 실행을 마쳤다. `tests/generated/`와 파생 manifest는 커밋하지 않는다.

[편람의 기존 검사 참조 목록](handbook-existing-test-inventory.json)은 42개 source의 위치와
불변 여부를 기록한 목록이다. 목록 수집만을 assertion 검토나 시각 판정으로 간주하지 않는다.
독립 Print 경계를 고정한 `issue_8153_handbook_print_boundaries` 등은 기존 근거에 연결된
관계·좌표 검사를 유지했고, 이번 양쪽 전쪽 실측에서도 모든 페이지가 98% 이상이다.
이번 보정은 기존 회귀의 기대값·baseline·assertion을 바꾸지 않았다.

[원 PR CI 참고값](original-ci.json)과 후보의 로컬 검증을 구분한다. 원 네 PR의 실패는 모두
fork enforcement 정책이며 기존 성공한 Full CI를 이번 통합 후보의 CI 성공으로 대체하지 않는다.
현재 본인 계정의 열린 PR은 없으며 remote devel은 고정 base와 같음을 재확인했다.
원격 게시 승인 뒤 실제 최종 head의 raw 증적을 내려받아 hash를 확인하고, 새 CI와 mergeability를
확인한 뒤 승인 범위의 통합·후속 기록을 진행한다.

## 최종 통합 (2026-10-31)

[통합 PR #8394](https://github.com/edwardkim/rhwp/pull/8394)를 merge commit `7d4741392bb2ffaf308f5ce1cf6ba7f4deeeb1c9`로 반영했다. 원 PR 네 건도 모두 MERGED이며 원 커밋이 조상으로 남는다.
제출 head `d5a7d933f83550e89aabc3b08795724873051f34`의 Full CI·CodeQL·Render Diff·Proptest·Adapter·CI 정책이 성공했다. 원격 기본 회귀 12,004 PASS / 0 FAIL / 45 skipped, Native 새 cache 21·planner 10·host surface 9 및 Direct PDF 22건의 실제 통과를 확인했다.
[최종 CI·self-review 영수증](integration-result.json). 앞 절의 원격 승인 전 상태는 로컬 검증 당시 기록이며 최종 상태는 이 절을 따른다. #536·#7501은 OPEN을 유지한다.
