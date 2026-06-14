# PR #1405 검토 — HWPX 직렬화 무손실 라운드트립 충실도

- PR: https://github.com/edwardkim/rhwp/pull/1405
- 제목: `fix(hwpx): HWPX 직렬화 무손실 라운드트립 — DocInfo·표·보조 항목 충실도`
- 작성일: 2026-06-14
- 작성자: `physwkim`
- 기여자 상태: `FIRST_TIME_CONTRIBUTOR`
- base: `devel`
- head: `physwkim:fix/hwpx-lossless-roundtrip` (`df487aa2e482d1ae42cc1ecea0866b838cea7e61`)
- 검토 브랜치: `review/pr-1405`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 한컴에서 저장한 HWPX를 `parse -> serialize`할 때 손실되던 DocInfo, 표, 보조 ZIP 엔트리,
`content.hpf`, header tail, numbering paraHead, font/typeInfo/substFont, border/fill 관련 속성을
대폭 보강한다. 변경 규모는 크지만 범위는 HWPX 라운드트립 충실도에 집중되어 있고, parser가 이미
보존하던 IR 값을 serializer가 누락하던 지점 또는 IR로 모델링하지 않는 원본 XML/ZIP 조각을 명시적으로
passthrough하는 지점으로 나뉜다.

first-time contributor의 대형 PR이므로 꼼꼼한 확인이 필요했지만, 로컬 전체 회귀와 GitHub Actions가
모두 통과했다. 특히 `hwpx_roundtrip_baseline`, `hwpx_roundtrip_integration`, `svg_snapshot`가 통과해
라운드트립 및 golden SVG 변경이 현재 테스트 계약과 맞음을 확인했다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| mergeStateStatus | `CLEAN` |
| 변경량 | 21 files, +1690 / -298 |
| 작성자 | `physwkim` |
| author association | `FIRST_TIME_CONTRIBUTOR` |
| maintainerCanModify | true |
| closing issues | 없음 |

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| Canvas visual diff | pass |
| WASM Build | skipped |

이 검토 문서와 오늘할일 커밋을 PR head에 push한 뒤 GitHub Actions를 다시 확인한다. CI 통과 사실만
문서에 반영하기 위한 추가 push는 하지 않는다.

## 3. 변경 검토

### 3.1 HWPX 원본 보조 항목 보존

`Document.hwpx_aux_entries`를 추가해 `version.xml`, `settings.xml`, `Preview/*`,
`Contents/content.hpf` 원본 바이트를 보존한다. serializer는 원본이 있으면 passthrough하고, HWP5 또는
synthetic document 경로처럼 원본이 없으면 기존 상수 fallback을 사용한다. 본문과 직접 결합되지 않는
metadata/preview/settings 손실을 줄이는 전략으로 적절하다.

### 3.2 header.xml 충실도

`DocInfo.hwpx_head_tail`, `DocInfo.hwpml_version`, `Font.subst_font`, `Numbering.raw_para_heads` 등
원본 HWPX에만 존재하던 세부 정보를 보존한다. `paraPr`, `charPr`, `borderFill`, `fontface` serializer는
parser가 보존한 bit/속성의 역매핑을 추가했다. raw splice 경로는 fallback을 유지하고 있어 HWP5 입력의
기존 생성 경로도 남아 있다.

### 3.3 표/section/시각 결과

`cellzoneList` 직렬화, table `pageBreak` 역매핑, `secPr tabStopVal/tabStopUnit`,
`xmlns:hwpunitchar` 선언을 추가했다. border width index/mm 매핑은 parser와 serializer가 단일 테이블을
공유하도록 바뀌었고, 이 영향으로 golden SVG 3건이 갱신됐다. `svg_snapshot` 통과로 golden 갱신의
결정성은 확인했다.

## 4. 로컬 검증

검토 브랜치: `review/pr-1405`

| 명령 | 결과 |
|---|---|
| `git diff --check upstream/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| `cargo build --release` | 통과, 1m 59s |
| `cargo test --release --lib` | 통과, 1787 passed / 0 failed / 6 ignored |
| `cargo test --profile release-test --tests` | 통과 |
| `wasm-pack build --target web --out-dir pkg` | 통과, 1m 14s |

`wasm-pack` 산출물 `pkg/`는 ignored 검증 산출물이므로 커밋 대상에 포함하지 않는다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| raw XML/ZIP passthrough가 편집 후 오래된 metadata를 유지 | 낮음 | 대상은 본문과 무관한 보조 항목 또는 모델 미표현 영역 |
| header tail/numbering raw splice의 구조적 취약성 | 중간 | 원본 경계가 없으면 fallback, 관련 unit test와 roundtrip test 존재 |
| border width 매핑 변경에 따른 시각 차이 | 중간 | golden SVG 3건 갱신, `svg_snapshot` 통과 |
| first-time contributor 대형 PR | 중간 | 로컬 전체 검증과 GitHub Actions 통과, maintainerCanModify true |

## 6. 권고

로컬 검증과 GitHub Actions 기준으로는 merge 준비 가능하다.

머지 전 마지막 확인:

- PR #1405 head에 이 검토 문서와 오늘할일 갱신 커밋을 push
- PR diff에 `mydocs/pr/archives/pr_1405_review.md`와 `mydocs/orders/20260614.md`가 포함됐는지 확인
- 문서 커밋 push 후 GitHub Actions 전체 통과 확인
- first-time contributor임을 고려해 merge 후 감사 코멘트에 검증 결과와 기여 범위를 구체적으로 언급
