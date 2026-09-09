# #6950 Stage 2 — 문단·표 배치 계약 구현

- Issue: [#6950](https://github.com/edwardkim/rhwp/issues/6950)
- 수정 [구현계획](../plans/task_m100_6950_impl.md): 2026-09-09 메인테이너 승인.
- 상태: A 기준점 확인 및 B 저장 호스트 통째 배치 절편 구현·집중 검증.
  **재조판/분할 경로의 결과 전달이 남아 있어 Stage 2 전체 완료는 아니다.**
- 입력/기준: [Stage 1](task_m100_6950_stage1.md)의 fixture·PDF·제품 SHA를 유지한다.

## 1. 런타임 기준점 확인과 기존 설명 정정

임시 계측으로 표 배치 직전 typeset과 layout의 실제 원점을 확인했다. 로그는 로컬
`output/6950/anchor-trace/{new,6718,6879}.log`에 보존했고 임시 로그 코드는 제거했다.

| 사례 | typeset 문단 시작(단 상대 px) | layout 문단 시작(쪽 절대 px) | layout 단 상단 |
| --- | ---: | ---: | ---: |
| 이번 샘플 문단 1 | 379.547 | 474.013 | 94.467 |
| #6718 문단 23 | 108.267 | 221.640 | 113.373 |
| #6879 문단 73 | 0.000 | 94.467 | 94.467 |

두 단계의 시작점은 단 상단을 더하면 일치한다. #6718의 핵심은 빈 호스트 rewind 스냅이나
이미 진행한 문단 원점이 아니었다. `native_multiline_visible_float_table_top`이 본문 전체 높이와
offset을 더한 후보를 만들고, 이어지는 제목 첫 줄 보정이 이를 다시 내리고 있었다.
여기에 일반 앵커 보정을 추가하면 중복된다. Stage 1의 계보 측정은 유지하되 상세 인과 설명을 보완한다.

이 기존 helper는 `a268e24d31`에서 HWP5·단일 표·3줄 이상이라는 조건으로 도입되었다.
호스트/표의 구조적 배치를 나타내는 공통 계약이 아니므로 그 조건 자체를 보호하지 않는다.
저장소의 `pdf/issue6718-27469-2020.pdf` 4쪽을 직접 열어 본문 뒤 표 관계를 확인했다.

## 2. 첫 구현 후보

`ParagraphFloatPlacement`로 앵커·표 상단·점유 하단을 단 상대 좌표로 함께 기록한다.
현재 연결 범위는 유효한 저장 줄에서 모든 호스트 줄이 표보다 앞서는 문단 기준 TopAndBottom 표다.
typeset의 결과를 `ColumnContent`로 전달하고 layout에서 소비한다. 기존 정상 TAC 경로는 유지한다.
저장 줄의 단조 증가만으로 두 사례를 구분하지 않는다. 두 사례 모두 같은 계약으로 계산한다.

첫 후보의 집중 CLI 실행(`output/6950/history/stage2-first/summary.json`):

- 이번 표 516.4 → 603.8px. 본문 네 줄은 그대로이며 후속 본문은 표 아래에서 시작한다.
- #6718 표 378.5 → 356.1px, 4쪽 본문 최대 넘침 3.2 → 0px.
- #6879 라벨 98.2px, 후행 표 182.1px 유지.

이 값은 실행 관찰이지 최종 시각 통과 판정이 아니다. 해당 기록의 소스 SHA는 dirty checkout의
부모 SHA이므로, 검증 후보의 최종 commit과 별도로 해석한다. 전체 Rust 회귀·WASM은 아직 미수행이다.

## 3. 같은 절편에서 보완한 책임 경계

- `8bea53b5f`: 통째 배치 fit **이전**에 배치 상자를 결정한다. 현재 흐름 높이에 표 높이만
  더한 예산 대신 확정된 점유 하단으로 현재 단에 들어가는지 판단하고, 같은 값을 예약·출력에 전달한다.
- layout의 기존 호스트 높이·제목 줄 보정은 확정 결과가 없는 경로에서만 실행한다.
  `layout_table`도 명시적인 확정 상단을 받으면 위치 속성을 다시 해석하거나 쪽 안으로 끌어올리지 않는다.
- `fa28cdbba`: 겹침 비허용 표는 선행 점유 구간을 정렬된 순서로 한 번씩 방문하여 표 상자만
  아래로 진행한다. 원래 앵커 줄과 표 높이는 보존한다. layout의 후속 exclusion도 전달된 점유 구간을 쓴다.
- `0698e2846`: 위 캡션을 포함한 예약 상자의 내부에서 표 본체를 배치한다. 캡션은 별도 앵커 보정이 아니다.

신규 공통 결과 전달의 적용 범위는 **유효한 저장 호스트 줄이 표보다 앞서고 통째 배치 가능한
문단 기준 TopAndBottom 표**다. TAC/Square/글앞·글뒤는 기존 의미를 유지한다. 저장 줄이 없거나
합성/되감김 줄인 경우에는 이 저장 좌표 결과를 만들지 않는다. 재조판 및 분할·이월은 기존 경로를
재사용하며, 아래 경계 테스트가 확인한 범위와 공통 결과 전달의 적용 범위를 혼동하지 않는다.

## 4. 집중 검증 기록

검증 worktree는 `/home/edward/mygithub/rhwp-6950-review`, 공유 target은
`/home/edward/mygithub/rhwp-shared-review-target`이다. 새 integration source는
`tests/cases/issue_6950_paragraph_end_topbottom_anchor.rs`에만 추가했다. 원본 sample/PDF는 변경하지 않았다.

실행 절차:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/run-rust-test.mjs --cargo-test <case> -- \
  --target-dir /home/edward/mygithub/rhwp-shared-review-target -j 2
```

| 검증 후보 | 대상 | 결과 |
| --- | --- | --- |
| `10f5b3643a` | 신규 실제 fixture/좌표/배치방식/저장 줄 계약 | 5/5 |
| `8bea53b5f` | 신규 + 저장 줄 무효화 + 본문 축소 3조건의 비겹침 | 7/7 |
| `8bea53b5f` | #6718 / #6879 / #6860 원본·재저장 / #5807 | 3/3, 4/4, 5/5, 2/2, 1/1 |
| `fa28cdbba` | 신규 + 선행 점유 구간 + 본문 축소 시 표 하단 검사 | 8/8 |
| `fa28cdbba` | 기존 #2439 `issue_2439_lineseg_indent_tests` | 5/5 |
| `0698e2846` | 위 캡션 포함 신규 계약 전체 | 9/9 |
| `0698e2846` | #6718 / #6879 / #6860 원본·재저장 / #5807 / #2439 최종 재실행 | 3/3, 4/4, 5/5, 2/2, 1/1, 5/5 |

본문 축소·저장 줄 무효화·캡션 테스트는 **직접 구성한 IR의 알고리즘 검사**이며 한컴에서
정상 저장·검증한 추가 문서라고 주장하지 않는다. 원본 fixture/PDF 검증과 분리한다.
테스트 source가 바뀌면 suite 배정도 바뀐다. 생성 suite가 낡아 0건으로 종료된 실행은
통과에 포함하지 않았고, `--prepare` 후 실제 실행 건수를 확인했다.

`cargo fmt --all -- --check`, `git diff --check`,
`node scripts/rust-unit-test-tiers.mjs --check`도 통과했다(unit-tier 4,205 tests/298 modules).
최종 후보의 집중 검증은 총 29건이다. review 환경의 manifest `--check`도 통과했다.

최종 CLI는 `0698e28467dbfb5e916891ecf7e9e9370ad6287d` 검증 worktree에서 다시 빌드했다
(`cargo build --locked -p rhwp --bin rhwp --target-dir <공유 target> -j 2`, 1분 21초).
실행 파일 SHA-256은 `5b8dcbbe3e3de12b0e483072119fd57ab570df6ad2ecaa441bf5f3bc790f4c9f`다.
`output/6950/history/stage2-final/summary.json`에 원본/#6718/#6879 실행을 기록했다.

같은 `본문 마지막 줄 하단 ≤ 표 상단` 조건을 과거/최종 CLI 산출물에 적용한 결과:

| 산출물 | 본문 하단 | 표 상단 | 비겹침 |
| --- | ---: | ---: | --- |
| 수정 전 `2450a6455` 제품 코드 | 580.1 | 516.4 | 실패 |
| 최종 후보 `0698e2846` | 580.1 | 603.8 | 통과 |

명령은 `node output/6950/check-flow-contract.mjs stage2-final`이다. 이는 같은 실제 입력에 대한
CLI 기하 조건의 red/green이며, 새 Rust 테스트 파일을 과거 revision에서 컴파일했다는 뜻은 아니다.

## 5. 원본 3쪽 표준 비교

`8bea53b5f` 제품 코드로 CLI SVG → Chrome 공통 webfont raster, PDF 96dpi를 비교했다.
WASM Studio 실행 결과가 아니며 원격 폰트 응답·대체 폰트에 따른 글꼴 차이가 포함된다.

- 비교: `output/6950/visual-after/para-table/compare/compare_001.png`
- overlay: `output/6950/visual-after/para-table/overlay/overlay_001.png`
- review: `output/6950/visual-after/para-table/review/review_001.png`
- 전체: `output/6950/visual-after/summary.json`
- 문자/쪽/개체 원장: `output/6950/fidelity-after/`
- 최종 CLI SVG: `output/6950/stage2-final-svg/20260909-para-table_001.svg` (전체 3쪽)

최종 `0698e2846` CLI로 다시 생성한 SVG 3개는 표준 비교에 사용한 `8bea53b5f` SVG 3개와
`diff -qr`로 바이트 동일함을 확인했다. 따라서 이 입력에서는 기존 raster 증적을 재사용한다.
다른 입력에서 두 후보가 동일하다는 주장이 아니며, 추가 캡션/점유 구간 변화는 별도 테스트로 검증했다.

| 쪽 | pixel match | 내용 픽셀 중심 보조 일치율 |
| --- | ---: | ---: |
| 1 | 80.42936% | 5.95666% |
| 2 | 88.50304% | 13.17671% |
| 3 | 84.50018% | 46.58850% |

1쪽 수정 전 보조값은 6.14793%였다. **겹침 해소와 전체 raster 점수 상승은 같은 주장이 아니다.**
현재도 상단 세 표의 위치·글꼴부터 한컴 PDF와 차이가 남는다. 대상 본문 네 줄 뒤에 표가 놓이는
관계는 확인했지만 전체 시각 정합 통과는 메인테이너가 판정해야 한다.

`fidelity_compare --text-only --export-all-svg --layout-ledger` 결과는 PDF/SVG/render tree 각
3쪽, 요청 3쪽 모두 완료다. 페이지별 문자 멀티셋 차이(reference-only/svg-only)는 전부 0/0이며,
layout 원장의 footer/frame/셀 텍스트 겹침 등 후보도 0이다. 문자 순서·글꼴·모든 시각적 동일성을
증명하는 값은 아니다.

## 6. 잔여와 다음 절편

1. 최종 후보 캡션·보호 테스트 및 CLI SHA/원본 3쪽 SVG 일치 확인까지 완료했다.
2. **저장 줄 없는 재조판과 분할·이월에서도 동일한 공통 배치 결과를 전달하는 연결은 미완료다.**
   기존 경로의 경계 테스트 통과만으로 구현계획 1.1의 전체 계약을 완료했다고 판단하지 않는다.
   다음 절편에서는 계산된 줄의 실제 앵커와 physical fragment 소유를 입력으로 전달해야 한다.
3. 메인테이너 시각 판정과 Stage 2 결과 승인 후에만 Stage 3 전체 lint/회귀·Docker WASM·PR 준비로 간다.
   이번 절편에서 push·PR·GitHub comment·병합·close는 수행하지 않았다.
