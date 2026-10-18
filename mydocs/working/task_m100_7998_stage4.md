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
