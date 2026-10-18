# #7998 P4 표·그림과 문단의 합성

- Issue: #7998
- 승인: 2026-10-18 P3 시각·문단 구조 통과 뒤 다음 절편 진행 승인.
- 시작: `4768e389f` / `task_m100_7998`, P3 런타임 checkpoint `b923b179a`.
- 계획: [P4](../plans/task_m100_7998_impl.md#43-분할정복-실행-순서와-절편별-완료-조건).

## 첫 절편의 위반 계약과 범위

독립 source 연산자로 작성한 일반 중첩 표, 여러 문단·그림, 그림이 먼저/여러 개인 셀은
현재 Native에서 99.98107–100%다. 그림이 셀 경계를 넘거나 본문과 겹치는 반례는 원래
내용 소유를 유지한다. 이 정상 경계를 새 결함이라고 주장하지 않는다.

점선 경계가 포함된 중첩 표는 `TableBuilder::build_inner`가 명시적으로 보류한다.
기존 source paint 분할은 한 층의 셀 영역만 받으므로 부모 셀이 자식 표의 영역까지
동시에 소유하면 중복 paint 또는 잘못된 이관을 만든다. 부모/자식 경계의 무조건 중복
허용이나 그림 숨김으로 이 제약을 제거하지 않는다.

P4 첫 절편은 엄격히 부모 셀 안에 있는 중첩 표 family의 source paint를 가장 안쪽
셀부터 배분하고, 이미 소비한 부분을 부모에서 제외한다. 선택한 family의 모든 단계가
같은 분할을 사용하며 일반 실선 family는 기존 경로를 유지한다. 실제 중첩 Table·셀 문단·
그림의 유일한 소유와 앞/뒤 문단, source 원문·paint 보존 및 편집·양 저장 형식을 검증한다.

source 경로는 `table_detection/table_hierarchy`의 childTableIndices·엄격한 셀 소속 →
`TableBuilder`의 child host Paragraph와 cell flow → family paint의 정수 공유 축과
가장 안쪽 셀 분할 → `table_paint::anchor`의 cell frame → 실제 nested Table 원점과
셀 draw 경로다. 요구/예약 높이는 기존 공통 cell flow를 소비한다. pagination·분할 정책은
이번 절편에서 변경하지 않는다. 원본의 부재한 경계로 병합을 확정하지 않는다.

## 독립 입력과 검증 경계

source 생성기·수정 전 producer·각 실행은 ignored
`output/pr-review/issue7998/p4-composition/`에 보존한다. 실제 저장 위치는
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-composition/`이다.
원본 PDF의 명시적 선·문자·그림 연산자와 생성 주석으로 기대 소속을 정하며 구현 결과를
정답으로 복사하지 않는다. 부모만 점선/자식만 점선/둘 다 점선, 일반 실선 중첩과
문단·그림 대조군을 비교한다. 시각 선행 조건 전에 새 fixture/정식 회귀를 등록하지 않는다.

실물은 기존 보고서·견적서의 전쪽 소속·내용과 동일 backend 전후 출력을 확인한다.
모호한 병합 후보와 compound 경계 그림은 이번 첫 절편의 지원으로 자동 승격하지 않는다.
미실행 경로와 남은 P4/P5는 별도로 기록한다. 원격 반영·PR 등록은 포함하지 않는다.

## 첫 합성 절편 결과 — 2026-10-18

메인테이너가 아래 검토 묶음과 실제 Studio에 대해 **“시각·중첩 구조 통과”**로 판정했다.
P4 전체 완료나 최종 PR 게이트 통과 판정은 아니다.

### 구현과 독립 기대값

`structured_lowering/composition.rs::source_paint_family`는 periodic 경계를 사용하는
후보의 조상 관계를 따라 family를 정한다. `TableBuilder::build_inner`는 그 family의 모든
실제 자식 Table에 같은 source-paint 방식을 전달한다. 일반 실선 family는 기존 경로다.
`composition::owners`가 실제 Table 계층의 가장 안쪽 셀을 먼저 나열하고,
`table_paint::partition`은 이미 소비한 조각을 제외한 물리 영역만 다음 owner에 배분한다.
`composition::table_mut`로 실제 해당 Table의 셀에 조각을 붙인다. 진단 후보 수만 늘린
것이 아니며, 자식 셀과 부모 셀의 겹치는 영역을 중복 발행하지 않는다.

원본의 문자·그림 순서는 기존 child host Paragraph와 공통 cell flow를 소비한다.
정수 공유 축·면적/선분 보존·source paint 순서 검사와 작업/fragment 한도는 유지했다.
renderer·pagination·serializer의 별도 좌표 보정이나 clamp는 추가하지 않았다.
분할·이어받기 정책 변경은 비해당이다. 실제 배치 검사는 자식 표·그림·글자의 셀 내부 포함,
부모 BEFORE/AFTER 문단과 자식 표의 순서, 그림과 글자의 비중첩까지 확인했다.

독립 입력은 [생성기와 원본 장부](../../pdf/issue7998/composition-sources.md)의 14개다.
일반 셀 그림·복수 그림·실선 중첩 4개와 침범/교차 반례 2개는 기존 정상 경계이며 새 수정
효과로 세지 않는다. 새 periodic family 8개는 부모/자식/양쪽 경계, 선두 자식, 자식 그림,
두 자식, 3단계 중첩, 공유 배경이다. 이전에는 실제 표 0개였고 현재는 각각 2개 또는 3개다.
source 연산자의 문자 장부와 경계 길이×0.5pt로 보존을 검사했다. 배경 23,800pt²는
자식 8,400pt²와 부모 잔여 15,400pt²로 실제 소유가 나뉘며 부모 밖 중복은 없다.

PDF dash 연산자(`[2 2] 0 d`)가 compound Picture로 내려오는 **별도 원본 7개**는 여전히
`compound boundary image needs native cell paint preservation`으로 보류된다.
독립 선분으로 그린 periodic 입력의 통과를 이 원본의 지원으로 바꾸어 보고하지 않는다.
원본·보류 증거는 외부 경로의 `nested-dash-*`에 보존했고 활성 회귀로 고정하지 않았다.

### 실행·시각·구조 검증

| 검증 | 관측 결과와 증거 |
|---|---|
| 정식 집중 회귀 | 기존 표 17건, 기존 본문 4건, 새 합성 4건, 합계 **25건 PASS**. `focused-existing-tables.log`, `focused-existing-body.log`, `focused-composition.log`. 새 검사는 자동 수집 `regression_suite_018`에서 실행했다. CI 개별 호출·기대값 완화는 없다. |
| 수정 전 FAIL / 수정 후 PASS | P3 라이브러리(`before/producer.json`)에서 현재 외부 계약 4건 중 실제 중첩 소유·배경 소유 2건 FAIL, 정상/보류 대조 2건 PASS. 현재 보정본 4건 PASS. `contract-before-current.log`, `contract-after-final.log`. 독립 원문·면적·포함/순서 관계를 검사하며 빌드 실패가 아니다. |
| 전쪽 원본·무편집 저장 | 54원본/65쪽 × PDF·HWP·HWPX = backend별 **162건/195쪽**. Native 최저 **97.45634%**, fresh WASM **97.41574%**. 오류·미달·누락·쪽수 불일치 0. 모든 저장본 PNG가 같은 backend의 직접 PDF 입력과 동일하다. `comparison.json`, `native-validation.log`, `wasm-validation.log`. |
| 자식 셀 편집·저장 | 8개 family에서 삽입 X/삭제 후 HWP·HWPX 재열기. Native 저장본 16쪽 최저 **99.95911%**, 실제 Studio WASM 저장본 16쪽 **99.95843%**. 위 전쪽에 더해 backend별 **211쪽** 비교. `native-edited-results.json`, `wasm-edited-results.json`. |
| 실제 Studio WASM API | 8개 family의 실제 Table/Cell 경로에서 삽입·삭제·양 저장 형식 재열기 통과, page error 0. `studio-result.json`. 저장 후 내부 control index가 달라질 수 있으므로 현재 트리에서 경로를 다시 얻고 의미 소속·원문을 대조한다. |
| P3 출력 보존 | 같은 backend의 기존 P3 **132쪽 모두 PNG byte-identical**, Native/WASM 차이 각각 0. 이는 P3 checkpoint 대조이며 최종 upstream/devel 대조의 대체가 아니다. |

전쪽 원본 TSV는 `native/visual/<key>-<format>/silhouette.tsv`와 `wasm/visual/…`에,
편집·복원 저장본은 `native-edited/visual/…`, `wasm-edited/visual/…`에 보존한다.
각 실행 manifest가 입력·기준 PDF·명령·글꼴·페이지를 고정한다. 독립 기준은 생성한 PDF
원본 자체이며 한컴 저장본 열기·인쇄 검증은 P5의 별도 게이트로 남는다.

14개 fixture와 새 정식 검사 4건은 양 backend·양 저장 형식의 90% 선행 조건과 변경 영역의
직접 확대 판독 뒤 등록했다. 기존 검사는 이미 관계 중심이며 절대 px 핀 교체는 없다.
새 검사도 글자/그림의 유일 소유·포함·앞뒤 순서·source 면적 보존을 사용한다.
private 견적서와 중간 실행 로그·JSON은 커밋하지 않는다.

### source와 producer

`source-runtime.json`의 런타임 입력 2,444개는 캡처 뒤 테스트·fixture·문서 추가와 구분해
재확인했다. manifest SHA-256:
`923fcc6c948e230607c1dd0c378e655bac137259bf194cc47ebd7e82ab1262db`.
Native SHA-256:
`4407cc5b5b016e796e64c2a5c1b452910369f6fa90e5c159255825970892491f`.
fresh Docker WASM SHA-256:
`c7a6c40044be28a27f1a2d953798a602c1ba5884f6685fda031a10ec9fab02d1`
(19,190,935바이트). Docker 완료 exit 0 뒤 복사한 사실은 `wasm-build-completion.json`에
기록했다. 실제 Studio loader 응답의 해시도 같았다(`studio-cdp.json`).

```bash
CARGO_BUILD_JOBS=8 cargo build --locked --profile release-test --features native-skia --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
docker compose -p rhwp --env-file .env.docker run --rm --no-deps \
  -e CARGO_BUILD_JOBS=6 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm
python3 output/pr-review/issue7998/p4-composition/validate.py native
python3 output/pr-review/issue7998/p4-composition/validate.py wasm
node output/pr-review/issue7998/p4-composition/verify_studio.mjs
python3 output/pr-review/issue7998/p4-composition/validate_edits.py native
python3 output/pr-review/issue7998/p4-composition/validate_edits.py wasm
node scripts/run-rust-test.mjs issue_7998_pdf_table_composition -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
```

Docker 최적화 완료 전에 복사한 중간 package로 시작했던 WASM 검증은 중단하고
`premature-package-diagnostic/excluded.json`에 제외 기록을 남겼다. 동일 Docker 빌드의
정상 종료를 기다린 뒤 최종 package로 영향 WASM 검증만 다시 실행했다. 위 표에는
중간 package 결과를 섞지 않았다. 이후 런타임은 바뀌지 않아 추가 빌드·전쪽 캡처를 반복하지 않는다.

### 직접 판정과 남은 범위

검토 묶음은 `/mnt/e/tmp/issue7998-semantic-table-review/p4-composition/index.html`
(`E:\tmp\issue7998-semantic-table-review\p4-composition\index.html`), 실제 Studio는
`http://localhost:7793/?review=issue7998-p4-composition&lang=ko`다. 원본·Native·fresh WASM,
실제 저장 IR과 owner overlay, 편집 복원본을 분리해 제공했다. 배경·3단계 중첩·자식 그림을
같은 배율로 확대 판독했고, 메인테이너가 **“시각·중첩 구조 통과”**로 수용했다.

새 중첩 입력 중 5쪽의 이전 positional 출력 대비 raw 점수 하락은 그대로 기록했다.
Native 최대 **0.01662%p**, WASM 최대 **0.01694%p**이며 최저는 99.95% 이상이다.
직접 판독과 이번 수용을 수치 삭제나 최종 devel 비교 예외로 해석하지 않는다.

남은 P4는 compound 경계 그림 7개, 병합/지역 격자의 독립 근거와 적용 범위,
테두리 스타일·행열 편집이다. 기존 수평 표현 보류·미확정 본문도 B12에 유지한다.
또한 `cell-picture-first`의 빈 그림 host에 `EDIT `를 12회 삽입한 기존 P3 진단에서는
그림 오른쪽이 셀보다 약 3.60px 넘는다(`edit-probe.log`). 긴 텍스트와 inline 그림의
줄 구성·측정/배치 소비 경계이며 이번 짧은 자식 셀 편집 통과로 해결됐다고 부르지 않는다.
이를 [B14](../plans/task_m100_7998_backlog.md)에 같은 #7998의 남은 P4로 등록했다.

P5의 최신 devel 병합·공통 대조군·요청 전체 자원/취소·Worker/package·전체 회귀·lint·
한컴 열기/인쇄·최종 PR 판단은 남아 있다. 이번 절편의 source·fixture·검사와 수용 기록을
로컬 커밋으로 보존하고 원격 push·PR 등록은 하지 않는다.

마지막 로컬 정합성 검사에서 runtime 지문 2,444개는 모두 같았고, 독립 생성기 재생성
PDF 14개가 커밋 대상 원본과 바이트 단위로 같았다. `cargo fmt --all -- --check`,
`git diff --check`, suite manifest `--check`는 통과했다. 검토 묶음의 로컬 참조 215개에
누락이 없으며 메인 Studio 7700의 JS/WASM도 P2부터 보존한 해시와 동일하다.
`final-local-checks.json`과 `maintainer-decision.json`에 source·판정·남은 범위를 고정했다.

## P4 두 번째 절편 착수 — compound 경계 그림

2026-10-18 다음 절편 진행 승인. 시작 checkpoint는 `f4842a060`이다.
앞 절편에서 별도로 보존한 PDF dash 원본 7개를 같은 바이트로 사용한다.
`path_paint::lower`는 점선의 phase·alpha·clip을 물체별 PNG에 보존하고
`lower_picture`가 원본 이미지 범위/선택 영역과 page frame을 만든다. 그러나
`table_paint::partition`은 이 Picture 종류를 무조건 거절하여 실제 Table을 발행하지 못한다.

이번 계약은 source frame의 물리 분할과 동일한 비율로 원본 이미지 선택 영역을 나누고,
같은 경계는 같은 정수 crop 좌표를 사용해 scale·phase·alpha를 보존하는 것이다.
셀마다 전체 이미지를 축소하거나 투명 여백을 무시해 위치를 맞추지 않는다. 실제 소비는
fragment의 Picture frame/crop → cell anchor → 공통 picture crop → Native/WASM 및
양 저장 형식 재열기까지 추적한다. 원본 bin data를 공유하고 이미 소비한 영역을 다시
부모에게 주지 않는다. 경계 출처·paint 순서·기존 작업/fragment 예산과 fallback은 유지한다.

독립 원본/수정 전 보류·구조/시각·실행 증거는 ignored
`output/pr-review/issue7998/p4-compound/`(외부 안정 경로의 `semantic-stage3/p4-compound/`)에
보존한다. compound 장식·내용 그림·전경과 겹치는 paint를 단순 separator로 오인하지 않는
반례를 대조한다. Native/fresh Docker WASM 직접 판독과 실제 셀 편집·양 저장 검증 뒤에만
fixture/정식 회귀를 등록한다. 테두리 스타일·행열 편집·B14는 이 절편의 완료 주장에 포함하지 않는다.

### 직접 출력에서 추가로 검출한 소비 경계

초기 crop 보존 검사만으로는 실제 배치·가시성을 입증하지 못했다. Native 직접 출력에서
① 일반 셀 Picture의 ‘같은 문단에 여러 float’ 보정이 overlay의 음수 오프셋까지 지우고
셀 안으로 clamp하여 원점을 이동시키는 문제, ② 경계 bitmap의 실제 두께를 한 셀에 모두
주면 그 셀의 clip이 다른 쪽 절반을 지우는 문제를 검출했다. 첫 출력은 90% 미달이었고,
원점 보정 뒤에도 선 두께 차이가 남아 정식 fixture를 등록하지 않았다.

`table_layout.rs`의 group-flow offset 보정은 흐름을 밀지 않는 overlay에 적용하지 않고,
overlay의 실제 원점도 봉쇄로 이동시키지 않는다. 기존
`cell_wrap_vertical_offset_hu`가 보존하는 signed overlay offset과 일치시킨다.
부분 표의 Picture 경로는 이 group reset/후행 clamp를 갖지 않으므로 같은 신규 보정은
비해당이다. 이번 PDF 원본은 각 1쪽의 완전한 표 경로이며 continuation 수정은 아니다.
TextBox 내장 표의 별도 Picture 경로 전체를 검증했다고 주장하지 않는다.

bitmap은 면적 객체다. source 축으로 선의 전체 두께를 한쪽에 배정하지 않고 실제 셀
영역으로 분할한다. 가장 안쪽 셀 → 부모 셀 → 쪽 잔여 영역이 각각 소비한 부분만 남긴다.
원본 이미지 선택 영역의 빈틈·중복 없음뿐 아니라 실제 Image node의 셀 내부 포함과
source crop에 대한 최종 배치도 검사해 위 두 결함이 검사 밖으로 빠지지 않게 한다.
초기 실패 캡처는 `pre-anchor-diagnostic/`, `pre-physical-diagnostic/`에 보존한다.
중단한 Docker 빌드(exit 137)와 그 산출물은 최종 WASM 증거로 쓰지 않는다.

물리 분할에서 다른 source path의 bitmap과 교차할 때 기존 guard가 이를 별도로 삽입한
내용 그림으로 오인해 보류하는 경계도 확인했다. `ruleSourceControls`의 출처로 path paint와
내용 Picture를 구분한다. path끼리는 원래 z-order와 물리 분할을 유지하며, 글자나 별도로
삽입한 그림을 나중에 덮는 path는 계속 보류한다. `compound-foreground`로 이 경계를 대조한다.

### Native 전쪽 선행 결과

최종 source의 지문은 `p4-compound/source-runtime.json`이며 2,444개 파일이다.
SHA-256은 `46993bd7c858b203bf24262875fec86c2e7d7517da1139106be6a52663510aeb`,
Native producer는 `ff29632219b8433ef294bcc9f0a2791cd036c19c2083205f08ebeadfe4e6f8a9`다.
수정 전 producer는 앞 절편의 `4407cc5b…`로 고정했다. root의 오래된 `.rlib`는 빌드 신원
증거로 사용하지 않았다. 수정 전에는 이 frozen CLI가 실제 발행한 HWPX를 재열어 Table
소유를 검사했고, 새 의미 소유 검사 1건은 0→2 기대 차이로 FAIL, 전경 보존 1건은 PASS다.
현재 Native 계약은 10개 합성 family의 실제 소속·crop/원본 좌표·글자·편집·양 저장 형식과
전경 반례를 통과했다(`contract-before.log`, `contract-after.log`).

58 PDF/69쪽의 PDF·HWP·HWPX, **174건/207쪽** 전쪽 Native 최저는 **97.45634%**,
오류·미달·누락·쪽수 차이는 0이다. 무편집 저장본 PNG는 직접 PDF 입력과 모두 같다.
앞 절편의 195쪽 중 기존 지원 범위 174쪽은 PNG가 같고, 그때 보류됐던 원본 7개×3형식의
21쪽은 실제 Table 발행 뒤 바뀌었다. 자식 셀 삽입·삭제·양 저장 형식 복원 20쪽도
90% 이상이다. 원본/저장과 편집 복원 결과를 서로의 대용으로 쓰지 않는다.

두 미세 하락은 숨기지 않는다. `nested-dash-parent`는 이전 positional 출력보다
Native raw **0.01656%p**, 새 `compound-phase-alpha`는 **0.48740%p** 낮다.
최저값은 각각 99.95631%, 99.48824%다. 후자는 반투명 bitmap 선택 영역의 셀 경계에서
래스터 농도가 달라지는 경우다. 편집 복원본은 99.48796%이며 원문·소속과 source 영역의
누락은 검출하지 않았다. 수치·확대 이미지·직접 판정을 구분하고 최종 PR gate를 해제하지 않는다.

실제 한컴 대조군 `cell_behind_text_para_anchor.hwpx` 2쪽과
`156489219_satellite_pm_release.hwp` 8쪽은 이전 Native PNG와 모두 같다.
전자는 99.70908% 이상이고 후자에는 base부터 5쪽 84.19045%가 있다. 이 원본을 새 정상
fixture로 등록하거나 그 점수를 이번 개선의 통과 근거로 사용하지 않는다. 기존 미달 범위는
P5의 최종 검토 대상으로 남긴다. 현재 변경의 픽셀 회귀는 검출하지 않았다.

정식 기존 표 17건·본문 4건, 관련 한컴 overlay·flow 대조 2건은 통과했다. fresh Docker
WASM·실제 Studio와 직접 시각 판정은 이 Native 결과와 별개로 진행한다. source가 같으면
완료한 빌드·캡처를 다시 실행하지 않으며 신규 fixture/회귀는 WASM 선행 조건 뒤 등록한다.

### 두 번째 절편의 최종 검증과 직접 수용

2026-10-18 메인테이너가 실제 Studio의 점선 중첩/내용 그림과 반투명 경계를 확인하고
**“시각·중첩 구조 통과”**로 판정했다. 요청 당시 Native raw 99.48824%와 이전 positional
출력 대비 0.48740%p 하락을 공개했다. 후행 WASM 결과도 같은 위치·두께를 보존하며 농도
차이를 기록했다. WASM의 해당 쪽은 99.47968%, 하락은 0.49549%p다. 부모 점선의 WASM
하락은 0.01438%p다. 높은 점수로 이 차이를 지우거나 최종 PR 게이트를 면제하지 않는다.

| 검증 | 최종 관측과 기존 증거 |
|---|---|
| 원본·무편집 저장 전쪽 | Native/fresh WASM 각각 **174건/207쪽**, 최저 **97.45634% / 97.41574%**. 오류·90% 미달·누락·쪽수 차이 0, 모든 무편집 저장본 PNG는 같은 backend의 직접 PDF 입력과 동일. `comparison.json`, `native/results.json`, `wasm/results.json`. |
| 편집·양 저장 형식 | 10개 family의 자식 셀 X 삽입·삭제 후 HWP/HWPX 복원 각각 **20쪽**, 최저 Native **99.48796%**, 실제 Studio WASM **99.47924%**. 합계 backend별 **227쪽**. `native-edited-results.json`, `wasm-edited-results.json`. |
| 실제 Studio | 원래 문자/셀 계층, 현재 경로의 삽입·삭제·저장·재열기 10개 모두 통과, page error 0. 실제 loader 응답 해시는 최종 package와 동일. `studio-result.json`, `studio-cdp.json`, `studio-cdp-alpha.json`. |
| 이전 지원 범위 | 앞 절편의 195쪽 중 **174쪽 PNG 동일**, 보류됐던 dash 7개×3형식의 21쪽만 의미 표 생성 후 변경. 이전 checkpoint와의 비교이며 최신 devel 검증의 대체가 아니다. |
| 한컴 원본 대조군 | 앞서 기록한 overlay 2쪽·flow 8쪽은 WASM도 **10쪽 모두 이전 PNG 동일**. overlay 최저 98.45233%, flow의 기존 5쪽 미달 84.52418%는 보존. `hwp-control-comparison.json`. 이 기존 미달은 P5 판정 대상으로 유지한다. |
| 정식 집중 회귀 | 선행 표/본문/한컴 대조 23건에 합성 6건을 더해 **29건 PASS**. 새 2건은 source PNG 전체 영역·비중복 crop·실제 셀 포함·글자 소속·양 저장 형식을 검사한다. `focused-compound-qualified.log`. 기존 자동 수집 suite를 사용하며 workflow 개별 호출은 추가하지 않았다. |

새 [독립 원본 11개](../../pdf/issue7998/compound-sources.md)는 전쪽 Native/fresh WASM과
확대 판독·메인테이너 판정 후 정식 fixture로 등록했다. PNG 영역의 합만 확인하지 않고 실제
최종 Image의 셀 내부 포함을 함께 검사한다. 수정 전 실제 frozen CLI의 HWPX에서 새 의미
소유 assertion이 실패한 증거와 초기 배치 실패는 위 선행 기록에 연결했다. 처음 정식 suite
실행은 생성 harness의 과거 배정 때문에 0건으로 끝났고 통과로 세지 않았다. canonical
`--prepare` 후 실제 6건 실행을 확인했다. harness 생성기는 ignored 산출물이며 기대값이나
기존 검사를 삭제·완화하지 않았다.

fresh Docker WASM은 빌드 exit 0 확인 뒤 복사한 **19,197,060바이트** package다.
SHA-256: `3f8ba4e23d4a984def562745fa716a69511de1dc58d2fe9cd91078304d9797ed`.
`wasm-build-completion.json`, `docker-wasm.log`에 완료와 producer를 기록했다.

```bash
docker compose -p rhwp --env-file .env.docker run --rm --no-deps \
  -e CARGO_BUILD_JOBS=6 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm
python3 output/pr-review/issue7998/p4-compound/validate.py native
python3 output/pr-review/issue7998/p4-compound/validate.py wasm
node output/pr-review/issue7998/p4-compound/verify_studio.mjs
python3 output/pr-review/issue7998/p4-compound/validate_edits.py native
python3 output/pr-review/issue7998/p4-compound/validate_edits.py wasm
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_BUILD_JOBS=8 node scripts/run-rust-test.mjs issue_7998_pdf_table_composition -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
```

최종 검토 묶음은 `/mnt/e/tmp/issue7998-semantic-table-review/p4-compound/index.html`
(`E:\tmp\issue7998-semantic-table-review\p4-compound\index.html`)이다. 원본·이전/현재 출력·
실제 저장 IR·소유 overlay·편집 복원본과 TSV를 분리해 제공한다. 확대 판독에서 자식 표의
테두리 양쪽 누락·원점 이동·그림/문자 누락은 검출하지 않았고 반투명/가는 경계의 농도
차이는 그대로 남겼다. source manifest의 런타임 2,444개는 빌드·캡처 뒤 동일하며 기존/신규
독립 생성 원본 25개도 저장소 원본과 byte-identical이다. 검토 묶음의 로컬 참조 206개에
누락이 없다. `cargo fmt --all -- --check`, `git diff --check`, suite manifest `--check`를
통과했고 메인 Studio 7700의 JS/WASM 해시도 보존됐다. 최종 정합성 결과는
`final-local-checks.json`에 보존한다.

이번 절편을 로컬 커밋으로 보존한다. 다음은 B14의 긴 inline 그림 host와 테두리 스타일·
행열 편집, 독립 근거에 따른 병합/지역 격자의 남은 범위다. P5의 최신 devel 병합·공통
대조군·전체 회귀/lint·Worker/package/자원 정책·최종 한컴 열기/인쇄와 PR 판정은 남아
있다. 이번 수용을 P4 전체 완료나 원격 push·PR 등록·이슈 종료 승인으로 해석하지 않는다.

## P4 세 번째 절편 — B14 긴 텍스트와 inline 그림 host

2026-10-18 다음 절편 승인, 시작 `9ee2b9dc0`.
기존 독립 `cell-picture-first.pdf`의 빈 그림 host에 `EDIT ` 12회를 삽입하는 동일
Native API로 오른쪽 약 3.60px 넘침을 재현했다. `p4-inline-host/probe-before.log`에
현재 source의 저장 줄·control 위치·실제 RenderTree를 보존했다. 실제 그림 크기·원문은
보존돼야 하며 글자와 그림의 점유 폭이 가용 폭을 넘으면 다음 물리 줄을 가져야 한다.

현재 입력은 PDF에서 발행한 IR을 실제 편집한 문서다. 단순 수동 LineSeg는 아니다.
`reflow_cell_paragraph_with_metrics`의 셀/문단 여백 상자 →
`reflow_line_segs_after_cell_text_edit` → `flow_inline_controls`가 문단 끝 위치의
그림을 제외한다. 실제 셀 배치는 마지막 run의 end TAC로 그림을 그려 소속을 재추정한다.
따라서 text-only 2줄 결과 뒤에 그림이 붙고, 후행 정렬은 공간 부족을 해소하지 못한다.
이 부분을 clamp/그림 축소로 해결하지 않는다. 기존 inline frame projection의 원시
UTF-16 control 슬롯을 줄 나눔 입력에 포함하고 그 LineSeg를 측정·배치·저장에 전달한다.
실제 줄/개체 포함·앞뒤 순서·원문/그림 한 번 보존 및 양 저장 형식으로 판정한다.

긴 글자 앞/뒤/명시적 개행, 같은 줄의 복수 그림, 바깥 여백이 있는 짧은 대조군을 대조한다.
유효한 무편집 저장 줄 재사용·floating/캡션 등 별도 배치 계약은 같은 처리라고 가정하지
않으며 적용되지 않는 경계를 기록한다. 독립 출력과 Native/fresh Docker WASM 시각 검증
선행 조건 뒤에만 정식 회귀를 등록한다. 원격 반영·P5 전체 검증은 이번 승인에 포함하지 않는다.

### B14 공통 줄 구성과 실제 소비 경로

변경한 값은 그림의 **raw UTF-16 슬롯과 사방 바깥 여백을 포함한 점유 폭/높이**다.
`document_core/commands/text_editing.rs::reflow_cell_paragraph_with_metrics`가 셀·문단
여백 상자를 정하고 `reflow_line_segs_after_cell_text_edit`에 전달한다.
`composer/line_breaking.rs::reflow_line_segs_impl`은 편집으로 무효가 된 저장 줄을
제거한 입력을 공통 frame에 넣는다. `inline_frame_input::project`는 가시 문자열을
바꾸지 않고 문단 끝/같은 글자 경계의 그림도 별도 raw 슬롯으로 보존한다. 기존
`inline_control_occupied_size_hwp` 결과에 그림 바깥 여백을 더한 상자를 fill이 소비한다.
frame의 줄·공백 메트릭을 `replace_line_segs_with_space_metrics`로 함께 발행한다.

그 결과를 composer의 실제 줄과 `typeset/paragraph/format.rs`의 줄 점유/advance가
소비한다. paint의 `paragraph_layout.rs::layout_empty_runs_line`도 생성 태그와
`control_line_seg_index`의 실제 소속이 맞는 그림 전용 줄을 공통 inline 경로로 그린다.
이 마지막 소비를 바꾸기 전에는 줄 나눔에서 여백을 예약했어도 셀 fallback이 원점을 다시
선택해 왼쪽/위 여백이 빠졌다. 최종 그림 원점은 기존 공통 바깥 여백·최고 개체의 기준선
정렬로 정하고 inline 위치를 등록해 fallback의 중복 배치를 막는다. 이미 존재하는 baseline
정렬/max 처리는 그대로이며 이번 수정에 별도 clamp·그림 축소·문서별 분기는 없다.

적용은 캡션 없는 inline Picture와 폭을 차지하지 않는 marker만 있는 host다.
유효한 무편집 저장 줄의 수용 계약, 캡션/floating·다른 layout owner가 섞인 경로,
`split_stale_cell_reflow`는 기존 경로를 유지한다. pagination/rowspan/continuation의 컷·
예약·이월 정책은 변경하지 않았으며 이번 절편으로 분할 일반성을 입증했다고 보고하지 않는다.

### B14 독립 근거·수정 전후·최종 검증

증적 루트는 ignored `output/pr-review/issue7998/p4-inline-host/`다. 실제 원본은
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-inline-host/`에 있다.
[입력·편집 순서·인쇄 출처](../../pdf/issue7998/inline-host-sources.md)에 작은 독립 Print
PDF와 인쇄 입력 HWPX의 해시를 연결했다. 원래 face의 재현과 명시적 Arial 대조군을 구분했다.
한컴 2020이 원래 Helvetica를 Haansoft Batang으로 대체한 `long-suffix`의 Native
**88.50536%**는 남는다. Arial을 편집 전에 선택한 7개 입력의 실제 Print는 ArialMT를
포함한다. 이 통과를 원본 글꼴 차이의 해결/예외 승인으로 바꾸지 않으며 B08·P5에 유지한다.

| 검증과 주장 | 실제 관측과 증거 |
|---|---|
| 원래 문단 끝 그림 결함 | 시작 `9ee2b9dc0`의 실제 API 재현에서 그림 오른쪽이 셀보다 **3.60px** 넘었다. frozen 이전 CLI의 같은 편집 HWPX는 독립 Print 대비 **68.57022%**, 물리 셀 포함 assertion FAIL이다. 현재 양 형식은 Native **91.74413%**, actual Studio WASM **90.96021%**, 셀 포함 PASS다. `probe-before.log`, `before-geometry.json`, `before-visual/`, `probe-after.log`, `native-edited-visual/`, `wasm-edited-visual/`. |
| 편집 대조군 7개×양 형식 | 긴 글자 앞/뒤·중간, 명시적 개행, 짧은 글자, 복수 그림, 사방 여백을 실제 API로 편집했다. 독립 한컴 2020 Print 대비 각각 **14쪽**, 최저 Native **91.24106%** / actual WASM **90.35264%**, 오류·미달·쪽수 차이 0. `native-arial-edited-results.json`, `wasm-arial-edited-results.json`, `arial-*-status.json`. |
| 삭제 후 빈 그림 host | 원래 긴 삽입을 삭제하고 HWP/HWPX로 저장·재열기한 **2쪽**은 원본 PDF 대비 Native **99.99115%** / actual WASM **99.99108%**다. `delete-probe.log`, `restored-studio.json`, `restored-results.json`. 다른 7개 삭제 대조는 API·소유·저장 보존을 검사했으며 이 원본과의 시각 복원 점수를 대신 적용하지 않는다. |
| 기존 지원 전쪽 | Native/fresh WASM 각각 **174건/207쪽**, 최저 **97.45634% / 97.41574%**. 양쪽 모두 오류·90% 미달·누락·쪽수 차이 0. 앞 절편 같은 backend의 **207쪽 PNG 모두 byte-identical**, 각 무편집 저장본도 직접 PDF 입력과 같다. `native/results.json`, `wasm/results.json`, `*-source-comparison.json`. 최신 devel 비교가 아니라 checkpoint `9ee2b9dc0` 대비다. |
| 실제 Studio 편집 구조 | Arial 7개와 원래 PDF 직접 입력 1개, 총 **8개**에서 삽입·merge·그림 속성·삭제·양 저장 형식·실제 RenderTree의 셀 포함과 그림 수를 확인했다. API의 원문 순서와 저장 후 소유를 확인했고 page error 0이다. 실제 loader 해시는 아래 fresh package와 같다. `studio-result.json`, `studio-cdp.json`, `studio-cdp-margin.json`. |
| 정식 집중 회귀 | composition **8건**(기존 6+신규 2), 관련 #6603 **1건**, #7160 **4건**, #6754 **2건**, #6122 **1건**, 합계 **16건 PASS**. `focused-inline-final.log`, `focused-6603.log`, `focused-7160.log`, `focused-6754.log`, `focused-6122.log`. 기존 자동 수집 suite를 사용하고 CI 개별 호출을 추가하지 않았다. |

정식 신규 2건은 위 Native/fresh WASM 선행 조건과 직접 판정 뒤 등록했다.
원문/paint 글자 누락·중복, 원래 이미지 데이터, 실제 셀 포함·글자/그림 비중첩·그림 순서와
공유 기준선, 양 저장 형식·삭제 복원을 보호한다. 98% 미만 편집 쪽에 절대 px 좌표나
SVG 해시를 고정하지 않았다. 결함 검출의 수정 전 증거는 위 frozen 이전 실행의 실제 셀
포함 assertion이며, 최종 Rust 함수 전체를 이전 라이브러리에서 실행한 것으로 보고하지 않는다.

초기 `before` 이름으로 복사하려던 라이브러리는 이미 첫 후보 빌드가 끝난 산출물이었다.
`libcandidate-initial.rlib`/`initial-candidate-before-library.json`으로 구분해 이전 producer
증거에서 제외했다. 실제 수정 전 증거는 빌드 전 `probe-before`와 frozen 이전 CLI다.
초기 Studio harness도 저장 후 control index가 바뀌는 것을 고정해 마지막 사례에서 실패했다.
현재 RenderTree에서 경로를 다시 구하도록 진단만 보정했고 8개 모두 실제 실행을 확인했다.
이 두 진단 오류를 제품 결함·수정 전 FAIL로 세지 않는다.

2026-10-18 메인테이너는 원래 재현과 바깥 여백을 실제 Studio에서 확인하고
**“시각·편집 구조 통과”**로 판정했다. 원래 Helvetica 대체 차이를 공개한 상태의 판정이다.
같은 판정을 다시 요청하지 않는다. 직접 확대 판독에서 원래 그림 크기·같은 줄의 두 그림
기준선·사방 여백과 뒤 문단을 대조했다. 가는 괘선/글꼴 잉크의 농도 차이는 남겼으며
시각 점수와 실제 편집 구조를 구분한다. 검토 묶음은
`/mnt/e/tmp/issue7998-semantic-table-review/p4-inline-host/index.html`이다.

### B14 producer·명령과 다음 경계

런타임 2,444개 source manifest SHA-256은
`6a59224df090e9fea8a5991e3c89bfb86f26831906268eef12e6fef62d020057`다.
Native: `501539465924136c6c8de5760a0677c931af60aa3b2916e9746fae81267fcfa6`.
fresh Docker WASM: **19,202,385바이트**,
`7ff8e7f6ddb12f7d9eba58e16965c71c93df130c4ad69ef006b9c6beab98022f`.
빌드 exit 0을 확인한 뒤 package를 복사했으며 실제 Studio 응답 해시까지 대조했다.
로그는 `native-build-row-consumer.log`, `docker-wasm.log`, `wasm-build-completion.json`이다.

```bash
CARGO_BUILD_JOBS=8 cargo build --locked --profile release-test --features native-skia --bin rhwp \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
docker compose -p rhwp --env-file .env.docker run --rm --no-deps \
  -e CARGO_BUILD_JOBS=6 -e CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 wasm
python3 output/pr-review/issue7998/p4-inline-host/validate.py native
python3 output/pr-review/issue7998/p4-inline-host/validate.py wasm
node output/pr-review/issue7998/p4-inline-host/verify_studio.mjs
python3 output/pr-review/issue7998/p4-inline-host/validate_edits.py native --arial-control
python3 output/pr-review/issue7998/p4-inline-host/validate_edits.py wasm --arial-control
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_BUILD_JOBS=8 node scripts/run-rust-test.mjs issue_7998_pdf_table_composition -- \
  --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review --features native-skia
```

완료한 같은 소스의 빌드·전쪽 캡처를 다시 실행하지 않았다. `cargo fmt --all -- --check`,
suite manifest `--check`, `git diff --check`와 source/package/검토 링크 정합성은
`final-local-checks.json`에 기록한다. 메인 Studio 7700의 JS/WASM은 보존하고 task Studio
7793만 갱신했다. 이번 변경과 보고를 로컬 커밋으로 보존한다.

B14의 이번 공통 inline 그림 계약은 구현·직접 판정을 마쳤다. P4의 테두리 스타일/행열
편집·독립 병합/지역 격자 근거·보류된 본문 수평 표현과 B08의 원래 글꼴 차이는 남는다.
P5의 최신 devel 병합·공통 대조군·전체 회귀/lint·Worker/package/자원/취소·최종 한컴
열기/인쇄와 PR gate도 남는다. 이번 판정은 P4 전체 완료나 원격 push·PR·이슈 종료 승인이 아니다.
