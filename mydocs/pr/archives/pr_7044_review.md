---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-12
---

# PR #7044 — 최초 재현 및 수정본 재검토

> 최신 상태: 2026-09-12 재검토 중 collaborator `jangster77`이 수정 head
> `69187adbefb92321a02bf99d0751c4f063a91d74`를 병합했다.
> 병합 커밋 `78bdfd9aa69417b4a88ffdd43aa76d3cad471f44`, 시각 12:16:17 KST.
> 아래 최초 검토와 보류 판정은 이전 head에 대한 이력이며, 최신 재검토는 마지막 절을 따른다.
> 최종 판정: **승인**. 2026-09-12 메인테이너가 수정본 시각 판정 통과를 확정했다.

## 범위와 현재 판정

- 요청: 기여자가 주장한 샘플 현상이 실제 현재 제품에서 발생하는지 독립 확인.
- 경로: `maintainer_general` + `intake_and_review` + `local_validation` + `visual_fixture_evidence`.
- PR: https://github.com/edwardkim/rhwp/pull/7044 — 관련 이슈 #7018, 기여자 `planet6897`.
- 검토 head: `d2cbc46c18dbb98454c034753f45c1ccf6561239`.
- 현재 `upstream/devel` 및 재현 WASM source: `d14f44eba7035a5604acf9c236b81a294722691f`.
- **증상 재현 확인. PR 최종 판정은 머지 보류(수정본 독립 검증·회귀 검증 미실시).**
- 메인테이너 시각 판정을 대신하지 않는다. 이번 단계에서 원격 댓글·review·merge·push는 하지 않았다.
- reviewer `edwardkim` 지정 완료. `gh pr edit`의 classic project GraphQL 오류는 `gh api`의 requested_reviewers REST 경로로 처리했다.

## 원본과 기준 PDF

PR에 포함된 파일을 별도 review worktree에서 그대로 사용했다. 원본 변형·PDF 재생성·중복 복사는 하지 않았다.

| 항목 | 원본 HWPX | 기준 PDF |
|---|---|---|
| 경로 | `samples/issue7018/2769535-records-inspection-plan.hwpx` | `pdf/2769535-records-inspection-plan-2020.pdf` |
| SHA-256 | `3210d66f119a38e44764e0e29780d8140bc53b51f00921356fa99f6c41996f2a` | `9b3b225082affc521ff7a8561d50c6f8c6a31fd39d092eb1595f1bedd6c41cd2` |
| SHA-1 | `c840f5168ea3b32643d1e332170abcc4078c678c` | `cd26939f0716f53e7ee1338ec68328d1eb3422c5` |

PDF는 2쪽, 595×841pt(A4), PDF 1.4, Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`이다.
원문과 2쪽 표·제목 대응을 확인해 한컴 기준 자료로 재사용했다.
기여자가 기록한 MCP `engine_profile=2020`과 실제 PDF Creator 제품 표기는 구분해야 한다.
이 메타데이터만으로 변환 요청이 잘못되었다고 단정하거나 기준 PDF를 무효화하지 않는다.

## 독립 재현

기존 Studio가 사용하는 `devel` Docker 빌드 WASM을 Node에서 직접 로드했다.
PR 코드를 빌드한 결과나 기여자의 before PNG를 재현 증적으로 대신 사용하지 않았다.

- WASM: `/home/edward/mygithub/rhwp/pkg/rhwp_bg.wasm`.
- SHA-256: `f0d34fd4c82ed07ed0a59250460c21786e59d131c826c5d40eb20bfc0be777b5`.
- 페이지 수: 2쪽. 검토 페이지: 2쪽(0-based index 1), 본문 문단 `pi=27`.
- HWPX 원문: `  마. 행정박물류 ` 뒤 자리차지 표(`treatAsChar=1`, `vertOffset=0`).

| 항목 | HWPX 저장 정보 / 기대 출처 | 현재 WASM 관측 |
|---|---|---|
| 글자 줄 | `textpos=0`, `vertpos=39764`, baseline `1020` HU = 13.6px | 런 y=605.8px |
| 표 줄 | `textpos=11`, `vertpos=41924`, baseline `13423` HU = 178.973px | 글자 런 높이가 179.0px로 잡힘 |
| 글자 기준선 | 자기 줄 사용 시 약 619.4px | SVG y=`784.7466666666667`px |
| 대상 표 상단 | 별도의 표 배치 | control layout y=636.4px |

기준선 차이 `(13423-1020)/75 = 165.3733px`가 기여자의 약 165.4px 하강 주장과 일치한다.
현재 생성한 비교 PNG를 실제 열어 확인했다. PDF에서는 소제목이 마지막 표 **위**에 있고,
현재 rhwp에서는 같은 제목이 마지막 표 **안의 하단 행**으로 내려가 있다.
이는 단순 글꼴 두께·자간 차이가 아니라 소제목의 수직 위치 오류다.

## 증적과 재현 명령

로컬 root checkout에서:

```bash
node output/7044/reproduce.mjs
venv/bin/python output/7044/compare.py
```

`compare.py`는 프로젝트 `tools/fidelity_compare/fidelity_compare.py`의
`pdf_to_png`, `svg_to_png`, `sheet` 함수를 재사용한다. 별도 비교 UI를 만들지 않았다.
PNG는 현재 WASM SVG와 PR 기준 PDF를 비교한 것이며 PR 수정 전후 비교는 아직 아니다.

- 비교: `/home/edward/mygithub/rhwp/output/7044/baseline/cmp-p002.png`
- SVG: `/home/edward/mygithub/rhwp/output/7044/baseline/page_002.svg`
- PDF 래스터: `/home/edward/mygithub/rhwp/output/7044/baseline/reference_002.png`
- WASM 래스터: `/home/edward/mygithub/rhwp/output/7044/baseline/rhwp_002.png`
- 문단·표 좌표: 같은 폴더의 `page_002.text-layout.json`, `page_002.control-layout.json`.
- 원본 XML·PDF 메타데이터·좌표 요약: `measurements.json`.
- 빌드·샘플 해시: `provenance.json`.

## 다음 검토 범위

1. PR head를 독립 실행해 소제목 기준선 복원과 표 좌표 유지 여부 확인.
2. `table_owns_its_line`의 설명(위치 같음)과 실제 조건(`seg_start >= utf16_pos`)의 적용 범위 점검.
   현재는 검토 항목이며 이 조건으로 회귀가 발생한다고 판정한 것은 아니다.
3. 관련 inline table 보호 사례, 전체 로컬 검증 게이트와 최신 CI를 확인한 뒤 최종 리뷰 판정.

현재 root `devel`, Studio dev 서버, 설치된 WASM은 변경하지 않았다.

## 조판 규칙 일반성 검토 — 2026-09-12

대상 head가 여전히 `d2cbc46c18dbb98454c034753f45c1ccf6561239`임을 `gh pr view`로 재확인했다.
이번 검토는 정적 코드·원본 XML 분석이며 PR 수정본 실행이나 변형 샘플의 한컴 검증은 하지 않았다.

**판정: 조판 원리를 근거로 한 국소 조건 보정이다. 줄별 소속과 baseline을 일관되게 처리하는 전역 규칙 구현으로 볼 수 없다.**
파일명·문구·고정 좌표로 분기하는 하드코딩은 아니다. 그러나 범용 속성을 사용했다는 것만으로
조건식의 충분성이나 모든 출력 경로의 일관성이 보장되지는 않는다.

### 1. P1 — 같은 문단의 마지막 텍스트 출력 경로 누락

`paragraph_layout.rs:2847`, `:2899`, `:2967`의 출력에는
`wrapped_below_table || table_owns_its_line`이 적용되지만,
`:3049`의 `remaining_bbox_h`에는 이전 `wrapped_below_table` 조건만 남아 있다.
따라서 `table_owns_its_line=true`, `wrapped_below_table=false`인 같은 상태에서도
중간 flush와 마지막 flush가 서로 다른 baseline을 선택한다. 이것은 코드에서 확정 가능한 불일치다.

원본 HWPX `pi=27`에는 다음 두 run이 있다.

- `charPrIDRef=16`: `  마. 행정박물류`
- `charPrIDRef=17`: 공백 하나와 표

제목은 스타일 변경 때 중간 flush되어 새 조건을 사용하고, 마지막 공백은 이전 조건을 사용한다.
눈에 보이는 제목만 검사하는 현 테스트로는 마지막 출력 경로 누락을 발견하지 못한다.
스타일 변경 없이 마지막 run으로 출력되는 동일 계열의 텍스트는 보정에서 빠질 수 있다는
코드 경로상의 근거가 있다. 해당 변형의 실제 한컴 출력까지 확인한 결함이라고 과장하지 않는다.

### 2. 문단 단위 첫 표/첫 텍스트 줄 판정

`:2666`은 첫 번째 표 높이를 담을 수 있는 첫 저장 줄을 표 줄로 고르고,
`:2678`은 그 줄이 아닌 첫 줄을 텍스트 줄로 고른다. 이는 PR 이전부터 존재한 선택 방식이다.
이번 PR은 여기에 `:2695`의 문단당 boolean을 추가해 여러 텍스트 run에 공통 적용한다.
현재 출력하는 run이 어느 저장 줄에 속하는지, 각 표가 어느 줄을 차지하는지는 개별 계산하지 않는다.
따라서 다중 줄/다중 표에 대한 일반화의 근거는 부족하다. 이들 사례에서 회귀를 실행 검출했다는 뜻은 아니다.

### 3. 줄 소유권 조건의 설명과 실제 판정 범위 불일치

`:2682` 설명은 표 줄이 본문 끝에서 시작하는 경우를 말하고,
`:2696` 설명은 표 위치와 저장 줄 시작이 같은 경우를 말하지만,
`:2713` 구현은 `seg_start > 0 && seg_start >= utf16_pos`이다.
본문 끝 검사도, 동일 위치 검사도 아니다. 단순히 `>=`를 `==`로 고치는 것으로 충분하다고 보지 않는다.

특히 `utf16_pos`는 `para.text`의 문자만 UTF-16 길이로 합산하지만,
`line_seg_text_start`는 `src/model/paragraph.rs:1901` 계약상 컨트롤 슬롯을 포함하는 HWP5 문단 축이다.
앞선 컨트롤 슬롯이나 HWPX 축 보정이 있는 문단에서는 같은 단위를 쓰더라도 서로 다른 위치 척도를
비교할 수 있다. 이를 정규화한 뒤 실제 줄 범위와 컨트롤 위치의 대응을 판단해야 한다.

### 4. 적용 경로와 검증 범위

호출부 `src/renderer/layout.rs:9164` 이후는 인라인 표로 분류되고 다른 인라인 수식·그림·도형이
없는 문단을 이 함수로 보낸다. 해당 보정은 공통 composer 전체의 줄 메트릭 계약을 바꾼 것이 아니다.
다른 경로까지 수정해야 한다는 주장은 아니며, 영향을 받는 이 경로 안에서도 baseline 선택을
모든 출력 지점에서 일관되게 해야 한다는 뜻이다.
추가된 테스트 네 개는 모두 같은 HWPX fixture를 읽는다. 단일 스타일의 마지막 run,
앞선 컨트롤 슬롯, 여러 줄/표에 대한 독립 계약 검증은 추가되지 않았다.

### 보류 해제에 필요한 보완

1. 현재 텍스트 run과 표의 소속 줄을 동일한 위치 축에서 판정한다.
2. 한 줄의 baseline 선택을 한 곳에서 결정하고 중간·마지막 flush에 같은 결과를 사용한다.
3. 원본 샘플 외에 단일 스타일 마지막 run, 슬롯이 선행하는 문단, 여러 줄/표 및 진짜 같은 줄의
   텍스트+표 사례를 검증한다. 합성 입력은 내부 계약 검사로 한정하고 한컴 정답지로 취급하지 않는다.

전체 엔진 재작성이나 무관한 조판 경로 변경을 요구하지 않는다. 해당 함수 범위에서라도
줄 소속과 출력 경로의 일관성을 확보한 뒤 수용 판단해야 한다. 원격 게시·코드 수정은 하지 않았다.

## 메인테이너 승인 후 댓글 게시

2026-09-12 메인테이너의 댓글 게시 승인에 따라 위 보완 요청을 게시했다.
게시 직전 head `d2cbc46c18dbb98454c034753f45c1ccf6561239`, OPEN 상태를 재확인했다.

- 댓글: https://github.com/edwardkim/rhwp/pull/7044#issuecomment-5637435899
- `gh pr comment --body-file` 사용. API로 게시 본문과 로컬 UTF-8 원문 일치 및 BOM·`??` 부재 확인.
- 판정: 머지 보류. 코드 수정·push·GitHub review event·close·merge는 수행하지 않았다.

## 수정본 재검토 — 2026-09-12

메인테이너의 재검토 요청에 따라 기여자 응답과 새 head를 대조했다.
응답: https://github.com/edwardkim/rhwp/pull/7044#issuecomment-5642247476

### 기존 지적 조치 확인

| 지적 | 수정본 확인 |
|---|---|
| 마지막 run 출력 누락 | `remaining_bbox_h`를 포함한 네 출력 지점이 같은 `stored_line_baseline_at`을 사용한다. |
| 첫 표 기준 문단 공통 boolean | `table_owns_its_line`을 제거하고 각 run 시작 위치가 속한 저장 줄의 baseline을 선택한다. |
| 본문 문자만 세어 컨트롤 슬롯 축과 혼용 | `char_offsets[char_idx]`와 `line_seg_text_start`를 비교해 정규화된 문단 위치 축을 사용한다. |

이 변경은 해당 인라인 표 문단 경로 안에서 **run의 소속 줄에 따라 기준선을 선택하는 규칙**이다.
샘플명·문구·고정 좌표로 분기하지 않는다. 전체 조판 엔진을 재설계한 것으로 확대 평가하지 않는다.
저장 줄 정보가 부족하거나 위치를 찾지 못하면 기존 fallback을 유지한다.

### CI 및 로컬 집중 검증

- 검토 head: `69187adbefb92321a02bf99d0751c4f063a91d74`.
- [Full CI](https://github.com/edwardkim/rhwp/actions/runs/34662415596): 성공.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34662415572): 성공.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34662415568): 성공.
- [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/34663193933): full 모드 성공.
- CI 증적은 위 동일 head 기준으로 재사용했다. 로컬 전체 회귀·세 Clippy 재실행을 했다는 뜻이 아니다.
- review worktree에서 `node scripts/rust-test-suite-manifest.mjs --prepare` 후
  `node scripts/run-rust-test.mjs issue_7018_tac_host_line_baseline -- --cargo-profile release-test --target-dir /home/edward/mygithub/rhwp/target/pr-review`:
  **6개 통과**, 필터 밖 185개 미실행. manifest `--check` 및 `git diff --check` 통과.
- 병합 전 fetch한 `upstream/devel=59a11f180ad1bd5cadcbbf0a6dc9d0162f4a0a21`과
  `git merge-tree --write-tree` 충돌 없음. 이는 실제 병합 커밋 검증을 대체하지 않는 사전 검사다.
- 이후 GitHub API로 실제 merge tree가 `48e377b89858bde43245e0b54b654fafc05f13ec`임을 확인했다.
  두 parent도 위 base와 검토 head이고, 사전 검사 tree와 동일하다.

### 독립 Docker WASM 및 PDF 비교 결과

- review worktree에서 `docker compose -p rhwp --env-file .env.docker run --rm wasm` 성공.
  WASM SHA-256: `8210efe72485a94f8b2aeba2b495dde4f4e71529e4a18ab60b8aa8b345818575`.
- `output/7044/recheck.mjs`로 이 WASM을 직접 로드했다. 원본 2쪽 유지,
  `pi=27` 제목·후행 공백 두 run의 y=605.8px, baseline 거리=13.6px,
  실제 기준선=619.4px. 표 상단=636.4px 유지.
- 2쪽 control 3개의 type·x·y·w·h·문단/컨트롤 ID가 기존 독립 재현 baseline과 동일함을 assertion으로 확인.
  비교 baseline source는 `d14f44eba`이며 최신 devel 전체 A/B라고 부르지 않는다.
- 메모리 안에서 `pi=27`을 단일 글자 스타일로 통일한 계약 입력도 마지막 run 하나,
  baseline 거리=13.6px로 통과. 이 변형은 저장·한컴 검증한 정상 샘플이 아니다.
- PDF는 원본에 대응하는 기존 한컴 PDF 그대로 사용했다. PDF는 표준 `pdf_raster_commands`의
  96dpi, SVG는 표준 `rasterize-svg-webfonts.mjs`의 zoom=1로 생성하고,
  `make_compares`·`make_overlay_page`·`make_review_panels`를 재사용했다.
  최초 보조 스크립트의 PDF 700px/브라우저 SVG 원래 크기 혼용과 페이지 인덱스 오류를 정정한 뒤
  비교·overlay·review를 재생성했다. 정정 전 수치는 판정에 사용하지 않는다.
- 1·2쪽 review PNG를 직접 열어 확인했다. 문제 제목이 표 위로 복원됐고 표와 겹치지 않는다.
  글꼴·셀 내부 배치의 잔여 차이는 보인다. 모든 잔여 차이의 원인이나 PR 귀속은 판정하지 않았다.

| 페이지 | pixel match | ink match | diff pixels / total |
|---|---:|---:|---:|
| 1 | 92.73391% | 11.66783% | 64789 / 891662 |
| 2 | 94.13713% | 14.03647% | 52277 / 891662 |

위 값은 threshold=32의 보조 픽셀 지표이지 조판 정확도·성공률이 아니다. 전체 visual sweep의
구조 분석이나 fidelity ledger를 실행한 결과도 아니다. 특히 ink match가 낮으므로
pixel match만으로 문서 전체가 일치한다고 해석하면 안 된다.

- 대표 비교 이미지: [수정본 2쪽](../assets/pr_7044_recheck_p002.png).
- 원시 산출물: `/home/edward/mygithub/rhwp/output/7044/recheck/`의
  `page_001.svg`, `page_002.svg`, `review_001.png`, `review_002.png`,
  `overlay_metrics.json`, `provenance.json`, `single-style-contract.json`.
- 재현: `node output/7044/recheck.mjs`,
  `VISUAL_SWEEP_CHROME=/home/edward/.cache/puppeteer/chrome/linux-146.0.7680.31/chrome-linux64/chrome venv/bin/python output/7044/recheck-compare.py`.

**재검토 결론:** 기존 세 코드 지적의 수정과 원본 증상 개선을 확인했다. 집중 테스트·동일 head CI·독립 WASM
증적을 확보했다. PR은 검토 도중 다른 collaborator가 이미 병합했으므로 별도 merge 절차를 반복하지 않는다.
메인테이너의 수정본 시각 판정을 대신 기록하거나, 이 검토를 문서 전체 완전 일치 보증으로 사용하지 않는다.

### 검증 범위의 한계

추가 테스트는 원본 fixture의 후행 공백까지 검사해 누락 경로를 보호한다. 반면 축 검사도
같은 fixture를 사용하므로 선행 컨트롤이 있는 모든 경우를 검증했다고 말할 수는 없다.
기여자의 10,184 run 및 여러 줄·표 A/B 결과는 기여자 제공 증적이며 로컬에서 재계측하지 않았다.
단일 스타일 변형은 별도의 내부 계약 확인으로 취급하고 한컴 정답지로 쓰지 않는다.

### 원격 상태 및 보존

재검토 중 원격 PR은 `jangster77`에 의해 MERGED로 전환되었다. 이 에이전트가
댓글·review event·push·merge를 수행한 것은 아니다. 확인 시 관련 이슈 #7018은 OPEN이었다.
루트 `task_m100_3587`의 응답 대기 작업과 기존 Studio WASM은 변경하지 않았다.

### 메인테이너 시각 판정 확정

2026-09-12 메인테이너가 “시각 판정 통과입니다.”라고 판정했다.
대상은 검토 head `69187adbefb92321a02bf99d0751c4f063a91d74`의 독립 WASM 비교 결과이며,
이번 PR의 소제목 기준선 복원 범위에 대한 승인으로 기록한다. 전체 문서의 픽셀 완전 일치를 뜻하지 않는다.

PR #7044의 MERGED 상태와 merge commit `78bdfd9aa69417b4a88ffdd43aa76d3cad471f44`를 재확인했다.
이슈 #7018의 보고 증상은 이번 수정·검증·메인테이너 판정으로 해결 요건을 충족한다.
조회 시 이슈는 여전히 OPEN이다. 후속 운영 기록·asset 반영과 devel 동기화가 아직 완료되지 않아
`post_merge.md` 순서에 따른 이슈 종료·댓글 및 검토 worktree 정리를 남긴다.
이번 턴에는 원격 게시·push·이슈 close 또는 worktree 삭제를 하지 않았다.

## 승인된 후속 처리 — 2026-09-12

- base route: `maintainer_general`; modifiers: `intake_and_review`, `local_validation`,
  `visual_fixture_evidence`, `post_merge`. 각 정본과 모 라우터의 절차를 확인했다.
- 메인테이너가 리뷰 기록·이미지 반영 → devel 동기화 → #7018 종료·댓글 → 검토 worktree 정리를 승인했다.
- 문서 처리: **maintainer 직접 반영**. admin 권한을 확인했으며 이 archive, 대표 PNG,
  오늘할일만 운영 기록 commit으로 반영한다. 원본·소스·테스트·workflow·golden 변경은 없다.
- 루트의 `task_m100_3587=211351775c638be056ef61972727932e3ff92c0d`는 보존한다.
  clean 상태에서 devel로 전환해 `78bdfd9aa`까지 fast-forward했다. #3587의 구현 재개나 병합은 하지 않는다.
- 원 PR fork branch `planet6897/rhwp:fix/7018-tac-host-baseline`는 삭제 대상이 아니다.
- 남은 원격 처리 결과는 아래 계획에 따른 이슈·PR 댓글로 기록한다. 이 문서의 과거 OPEN/보류 표기는 당시 상태다.
- 운영 기록 검증: `python3 scripts/check_markdown_links.py mydocs/pr/archives/pr_7044_review.md mydocs/orders/20260912.md`
  2개 문서 내부 링크 이상 없음. `git diff --check` 통과. 대표 PNG는 이전 단계에서 직접 열어 확인했다.
- 병합 후 [CI 34669914465](https://github.com/edwardkim/rhwp/actions/runs/34669914465)의
  `Build & Test`, Lint, Native Skia, archive A–D tests 성공을 확인했다.
  기록 시 부가 단계 `Refresh nextest target duration data`는 진행 중이다.

## Merge 후 contributor PR comment 계획

1. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 연결한다.
2. 검토 head `69187adbe`, code merge `78bdfd9aa`, 기존 세 지적 조치, 집중 테스트 6 PASS,
   동일 head CI·CodeQL·Render Diff, 독립 Docker WASM 성공과 메인테이너 시각 판정 통과를 적는다.
3. 원본 1·2쪽을 직접 확인했고 대표 이미지는 2쪽이다. 전체 visual sweep 구조 분석은 실행하지 않았으므로
   flagged 후보 수는 **미계측**이다. `0/N`을 만들어 쓰지 않는다. 대표 2쪽 pixel match=94.13713%,
   visual_accuracy_proxy_percent(ink match)=14.03647%, threshold=32를 함께 표기한다.
   픽셀 값은 정확도·통과율이 아니며 잔여 글꼴·셀 내부 배치 차이까지 해결했다고 주장하지 않는다.
4. 이미지 안정 경로는 `mydocs/pr/assets/pr_7044_recheck_p002.png`다. 이미 원 코드 PR이 병합된 상태여서
   code merge SHA 대신 **실제 asset을 담아 devel에 반영한 운영 기록 commit SHA**를 고정한다:
   `https://raw.githubusercontent.com/edwardkim/rhwp/<운영-기록-commit-sha>/mydocs/pr/assets/pr_7044_recheck_p002.png`.
5. asset의 원격 blob과 devel 포함을 확인하고 최종 devel 동기화 후 `gh --body-file`로 게시한다.
   게시 후 API에서 UTF-8 본문·이미지 URL 일치를 검증한다. 이슈 #7018에는 해결 근거와 code merge,
   검증·시각 판정을 적고 completed로 종료한다. 동일 증적 댓글은 중복 게시하지 않는다.
6. 필수 후속 처리 완료 후 검토 전용 `/home/edward/mygithub/rhwp-review-7044`를 제거한다.
   공유 `target/pr-review`, 기본 작업공간, #3587, #7040 worktree, contributor fork는 보존한다.
