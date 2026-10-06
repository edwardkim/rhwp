# #7084 Stage 2 — A 절편 공통 보충 메트릭

- 일자: 2026-09-13
- 계획: [승인된 구현계획](../plans/task_m100_7084_impl.md)
- 상태: **A 절편 구현·집중 15건·native Clippy 통과. B Studio 연결은 미착수.**
- 통합 기준: `upstream/devel`의 `70bf40af2a2818e72bd58b4fa66e2d4c06de2b51`.
  계획 보존 `1f33a7059` 후 `c71cea458`에서 충돌 없이 통합했다. #7106 창 제목 변경은 보존했다.

## 구현한 규칙

지정 폰트 DB에서 폭을 찾지 못하여 일반 `heuristicHalfwidth`로 떨어진 독립 scalar에만,
현재 렌더링 세션의 보충 자연 advance를 사용한다. 장평·자간은 이후 기존 계산에서 한 번 적용한다.
DB hit·공백/탭·반각 형태·전각·좁은 구두점·합성 PUA의 의미 있는 폭 규칙을 덮어쓰지 않는다.

- `supplemental_metrics.rs`: 문서/폰트/backend 세대, 정확한 크기·family·bold/italic 키,
  backend 실측과 검증한 font bytes/cmap/hmtx 근거를 구분한다.
- 일괄 등록: 잘못된 세대, 중복 키, 비유한/음수 진행폭, 자원 상한 초과를 거부하며 이전 자료는 유지한다.
  동일 내용 재등록은 기존 snapshot을 유지한다. 다른 자료 등록·세대 변경·세션 폐기는 이전 참조도 무효화한다.
- 상한: 4,096항목, UTF-8 key 총량 4MiB, font 32MiB/개·64MiB/세션.
  동일 hash의 bytes는 snapshot 안에서 공유한다. 레이아웃 루프에서 font I/O나 parsing을 하지 않는다.
- `ResolvedStyleSet`에서 `TextStyle`로 읽기 참조를 전달한다. 직렬화에서는 제외하며 원본 Document를 변경하지 않는다.
- 전체 폭, 문자 위치, 비반올림 폭과 trace가 같은 `char_width_decision`을 소비한다.
  기존 snapshot 없는 경로에는 추가 grapheme 분할/할당이 없다.
- 여러 scalar로 이루어진 VS16/ZWJ/결합 문자 묶음은 전체 기존 경로를 유지한다. 부분 폭 교체를 하지 않는다.

## 검증 구분

집중 테스트는 통제 provider의 독립 입력값을 이용한 **계약 검사**다.
자연 폭을 임의의 정방형이나 Stage 1의 특정 Windows 측정값으로 고정하지 않는다.
실제 브라우저/한컴 일치 검증은 B·C 절편에서 수행한다.

- 초기 `cargo check --locked --lib --target-dir target/pr-review`: 성공(34.19초).
- 최종 제품/테스트 SHA: `cccc33a360097ec5a9d7d4c11f5a09cf3c928ae3`.

### 최종 집중 검증

검증 전용 detached worktree `/home/edward/mygithub/rhwp-rust-review-7084-a`에서
위 SHA를 checkout했다. target은 기존 `/home/edward/mygithub/rhwp/target/pr-review`를
공유하여 순차 실행했다. generated suite/manifest는 해당 worktree에만 생성했고 커밋하지 않았다.

| 명령 | 결과 |
| --- | --- |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | 성공, 최종 새 source는 `regression_suite_003`에 자동 배정 |
| `cargo fmt --all -- --check` | 성공 |
| `node scripts/run-rust-test.mjs --cargo-test issue_7084_supplemental_metrics -- --target-dir /home/edward/mygithub/rhwp/target/pr-review` | **15 passed / 0 failed**, 실행 0.11초, 컴파일 포함 1분 05초 |
| `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` | 성공, 58.78초 |
| `node scripts/rust-test-suite-manifest.mjs --check` | 성공, 1,306 sources / 48 integration targets |
| `git diff --check` | 성공 |

15건은 다음을 검증한다: 전체 폭/문자 위치, 비이모지 미등록 glyph, DB hit·공백·탭·PUA 등의
기존 규칙, 장평·양수/음수 자간, 반복 대시, snapshot 교체, 크기·bold/italic·첨자 식별,
multi-scalar 전체 보류, 잘못된 입력/0 advance, 실패/동일 등록 원자성, key 총량 상한,
문서·폰트·backend 세대 변경, Canvas 실측의 backend 경계, 직렬화 제외/세션 폐기,
실제 Noto Sans KR의 cmap/hmtx·source identity.

### 첫 실행의 실패와 정정 근거

후보 `dc06a7202`는 13건 중 12건 통과, 1건 실패였다. 이는 새 테스트가 기존
`EmbeddedTextMeasurer::estimate_text_width`의 `total.round()`를 0.01px 반올림으로
잘못 가정한 테스트 오류다. 기존 API는 **정수 px 반올림**, 문자 위치는 raw px를 반환한다.
제품의 반올림을 변경하거나 기존 golden을 완화하지 않고, 새 테스트를
`전체 폭 == 마지막 문자 경계.round()`의 정확한 계약으로 정정했다.

함께 반복 대시의 기존 전용 폭 규칙을 보충 측정에서 제외하고, font 자료 동등 비교는
항목마다 전체 bytes를 다시 비교하지 않도록 이미 검증한 hash·face·glyph로 처리했다.
이 보완 후의 최종 SHA에서 위 검증을 다시 통과했다. 이 첫 실패를 제품 결함의 RED 증거로
보고하지 않는다. 브라우저 압축에 대한 실제 RED→GREEN은 B·C에 남아 있다.

## 아직 남은 연결과 보호 경계

현재 제품 세션에서는 보충 자료를 등록하지 않으므로 기본 출력은 아직 변경하지 않는다.
다음 B에서 fonts ready → 요청 수집/측정 → snapshot 등록/캐시 무효화 → 재조판 → 동일 paint를 연결한다.
검증한 font bytes의 존재만으로 backend가 그 font를 로드했다고 보지 않는다.
Canvas2D 실측값은 다른 backend에 등록할 수 없다. exact source의 paint capability 확인은 B에 남아 있다.
세대가 바뀌면 이전 참조는 적용되지 않지만, 이미 계산한 layout/paint 캐시의 폐기는 세션 소유자가 해야 한다.

font parsing/hash는 준비 단계의 API에서 수행한다. 반복 요청 중복 제거와 provider cache,
문서 스타일 주소·UTF-16 범위·paint descriptor 일치 검증, 세로쓰기 등 capability 제한은 B에서 연결한다.
이번 A의 단일 scalar 크기 키만으로 전체 문서의 cluster shaping 지원을 주장하지 않는다.
WASM/Studio 빌드 교체·전체 회귀·원격 push·PR·GitHub 댓글은 수행하지 않았다.
