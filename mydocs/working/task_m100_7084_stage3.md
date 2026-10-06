# #7084 Stage 3 — B 절편 측정 준비부 및 남은 적용 경계

- 일자: 2026-09-13
- 계획: [구현계획](../plans/task_m100_7084_impl.md)
- 상태: **B 실제 Studio 연결·Docker WASM·Chrome 실측 완료. 작업지시자 HWP/HWPX 시각 판정 통과. C 진입 승인.**
- 선행: [A 결과](task_m100_7084_stage2.md), 인계 `b9b271238`.
- 최초 준비부 제품/테스트 SHA: `eef7b3293f803ca3f8b702419c381f512e808b44`.
- 공통 설정 후속 제품/테스트 SHA: `2364c51f37fefde1b04b68e7c2509e6828659393`.

## 1. 이번에 구현한 범위

`rhwp-studio/src/core/supplemental-text-metrics.ts`는 Canvas2D의 일괄 측정 준비부다.
아직 WasmBridge/RendererSession에서 호출하지 않으며, Rust snapshot 등록도 하지 않는다.
기존 Studio 화면이 개선되었다는 뜻이 아니다.

- 문서·폰트 세대에 귀속된 cache를 소유한다. 폰트 준비 대기 중 세대 변경/폐기 시 늦은 결과를 버린다.
- `fontsReady` 완료 후 요청 descriptor를 **일반 Canvas setter**에 넣는다.
  presence probe용 raw setter로 Studio 대체 폰트 처리를 우회하지 않는다.
  설정 후 `context.font`를 읽어 실제로 해석된 descriptor를 함께 반환한다.
- 결과는 `canvas2d` / `backendMeasured`로 표시한다. 실제 fallback face를 식별한 `verifiedSource`라고 하지 않는다.
- Canvas가 잘못된 CSS font 입력을 무시하고 이전 font를 유지하는 경우를 두 초기 설정값으로 검출한다.
- NaN/Infinity/음수 advance, 중복 키, 여러 scalar cluster, 제어문자/단독 surrogate를 거부한다.
  정상 0 advance는 허용한다.
- 등록 후보 전체가 성공해야 cache를 교체한다. 중간 실패는 앞서 측정한 항목도 부분 게시하지 않는다.
- 동일 세대·키·문자·descriptor 재요청은 추가 Canvas 생성/측정 없이 cache를 쓴다.
  반환 자료 수정이 cache를 바꾸지 않도록 복사한다.
- 4,096항목·UTF-8 key 4MiB 상한을 요청 및 누적 cache에 적용한다. 큰 문자열은 encoding 전에
  UTF-16 길이로도 하한 검사를 하며, 해결된 descriptor도 cache 예산에 포함한다.

## 2. 코드 연결에서 확인한 경계

### 2.1 backend별 조판 문맥

`DocumentCore.styles`는 Canvas2D 전용이 아니다. `build_page_layer_tree_with_profile`의 결과는
Canvas2D, SVG, CanvasKit 및 PDF 경로에서 사용된다. `RenderProfile::Screen/Print`도 backend와 다르다.
여기에 Canvas 측정 snapshot을 항상 등록하면, 다른 backend가 같은 폰트 선택을 보장하지 못하면서도
브라우저 진행폭으로 만든 pagination/char positions를 사용하게 될 수 있다.

A의 `MetricContext.backend`와 등록 거부 검사는 **자료의 세대와 출처**를 보호한다.
그 자체로 이미 계산된 문서 코어의 공통 pagination/page-tree cache가 backend별로 분리되지는 않는다.
따라서 공통 스타일에 연결했다는 사실만으로 B의 backend 보호 요건을 통과시킬 수 없다.

기존 Studio `view/renderer-session.ts`에는 `documentRevision`, `resourceGeneration`,
`beginDocument`, `invalidateDocument`, `resolve`, `dispose`가 있다. 이 소유권을 활용하되,
실제 backend 변경/내보내기에서 **측정 문맥과 조판 cache가 함께 선택·무효화**되는 연결이 필요하다.
원본 Document 복사·저장 변경이나 단순 paint 단계의 scale 제거로 대신하지 않는다.

### 2.2 자연 advance와 축소 장평의 단위

`web_canvas.rs::draw_text_positioned`는 첨자를 적용한 뒤
`condensed_ratio_draw_params`로 실제 font size와 가로 배율을 구한다.
축소 장평에서는 기존 규칙에 따라 각각 √ratio를 사용한다.
반면 A의 `SupplementalMetric`은 첨자 적용 크기에서의 **장평 적용 전 자연 advance**를 기대한다.

최초 준비부의 `naturalAdvancePx`는 요청받은 **실제 CSS descriptor 크기에서의 측정값**이었다.
이를 A에 그대로 넣으면 축소 장평을 중복 적용할 수 있다. 다음 연결에서 공유 font 설정 결과를 통해
측정 크기와 기준 크기를 명시적으로 환산하고, 측정 키와 실제 paint descriptor의 정합을 검사해야 한다.
장평 100%의 emoji 한 개만 통과하는 조건을 전체 서식 통과로 간주하지 않는다.
이 단위 문제는 아래 5절에서 공통 설정·환산과 `measuredAdvancePx` 명칭으로 보완했다.

## 3. 실행 검증

위 source SHA와 같은 내용에서 실행했다.

```bash
cd /home/edward/mygithub/rhwp/rhwp-studio
node --test tests/supplemental-text-metrics.test.ts
npx tsc --noEmit
```

- 집중 테스트 **10 passed / 0 failed**, 128.22ms.
- 전체 프로젝트 TypeScript 타입 검사 성공.
- `git diff --check` 성공.

테스트는 통제 Canvas mock으로 독립적인 advance와 setter 치환/거부 동작을 제공한다.
실제 OS 폰트·브라우저·한컴 출력 일치의 증거가 아니다. 신규 WASM 빌드, browser 시각 판정,
전체 npm/Rust 회귀는 수행하지 않았다. 이 준비부를 아직 호출하지 않으므로 기존 렌더링 동작은 유지한다.

## 4. 최초 인계 당시 다음 순서 — 같은 B 절편의 잔여 작업

1. 공유 font 설정 결과에 실제 descriptor·그리기 크기·자연 advance 기준 단위를 묶는다.
2. RendererSession 수명과 Rust 측정 문맥을 연결하고 backend별 cache 적용/복원을 검증한다.
3. 문서에서 실제 누락 요청을 수집하여 준비부 → A snapshot → 재조판 → 같은 paint로 연결한다.
   편집으로 추가된 glyph·서식, 폰트 교체, SVG/CanvasKit 내보내기도 포함한다.
4. Docker WASM으로 두 원본 HWP/HWPX를 다시 열어 압축·뒤 문자·장평을 확인하고 작업지시자 판정을 요청한다.

B의 수용 기준을 충족하지 않았으므로 C 완료나 PR 준비로 넘어가지 않는다.
이번 turn에서는 별도 이슈·브랜치를 만들지 않았고 원격 push·PR·GitHub 댓글도 하지 않았다.

## 5. 후속 구현 — 공통 폰트 설정과 측정 단위

작업지시자의 다음 절차 승인으로 4절의 1번을 구현했다. `CanvasTextFont`를
`WebCanvasRenderer::draw_text_positioned`와 `SupplementalMetric::from_canvas_measurement`가
함께 사용한다. 기존 CSS font family 체인, bold/italic, 첨자 크기·baseline 이동,
축소 장평의 √ratio 규칙 및 CSS 소수 셋째 자리 표기를 보존한다.
원본 문서 스타일, 정적 DB, 폰트 배포·설치, 전역 fit 정책은 바꾸지 않는다.

### 5.1 단위와 적용 규칙

| 기호 | 의미 |
| --- | --- |
| M | 최종 CSS 크기에서 Canvas `measureText`가 반환한, 아직 가로 transform을 하지 않은 진행폭 |
| P | 기존 Canvas 그리기의 가로 배율. 축소 장평에서는 √ratio, 확대에서는 ratio |
| R | 공통 조판의 문서 장평 배율. 기존 계약대로 0 이하이면 1 |
| N | snapshot에 등록할 조판용 장평 적용 전 진행폭: **M × P / R** |

따라서 자간을 적용하기 전 `N × R = M × P`가 된다. CSS 크기의 반올림이나
실제 크기에 따른 폰트 메트릭 차이를 다시 nominal 크기로 환산해 없애지 않는다.
명목 크기에서 재측정한 폭이라는 주장이 아니라, **실제 그리기 진행폭과 일치하는 조판용 기준 폭**이다.
첨자는 이미 측정 descriptor 크기에 들어 있으므로 환산 과정에서 0.7을 다시 곱하지 않는다.

- TypeScript 필드를 `naturalAdvancePx`에서 **`measuredAdvancePx`**로 바꾸어
  요청 CSS 크기의 실측값과 Rust의 조판용 기준 폭을 혼동하지 않게 했다.
- 등록 시 요청 descriptor가 현재 스타일의 공통 설정과 정확히 같은지 검사한다.
  브라우저의 치환·정규화 후 읽은 descriptor는 별도로 보존하며 exact face로 승격하지 않는다.
- snapshot key에 장평을 포함했다. 같은 크기·문자라도 장평이 바뀌면 기존 CSS 측정을
  재사용하지 않고 미측정으로 돌아간다. 자간은 기존 공통 조판에서 후속 적용한다.
- 비유한 장평, 비정상 advance, CSS 표기상 0px가 되는 극소 크기는 등록하지 않는다.
  등록 거부를 기존 그리기 fallback까지 변경하는 근거로 삼지 않는다.

### 5.2 검증 기록

`2364c51f3`과 같은 제품·테스트 내용으로 전용 detached 검증 worktree에서 실행했다.
최초 후보 `547855d1d`에서 집중 테스트 파일의 포맷만 보완한 것이 `2364c51f3`이다.

- Rust 집중 테스트 **21 passed / 0 failed**, 실행 0.13초, 컴파일 포함 1분 30초.
  기존 A 15건과 공통 설정·장평/첨자 환산·반올림 보존·descriptor 불일치·장평 key·잘못된 입력 6건.
- Studio 집중 테스트 **10 passed / 0 failed**, 142.66ms; 전체 TypeScript 타입 검사 성공.
- 전용 worktree의 `cargo fmt --all` 및 `cargo fmt --all -- --check` 성공.
- 최초 루트 포맷 실행은 오래된 generated suite가 존재하지 않는
  `issue_6332_snapshot_budget_coupling.rs`를 참조해 실패했다. source 결함이나 회귀로 분류하지 않았다.
  루트 파생물을 임의 삭제하지 않고 전용 worktree에서 manifest를 준비해 검증했다.
- 격리 worktree에서 `natural_advance`의 환산만 제거한 음성 대조는 집중 검사 1건이
  의도대로 실패했다. 장평 64%, 측정 27.531px에서 기대 **22.0248px**, 오구현 **17.61984px**.
  단위 환산 누락을 실제 제품 측정 API 호출로 검출했으며 사용자 작업트리는 바꾸지 않았다.
  대조 후 원 구현으로 복원하고 `2364c51f3`과 제품·시험 파일이 같은지 대조했다.
- 복원 후 Rust 집중 **21 passed / 0 failed**, 0.13초(컴파일 31.47초).
- native Clippy `-D warnings` **성공**, 1분 00초.
- WASM32 lib Clippy `-D warnings` **성공**, 55.89초. Docker WASM 패키지 빌드와는 구분한다.
- 첫 manifest 최종 검사는 test source 포맷 후 길이 가중치가 바뀌어 harness drift를 검출했다.
  포맷이 끝난 source에서 review 전용 `--prepare`를 다시 실행해 자동 배정을 갱신했다.
  source test를 삭제하거나 expected 값을 바꾸지 않았다.
- 최종 manifest **1,306 sources / 48 integration targets** 검사 및 fmt check 성공.
  최종 자동 배정으로 집중 테스트를 다시 실행해 **21 passed / 0 failed**, 0.11초(컴파일 3.98초)를 확인했다.

주요 실행 명령(검증 worktree, target은 기존 공유 cache):

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
cargo fmt --all
# 포맷 후 source 크기 변화도 자동 배정에 반영
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check
cargo fmt --all -- --check
node scripts/run-rust-test.mjs --cargo-test issue_7084_supplemental_metrics -- \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
```

workspace build/all-target Clippy, 전체 Rust·Studio 회귀, Native Skia·Render Diff는 아직 미실행이다.
이번 검증을 PR 직전 전체 게이트 통과로 표시하지 않는다.

집중 테스트의 제어 provider 입력은 한컴 시각 정답지가 아니다. 아직 Docker WASM 재빌드와
실브라우저 시각 판정을 하지 않았으며, 사용자에게 emoji 압축 해결을 확인 요청할 단계가 아니다.

### 5.3 B의 남은 작업

4절의 2~4번이 남았다. 세션이 backend·폰트 세대를 소유하고 공통 pagination/cache를 함께
무효화하는 연결, 실제 누락 요청 수집·등록·재조판, 유효 측정과 동일 descriptor로 그리기,
Docker WASM 두 원본 검증 순서로 진행한다. 특히 기존 fit helper의 첨자 보정까지 그대로
적용하면 새 측정에 첨자가 이중 적용될 수 있으므로, 유효 보충 자료를 소비하는 paint 경계에서
같은 측정인지 확인해야 한다. **공통 설정 연결만으로 fit·세션 수명·타 backend 보호가 완료되지는 않는다.**

## 6. 코어 측정 세션·출력 경계

제품/테스트 후보: `0527ce2786` (`2498b605c` 세션 구현, `6bc64c0fc` 셀 서식 배치 보완 포함).

### 6.1 구현

- `DocumentCore`가 Canvas 측정 저장소를 소유한다. 등록과 활성화를 분리하며 기본 상태는
  비활성이다. 문서/폰트 세대가 바뀌면 이전 저장소를 폐기하여 외부에 남은 스타일의 자료도 무효화한다.
- 활성화·해제 및 활성 상태의 자료 변경은 기존 `rebuild_derived_state`를 통해 스타일·문단 구성·
  표/문단 측정·페이지네이션·페이지 트리를 함께 갱신한다. 같은 자료 등록과 같은 상태 선택은 재계산하지 않는다.
- 실제 WebCanvas의 페이지·부분 영역 그리기는 명시적인 Canvas layer-tree 진입점을 사용한다.
  일반 layer-tree, SVG, HTML 등 portable 출력은 Canvas 측정이 활성인 동안 오류로 거부한다.
  다른 출력은 페이지 수를 열거하기 **전에** portable 문맥을 선택해야 한다. 출력 실패를 숨겨 진행하지 않는다.
- 새 문서 설정/빈 문서 생성은 소유자를 해제한다. 편집 batch 중 세대 변경·자료 등록·문맥 전환은
  `EditInProgress`로 거부하여 편집의 지연 재조판과 섞이지 않게 한다.
- 셀 서식 배치가 해석된 스타일을 직접 교체하는 경로에도 snapshot을 유지했다.
  색상 변경으로 서식 ID가 달라져도 같은 폰트 측정은 유지하고, 다른 폰트/크기는 기존 값 기반 key로 구분한다.

이 절은 **Rust 코어에서의 세션 소유권과 출력 거부 경계**다. TypeScript 측정 준비부의 실제 호출,
WASM 등록 API, 누락 문자 요청 수집, Studio 내보내기 전후의 문맥 복원, paint descriptor/fit 정합은
아직 연결하지 않았다. 따라서 사용자 화면을 개선했다고 판단하지 않는다.

### 6.2 검증 및 정정

- 원본 `c-form-labnote-001-stage11-filled.hwp/.hwpx`를 그대로 사용했다. 새 합성 문서나 정답지 변경은 없다.
- 세션 집중 검사 4건이 통과했다: HWP/HWPX 활성화·해제, 세대 변경/늦은 결과 거부,
  문서 교체/편집 배치 보호, 셀 색상 변경 후 측정 유지.
- 활성화 시 실제 replay 좌표가 달라지고, 해제 시 원래 좌표와 SVG가 복구되는 것을 검사한다.
  전환 전후 전체 Document의 Debug 표현도 비교하여 원본 IR 불변을 확인했다.
- 첫 테스트 컴파일은 Document가 Serialize를 구현하지 않는데 JSON 직렬화를 호출하여 실패했다.
  제품 타입을 바꾸지 않고 기존 Debug 표현을 비교하도록 테스트를 정정했다.
- 첫 실행 2건 실패는 K0 런의 선택적 `layout_positions=None`을 비교한 테스트 오류였다.
  K0는 painter에서 스타일로 좌표를 계산하므로 그 경로를 사용하도록 정정했다. 기대 폭 완화는 없다.
- 격리 검증 worktree에서 문맥 전환의 `rebuild_derived_state` 호출을 제거한 음성 대조는
  **2건 실패**를 검출했다. 이전 폭의 마지막 좌표 32.5333px와 새 측정의 44.1694px가
  서식 변경 전후 섞이는 것을 검출했으며, 대조 후 원 구현으로 복구하고 Git 차이 0을 확인했다.
- 원 구현 복원 후 세션 **4 passed / 0 failed**, 0.15초(컴파일 28.74초), 기존 보충 측정
  **21 passed / 0 failed**, 0.12초(컴파일 4.45초).
- native Clippy `-D warnings` 성공, 1분 00초. WASM32 lib Clippy `-D warnings` 성공, 55.48초.
- 검증 worktree의 `cargo fmt --all -- --check`, manifest `--check` 성공:
  **1,307 sources / 48 integration targets**. 파생 suite/manifest/Cargo는 커밋하지 않았다.
- Studio 준비부 집중 **10 passed / 0 failed**, 136.45ms, `npx tsc --noEmit` 성공.
- 최종 검증 source는 `0527ce27864694ce21b425abba718abca60182de`와 일치한다.

검증은 5.2절과 같은 격리 worktree 방식이며 세션 집중 명령만 다음을 추가했다.

```bash
node scripts/run-rust-test.mjs --cargo-test issue_7084_canvas_metric_session -- \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review
```

workspace build/all-target Clippy·전체 회귀·Native Skia·Render Diff·Docker WASM·실브라우저
시각 판정은 이번 절편에서 실행하지 않았다. PR 직전 전체 게이트 완료로 보고하지 않는다.
임시 검증 worktree는 검사 후 제거하며, 원본 샘플과 공유 target cache는 유지한다.
원격 push·PR·GitHub 댓글은 수행하지 않았다.

## 7. 실제 Studio 연결 및 브라우저 검증

제품·테스트 SHA: `bdd63167ccad59d0560fd8a6a57e018057ba8c06`.
앞선 1~6절의 미연결 표시는 해당 절편 당시 상태다. 이번 절편에서 실제 연결까지 진행했다.

### 7.1 연결 내용

- Rust에서 본문·셀·글상자·캡션·머리말/꼬리말 등 문단을 순회하여 일반 추정 폭으로
  떨어지는 단일 scalar 요청을 수집한다. 기존 DB hit·공백·특수 조판 규칙은 요청하지 않는다.
  요청 수/bytes 상한과 순회 작업량 4MiB·깊이 128 상한을 두며 부분 성공을 게시하지 않는다.
- 요청은 문서/폰트 세대 외에도 요청 ticket과 문서 epoch·구역 revision·DPI에 묶는다.
  응답은 서버가 보존한 스타일과 대조한다. 부분·중복·다른 descriptor·편집 전 응답은 거부한다.
- WASM API와 Studio coordinator를 연결했다. 폰트 준비 후 일괄 측정·등록·문맥 선택을 수행한다.
  문서 해제/교체·늦은 view revision·폰트 세대 변경에서 이전 비동기 결과를 버린다.
  같은 세대의 기존 key는 측정 cache를 사용한다. 문단 순회 자체는 settled revision마다 수행한다.
- 일반 Canvas paint와 효과 paint에서 등록 descriptor와 실제 Canvas descriptor를 대조한다.
  유효 측정에는 잘못된 추정 폭에 맞추는 추가 fit을 적용하지 않으며, 기존 장평 transform은 유지한다.
  descriptor 불일치는 page render 오류로 반환한다. WASM을 강제로 throw해 Rust 정리를 건너뛰지 않는다.
- CanvasKit 선택/preflight 및 SVG·인쇄는 portable 문맥을 사용한다. 인쇄는 그 문맥에서 페이지 수·
  SVG·용지 정보를 함께 확보하고 화면 문맥을 복구한 뒤 비동기 UI 처리를 계속한다.
  문서 원본·정적 폰트 DB·저장 서식·폰트 배포는 변경하지 않았다.

### 7.2 실행 결과

| 검사 | 결과 |
| --- | --- |
| 실제 샘플 세션/요청 집중 | 7 passed, 0 failed; 실행 0.14초, 컴파일 포함 1분 39초 |
| 기존 보충 메트릭 집중 | 21 passed, 0 failed; 실행 0.12초 |
| Studio 준비·세션·인쇄 등 집중 | 25 passed, 0 failed; 145.35ms |
| 전체 TypeScript 타입 검사 | 통과 |
| 전용 worktree fmt check | 통과 |
| native / WASM32 lib Clippy | 순차 통과, 각각 1분 04초 / 1분 06초 |
| review manifest check | 1,307 sources / 48 integration targets 통과 |
| Docker release WASM | 최적화 포함 7분 58초, exit 0 |
| Windows Chrome 152.0.7977.83, Canvas2D | 두 원본·서식 변경·문맥 복원 통과; pageerror/측정 경고 없음 |

실제 전달된 WASM과 디스크 파일의 SHA-256은 모두
`20cc5759452bc2e7c245897a89f6019ec1f67a71a845cfece8fe2c8d9d6f97fd`이며 HTTP 200이다.
기존 `localhost:7700` dev 서버를 사용했다. 검증 탭만 생성/종료했고 기존 사용자 탭은 보존했다.

두 원본 모두 2쪽 `실물😀`에서 `supplementalBackendMeasured` 출처를 확인했다.
장평 100%에서 측정 진행폭 **18.302703857px**, 가로/세로 transform 비 **1.0**이다.
같은 문서·브라우저에서 측정 문맥만 끈 음성 대조는 **0.364244908** 압축을 재현했다.
다시 활성화하면 1.0으로 복구한다. 이 실측값을 제품 상수나 다른 OS의 golden으로 넣지 않았다.

원본을 변경 저장하지 않은 메모리 내 서식 대조도 두 포맷에서 통과했다.

- 장평 64%: 기존 √ratio 규칙에 따른 CSS 10.667px·가로 배율 0.8. 추가 fit 없음.
- 위첨자: CSS 9.333px·가로 배율 1.0. 첨자 크기를 두 번 적용하지 않음.
- 장평 120% + 굵게/기울임: CSS `italic bold 13.333px`·가로 배율 1.2.
- 폰트 세대 무효화 직후 비활성, 재준비 후 활성. 인쇄용 SVG 조회 후에도 화면 문맥 복원.

### 7.3 시각 자료와 판정 범위

재현 스크립트: `output/7084/studio-connected-check.mjs`.
실행: `node output/7084/studio-connected-check.mjs` (기존 CDP 19222·Vite 7700 필요).
실측/요청 출처·전달 asset 해시는 `output/7084/connected/runtime.json`에 남겼다.
그리기 전후 PNG는 `output/7084/connected/{hwp,hwpx}-{before,after}.png`다.
여기서 before는 같은 제품에서 새 측정 문맥을 비활성화한 통제 대조다.

한컴 PDF는 기존 `output/7084/oracle/stage11-filled-{hwp,hwpx}-2020.pdf`를 재사용했다.
그 PDF의 **2쪽을 96dpi로 다시 rasterize**하여 실제 Canvas 794×1123px와 비교했다.
새 SVG를 개선 화면으로 대신하지 않았다. 표준 `scripts/visual_sweep.py`의
`make_compares`·`make_overlay_compares`·`make_review_panels`를 그대로 재사용했다.
전체 sweep의 구조 후보 검출을 실행했다는 의미는 아니다.

각 포맷의 아래 경로에 판정 자료가 있다.

- `output/7084/connected/hwp/compare/compare_002.png`
- `output/7084/connected/hwp/overlay/overlay_002.png`
- `output/7084/connected/hwp/review/review_002.png`
- `output/7084/connected/hwpx/compare/compare_002.png`
- `output/7084/connected/hwpx/overlay/overlay_002.png`
- `output/7084/connected/hwpx/review/review_002.png`

두 포맷 모두 PDF 대비 pixel match 90.78317%, 내용 픽셀 중심 자동 일치율 보조값
`visual_accuracy_proxy_percent` **6.68869%**다. 페이지의 표 간격·선 위치 및 폰트/컬러 표현 차이가
남으므로 이 숫자를 이모지 수정 성공률이나 전체 한컴 호환 판정으로 해석하지 않는다.
동일 제품의 측정 비활성/활성 대조에서는 **172개 픽셀**, bbox **[71,250,85,262]**만 달랐다
(임계값 32). 이번 두 원본에서 표/문단 위치 차이는 새 측정 선택으로 생긴 차이가 아니다.

### 7.4 인계 및 잔여 검증

2026-09-13 작업지시자가 실제 Studio의 HWP/HWPX 이모티콘 렌더링을 **성공**으로 판정했다.
이 판정은 두 원본의 이모티콘 렌더링에 대한 것이며 페이지 전체의 한컴 동일성이나
모든 backend·복합 문자 지원을 승인한 것으로 확대하지 않는다. 이어서 C 통합 검증 진입을 승인했다.
제품/테스트 SHA는 `bdd63167ccad59d0560fd8a6a57e018057ba8c06`이며 이슈 완료는 아직 아니다.

C에서는 전체 Rust/Studio 회귀·workspace/all-target lint·Native Skia·Render Diff·비용을 검증한다.
정적/portable SVG·PDF 및 CanvasKit의 이모지 자체를 고친 것은 아니다.
복합 emoji(VS16/ZWJ), run 경계를 가로지르는 묶음·세로쓰기 등 지원 경계와 실패 후 재준비 경로도
전체 완료 선언 전에 점검한다. 이번 단독 emoji 실측을 그 범위의 검증으로 확대하지 않는다.
인쇄의 portable 문맥 확보는 동기식이며 큰 문서의 응답 지연은 아직 계측하지 않았다.
검증용 worktree만 제거하고 공유 target·원본·산출물은 보존한다. 원격 push·PR·댓글은 하지 않았다.
