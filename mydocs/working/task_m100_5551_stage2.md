---
kind: report
status: active
canonical: mydocs/working/task_m100_5551_stage2.md
last_verified: 2026-09-11
---

# #5551 Stage 2: 구역 주소 잔여 결함 분석과 추가 보정 계획

- Issue: [#5551](https://github.com/edwardkim/rhwp/issues/5551)
- 선행 분석: [Stage 1](task_m100_5551_stage1.md)
- 기준 commit: `376c6b605c6be3b735bf6b8b9464fcd16b833a10`
- 1차 보정 기준 commit: `4198c6df4`
- 작업 브랜치: `fix/5551-caption-owner-20260911`
- **단계 판정: 추가 보정 및 PR 준비용 로컬 검증 통과. 전체 회귀는 PR 준비 단계에서 한 번 실행했다. 원격 PR 생성·CI 검증은 미실행이다.**
- 아래 1~6절은 1차 보정의 검증 기록이다. 추가 보정 이후의 통과 결과가 아니다.

## 1. 전체 회귀

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/review-5551-20260911 CARGO_BUILD_JOBS=8 \
  cargo nextest run --locked --cargo-profile release-test --tests \
  --test-threads 8 --no-fail-fast
```

실제 최종 결과:

```text
Summary [290.763s] 9478 tests run: 9478 passed (2 slow), 46 skipped
```

exit code는 0이다. 테스트 실행 시간은 약 4분 51초이며 컴파일 시간은 제외한다.
46개 skip을 실행·통과로 합산하지 않았다. 신규 캡션 테스트 5개와 기존 컨트롤 쿼리에
추가한 부분 주소 보존 assertion도 이 검증 대상에 포함됐다.

## 2. WASM 빌드

```sh
CARGO_TARGET_DIR=target/review-5551-20260911 CARGO_BUILD_JOBS=8 \
  scripts/wasm-pack-locked.sh --target web \
  --out-dir /tmp/rhwp-5551-validation/pkg
```

- release 컴파일: 2분 56초, 성공.
- wasm-bindgen 처리·wasm-opt를 포함한 빌드: 3분 16초, 성공.
- 플랫폼용 wasm-bindgen 다운로드 경고 후 설치 fallback을 거쳐 정상 완료했다.
- 기존 사용자 `pkg`와 실행 중인 Studio/Vite는 교체하지 않았다.
- WASM SHA-256: `a908d5ce030aab5bd8860b041b780e6616a46c71c81ee6937f9abdbb8f35c67a`

## 3. 실제 공개 API 검증 방법

임시 로컬 HTTP 서버와 격리된 Chrome을 사용했다. 새 WASM의 실제 `HwpDocument`에
HWP/HWPX 바이트를 로드하고 다음 메서드를 호출했다. 모의 WASM이나 반환값 대역은 사용하지 않았다.

- `getDocumentInfo()`와 `pageCount()`
- 모든 페이지의 `getPageRenderTree()`와 `getPageControlLayout()`
- 동일 페이지의 반복 조회 결과 비교
- `renderPageSvg()`와 `renderPageToCanvas()`
- Canvas 크기 및 실제 비백색 픽셀 존재 확인

캡션 검증은 문자열·좌표 추정이 아니라 반환된 소유 주소와 같은 페이지의 컨트롤 주소를
직접 대조했다. DOM 캡처는 Canvas 영역만 포함하며 PC 전체 화면을 촬영하지 않았다.
문서별 대표 페이지를 렌더링했으며 모든 페이지를 한컴 PDF와 시각 대조한 것은 아니다.

## 4. 문서별 실제 결과

| 입력 | 구역 / 페이지 | 컨트롤 | 캡션 줄 | 주소 대조 결과 |
| --- | --- | --- | --- | --- |
| 3구역 합성 HWPX | 3 / 5 | 18 | 24 | 표·그림·도형 모두 일치, 여러 줄 캡션 8그룹 확인 |
| `samples/hwp3-table-caption.hwp` | 1 / 1 | 1 | 1 | 표 캡션 일치 |
| `samples/issue6284/child_policy_top_caption_charts.hwpx` | 1 / 34 | 206 | 16 | 그림·표 캡션 일치, 여러 줄 캡션 4그룹 확인 |
| `samples/issue5802/hf_cross_section_inherit.hwp` | 2 / 2 | 4 | 0 | 꼬리말 그룹 2개에서 문서 구역 범위 밖 `secIdx` 발견 |

총 42페이지에서 메타데이터를 조회했다. 세 문서의 캡션 줄 41개 모두 같은 페이지의
컨트롤 주소와 연결됐다. 알려진 `secIdx` 키의 누락은 이 네 입력에서는 없었다.

합성 문서는 기존 회귀 테스트의 IR 입력을 현재 native 코드로 HWPX 직렬화한 것이다.
마지막 구역은 컨트롤이 없는 문단으로 구성했다. `sectionCount=3`이지만 컨트롤 및
캡션 소유 구역은 `[0, 1]`인 것이 의도한 정상 결과다.

- 합성 HWPX SHA-256: `50ed147dca9df7cf282b86e2802f388dafd46b837456dea47ca7497b758ea62a`
- 한컴 원본·PDF 정답지로 간주하지 않는다.

## 5. 실제 동작에서 발견한 잔여 결함

`hf_cross_section_inherit.hwp`는 `sectionCount=2`인데 두 페이지의 꼬리말 그룹이
다음 값을 반환했다.

```json
{
  "type": "group",
  "secIdx": 4294967295,
  "paraIdx": 0,
  "controlIdx": 0,
  "stableIndex": [4294967295, 0, 0]
}
```

원인은 `src/renderer/layout.rs`의 `layout_header_footer_paragraphs`에서 TAC 도형의
인라인 좌표 등록·조회 키를 본문과 구분하기 위해 `usize::MAX`를 사용하는 데 있다.
`src/document_core/queries/rendering.rs`의 Group 직렬화는 이 내부 값을 실제 문서
구역처럼 그대로 방출한다. WASM에서는 `usize::MAX`가 `4294967295`다.

이 값은 렌더 내부에서 의도된 센티널이지만 **문서의 실제 구역 번호는 아니다**.
필드 존재 여부만 확인한 첫 실행은 이를 놓쳤다. 이후
`0 <= secIdx < sectionCount` 검사를 추가한 실행은 이 두 컨트롤 때문에 exit code 1로
실패했다. 이를 정상 구역이나 expected skip으로 재분류하지 않았다.

### 후속 보정 시 지켜야 할 경계

- 내부 인라인 등록·조회 키를 곧바로 `0`이나 현재 페이지 구역으로 바꾸지 않는다.
  본문 캐시 충돌과 상속된 머리말·꼬리말의 잘못된 소유 주소를 만들 수 있다.
- 실제 소유 구역은 Header/Footer 참조의 `source_section_index`에서 전달해야 한다.
- 그룹의 내부 배치 식별자와 공개 문서 주소를 분리하고, `stableIndex` 및 편집·선택 경로의
  호환성을 함께 확인해야 한다.
- 이번 전체 회귀 통과를 근거로 이 잔여 결함까지 해결됐다고 판단하지 않는다.

## 6. 증적과 미실행 범위

임시 실행 증적은 `/tmp/rhwp-5551-validation/`에 보관했다.

- `full-regression.log`: 전체 회귀 최종 summary.
- `wasm-build.log`: 실제 새 WASM 빌드 결과.
- `browser.mjs`, `browser-results.json`, `browser.log`: 공개 API 검증과 실패한 범위 검사.
- `fixture.rs`, `multisection-caption.hwpx`: 합성 입력 생성 근거.
- `browser-synthetic.png`: 캡션이 있는 대표 페이지의 실제 Canvas 캡처.

로그·임시 스크립트·중간 JSON은 커밋 대상이 아니다. 이번 실행은 Studio E2E 전체,
Native Skia 별도 feature 테스트 또는 모든 문서의 한컴 PDF 시각 대조를 수행한 것이 아니다.
원격 push, 이슈 close 및 PR 생성은 수행하지 않았다.

## 7. 추가 보정 전 분석

### 7.1 공개 주소와 내부 배치 키가 섞이는 경계

`layout_header_footer_paragraphs`는 본문과의 캐시 충돌을 피하려고 머리말·꼬리말 TAC
도형의 구역 키에 `usize::MAX`를 사용한다. 등록 단계와 조회 단계가 같은 값을 써야
하므로 이 키 자체를 실제 구역 번호로 바꾸는 방식은 채택하지 않는다.

머리말·꼬리말을 페이지에 배치하는 `build_header`/꼬리말 경로에는 이미
`HeaderFooterRef::source_section_index`가 있다. 상속된 꼬리말은 현재 페이지 구역이
아니라 원본 컨트롤을 정의한 구역을 사용해야 한다. 현재
`propagate_header_footer_ref`는 그림의 주소만 정규화하므로 그룹·도형에서는 이 정보가
공개 쿼리까지 전달되지 않는다.

### 7.2 구역 번호만 바꿀 때 생기는 편집 위험

Studio의 `input-handler-picture.ts`는 일반 도형·그룹의 `secIdx`, `paraIdx`,
`controlIdx`를 본문 도형 속성·이동·삭제 API에 전달한다. 머리말·꼬리말 그림에는
별도의 `headerFooter` dispatch가 있지만 도형·그룹에는 같은 편집 경로가 없다.

따라서 그룹의 `secIdx`만 `0`으로 바꾸면 subList 문단 주소가 본문 주소로 오인되어
다른 개체를 대상으로 편집할 위험이 있다. 구역 필드 하나를 치환하는 보정은 하지 않는다.

## 8. 선택한 추가 보정 방향

1. 렌더 노드의 내부 주소·인라인 캐시 키와 별개로 머리말·꼬리말 원본 참조를 전달한다.
2. 공개 컨트롤 쿼리는 이 참조의 실제 소유 구역을 사용하고 `headerFooter` 문맥을 함께 노출한다.
3. 내부 등록·조회와 페인트 정렬에 사용하는 키는 유지한다. `stableIndex`를 본문 구역
   분포에 그대로 사용하는 것이 가능한지 문서화하며, 공개 주소와 정렬 키를 혼동하지 않는다.
4. Studio에서 아직 지원하지 않는 머리말·꼬리말 도형 편집을 본문 편집으로 잘못 dispatch하지
   않도록 명시적으로 구분한다. 기존 머리말·꼬리말 그림의 지원된 편집 경로는 유지한다.
5. 캡션 소유 정보도 내부 센티널을 실제 구역으로 방출하지 않도록 점검한다.

## 9. 추가 보정 수용 기준

- `hf_cross_section_inherit.hwp`의 두 페이지에서 범위 밖 `secIdx`가 없어야 한다.
- 상속된 꼬리말은 현재 페이지가 아닌 실제 원본 구역에 연결되어야 한다.
- 내부 센티널 키를 바꾸지 않아 TAC 도형 등록·조회와 렌더 결과가 유지되어야 한다.
- 머리말·꼬리말 그룹이 본문 도형 편집 API로 전달되어서는 안 된다.
- 기존 그림의 머리말·꼬리말 문맥과 정상 본문 컨트롤 주소가 유지되어야 한다.
- 캡션 집중 테스트, 새 구역 주소 회귀, 전체 회귀, 새 WASM 및 Chrome 공개 API 검증을
  보정 후 다시 수행한다. 현재의 실패를 assertion 제거나 expected skip으로 넘기지 않는다.

## 10. 커밋과 단계 순서

1. `4198c6df4`: 완료된 1차 보정과 Stage 1 기록. Stage 2 문서는 이 커밋에 포함하지 않는다.
2. 현재 Changes에서 Stage 2 분석을 작성한다. 분석 문서만 먼저 커밋하지 않는다.
3. 같은 회차에서 추가 코드 보정과 테스트를 수행한다.
4. 분석 당시의 사실과 실제 실행 결과를 구분하여 결과보고를 작성한다.
5. 해당 회차의 분석·코드 수정·결과보고를 함께 커밋한다.
6. 다음 분석은 그 커밋 이후의 새 Changes에서 시작한다.

작업 순서는 **분석 → 코드 수정 → 결과보고 → 커밋 → 분석 → 코드 수정 → 결과보고**다.

7~9절은 추가 코드 수정 전의 분석이다. 아래는 실제 추가 보정과 검증 결과이며,
1~6절의 이전 코드 검증 기록과 구분한다.

## 11. Stage 2 코드 보정 결과

- `RenderNode.header_footer_source`에 실제 원본 구역과 머리말·꼬리말 참조를 보존한다.
  이 내부 메타데이터 자체는 일반 렌더 트리 JSON에 직렬화하지 않는다.
- `getPageControlLayout()`은 해당 참조로 `secIdx`를 출력하고 `headerFooter`에 원본
  머리말·꼬리말 컨트롤 위치를 함께 제공한다. 내부 캐시 주소와 `stableIndex`는 바꾸지 않는다.
- Studio의 본문 선택·연결선·표/도형 위치 조회는 머리말·꼬리말 내부 주소를 본문 주소로
  오인하지 않도록 구분한다. 전용 편집 경로가 있는 직접 머리말·꼬리말 그림은 유지한다.
- 현재 `CaptionOwner`는 본문 컨트롤 주소 구조다. 머리말·꼬리말 subList 캡션에는
  내부 센티널이나 잘못된 본문 주소를 넣지 않고 소유 필드를 생략한다. 이 추가 주소 구조의
  지원까지 완료했다고 주장하지 않는다.
- 테스트 추가 과정에서 Node의 확장자 없는 TypeScript import 오류가 발생했다.
  해당 import를 보정한 뒤 집중 테스트와 Studio 전체 테스트를 재실행해 통과했다.

## 12. Stage 2 검증 결과

검증 대상은 `4198c6df4` 이후 현재 작업 트리의 Stage 2 코드다. 아직 커밋·push하지 않았다.
기존 사용자 `pkg`와 Studio/Vite는 교체하지 않고 전용 target과 임시 WASM 패키지를 사용했다.

| 검증 | 실제 결과 |
| --- | --- |
| Rust 캡션·구역 주소 집중 테스트 | 6개 통과, 실패 0, 8 thread 설정. 전체 회귀 실행이 아님 |
| Studio 선택·연결선 집중 테스트 | 4개 통과 |
| TypeScript `npx tsc --noEmit` | exit 0 |
| Studio `npm test` | 1,661개 통과, 2개 skip, 실패 0 |
| WASM release 빌드·wasm-opt | exit 0, 총 2분 37초 |
| 새 WASM의 Chrome 공개 API 검증 | exit 0, 4개 입력·42페이지 |
| 캡션 소유 컨트롤 대조 | 41줄 모두 일치, 미연결 0 |
| 컨트롤 구역 주소 | 누락 0, 문서 구역 범위 밖 값 0 |
| 보정 전후 페이지 SVG | 42페이지 모두 SHA-256 일치 |

실제 `HwpDocument`에 입력 바이트를 로드하고 `getPageRenderTree()`의 `captionOwner`를
같은 페이지의 `getPageControlLayout()` 주소와 대조했다. 여러 줄 캡션은 같은 소유 주소와
ordinal을 유지하며 반복 조회 결과도 같았다. 대표 페이지의 SVG와 Canvas 출력도 실행했다.
SVG 해시 일치는 이번 메타데이터 보정으로 렌더 출력이 바뀌지 않았다는 근거이며,
한컴 PDF와 전체 시각 대조를 했다는 뜻은 아니다.

| 입력 | 페이지 | 캡션 줄 | 결과 |
| --- | --- | --- | --- |
| 3구역 합성 HWPX | 5 | 24 | 표·그림·도형 주소 일치, 컨트롤 없는 마지막 구역에 주소를 꾸며 넣지 않음 |
| `samples/hwp3-table-caption.hwp` | 1 | 1 | 실제 HWP3 표 캡션 주소 일치 |
| `samples/issue6284/child_policy_top_caption_charts.hwpx` | 34 | 16 | 실제 HWPX 그림·표 캡션 주소 일치 |
| `samples/issue5802/hf_cross_section_inherit.hwp` | 2 | 0 | 두 꼬리말 그룹의 `secIdx=0`, 실제 footer 원본 참조 노출 |

마지막 입력의 꼬리말 그룹은 공개 주소가 `secIdx=0`으로 바뀌었지만 정렬 키는
`stableIndex=[4294967295,0,0]`을 유지했다. 네이티브 집중 테스트는 빈 구역을 앞에
추가한 경우에도 두 그룹이 실제 원본 구역 `1`을 가리키는지 확인했다.

임시 증적은 `/tmp/rhwp-5551-validation/`의 `stage2-focused.log`,
`stage2-typescript.log`, `stage2-studio.log`, `stage2-wasm-build.log`,
`stage1-browser-baseline.json`, `stage2-browser-results.json`, `stage2-browser.log`에 있다.
이 로그·중간 JSON·임시 검증 스크립트는 커밋하지 않는다.

### 집중 검증 시점의 미실행 및 다음 게이트

다음은 12절 집중 검증 당시 상태다. 이후 PR 준비 단계의 실제 실행 결과는 13절에 기록한다.

- **전체 회귀는 이번 Stage 2에서 실행하지 않았다. 사용자 지시에 따라 PR 직전에 한 번 수행한다.**
- Native Skia 별도 feature 검증, Rust Clippy 전체 묶음, Studio 전체 브라우저 E2E는 이번 실행에 포함하지 않았다.
- Studio 선택·연결선 방어 로직은 Node 집중 테스트로 검증했다. 실제 Chrome 검증은 새 WASM의
  PageRenderTree·컨트롤 주소·렌더 출력 범위이며 Studio UI의 클릭·삭제 E2E까지 수행한 것은 아니다.
- 원격 push, PR 생성, merge, 이슈 close는 수행하지 않았다.

## 13. 커밋 후 PR 준비 검증

사용자 지시에 따라 Stage 2 코드·결과보고와 절차 보완을 먼저 `888725201`로 커밋한 뒤
PR 준비 검증을 수행했다. 아래 결과는 이 코드 commit을 대상으로 하며 이후 소스 수정은 없다.
이 절의 결과 기록은 문서 변경만 해당한다.

### 적용 경로와 범위

- base route: `collaborator_self_merge.md`.
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`,
  `rework_and_exceptions.md`의 1,000줄 초과 PR 경계.
- 본인 PR 준비이므로 reviewer를 자동 지정하지 않는다.
- 최신 `upstream/devel`을 fetch했고 기준 tip은 `d408532ce`였다.
  `git merge-tree --write-tree HEAD upstream/devel`은 exit 0으로 텍스트 병합 충돌이 없었다.
  작업 브랜치에 rebase/merge하지는 않았으므로 전체 회귀 대상은 `888725201`이며
  최신 devel을 합친 가상 후보의 런타임 검증이라고 표현하지 않는다.

### 실제 검증 결과

| 항목 | 결과 |
| --- | --- |
| `cargo fmt --all -- --check` | 통과 |
| native root Clippy (`-D warnings`) | 통과 |
| WASM32 lib Clippy (`-D warnings`) | 통과 |
| workspace build | 통과 |
| workspace all-target Clippy (`-D warnings`) | 통과 |
| 파생 suite 정합성 | 재준비 후 통과: 1,259 sources / 28 suites + 20 exceptions |
| source-side test 계약 | 통과: 4,205 tests / 298 modules, 기준선 상향 없음 |
| Native Skia lib | rhwp 3,930개 통과·13개 ignored, 함께 실행된 workspace lib 182개 통과 |
| Native Skia placeholder | 2개 통과 |
| Native Skia direct PDF | 4개 통과 |
| default-feature 전체 회귀 | **9,479개 통과·46개 skip·실패 0**, 8 thread, 한 번 실행 |
| Markdown 내부 링크 | CONTRIBUTING·Stage 1·Stage 2 세 문서 통과 |
| whitespace | `git diff --check upstream/devel...HEAD` 통과 |

전체 회귀 명령과 최종 요약:

```sh
CARGO_TARGET_DIR=target/review-5551-20260911 CARGO_BUILD_JOBS=8 \
  cargo nextest run --locked --cargo-profile release-test --tests \
  --test-threads 8 --no-fail-fast
```

```text
Summary [365.225s] 9479 tests run: 9479 passed (2 slow), 46 skipped
```

365.225초는 테스트 실행 시간이며 빌드 시간을 포함하지 않는다. 마지막 대형 표 테스트까지
완료하고 exit 0을 확인했다. skip·ignored는 통과 수에 합산하지 않았다.

처음 suite 계약 검사에서 28개 generated harness drift가 보고됐다. 소스나 baseline을
완화하지 않고 `node scripts/rust-test-suite-manifest.mjs --prepare`로 검증용 하네스를
다시 준비한 뒤 suite 계약·포맷 검사를 통과했다. generated 파일은 커밋하지 않는다.

새 WASM·Chrome의 PageRenderTree 검증, TypeScript·Studio 전체 테스트 결과는 12절을
재사용한다. 그 검증 뒤 제품 코드 변경은 없었다. 새 WASM SHA-256은
`96a7e033501d3765f6ea11626755d7ef1dd277da8940e7b53d9e2d912a2b3dd7`이다.

### PR 준비 상태

- 해결 대상은 #5551의 본문 캡션 소유자 계약과 함께 요청된 알려진 구역 주소 보존이다.
- 머리말·꼬리말 subList 캡션 소유 구조 확장과 Studio 전체 클릭·삭제 브라우저 E2E는
  이번 완료 범위로 주장하지 않는다. 지원하지 않는 소유 주소는 추정 없이 생략한다.
- 사용자-visible 페이지 배치 변경을 주장하지 않으며 새 기준 PDF나 visual sweep을 추가하지 않았다.
  42페이지 보정 전후 SVG 동일성과 실제 Canvas 렌더 실행을 메타데이터 보정의 무변경 근거로 사용했다.
- PR 제목·본문 초안은 `/tmp/rhwp-5551-validation/pr-body.md`에 준비한다.
  아직 없는 PR 번호로 `pr_N_review.md`나 오늘할일을 미리 만들지 않는다.
- remote push와 PR 생성 뒤 채번된 번호로 self-review·오늘할일을 같은 PR의 trailing commit에
  추가하고 최신 head CI를 확인해야 한다. 이번 요청에서는 원격 변경을 수행하지 않았다.
- 검증 로그와 중간 산출물은 `/tmp/rhwp-5551-validation/pr-*.log`에만 보관하고 커밋에서 제외한다.
