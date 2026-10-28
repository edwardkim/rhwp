# PR #6996·#6999·#7007·#7011 누적 검토 및 검증 기록

## 통합 상태: 로컬 검증 완료, PR 제출

사용자가 추가 검증 후 PR 생성을 승인했다. 메인터너 보정은 `967b8d881d437d1254110b46e903a11edff29064`에 커밋했고 최신 `upstream/devel` `fad12365147f86ed01b7d76f3c90f02437b9b081`을 `2e6567618a2dbaa5c79b2dc86396ce84d38867e7`로 병합했다. 병합으로 추가된 것은 원 PR #6999의 메인터너 리뷰 문서 2개뿐이며, 검증한 제품·테스트·워크플로 소스에는 변화가 없었다.

검증 도중 #6999가 devel에 먼저 병합됐다. 이번 PR의 신규 통합 대상은 **#6996·#7007·#7011**이다. #6999 체리픽 이력은 보존하되 신규 수용·댓글·close 대상으로 중복 처리하지 않는다. 아래 네 건의 적용 기록과 최초 실패 기록은 검토 이력이다.

최신 결과: #7011 빈 문단 진단 오탐을 메인터너 보정하여 집중 114개 및 전체 회귀 9,468개가 모두 통과했다(기존 skip 46개). 기준선은 바꾸지 않았다. 아래 초기 실행 기록은 실패 이력으로 보존하며, 최신 판정과 보정 fingerprint·실행 시간·증적은 [#7011 최종 리뷰](pr_7011_review.md)를 따른다. 보정은 `967b8d881`에 커밋했고 추가 로컬 gate도 완료했다. 통합 PR의 원격 CI 완료를 뜻하지 않는다.

정정: 초기 JSON `page: 4`는 실제 5쪽이다. 아래 초기 기록의 4쪽 해석은 오류이며 최종 코멘트용 증적을 1·5쪽으로 교체했다. 5쪽의 빈 TextRun만 본문 overflow 검사에서 제외하고 표 우측 초과 신호는 유지했다.

2026-09-11, 검토자 `jangster77`. 이 문서는 원 PR별 기능 판정과 통합 브랜치의 제출 조건을 구분한다. 원 PR의 CI 성공을 통합 head의 CI 성공으로 간주하지 않는다.

- 검토 경로: `collaborator_external_pr` + `intake_and_review` + `multi_pr_update_branch` + `local_validation` + `visual_fixture_evidence`.
- 원 PR 네 건에 검토 시작 전 `jangster77` 리뷰를 요청했다. 기존 리뷰 요청은 유지했다. 통합 PR의 owner 자동 지정은 하지 않는다.
- 기준: `upstream/devel`의 `6806950b1ab57e6d97978b0986d63c689b12457e`.
- 브랜치: `review/planet6897-6996-7011-20260911`.
- 검증 head: `13f544e97f647003b73334c32fae31ee43b6affb`.
- 제품 코드 마지막 커밋: `5f554e802c2628a9b4b256d6d7f42b3b5b5760ca`.
- 마지막 `13f544e97`은 #7011 재현 HWP와 출처·해시 기록만 등록한 검토자 커밋이다.
- 원격 push, 통합 PR 생성, 원 PR 코멘트·close, 이슈 close는 수행하지 않았다.

## 적용 순서와 출처

| 원 PR | 원 커밋 | 로컬 체리픽 | 내용 |
| --- | --- | --- | --- |
| #6996 | `4066fb2676fc91698a1fbc4a956167d056754e64` | `15620f54e` | 저장된 물리 쪽 경계 보호 |
| #6996 | `bcf997f8713d5388ff164f309c44f39ed70c552b` | `ba2f57803` | 리뷰 지적 후 CellUnit 매핑·복수 경계 보완 |
| #6999 | `8bf8211df45fb8d4e293c97b89b91d857a20fc2f` | `34bbe46c5` | 쪽번호 컨트롤 중복 및 축 처리 |
| #6999 | `d563b48d761a4382106a00fd8daa90d7fb846222` | `1b9183c4d` | HWPX 원본 축 보완 |
| #6999 | `2b479c8b31735278afe2452c6740f26fba25eee9` | `91b439a49` | 원 PR 메인터너의 중첩 문단 상태 복원 |
| #6999 | `cd1a11dc06eff3f867c921c2a509b078cbf21871` | `97d16a4c6` | 원 PR 메인터너의 검증 기록 |
| #7007 | `9ed0354173aa6a55ae40e60e8deca69f46f6e6c0` | `73da73c19` | 큰 인라인 표의 위 바깥여백 |
| #7011 | `a0fa74b55a038ce316695234d89649e00128b5c5` | `5f554e802` | 글자처럼 취급 개체 줄의 최소 진행 높이 |

모두 `cherry-pick -x`로 출처를 보존했다. #6999의 devel 병합 커밋 `9c94c5fe61d2075104a3a622288eb3852d5507e8`은 중복 병합하지 않았다.

`src/serializer/hwpx/section.rs` 충돌은 최신 devel의 `PositionedMarkpens`와 빈 markpen 처리, 원 PR의 5요소 반환값을 함께 유지해 해소했다. 누적 적용 후 `section.rs`, `context.rs`는 #6999 최종 원격 head와 diff가 없었다. 원 PR에 이미 있는 `2b479c8b3` 보정을 이번 검토자가 새로 구현한 것으로 기록하지 않는다.

## 원 PR 최신 상태 재확인

네 건 모두 OPEN, non-draft, `MERGEABLE` / `CLEAN`이며 최종 재조회 head가 위 마지막 원 커밋과 같았다. 각 CI의 Build & Test, 실행된 기본 테스트 shard, Lint, CodeQL 언어별 분석, Adapter, Proptest는 성공했다. #6996·#7007·#7011의 Native Skia 및 Canvas visual diff도 성공했다. #6999 Native Skia는 SKIPPED였으며 Canvas visual diff 실행은 없었다.

| 원 PR | CI | CodeQL 언어별 분석 workflow | CodeQL 요약 check |
| --- | --- | --- | --- |
| #6996 | [34563120407](https://github.com/edwardkim/rhwp/actions/runs/34563120407) | [34563120434](https://github.com/edwardkim/rhwp/actions/runs/34563120434) | SUCCESS |
| #6999 | [34564676991](https://github.com/edwardkim/rhwp/actions/runs/34564676991) | [34564676946](https://github.com/edwardkim/rhwp/actions/runs/34564676946) | NEUTRAL |
| #7007 | [34552066468](https://github.com/edwardkim/rhwp/actions/runs/34552066468) | [34552066486](https://github.com/edwardkim/rhwp/actions/runs/34552066486) | NEUTRAL |
| #7011 | [34561866033](https://github.com/edwardkim/rhwp/actions/runs/34561866033) | [34561866054](https://github.com/edwardkim/rhwp/actions/runs/34561866054) | NEUTRAL |

`NEUTRAL`을 `SUCCESS`로 바꿔 기록하지 않는다. 병합 단계의 required check 정책 확인은 별도이며, 이 기록은 원 PR의 실행 결과 조회다.

## 초기 로컬 검증 이력 (보정 전)

검증 후 사용자 요청으로 HWP를 `samples/issue6928/148769979_274 반도체포럼 개최(이귀남 최종).hwp`, PDF를 `pdf/148769979_274 반도체포럼 개최(이귀남 최종).pdf`로 이름 변경했다. 아래 실행 명령과 실패 진단의 영문 sample 경로는 당시 실제 실행 경로를 보존한 것이다. 내용 변경 없이 이름만 바꿨으며, 이름 변경 이후 테스트는 재실행하지 않았다. 경로 기반 partition은 재실행 시 달라질 수 있다.

macOS, 검토 전용 `target/pr-review`, 테스트 스레드 8개. 기존 target이나 다른 worktree를 삭제하지 않았다. 새 sample 두 건을 보안 코퍼스 검사에 명시적으로 포함했다.

```sh
export RHWP_SECURITY_SWEEP_SAMPLES_JSON='["samples/issue6973/83818-appraisal-rules-amendment.hwpx","samples/issue6928/148769979-semiconductor-forum.hwp"]'

cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast \
  -E 'test(/issue_6973|issue_4068_clamped|issue_6869|issue_5943|issue_6956|issue_3820_p33|security_corpus_regression/)'

cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast

cargo fmt --all -- --check
node scripts/rust-test-suite-manifest.mjs --check
```

| 검증 | 결과 |
| --- | --- |
| 집중 테스트 | 29개 통과, 0개 실패, 실행 1.045초, exit 0 |
| 집중 테스트 구성 | #6996 3개, #6999 관련 16개, #7007 및 기존 #3820 p33 4개, 보안 코퍼스 6개 |
| 전체 기본 기능 회귀 | 9,464개 실행, 9,463개 통과, 1개 실패, 46개 skip, 실행 321.336초, exit 100 |
| 새 sample 보안 코퍼스 | hidden/injection/unicode 세 탐지기 검사 통과 |
| fmt | 통과, exit 0 |
| generated suite 정합성 | 통과: 1,256 sources, 5,328 static test attrs, 28 suites + 20 exceptions, 48 integration targets |
| 제품 소스 체리픽 후 공백 검사 | `git diff --check upstream/devel...HEAD` 통과 |

집중 빌드는 4분 27초 걸렸다. 전체 회귀 시간은 컴파일 시간을 포함한 성능 개선 수치가 아니다. 집중 실행 ID는 `13460846-4052-4e51-bcea-9c918ad4593f`이다. nextest의 `report-skipped` 미인식 경고는 있었지만 집중 실행 실패는 아니었다.

### 실패 한 건

`body_overflow_baseline::body_overflow_does_not_grow_partition_14`가 새 sample `issue6928/148769979-semiconductor-forum.hwp`의 baseline 없는 본문 하단 초과 1건 때문에 실패했다.

`rhwp layout-anomaly ... --json` 재조회 결과:

- 문서 5쪽, 해당 쪽 4.
- `Page/Body/Column0/TextLine8`: y=1013.3867px, h=20px, 본문 하단=1028.04px, `overBottom=5.3467px`.
- 같은 쪽 `Table7`의 `overRight=3.7733px`는 별도 우측 경계 신호다. 본문 하단 테스트 실패 1건과 혼동하지 않는다.
- 4쪽 PDF/PNG 대조에서 눈에 띄는 용지 밖 글자 잘림은 보이지 않았다. 노드 bbox의 하단 초과를 실제 글리프 잘림이나 #7011이 새로 만든 회귀로 단정하지 않는다.
- 새 sample로 드러난 기존 현상인지, 빈 줄 등 탐지 대상의 의미 문제인지, 제품 변경의 회귀인지 아직 분리되지 않았다. 동일 원본을 base와 candidate에서 비교하고 해당 TextLine 내용을 추적해야 한다.
- 기준선 상향, sample 제외, 테스트 무시로 통과시키지 않았다. 해소 후 해당 partition과 전체 회귀를 다시 실행해야 한다.

### 추가 병합 전 로컬 검증 완료

최초 리뷰 시 대기했던 항목을 아래와 같이 실제 실행했다. 원 PR CI로 대체하지 않았다. 검토 전용 `target/pr-review`를 사용하고 Cargo 작업은 순차 실행했다.

| 검증 | 결과 |
| --- | --- |
| Native Skia lib | 4,112개 통과, 기존 ignore 13개. rhwp 3,930 + 내부 crate 15/165/2 |
| Native Skia PNG placeholder | 2개 통과 |
| Native Skia p37 직접 PDF | 4개 통과 |
| 기본 Clippy | `--locked -- -D warnings` 통과 |
| WASM Clippy | `-p rhwp --lib --target wasm32-unknown-unknown -- -D warnings` 통과 |
| workspace build | `--locked --workspace` 통과 |
| workspace Clippy | `--locked --workspace --all-targets -- -D warnings` 통과 |
| WASM build | `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` 성공, 2분 49초 |

Native Skia는 `--features native-skia`로 lib 전체와 `run-rust-test.mjs`의 `issue_2225_missing_picture_placeholder`, `render_p37_direct_pdf_export`를 실행했다. 테스트 스레드는 8개였다. WASM은 Docker 없이 실행했고 wasm-opt까지 완료했다. wasm-bindgen 사전 빌드 바이너리 탐색 경고 후 cargo install 대체 경로로 성공했으며, 경고를 빌드 실패로 오인하지 않는다. 출력 `pkg`와 검증 로그는 PR에 포함하지 않는다.

## 시각 증적 생성 조건

- 실행 바이너리: `target/pr-review/release-test/rhwp`, 위 검증 head에서 집중 테스트와 함께 빌드.
- 바이너리 SHA-256: `e07607d564c2d689364bd2241e3027cc21e03179b5e0bf0a6c16488f3efdcc92`.
- `scripts/visual_sweep.py`, SVG font-style export, webfont 래스터화, PDF 96dpi 비교. Studio 직접 화면 캡처가 아니다.
- #6996 7~9쪽, #7007 33~35쪽, #7011 1·4쪽을 생성하고 직접 열어 대조했다.
- 전체 문서 수는 각각 rhwp/PDF 9/9, 215/215, 5/5쪽이다. summary의 선택된 페이지 수를 문서 총쪽수로 오인하지 않는다.
- 최종 PNG만 `mydocs/pr/assets/pr_6996_7011_planet6897_20260911/`에 보관한다. 로그, 중간 SVG·JSON·연락판은 `/tmp`에 두고 커밋 대상에서 제외한다.
- 새 한컴 PDF는 `pdf/148769979_274 반도체포럼 개최(이귀남 최종).pdf` 한 건이다. 나머지 신뢰 가능한 기존 PDF는 재사용했다.
- 자동 잉크 일치율은 높을수록 기준 이미지와 유사한 보조값이다. 사람 판정 정확도가 아니며, 픽셀 차이만으로 승인하거나 보류하지 않는다.

## 문서별 최종 결론 및 남은 작업

| 원 PR | 기능 검토 판정 | 제한 |
| --- | --- | --- |
| [#6996](pr_6996_review.md) | 승인 | 저장된 마지막 페이지 분리 범위. 전역 위치 차이·10k 자료 전체 재검증은 별도 |
| [#6999](pr_6999_review.md) | 승인 | 문단별 쪽번호·축·중첩 범위. 한컴 실물 5건 재변환은 이번에 미실행 |
| [#7007](pr_7007_review.md) | 승인 | 33~35쪽 위 여백 및 기존 계약 범위 |
| [#7011](pr_7011_review.md) | 승인 | 빈 문단 진단 오탐 보정 후 집중 114개·전체 9,468개 통과 |

로컬 기본 회귀·추가 gate는 완료했고 사용자 승인에 따라 PR을 제출한다. 신규 통합 대상 세 건의 head는 변하지 않았으며 #6999는 이미 devel에 병합됐다. 생성 후 통합 PR의 최종 head CI와 별도 병합 절차는 아직 남아 있다.

## 후속 코멘트 및 이슈 처리 계획

원 PR별 문서의 코멘트 계획을 사용한다. 지금은 게시하지 않는다. 향후 통합 PR 및 devel CI까지 성공한 경우 실제 merge SHA와 실행 결과를 넣고, 기존 동일 코멘트 유무를 확인한 뒤 UTF-8 body-file로 한 번만 게시하거나 기존 코멘트를 수정한다.

이미지는 링크만 나열하지 않고 `![설명](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6996_7011_planet6897_20260911/<PNG>)`로 본문에 직접 표시한다. `<merge-commit-sha>`는 실제 병합 SHA로 치환해야 하며 미치환 상태로 게시하지 않는다. 비교 방법은 저장소 `mydocs/manual/pr_review/visual_fixture_evidence.md`의 병합 SHA 고정 링크로 함께 안내한다. 원 PR을 직접 병합한 것인지 체리픽으로 수용한 것인지 구분하고, 이슈는 개별 해결 범위를 확인한 뒤에만 처리한다.
