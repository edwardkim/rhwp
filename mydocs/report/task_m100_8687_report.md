# #8687 목록형 원본 글꼴의 관측과 정상 대체

Issue: #8687

## 수정 결과

`산돌고딕 L\,한컴돋움` 같은 원본 이름이 관측 치환 key로 들어가 환경 전체를 거부하던
오류를 공통 요청 생산자에서 막았다. 이 원본 레코드를 본문이 실제 사용하는 경우에는
미해소된 쉼표 이름을 내부 CSS 체인으로 분해하지 않고 같은 레코드의 유효한 비내장 TTF 대체를
공통 style 결과로 전달한다. 독립 한컴 Print와 같은 `함초롬돋움` 출력으로 확인했다.
정상 단일 face 우선·이미 해소된 별칭·HFT·내장 프로그램·명시적 환경의 계약은 유지한다.
원본 레코드·직렬화·LineSeg·배치 수치·FontEnvironment 검증 허용치는 바꾸지 않았다.

- 초기 검증 기준 devel: `08963f6825956984d38b0d9392f44777db7107ab` (ff-only 동기화, divergence 0/0).
- 작업 브랜치: `task/8687-font-fallback-name-validation`.
- 관측 요청 보정: `86aaa4f4ffbc1d9cff84efd187f3130424c0844e`.
- 검증을 실행한 제품 source: `e4aefbcce7bf3db783ec39f3223e79b53082fbc1`.
- 최종 rebase 제품 source: `87c073fa7094603f6915c43851da7ed66697565b`; 최종 PR base `45684640be5eddcf5972b063260250345b34d928`.
- 선행 분석·RED와 중간 수정 근거: [계획](../plans/issue_8687_font_fallback_names.md).
- 입력·Print·전쪽 TSV·대표 PNG·실제 자원: [증적](../pr/assets/issue_8687/README.md).

## 원인과 공통 소비 경로

#8565 통합 `5551091085418c17d8321c0fee2626365911481b`의 원본 대체 관측과 현재 base를
대조했다. 당시·현재 `host_font_environment.rs`의 bytes가 같고, 문서 요청 생산자의
단일 face 검사와 선언 대체 관측 생산자의 검사가 불일치했다. #8692의 자기 참조 glyph 대체와는
원인이 다르다. 문서별 이름·수치·ID 분기를 추가하지 않았다.

`FontFace → declared_font_fallback_requests → prepare_declared_font_fallbacks_with /
WASM getDeclaredFontFallbackRequests → observed FontEnvironment`에서 같은 `valid_name`을 공유한다.
관측 key와 대체 모두 검증하므로 잘못된 한 선언이 정상 요청을 오염시키지 않는다.

실제 본문 소비는 `FontFace/substFont → try_lookup_font_name_decision →
normalized_face/css_family_chain → 언어별 char style·메트릭 → TextStyle.font_family →
Native declared_family_candidates / Canvas·SVG`다. 최초 guard만 수정했을 때 남던
쉼표 분해를 공통 style 생산자에서 보정했다. 측정과 표시가 같은 결과를 소비한다.

## 수정 전후의 실행 근거

| 검사 | 수정 전 | 수정 후 |
| --- | --- | --- |
| 최초 API 4건 | 기준 제품 1 PASS / 3 FAIL, 최소 환경에서도 동일 | 4 PASS |
| 실제 사용 중인 목록 이름 체인 | 첫 guard 수정 source로 5 PASS / 1 FAIL | 최종 source 6 PASS |
| 실제 Native screen export | 기준 종료 1, 환경 검증 오류, PNG 0개 | 두 재현 입력 모두 종료 0, 1쪽 PNG |
| used Native Print raw | guard만 적용 88.76426% | 91.21255% |
| used fresh WASM Print raw | 별도 기준 WASM 실측 없음; 기존 증거 재사용 승인 | 91.09617% |
| 정상 control Native | 91.10204% | 91.10204%, PNG bytes 동일 |

신규 검사는 합성 FontFace의 요청·원본 보존·실제 style 결정 API 관계 검사다.
설치 여부로 skip하지 않으며 특정 시스템 글꼴이나 실물 px/쪽수 핀을 추가하지 않았다.
기존 동일 원본 테스트·corpus 참조를 확인해 새 실물 golden을 중복 등록하지 않았다.
Native focused 추가 29건(#8153 2건, #6389 1건, PUA protocol 20건, #8533 6건)도 통과했다.

## 시각 판독

독립 Print는 원본 저장 세대에 맞는 실제 한컴 2024 `PrintMethod=0`, 1-up이다.
재현 입력·정상 대조군 모두 Native/fresh WASM 전쪽 90% 이상이고 PDF 쪽수와 일치한다.
본문 3줄·머리말·꼬리말·BOX의 소유·내용 순서·누락·중복을 확인했다.
같은 3배 확대에서 glyph·장평·자간·기준선·줄간격을 직접 대조했다. 수식·첨자는 없다.
BOX의 기존 높이·기준선 차이는 #8165 수용 기록 및 기준 control과 같으며 새 회귀가 아니다.

공통 producer를 소비하는 Native PNG·fresh WASM Canvas·SVG raster와 실제 Studio
Canvas2D/CanvasKit software 화면을 확인했다. Studio는 초기화 WASM bytes까지 확인했고
CanvasKit의 실제 선택·readiness 통과·자원 오류 0·미등록 글꼴 fallback 0을 기록했다.
독립 Print와의 일치는 screen backend끼리의 일치만으로 대체하지 않았다.

## 필수 최종 검증

실행 source는 `e4aefbcce7bf3db783ec39f3223e79b53082fbc1`이며 모든 아래 명령의 종료 코드는 0이다.
[원본 결과 요약](../pr/assets/issue_8687/validation-results.json)에 빌드 포함 시간과 수량을 보존했다.

| 검사 | 실제 결과 |
| --- | --- |
| 최소 글꼴 기본 집중 nextest | 32 PASS |
| 최소 글꼴 release-test 전체 nextest | 12,758 PASS, 0 FAIL, 기본 제외 45건, test 980.817초 |
| 최소 글꼴 Native 집중 | 신규 6 + 관련 29 = 35 PASS |
| Native Skia lib | 4,040 PASS, 기존 ignored 12 |
| Native Skia missing picture placeholder | 2 PASS |
| Native Skia direct PDF export | 22 PASS |
| 최적화 fresh WASM | 표준 locked wasm-pack + wasm-opt, 종료 0; pkg/public JS·WASM 해시 일치 |
| fmt / Native Clippy / WASM Clippy | 모두 PASS (`-D warnings`) |
| workspace build / all-target Clippy | 모두 PASS |
| manifest check (초기 base) | 2,098 sources, 48/48 integration targets, PASS |
| 실제 Studio 글꼴 E2E | 기존 assertion 11 PASS; 표시된 대체 글꼴 선택창을 처리한 실행 |
| 실제 Studio Canvas2D / CanvasKit software | 세 입력 × 두 backend; 실제 초기화 JS·WASM 신원·원본 본문 보존 확인 |

Cargo 명령은 공용 `target/pr-review`에서 직렬 실행했다. 최소 글꼴 검증은
[보존 wrapper](../pr/assets/issue_8687/minimal-font-profile.sh.txt)를 사용한다.
기본 전체 명령은 다음과 같다.

```bash
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
cargo test --locked --profile release-test --target-dir target/pr-review \
  --features native-skia --lib -- --test-threads 12
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --target-dir target/pr-review --features native-skia
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown \
  --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
```

편람은 최소 글꼴·자동 웹폰트(명시 font-path 없음)에서 기준·후보 모두 383/383쪽,
최저 raw 99.96465%, strict gate `passed`, 종료 0이다. 383쪽 PNG가 양쪽에서 모두
byte-identical이다. 전쪽 최저인 130쪽의 남은 차이는 작업지시자가 해결 불가능한 차이로
판정하고 PASS를 명시했다. raw와 기준 TSV는 그대로다.
[편람 전쪽 증적](../pr/assets/issue_8687/handbook/results.json)을 보존했다.

## 최신 devel rebase와 승인된 검증 재사용

`devel`을 `45684640be5eddcf5972b063260250345b34d928`으로 ff-only 동기화했고,
`devel...upstream/devel`은 0/0이다. 작업 브랜치의 두 수정 커밋을 이 base로 rebase했다.
충돌 0건이며 `git range-diff`에서 두 커밋 모두 `=`다.

| 이전 | rebase 이후 |
| --- | --- |
| `86aaa4f4ff` | `134b19d809` |
| `e4aefbcce7` | `87c073fa70` |

작업지시자의 `rebase, 충돌없으면 현재 테스트 자료 그대로 사용해도됨.` 지시에 따라
위 실행 source의 회귀·최적화 WASM·시각 결과를 재사용했다. 새 base의 추가 14커밋은
표·개체 점유 변경이며, 글꼴 제품 파일 4개와 새 검사 파일의 Git blob은 전후 모두 같다.
전체 제품 tree가 같다고 주장하거나 기존 PNG를 새 head의 재출력으로 기록하지 않는다.
[재사용 승인·SHA·range-diff·파일 blob 대조](../pr/assets/issue_8687/rebase-reuse.json)를 보존했다.

별도의 기준 WASM 빌드·수정 전 WASM TSV는 만들지 않았다. 수정 전 실제 Native export 오류와
정상 Native 기준·후보 대조, 후보 Native/fresh WASM의 독립 Print 전쪽 실측을 사용한다.
충돌 없는 rebase 뒤 기존 자료를 사용하도록 승인된 범위로 제출 자료를 정리했다.
기준 WASM 미측정을 통과한 검사나 실제 측정값으로 표시하지 않는다.

rebase 뒤에는 자동 suite를 재생성하고 최신 base에 대한 manifest check와 `git diff --check`를
확인했다. 새 회귀의 자동 수집을 확인했으며 파생 suite·manifest를 커밋하지 않는다.

## 원격 상태

이 보고서는 PR 생성 전 검증 기록이다. 본인 계정의 열린 PR은 게시 전 확인에서 없었다.
작업지시자가 `pr`로 upstream push·Open PR 생성을 명시 승인했다. 원격 생성 뒤 정확한
code head CI가 성공하면 정식 review·오늘할일을 해당 절차에 따라 추가한다. merge는
이 승인 범위에 포함하지 않는다.
