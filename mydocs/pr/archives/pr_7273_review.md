---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7273 검토

## 판정

**승인 — 명시한 부분 개선 범위.** 필수 로컬 검증과 통합 code candidate CI를 확인했다. 전체 PDF 일치를 의미하지 않는다.

## Metadata와 체리픽

- 원 PR [#7273](https://github.com/edwardkim/rhwp/pull/7273): 수정(layout): 겹침 걸음 사다리의 빈 줄이 저장 전진을 점유로 쓴다 — 중첩 표 −15.9px (#6923 잔여 축 A)
- 작성자 planet6897, nondraft, 검토 전 reviewer jangster77 지정.
- 원 head `50fd51608da60bbe03520e7c6b7a7bed3f5a9f46` → cherry-pick `072ca425526da215f15596603baf3aaf6b581ca9`.
- base `5b481c8b2c5a0aba46eaaa55fe506178caf1ab4c`, branch `codex/planet-7272-7275-20260919`.
- #7272 → #7273 → #7274 → #7275 순서로 충돌 없이 적용. 원 commit rewrite 없음.
- 메인터너 보정 #7274 `7f0a71b1923c1c1342008393cee40b0ca460b71d`, #7275 `e93d7f23684396163876c2a10733f912472fe932`. 최종 컴파일 source `e93d7f23684396163876c2a10733f912472fe932`.
- integration code candidate: `93ae8441134c094d9de0db70a33c29affebf274a`. 원 PR CI를 보정 통합 head CI로 세지 않는다.

## 분석·검증 결과와 남은 범위

저장된 겹침 줄 사다리에서 빈 문단의 840HU + 352HU 전진을 점유로 보존한다.
공통 `stored_overlap_spacer_advance_hu` 결과를 `cell_units` 측정과 `layout_horizontal_cell_paragraphs` 배치가 소비한다.
실물 7쪽 문서와 한컴 PDF를 기존 tracked 경로 그대로 사용했다. 임의 PDF 재생성·fixture 중복 추가 없음.
Native 1~3쪽 직접 sweep. 빈 줄 전진 수정의 세로 기하 변화는 전체7쪽 중1쪽만이며, 중첩 표 top 472.8→488.7 (PDF488),
bottom917.8→933.7 (PDF932)이다. 감싼 표 bottom987.9→1003.8 (PDF1022)의 약18px는 #7095 잔여.
PDF 텍스트 대조는 1쪽만 reference_only1/svg_only1, 2~7쪽은 모두0. PDF에서 없는 상단 logo는 rhwp에 있고
폰트 굵기 차이가 있으므로 전체 fidelity가 일치한다고 쓰지 않는다. 닫힌 #6923의 과거 완료 범위와 #7095를 구분한다. #7275 추가 후5쪽에서 x만3.77px 이동하며 위 세로 결과는 그대로다.

분석 → 코드 수정·검증 → 결과보고 → 커밋 순서를 적용했다. contributor 원 commit은 -x로 보존하고 보정 commit을 분리했다. 로그·JSON·TSV·파생 suite는 커밋하지 않는다. Studio/npm/편집command·Undo 변경은 비해당이고, 별도 성능 A/B는 미측정이다.

## 최종 검증과 한계

- focused `56 PASS`(14.680초), 원 PR 실제 반례의 red/green 확인.
- 전체 nextest: `Summary [ 414.805s] 10095 tests run: 10095 passed (6 slow, 1 leaky), 50 skipped`.
- Native Skia lib `4,112 PASS / 13 ignored`, placeholder `2 PASS`, 직접PDF export `4 PASS`.
- fmt, Native/WASM32/workspace all-target Clippy, workspace build, manifest 및 unit-tier 정책(base 고정) 모두 exit0.
- OVR5문서48개체: 기존2px허용범위에서회귀0. KTX만최대1.9px수평이동, 나머지4문서는이동0; 이를전체좌표동일로쓰지않는다.
- Native/fresh WASM 14문서711쪽 tree구조·텍스트·bbox차이0. 각32쪽PDF비교, 실제PNG31/32쪽동일. stack4제목의폰트공급CSS차이는같은CSS통제에서PNG동일로확인했다.
- 최종 compare·standalone overlay·review 및 전후/대응비교 PNG201개보존. 전체PDF일치판정은아니다.
- 필수 명령: `cargo nextest run --locked --cargo-profile release-test --tests --no-fail-fast`; `cargo test --locked --profile release-test --features native-skia --lib`; `cargo fmt --all -- --check`; `cargo clippy --locked -- -D warnings`; `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown -- -D warnings`; `cargo build --locked --workspace`; `cargo clippy --locked --workspace --all-targets -- -D warnings`; manifest/unit-tier `--check --base-ref 5b481c8b2c5a0aba46eaaa55fe506178caf1ab4c`.

검증 빌드 SHA256:
- `rhwp-maintainer4`: `ded73871ffae61ed1161a26767d4cd843c4bf52d7d4d51b5fc93363a2da60d54`
- `rhwp.js`: `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`
- `rhwp_bg.wasm`: `ec21e0b957c22a17170a4448dc6a87e3ffc843ffb82b96a849b85804444bd1d3`


검증 작업공간 `/private/tmp/rhwp-verify-7272-7274-20260919`와 전용 target `target/planet-7272-7274-20260919`를 썼다. 작업공간 git HEAD만으로 source를 식별하지 않고 실제 컴파일한 source/test/fixture28개를 최종 code source와 바이트 대조했다. Cargo는 순차 실행했다. test source 변경 뒤 --prepare와 base 고정 --check를 다시 수행했다. 앞 단계의 stale generated suite로 발생한 정책 실패는 재생성 후 해소했으며 성공으로 세지 않았다.

Docker daemon 연결 실패로 fresh WASM은 `scripts/wasm-pack-locked.sh --target web --out-dir <fresh-pkg> --no-opt` 호스트 경로를 사용했다. 로컬 최적화 Docker 빌드 성공 주장은 하지 않는다.

## Visual Sweep 범위

[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 따른다. 최종 Native/fresh WASM 각각14문서32쪽의 compare·standalone overlay·review를 새로 캡처했다. 직접 판독한 대표14문서의 경계·겹침·글꼴·잔여 차이와 전후 좌표 비교를 구분한다. 전체711쪽 tree parity는 전체711쪽 PDF 시각 일치를 뜻하지 않는다. 같은 내용의 ghost rhwp15/PDF14 대응 비교를 추가했으며 물리 페이지 번호를 속이지 않았다.

명령은 다음 형식이며 아래 입력/PDF/쪽 조합을 사용했다. Native binary는 `rhwp-maintainer4`, fresh package는 `wasm4`다.

```sh
venv/bin/python scripts/visual_sweep.py --file-target <key> <input> <pdf> \
  --rhwp-bin <verified-native> --pages <pages> --dpi 96 --out <out>
# fresh WASM에는 --wasm-pkg <fresh-pkg> 추가
```

Native/WASM 실제PNG는31/32쪽이 바이트까지 동일했다. stack4쪽 제목 영역1593픽셀만 달랐으며, SVG의CSS를 제외한 내용·좌표는동일하다. WASM sweep의 문서 전체 폰트 별칭CSS 때문에 대체 글꼴이 달라지고, Native에같은CSS를 공급한 통제 캡처는 WASM PNG와 바이트까지 같았다. 원래 backend별 캡처를 보존하며32쪽완전동일로쓰지않는다.

자동 점수를 사람의 판정 정확도나 전체 fidelity 통과로 사용하지 않는다.
높을수록 기준 PDF와 PNG가 더 비슷하고, 낮을수록 잉크 위치·형태 차이를 검토해야 하는 내용 픽셀 중심 보조값이다.


## 입력·기준 PDF·시각 지표

| key / 쪽 | 입력 | 독립 기준 PDF | 최종 Native 내용 픽셀 보조값 |
| --- | --- | --- | --- |
| wrapper / 1-3 | `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame.hwp` | `tests/fixtures/issue6923/148738070_wrapper_table_stored_page_frame-2020.pdf` | 1: 37.89591%, 2: 26.6657%, 3: 20.36042% |

기존 Git 추적 입력/PDF는 이름을 바꾸거나 중복 추가하지 않았다. 신규 입력은 기존 tracked 파일 SHA 대조로 중복이 없음을 확인했다. PDF Creator 연도/빌드 또는 형식1.4만으로 기준을 배제하지 않았다.

## 통합 candidate CI와 trailing 기록

Code candidate `93ae8441134c094d9de0db70a33c29affebf274a`에서 다음 원격 검증이 모두 성공했다. 원 PR CI와 구분하며 같은 통합 branch·head의 결과다.

- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/35439418133) — success
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/35439418242) — success
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/35439417978) — success
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/35439418284) — success
- [CI](https://github.com/edwardkim/rhwp/actions/runs/35439418176) — success

필수 로컬 검증을 완료한 source/test는 이후 변경하지 않았다. 이 기록은 해당 code candidate 뒤 trailing docs-only commit이며 최종 head의 preflight·재사용·필수 checks를 확인한 뒤 merge한다.

개별 검토와 오늘할일은 녹색 code candidate 뒤의 동일 PR trailing docs-only commit으로 반영한다. 최종 head 검사·mergeability·merge 완료와 후속처리는 별도 확인하며 미리 완료로 쓰지 않는다.

## 대표 증적과 Merge 후 contributor PR comment 계획

실제 merge SHA·CI URL·소스와 보정 출처·검증 범위·남은 차이를 한국어로 설명하고 기여에 감사한다. 위 Visual Sweep 정본과 연결하고 아래 최종 review 및 standalone overlay를 원 PR과 관련 issue comment에 **이미지로 직접 표시**한다. compare도 링크로 제공한다. URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<actual-merge-sha>/mydocs/pr/assets/prN_review/<filename>` 형식이며 N은 이 원 PR 번호다. UTF-8 파일과 --body-file로 게시하고 SHA·한국어·이미지 URL을 다시 확인한다. 원 PR은 통합 provenance를 설명한 뒤 close한다. #7092·#7095·#6935·#7063은 남은 범위가 있어 열어 두고, 이미 닫힌 #6923은 추가 개선 범위만 설명한다.

- [native_wrapper_compare_001.png](../assets/pr7273_review/native_wrapper_compare_001.png)
- [native_wrapper_compare_002.png](../assets/pr7273_review/native_wrapper_compare_002.png)
- [native_wrapper_compare_003.png](../assets/pr7273_review/native_wrapper_compare_003.png)
- [native_wrapper_overlay_001.png](../assets/pr7273_review/native_wrapper_overlay_001.png)
- [native_wrapper_overlay_002.png](../assets/pr7273_review/native_wrapper_overlay_002.png)
- [native_wrapper_overlay_003.png](../assets/pr7273_review/native_wrapper_overlay_003.png)
- [native_wrapper_review_001.png](../assets/pr7273_review/native_wrapper_review_001.png)
- [native_wrapper_review_002.png](../assets/pr7273_review/native_wrapper_review_002.png)
- [native_wrapper_review_003.png](../assets/pr7273_review/native_wrapper_review_003.png)
- [wasm_wrapper_compare_001.png](../assets/pr7273_review/wasm_wrapper_compare_001.png)
- [wasm_wrapper_compare_002.png](../assets/pr7273_review/wasm_wrapper_compare_002.png)
- [wasm_wrapper_compare_003.png](../assets/pr7273_review/wasm_wrapper_compare_003.png)
- [wasm_wrapper_overlay_001.png](../assets/pr7273_review/wasm_wrapper_overlay_001.png)
- [wasm_wrapper_overlay_002.png](../assets/pr7273_review/wasm_wrapper_overlay_002.png)
- [wasm_wrapper_overlay_003.png](../assets/pr7273_review/wasm_wrapper_overlay_003.png)
- [wasm_wrapper_review_001.png](../assets/pr7273_review/wasm_wrapper_review_001.png)
- [wasm_wrapper_review_002.png](../assets/pr7273_review/wasm_wrapper_review_002.png)
- [wasm_wrapper_review_003.png](../assets/pr7273_review/wasm_wrapper_review_003.png)
