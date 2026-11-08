# PR #8132 리뷰 — Base14 대체 글꼴 보존과 Studio 준비 성능

## 최종 판정

**승인 — #8109의 명시적 대체 생산·저장·실제 소비와 성능 보정, 전쪽 devel 비교 및 code head CI 충족.**
검토 code SHA는 `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`다. 원격 merge·이슈 close는 수행하지 않았다.
남은 merge 조건은 이 문서-only trailing head의 실제 required CI·mergeability 재확인과 작업지시자의
별도 merge 승인이다. 코드 검증 판정과 원격 승인 권한을 구별한다.

## 접수 정보

- [PR #8132](https://github.com/edwardkim/rhwp/pull/8132), 작성자 edwardkim, base devel.
- 기준 base `80968497c2c178eeecb3b2e1d04d086ad27a0dc7`, code head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- 관련 [#8109](https://github.com/edwardkim/rhwp/issues/8109)는 참조만 하며 자동 close 문구를 사용하지 않았다.
- collaborator self PR 경로로 처리했고 reviewer를 지정하지 않았다. Open PR·본문 한글·실제 이미지4개
  표시를 확인했다. 미커밋 PDF 표 추론 계획 문서는 요청대로 원문 그대로 포함했다.

## 구현 주장과 실제 소비

| 주장·적용 범위 | 독립 근거·실제 검사 | 판정 |
| --- | --- | --- |
| Type1·표준 Helvetica4종·program absent만 대체 선언을 만든다 | ISO/Core14·scoped FontInfo → span font → lowering의 조건을 [1단계](../../working/task_m100_8109_stage1.md)·[2단계](../../working/task_m100_8109_stage2.md)에 추적했다. 같은 이름의 Form resource·embedded/unreadable/unknown/type 비적용 대조군과 수정 전5 FAIL/후6 PASS를 확인했다. | 충족 |
| 원본·측정과 실제 대체·양 저장을 연결한다 | 원본 이름·Core14 AFM·CharShape를 보존하고 HWP alternative face/type을 공통 모델로 읽는다. 동일 입력·편집·역삭제·undo/redo·HWP/HWPX 저장/재열기와 독립 한컴 Print를 대조했다. 실제 CanvasKit physical face·style 및 Canvas2D 후보 전달을 [3단계](../../working/task_m100_8109_stage3.md)에 연결했다. | 충족 |
| 불완전 preflight가 불필요한 Tree를 생성하지 않는다 | 실제 제품 콜백의 전후 검사에서 incomplete Tree 조회1 → 0회, auto77쪽 조회77 → 0회다. 최종 fallback은 autoPreflightIncomplete / Canvas2D로 같았다. | 충족 |
| 정상 font 준비가 선언을 공유하고 가용성을 갱신한다 | report 수명의 WeakMap에 family/bold/italic만 저장한다. 같은 report의 transform+준비2 → 1회, 새 report는 다시 수집했다. 공급자 detach/재등록·늦은 local 갱신·네 style 실제 outline·font 실패 fallback도 통과했다. host catalog의 기존 별도 요청 수집 순회는 유지된다. | 충족 |
| 성능과 품질을 함께 유지한다 | 같은 환경9쌍에서 auto1,284.9 → 626.2ms, 직접 Canvas2D487.7 → 487.5ms다. 새 실제 Studio4쪽은 이전 PNG와 byte-identical이며 독립 Print 확대 판독도 완료했다. | 충족 |

조판 규칙·좌표 변경: 새 원점/줄 높이/advance/크기/alpha/clamp 규칙을 추가하지 않았다.
문서 ID나 우연한 수치로 예외를 만들지 않았다. 필수 경계와 실제 소비를 위 증거에 연결했다.
분할·이어받기·pagination 변경은 비해당이다. baseline·래칫·golden 완화나 렌더링 fixture 신규 추가는 없다.
기존 #4961 기대값 보완의 원 PR·독립 DocInfo 근거와 전후 검증은3단계에 기록했다.

## 검증 입력·실행 결과·출력 비교

원본·독립 Print·SHA는 [전쪽 입력 원장](../assets/issue_8109/stage3/inputs.tsv)에 있다.
기준 한컴2020 Print method0 one-up의 실제 생성 정보와 저장본 대응을 확인했다. 입력은 정식 커밋 경로를
사용하고 Node/WASM 실행 파일·Git blob이 같은지 검증했다. 상용 폰트 bytes와 비공개 원본은 포함하지 않았다.

- Rust lint 전체·release/lib·nextest11,215 PASS·Native Skia3종·doc·최적화 WASM을3단계에서 통과했다.
  이번 추가 변경은 Studio5개와 renderer 사전 계약 검사1개 및 문서/증적이다. Rust source/test/fixture/manifest/workflow diff와
  producer SHA가 같아 해당 결과를 재사용했다. [4단계](../../working/task_m100_8109_stage4.md)에 이 경계를 명시했다.
- 최종 Studio TypeScript·npm2,350 PASS/0 FAIL/2 skip·build, 실제 font E2E와 host 양 backend를 다시 통과했다.
  새 결정적 성능 검사2개는 보정 전 actual callback에서 FAIL / 보정 후 PASS했다.
- Native37·fresh WASM37·SVG6 전쪽은 각1=1쪽이고 raw 최저99.63470% / 98.35575%다.
  같은 producer의 고정 devel 대비0.1%p 이상 하락0쪽이며 전쪽 TSV84개를 PR 본문 raw 링크로 제출했다.
- 새 Studio 목록/네 style의 CanvasKit/Canvas2D4쪽은100 / 99.94072 / 99.40245 / 99.70580%다.
  독립 Print와4배 확대해 목록·계속 줄·뒤 본문·네 style·기준선·간격을 확인했다.
  [현재 소스/캡처 신원](../assets/issue_8109/stage4/producer.tsv), [시각 판독](../assets/issue_8109/stage4/README.md),
  [성능 raw](../assets/issue_8109/stage4/performance.tsv)를 연결한다.

최신 base는 변하지 않아 rebase·새 devel 기능 유입이 없었다. merge-tree 종료0·공백·변경 문서 링크가
통과했고 PR 생성 뒤 MERGEABLE을 확인했다. trailing push 전 다시 같은 검사를 실행한다.

## 정확한 code head GitHub CI

- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39595160625): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- [Cancel stale PR runs](https://github.com/edwardkim/rhwp/actions/runs/39595160792): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39595160713): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39595160482): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39595160825): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/39595160559): success, head `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`.

위 성공은 동일 PR·source repository·code SHA에서 API로 확인한 결과다. 리뷰 문서는 이 성공 뒤에만 작성했다.
trailing head의 fast-pass/required check는 push 이후 별도로 확인한다. 원시 API는 ignored stage4에 보존했다.

최초 `89cf12827`의 Render Diff는 픽셀 비교 전 과거 직접 인자 표기의 source guard에서 실패했다.
원 PR #4517·통합 보정 #7697의 준비 순서를 추적한 뒤 문서 후보 해소도 검사하도록 갱신했다.
로컬 renderer 계약·폰트 coverage·Canvas3/3·직접 PDF3/3·auto readiness8/8을 통과했고,
위 최종 code head의 새 CI에서 모두 성공했다. report-only PDF4 warning·0 error는 직접 호환 gate 및
독립 Print 시각 검증과 구별했다. [4단계 후속 기록](../../working/task_m100_8109_stage4.md#pr-8132-첫-ci의-source-guard-보정)에 연결한다.

## 사용자 승인 뒤 self-review 재확인

검토 문서 head는 `3cb689cc7776af186bfde74820379510ebeb9bf9`다. code head 이후 변경은
문서4개뿐이며 source·test·fixture·workflow·baseline은 같았다. Type1 program의 삼상태,
실제 scoped resource 소속, 대체 선언/저장/paint 소비와 report 수명의 WeakMap을 다시 대조했다.
차단할 코드 결함은 발견하지 못했다.

기존 시각 원장에서 저장·편집 결과26개의 devel 기준 값이 비어 있는 증거 부족을 발견했다.
고정 devel의 보존된 producer로 Native26·WASM26·SVG6을 같은 입력·독립 Print·글꼴·DPI에서
추가 실행했다. [기준 TSV58개](../assets/issue_8109/stage4/devel/)와 기존 기준22개를 합친
[전쪽80행 비교](../assets/issue_8109/stage4/devel-comparison.tsv)는 쪽수1=1,
최대 하락0.00000%p, 0.1%p 이상 하락0쪽이다. 입력45개는 HEAD blob·실행 파일·SHA가 일치했다.
원장의 기준 값과 `base_tsv`를 채웠고 후보 TSV·PNG와 구현은 변경하지 않았다.

같은 Rust suite binary의 #8109 집중6개, Studio 집중24개와 실제 Chrome155 글꼴 준비 E2E를
다시 통과했다. 실제 outline/style·대체/원 face·늦은 local 갱신·선언 공유·새 결정·실패 fallback을
확인했고 대표 목록/네 style 확대 이미지도 독립 Print와 다시 판독했다. 최초127.0.0.1 접속은
WASM URL에서 HTML이 반환돼 초기화가 실패했으며, 기존 검증과 같은 localhost에서 정상
WASM 응답과 전체 E2E 성공을 확인했다. 이 초기화 실패를 제품 회귀 재현으로 세지 않았다.

`3cb689cc7`의 [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/39599138040/job/119219469661)는
실제 success였다. [CI preflight](https://github.com/edwardkim/rhwp/actions/runs/39599138040/job/119218942447)의
실제 판정은 `fast_pass=true`, candidate `dce8337ecdf34bcb9e2b3d8a69f046f44d1cc500`다.
전체 check는12 success·21 skipped, 실패·대기0이었다. 최신 devel은 같은 `80968497c`이며
merge-tree 종료0·head tree 일치를 확인했다. 광범위 Rust/Native 회귀는 정확한 녹색 code head의
위 CI와 기존 실행 증거를 재사용했다. 이번 증거·리뷰 기록만 한 번 trailing으로 반영하며,
문서 기록이나 다른 PR의 merge만을 이유로 code head를 반복 rebase하지 않는다.
새 trailing head의 required check 확인과 별도 사용자 merge 승인은 여전히 필요하다.

## 수용 범위와 남은 한계

새 PDF import의 명시적 대체와 유효한 저장 대체 필드의 실제 소비가 수용 범위다.
embedded-program 비적용 대조군의 기존 농도/폰트 차이는 남는다. 과거 무선언 파일의 PDF 출처 복원,
embedded font 렌더링 확장, 원문 계획의 PDF 표 추론 구현과 #8106–#8108/#8110은 완료 주장에 포함하지 않았다.
Native의 작은 PDF 약1.6ms 증가와 WASM90,438바이트 증가도 성능 보고에 유지했다. 성능 측정은 view 준비
반환까지이며 첫 paint/FPS·장시간 heap/GPU 메모리의 보편적 보증은 아니다.

## Merge 후 contributor PR comment 계획

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다.
병합·후속 게시가 별도 승인되고 asset의 devel 반영을 확인한 뒤에만 한국어 존댓말 본문을 UTF-8
`--body-file`로 게시한다. merge SHA·최종 CI URL·위 수치·수용 범위·남은 차이를 함께 알리고 API로 재조회한다.
대표 이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/issue_8109/stage3/body-list-typed-hwp-wasm-review.png`
및 같은 경로의 `body-list-typed-hwp-wasm-overlay.png`, stage4의 `studio-zoom.png`다.
현재는 merge/comment/issue close를 실행하지 않는다.
