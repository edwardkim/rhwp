---
kind: review
status: archived
last_verified: 2026-10-06
---

# PR #7610 리뷰 — 용지 기준 overlay 표의 행 분할·소실

## 최종 판정

**승인 — 작업지시자가 승인한 부분 해결을 통합한다.** Paper 기준 표의 행 소실·잘못된 분할은 검증됐다. 작업지시자는 “분할정복입니다. 이 부분의 해결로 PR은 머지시키는게 올바른 판단”이라고 지시했다. 이에 따라 Page 기준 한컴 출력과 제보 원본의 미검증은 #7441의 후속 범위로 유지하고 이번 부분 통합의 blocker에서 제외했다. Page까지 검증 완료로 승격하지 않는다.

- 원 PR 검토 head: `cd879723e87dd19913cb99d0d17ec2970f1aa380`. 메인터너 보정 code는 추가하지 않았다.
- 통합 승인 범위: 현재 head의 부분 해결. 이후 #7441에서 Page 기준 overlay 표의 정상 한컴 출력과 제보 원본을 확보해 적용 경계·다음 쪽 내용 보존을 검증한다. 이 미검증을 이번 PR의 해결 주장에 포함하지 않는다.
- 실행으로 확인한 새 회귀: 없음. 앞선 증거 부족에 따른 보류는 작업지시자의 부분 통합 지시에 따라 해제했다. 원칙별 미검증 기록은 유지한다.
- #7441은 `Refs`로 유지한다. 전체 이슈 해결이나 issue close로 확대하지 않는다.
- reviewer `edwardkim`; [승인 review](https://github.com/edwardkim/rhwp/pull/7610#pullrequestreview-5429365574) 게시와 UTF-8 본문 재조회 완료. 원 code PR은 `059c12916d82c63a548734e448ddd0da9324ca6d`로 2026-10-06T13:50:44Z에 merge됐다.
- 작성 시점 상태는 open / non-draft / mergeable=true / clean / exact-head CI 성공이다. 병합 전에 최신 head·base·CI·작업지시자 승인을 다시 확인해야 한다.

## 접수 정보와 승인된 범위

| 항목 | 확인 결과 |
| --- | --- |
| PR·작성자 | [#7610](https://github.com/edwardkim/rhwp/pull/7610), johndoekim / CONTRIBUTOR |
| base / head | devel `a048da32d2b76867283aa6dbdf2578816a0e6238` / `cd879723e87dd19913cb99d0d17ec2970f1aa380` |
| 규모 | 4 commits, 14 files, +394 / -0; production 변경 13줄, 회귀 2개와 probe 2개, 대표 PNG 10개 |
| 라우팅 | maintainer_general + intake_and_review / local_validation / visual_fixture_evidence |
| 이슈 | [#7441](https://github.com/edwardkim/rhwp/issues/7441), `Refs`; 종료하지 않음 |
| 현재 CI | [CI 37463326904](https://github.com/edwardkim/rhwp/actions/runs/37463326904), 34 check runs에 대기·실패 없음 |

[메인터너 범위 승인](https://github.com/edwardkim/rhwp/issues/7441#issuecomment-5995104948)은 원본이 없으면 저장소 문서·합성 재현으로 부분 PR을 진행하고, 이슈를 닫지 않은 채 처리 범위를 comment에 남기는 2번안이다. [기여자의 범위 보고](https://github.com/edwardkim/rhwp/issues/7441#issuecomment-6016215966)와 현재 PR 본문의 구분을 확인했다. 승인된 범위 자체를 다시 확인받도록 요구하지 않는다.

본문 안에 들어가는 Paper 기준 표의 불필요한 컷과 행 소실을 해결한 것은 수용 근거가 있다. 제보자의 실제 원본, 합성 제보 조건의 한컴 대응, Page 기준의 한컴 분할 정책은 별개의 미검증이다. 현재 코드가 바꾸는 Page 경로의 미검증은 작업지시자 지시에 따라 열린 #7441에서 후속 관리한다. 이 판단은 부분 통합 결정이며 Page 동작을 한컴과 일치한다고 선언하지 않는다.

## 원인과 실제 소비 경로

[한컴 공식 위치 도움말](https://help.hancom.com/hoffice/multi/ko_kr/hwp/table/tableattribute/table%28position%29.htm)은 Para의 세로 기준이 표가 있는 줄, Page는 편집 가능한 본문 위·아래, Paper는 용지 끝임을 구분한다. 쪽 영역 제한의 문단 기준 설명도 확인했다. 이는 좌표 기준의 독립 근거이며, Page 표를 항상 분할하지 않는다는 한컴 출력 증거로 확대하지 않는다. [여러 쪽 지원](https://help.hancom.com/hoffice/multi/ko_kr/hwp/table/tableattribute/table%28many%29.htm) 설명만으로도 Page overlay의 구체적인 분할 정책을 확정할 수 없다.

기존 `current_height + vertical_offset`를 본문 가용 높이에서 빼는 식은 Paper의 용지 원점을 문단 흐름 원점으로 해석했다. 표가 본문 안에 들어가도 잘린 행이 다음 쪽으로 넘어갔으며, 다음 쪽 표와의 겹침 방지 paint 상한이 0이면 행이 사라졌다. 새 조건은 해당 컷 계산을 Para에만 적용한다.

| 단계 | 실제 코드와 값의 소비 |
| --- | --- |
| Shape·요구 높이 생성 | `src/renderer/typeset/controls.rs:88–125`가 Shape를 먼저 발행하고 `FormattedTable`을 만든 뒤 continuation을 호출한다. Shape 자체를 숨기는 수정이 아니다. |
| 시작·끝 컷과 예약 | `src/renderer/typeset/controls/decoration_table.rs:14–94`: Para는 종전 room·prefix sum으로 first_unfit·remaining_px를 계산하고, 저장 vpos 되감기가 있으면 reserve_px를 만든다. Paper/Page는 None 반환으로 컷과 이월 예약을 만들지 않는다. |
| 누적 높이·유닛 소유 | `src/renderer/typeset/state.rs:326–355`가 Some의 현재 쪽 컷·다음 쪽 pending을 저장한다. `state/transition.rs:589–618`이 다음 쪽 예약의 최댓값으로 흐름을 전진시킨다. 새 None 경로에는 가짜 작은 조각을 예약한 뒤 paint에서 늘리는 처리가 없다. |
| 앵커 쪽 배치 | `src/renderer/layout.rs:16296–16358`은 typeset의 overlay_cut만 `NestedTableSplit`로 넘긴다. None이면 표 전체를 앵커 쪽에 그리며 임의 컷을 재계산하지 않는다. |
| 이어받기 배치 | `src/renderer/layout.rs:9971–10081`은 이어받은 행의 실제 공간을 다음 쪽 기존 표 상단으로 제한한다. 이번 수정은 Paper의 잘못된 이월을 이 경로에 보내지 않는다. Para 정상 대조군은 기존 이어받기를 유지했다. |

Paper의 본문 안 fit 사례와 꼬리말까지 뻗은 배경 표를 구분했다. 빈 셀의 물리 공간·내용을 지우거나 좌표를 clamp하는 변경은 없다. rowspan 내부 컷·중첩 recursive cut·TAC 줄 구성은 변경 경로가 아니다. 기존 LineSeg 수용 조건과 편집 후 재조판 규칙도 변경하지 않았다.

## 실행한 검증

| 항목 | 결과와 증거 |
| --- | --- |
| 충돌 / diff | 최신 base의 `git merge-tree --write-tree upstream/devel upstream/pr7610-head` exit 0; tree `0d9da58b666f73853369208aa78dc736b680908d`; diff check 통과 |
| 파생 suite / 정책 / fmt | `node scripts/rust-test-suite-manifest.mjs --prepare`, `--check --base-ref a048da32d2b76867283aa6dbdf2578816a0e6238`, `cargo fmt --all -- --check` 통과 |
| 새 정식 회귀 | `node scripts/run-rust-test.mjs issue_7441_paper_overlay_table_split -- --cargo-profile release-test --target-dir target/pr-review`: **2 PASS** |
| 정상 대조군 | 같은 runner로 issue_4514_overlay_table_flow 1 PASS, issue_5792_overlay_table_split_overlap 1 PASS, issue_6344_empty_page_false_positive 2 PASS. 총 **6 PASS** |
| 음성 대조 | `a048da32d2b76867283aa6dbdf2578816a0e6238`와 production code가 같은 테스트 추가 commit `bf7adb3e1c97b67ee0cb01ce89484dd27d79d973`의 격리 worktree에서 `cargo build --locked --lib --profile release-test --target-dir target/pr-review`를 실행했다. 동일 회귀 source를 그 lib에 링크해 **2 FAIL**: complex p1 row 14, ipc p1 rows 14·15 소실. 컴파일·환경 실패가 아니다. |
| 양성 대조 | 같은 source를 검토 head lib에 링크하면 **2 PASS**; 정식 suite 결과와 일치. 독립 기대값을 바꾸지 않았다. |
| Native / WASM | Native CLI release-test 빌드 성공. `docker compose -p rhwp --env-file .env.docker run --rm wasm` fresh 빌드 성공(7m57s). pkg와 Studio public의 JS·WASM SHA 일치. |
| 경계 probe | `check_7441_footer_area_cover.rs`를 현재 lib에 직접 링크해 실행: cover rows 0·1은 p1, p2 cover 없음, MEMO y=113.4px. 독립 한컴 2020 Print PDF p2 MEMO yMin=84.93pt(113.24px). |
| 전체 CI / lint / Skia | 같은 PR head의 full CI·3종 lint·archive workers·Native Skia·Canvas visual diff·CodeQL·adapter·proptest 성공을 재사용했다. source 보정 없이 광범위 로컬 nextest·Clippy를 반복하지 않았다. CI WASM Build skipped는 로컬 fresh 빌드로 보완했다. |
| 미실행 | 제보 합성 probe의 reviewer 재실행, Page 한컴 출력, 온새미로 전체 47쪽 sweep·수정 전후 SVG 동일성의 reviewer 재실행. 온새미로 전쪽 동일성·89.90%는 기여자 보고로 구분한다. |

Cargo는 공유 `/home/edward/mygithub/rhwp/target/pr-review`만 사용했다. Docker는 기존 rhwp named volumes를 재사용했다. 새 target이나 cache 초기화는 하지 않았다. 음성 대조는 사용자의 source를 변경하지 않고 별도 worktree에 수행했다. 로컬 nextest 0.9.137의 report-skipped 설정 경고가 있었으나 선택한 6건은 실제 실행됐으며 필터 제외 건수를 실패나 통과에 더하지 않았다.

새 회귀는 실제 렌더 트리를 검사하지만, 렌더된 표 목록만 순회하고 행을 BTreeSet으로 모으므로 표 전체 누락이나 셀 중복을 모두 검출하는 검사라고 과장하지 않는다. 이번 행 소실 검출은 음성 대조·전쪽 raster·대표 직접 판독으로 확인했다.

## 검증 입력 커밋 확인

**충족.** 아래 실제 입력은 `cd879723e87dd19913cb99d0d17ec2970f1aa380`의 Git blob 또는 LFS OID와 동일했다. 기준 PDF는 메인터너 `630a59867`의 Oracle 재산출본을 재사용했다. complex·ipc PDF는 Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.6, 각각 4·10쪽, A4 가로(841×595pt)다. 저장 제품은 PR이 확인한 한컴 2010이며, 제품 연도·PDF 형식 버전만으로 유효한 Print 기준을 거부하지 않았다.

| 입력·기준 경로 | SHA-256 | commit 일치 |
| --- | --- | --- |
| `samples/table-complex.hwp` | `9b5334f0fc164a3168e41042ef5f535ab7180e563b8fdfb59b0c157c895bbc22` | Git blob equality |
| `pdf/table-complex-hwp-2020.pdf` | `af53a2027a40505988a374afe5a98c5188bc42d946cf7b1f8b77a919c0b27d69` | Git blob equality |
| `samples/table-ipc.hwp` | `c407e18cd3ddf89a9da1ef151d79e172d750153787b2e100a7c9b22a12448ec2` | Git blob equality |
| `pdf/table-ipc-hwp-2020.pdf` | `6cae92bfa9b70ab4fd5aa8b6a4010d9733b5da10b931d167320e14de6ec4ef56` | Git blob equality |
| `samples/[2027] 온새미로 1 본교재.hwp` | `e8592e74c9a8425c4ee2c5824d012ebe45e9f6dd36880b784ba594b4fd0a31ce` | Git blob equality |
| `pdf/[2027] 온새미로 1 본교재-hwp-2020.pdf` | `450c793f33a134198936504b38d3b21c01a343a11c09c13e5a7a00e39e2d58c7` | Git blob equality |
| `pdf/[2027] 온새미로 1 본교재-2024.pdf` | `fd280e8771851af1e6d17049355bb880aae592c7b3e758ecceee8f0a5e3b7f6e` | Git blob equality |
| `samples/issue4514/sample1-repro.hwp` | `72d3be39c8af8779387e7657cb9cd5823fda62dff1c8700ab1bfe73592baf793` | Git blob/LFS OID equality |
| `samples/issue5792/2700727_animal_facility_standards.hwpx` | `43f4d4a0e6134c787278b139da49ea88e560aee199f2c19f37c787ecb71aeb86` | Git blob/LFS OID equality |

Page 적용 경계의 정상 저장본·기준 PDF는 없다. 이 범위의 입력 확인은 **미검증**이다. 제보 원본 미제공은 승인된 부분 처리의 제한으로 기록한다.

## 전쪽 Visual Sweep과 직접 판독

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md)의 전쪽 TSV와 대표 review/standalone overlay 절차를 실행했다. 비교는 96dpi / print / Chrome 154.0.8037.57 / webfont rasterizer이며, 실제 존재하는 Windows Fonts·Nanum·DejaVu 경로를 확인하고 `--embed-fonts=full`로 굴림·바탕을 공급했다. 글꼴 예외·점수 허용치 변경은 사용하지 않았다.

```bash
export VISUAL_SWEEP_CHROME=/home/edward/.cache/puppeteer/chrome/linux-154.0.8037.57/chrome-linux64/chrome
export RHWP_FONT_PATH=/mnt/c/Windows/Fonts:/usr/share/fonts/truetype/nanum:/usr/share/fonts/truetype/dejavu
python3 scripts/visual_sweep.py --silhouette-only --embed-fonts=full \
  --file-target table-complex samples/table-complex.hwp pdf/table-complex-hwp-2020.pdf \
  --file-target table-ipc samples/table-ipc.hwp pdf/table-ipc-hwp-2020.pdf \
  --rhwp-bin target/pr-review/release-test/rhwp --dpi 96 \
  --out output/pr-review/pr7610-20261006/native-font-scores
# fresh WASM: 같은 입력에 --wasm-pkg pkg, --out .../wasm-font-scores를 지정해 실행했다.
```

| backend / 문서 | 비교 쪽 / 전체 | 최저 2px 실루엣 일치율 | 90% 미만·누락·측정 불가 | TSV |
| --- | --- | --- | --- | --- |
| native-font-scores / table-complex | 1–4 / 4 | 99.99871% (p2) | 없음 | `output/pr-review/pr7610-20261006/native-font-scores/table-complex/silhouette.tsv` |
| native-font-scores / table-ipc | 1–10 / 10 | 99.96463% (p7) | 없음 | `output/pr-review/pr7610-20261006/native-font-scores/table-ipc/silhouette.tsv` |
| wasm-font-scores / table-complex | 1–4 / 4 | 99.99871% (p2) | 없음 | `output/pr-review/pr7610-20261006/wasm-font-scores/table-complex/silhouette.tsv` |
| wasm-font-scores / table-ipc | 1–10 / 10 | 99.96463% (p7) | 없음 | `output/pr-review/pr7610-20261006/wasm-font-scores/table-ipc/silhouette.tsv` |

Native/fresh WASM은 전체 14쪽의 PNG가 byte-identical이었다. 전쪽 TSV의 `not_evaluated`는 직접 시각 승인으로 간주하지 않았다. 대표 complex p1·p3은 `--pages 1,3`, Native와 fresh WASM 각각 일반 모드로 재출력했다.

- 현재 CLI SHA-256: `1b3e51860f13397f245bcc40df604d6e7906c6cf608cd595f7f6e78eb3691f35`.
- fresh WASM SHA-256: `5758a9b45bb3a6a108c37f06cc0adab6496345800eee53ef5ef0d6505a445510`.
- 직접 연 PNG: `output/pr-review/pr7610-20261006/native-review/table-complex/review/review_001.png`, `review_003.png`; fresh WASM 대응 `wasm-review/.../review_001.png`, `overlay/overlay_003.png`.
- p1 마지막 잡수입 소계 행, p3 마지막 정서지원프로그램·급식프로그램 행이 셀과 테두리 안에 복원됐다. 한컴의 동일 쪽·영역과 직접 대조했고 표 바닥·행 경계·쪽 소속이 맞았다. 수정 전 PR PNG의 같은 영역에는 해당 행이 없었다.
- 대표 PNG 두 쪽의 `pr_review_gate`는 두 backend 모두 passed, 후보 flagged pages 0. pixel match 평균 92.27302%, 최저 91.17266%; visual_accuracy_proxy/엄격 내용 ink match 평균 12.6122%, 최저 11.74426%; 2px 실루엣은 100%. 낮은 엄격 ink 값은 별도 잔여 차이이며 **전체 fidelity 통과라고 쓰지 않는다**. 수용 근거는 주장한 행 복원·외곽·소속의 직접 확인이다.
- table-ipc는 민감 정보 목록에 있어 원본·본문 PNG를 공개하지 않았다. 로컬 전체 10쪽 비교·행 검사와 글자를 제거한 `private-diagnostics/ipc-grid-native.png`를 직접 확인했다. 표 외곽과 행 선의 연속·바닥이 기준과 대응했다. 이 괘선 진단은 본문 글자 직접 판독을 대신하는 완전 검사가 아니다.
- 기존 contributor PNG의 한글 도구 라벨에는 tofu가 있었으나 제품 본문과 구분했다. 현재 Linux 재출력 라벨·수치·legend는 읽힌다. 도구 라벨 후속 추적 초안은 ignored output에만 준비하며 이번 source의 실행 회귀로 분류하지 않는다.
- PR 본문에는 fork `johndoekim/rhwp`의 정확한 `cd879723e87dd19913cb99d0d17ec2970f1aa380` raw URL로 Native/fresh WASM p1·p3 review/overlay가 실제 Markdown image로 표시돼 있다. 해당 10 asset 추가 외에 `addf23e03`→head의 source/test/Cargo 변경은 없었다.

TSV·manifest 해시:

- `output/pr-review/pr7610-20261006/native-font-scores/table-complex/silhouette.tsv`: `a2231896f1d6384469eda829f131335ea9faf2f5de089e49832526bcbd83177a`
- `output/pr-review/pr7610-20261006/native-font-scores/table-complex/silhouette_manifest.json`: `adb441f107756b8f51a67f1538ba1261a5c13b93ee6214b4003a1f07ffb9727d`
- `output/pr-review/pr7610-20261006/native-font-scores/table-ipc/silhouette.tsv`: `b7d47266c619b253afc725b66ac3e1b4c112680afac6a92dd0969605322550c0`
- `output/pr-review/pr7610-20261006/native-font-scores/table-ipc/silhouette_manifest.json`: `dc5c63cca88157da79f9f10f83b2c44aee1a45e1fd87898a898f969fdca34c1d`
- `output/pr-review/pr7610-20261006/wasm-font-scores/table-complex/silhouette.tsv`: `a2231896f1d6384469eda829f131335ea9faf2f5de089e49832526bcbd83177a`
- `output/pr-review/pr7610-20261006/wasm-font-scores/table-complex/silhouette_manifest.json`: `0a13080e1a7ada4cac92e20feed59e243a3d0f00915d5e417667823de5948fbb`
- `output/pr-review/pr7610-20261006/wasm-font-scores/table-ipc/silhouette.tsv`: `b7d47266c619b253afc725b66ac3e1b4c112680afac6a92dd0969605322550c0`
- `output/pr-review/pr7610-20261006/wasm-font-scores/table-ipc/silhouette_manifest.json`: `efeac9afe06546419f0a08013806af76901dc998ca35f8b7102a44b4adbf5a6f`

## 조판 원칙 준수 검토

| 원칙 | 판정 | 근거·제한 |
| --- | --- | --- |
| 구현 근거와 일반성 | 충족(Paper); 미검증(Page 분할 정책) | 독립 좌표 규칙·동일 원본 한컴 PDF, 속성 도메인 분기. 샘플 ID·숫자 예외·clamp 없음. Page의 실제 분할 정책은 위치 사양만으로 입증되지 않음. |
| 측정·배치 일관성 | 충족(검증 범위) | 위 실제 소비 경로에서 typeset 컷·예약을 layout이 공유. Paper에 None이면 컷/예약/이어받기 모두 생성하지 않음. backend 전체 PNG 동일. |
| 분할·이어받기 계약 | 충족(Paper/Para 대조); 미검증(Page) | Paper 본문 안 fit·실물 표 완전성, Para 두 기존 회귀, cover 다음 MEMO 보존을 실행. Page의 fit/넘침·다음 내용 경계는 미실행. |
| 줄 소속과 점유 높이 | 비해당(줄 재구성 변경); 흐름 예약은 위 계약 적용 | TAC·LineSeg 수용·기준선·빈 문단 줄 구성을 바꾸지 않았으며 Para 예약식도 동일. |
| 사례·증거 독립성 | 충족(Paper); 미검증(Page) | 정상 저장본·독립 PDF·새 회귀 2건 before FAIL/after PASS. 합성 제보는 진단으로만 취급. |
| 기준값 변경 | 비해당 | baseline/golden/래칫/허용치 변경 없음. 새 두 회귀 관련 전쪽 Native/fresh WASM 최저가 90% 이상. |
| 주장과 검증 범위 | 충족(구분·기록); 미검증(Page 실행) | source·binary·WASM·입력 해시·전쪽 TSV·대표 직접 판독. Page와 제보 원본을 실제 해결로 승격하지 않음. |

## Merge 후 contributor PR comment 계획

부분 해결 승인과 Paper 검증 결과, #7441의 열린 후속 범위를 담은 한국어 본문을 `output/pr-review/pr7610-20261006/review-comment-ko.md`에 준비했다. 앞선 보완 요청 초안은 현재 승인 판정으로 갱신했다.

merge 뒤 최신 CI URL, 처리 범위·남은 문제, [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment), 전쪽 지표와 사람이 확인한 행 복원을 contributor comment에 남긴다. 이미 source에 포함된 `mydocs/pr/assets/issue_7441_paper_overlay_table/` asset은 다음처럼 merge SHA로 고정한다.

```text
https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7441_paper_overlay_table/table-complex-p003-native-review.png
https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7441_paper_overlay_table/table-complex-p003-wasm-overlay.png
```

실제 merge SHA 확인 뒤 UTF-8 without BOM의 `--body-file`로 게시하고 API로 한국어·본문 동일성을 재조회한다. #7441에는 처리된 Paper 부분과 남은 원본·Page 범위를 구분하며 종료 표현을 넣지 않는다.

## 병합 후 기록

- 원 PR은 작업지시자가 승인한 분할정복·부분 통합으로 병합됐다. Paper 검증 결과를 수용하고 Page·제보 원본의 미검증은 열린 #7441의 후속 범위로 유지했다.
- 후속 문서 처리: maintainer 직접 반영. archive review와 오늘할일만 운영 기록 commit으로 보존하며 source/test/workflow/baseline은 섞지 않는다.
- 기본 작업공간 devel을 원 merge SHA로 fast-forward했다. #7441은 OPEN, closed_at=null임을 확인했다.
- 병합 후 workflow는 duration refresh와 issue-close 운영 workflow만 확인했다. 검증 CI·CodeQL·Adapter·Proptest·Oracle은 dispatch하지 않았다.

- 전쪽 TSV·silhouette manifest·대표 PNG·API receipt·실행 로그는 기본 저장소 ignored `output/pr-review/pr7610-20261006/`에도 보존했다. 임시 worktree의 진단 binary·SVG/font raster 중간 파일과 영구 요약을 구분한다.

- [PR 병합 결과 comment](https://github.com/edwardkim/rhwp/pull/7610#issuecomment-6017805110)와 [#7441 부분 처리 comment](https://github.com/edwardkim/rhwp/issues/7441#issuecomment-6017804387)를 게시하고 API로 한국어 본문·UTF-8/BOM·이미지 URL을 재조회했다. #7441은 OPEN으로 유지했다.
- [Duration refresh 37474041054](https://github.com/edwardkim/rhwp/actions/runs/37474041054) success. `ready:true`, `successful-pr-worker-measurements`, source head `cd879723e87dd19913cb99d0d17ec2970f1aa380`, source CI run `37463326904`를 확인했다.
- 재현 요약·TSV·manifest·대표 PNG·로그를 기본 저장소 ignored output에 보존한 뒤 clean 상태의 `/tmp/rhwp-pr7610-review-20261006`, `/tmp/rhwp-pr7610-negative-20261006`과 전용 local review branch·fetch alias를 제거했다. contributor fork branch·공유 target·다른 PR worktree는 보존했다.
