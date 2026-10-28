---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7274 검토

## 판정

**메인터너 보정 후 수용 가능 — 원 contributor head 직접 승인은 아님.** 필수 로컬 검증과 통합 code candidate CI를 확인했다. 전체 PDF 일치를 의미하지 않는다.

## Metadata와 체리픽

- 원 PR [#7274](https://github.com/edwardkim/rhwp/pull/7274): fix(layout): 조각의 시작 컷과 끝 컷 인덱스 공간을 구별한다 (#6935 중복 조판)
- 작성자 planet6897, nondraft, 검토 전 reviewer jangster77 지정.
- 원 head `e52a98ff8731d1c70a1be4e2d6dcd16113b6433d` → cherry-pick `a1b40bdb6881c74e9724c2e17d5176deb3b3cfa8`.
- base `5b481c8b2c5a0aba46eaaa55fe506178caf1ab4c`, branch `codex/planet-7272-7275-20260919`.
- #7272 → #7273 → #7274 → #7275 순서로 충돌 없이 적용. 원 commit rewrite 없음.
- 메인터너 보정 #7274 `7f0a71b1923c1c1342008393cee40b0ca460b71d`, #7275 `e93d7f23684396163876c2a10733f912472fe932`. 최종 컴파일 source `e93d7f23684396163876c2a10733f912472fe932`.
- integration code candidate: `93ae8441134c094d9de0db70a33c29affebf274a`. 원 PR CI를 보정 통합 head CI로 세지 않는다.

## 분석·검증 결과와 남은 범위

원 PR은 텍스트 컷에 `start_cut_is_block`을 추가했지만 producer는 `is_block_split = end || start`를 유지했고,
행 높이는 같은 flag로 start_block을 선택하고 `cell_cut_window(true,true,...)`를 호출했다.
즉 시작 행 공간 컷 `[5,9]`를 앞 rowspan 블록 전체의 컷으로 다시 해석했다.
실물 PDF를 포함한 테스트가 없고 수동 PageItem의 시작 flag만 바꾸는 단위 계약으로 최종 표 크기를 보지 못했다.

실제 18179365 2쪽에서 원 통합은 table height1150.8px, bottom1252.8px로 paper1122.5px를130.3px 넘었다.
실물 HWP를 기존 tracked 파일 전체와 SHA256 대조해 중복 없음 확인 후 tests/fixtures/issue6935에 추가했다.
한컴 engine2020 기준 PDF는 같은 입력으로 start→status→download 성공한3쪽이며 pdf/ 아래 보존한다.
추가 검사 수정 전: 글자 총량 PASS, paper bottom FAIL(exit101). 빌드 실패를 결함 재현으로 세지 않는다.

최종 보정은 행 높이의 start_block·per-row 컷·block-sum에서 `start_cut_is_block`을 소비하도록 한다.
시작 컷 공간을 텍스트 배치와 행 높이에 동일하게 전달하며 특정 문서 ID나 출력 clamp는 없다.
`is_block_split` producer의 기존 start OR end 및 마지막 조각의 start flag는 유지한다.
이 flag는 기존 조각 전체의 블록 분할 gate이며 end 공간만이라는 주석을 바로잡았다.

첫 보정에서 producer까지 end-only로 확대했다가 전체 nextest에서 10089 PASS/3 FAIL/50 skipped를 확인했다.
simsa 입력의 셀 넘침24→26·본문 넘침2→3, cyber 입력의 셀 넘침0→8이었다.
역방향 block-start→row-end에서 기존 예약/배치 계약을 바꾸는 확대를 철회하고 시작 공간 수정만 유지했다.
래칫 허용치를 변경하지 않았다. 첫3개 PR 통합 단계의 27 focused는 이 세 실패 항목을 포함해 모두 PASS다.
simsa는 8쪽 전체 render tree가 원 통합과 정확히 같고, cyber 45쪽 중29쪽만 바뀌며30쪽 회귀는 복구됐다.
cyber28~31쪽 PDF 비교에서29쪽 table height646.7→690.4px로 아래 경계가 기준에 가까워지고 텍스트798자는 보존된다.
28·30·31쪽은 원 통합과 동일하며 큰 기존 페이지별 행 배분 차이는 해결했다고 주장하지 않는다.
27 focused 중 1 leaky 표시는 nextest가 종료 후 남은 핸들을 감지한 것으로 실패로 분류되지는 않았으며 숨기지 않는다.
보정 Native 2쪽 tableheight928.1,bottom1030.1로 paper 밖 넘침 제거. 전체1958자,3쪽,3쪽의 뒤 주석이 유지된다.
base의 RenderTree2289자(331extra)/SVG2273자(315extra)가 통합·보정본1958자로 줄었다.
PDF 대비 whole-document 글자별 누락·추가0. 1쪽reference_only26/2쪽svg_only26/3쪽0은 그대로다.
base대비 table/cell 기하 변화는2쪽에만 있고,1·3쪽은 동일하다. 첫3개 PR 시점 OVR5문서48개체의bbox변화0. #7275를 포함한 최종 OVR은 아래 별도 결과를 따른다.

해결하지 않은 별도 소비 차이:2쪽 reserved920.4px vspaint927.9px(테두리포함928.1), bodybottom보다9.5px아래다.
블록 컷 워크와 rowspan padding의 잔여 차이이며 원래232.1px 본문 넘침 중222.6px만 이번 컷공간 보정으로 해소했다.
PDF와 같은 페이지별 내용 소속/행 경계/continuation 테두리까지 완료했다고 쓰지 않는다. #6935는 반드시 OPEN 유지.

위 simsa/cyber 동일성 비교는 #7275 추가 전7f0a71b 단계다. 최종 simsa는 전체 회귀의 셀·본문 넘침 검사 범위이며 추가 PDF sweep을 했다는 주장은 아니다.

#7275를 추가한 최종본에서도 collision3쪽의 모든 tree 좌표·텍스트는 동일하다. cyber45쪽 중17쪽은 수평으로만1.8~3.8px 이동하며 세로 결과는 유지된다. 원 통합→시작 컷 보정과 그 뒤 수평 원점 보정의 비교 기준을 섞지 않는다.

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
| collision / 1-3 | `tests/fixtures/issue6935/18179365_high_voltage_collision.hwp` | `pdf/18179365_high_voltage_collision-2020.pdf` | 1: 12.27155%, 2: 10.45796%, 3: 11.54104% |
| cyber / 28-31 | `samples/issue6795/1341000-201100013-cyber-university-application.hwp` | `pdf/1341000-201100013-cyber-university-application-2020.pdf` | 28: 10.25943%, 29: 8.95864%, 30: 10.62987%, 31: 8.58778% |

기존 Git 추적 입력/PDF는 이름을 바꾸거나 중복 추가하지 않았다. 신규 입력은 기존 tracked 파일 SHA 대조로 중복이 없음을 확인했다. PDF Creator 연도/빌드 또는 형식1.4만으로 기준을 배제하지 않았다.

실제 collision 입력과 기준 PDF의 SHA·engine2020·전처리 없음·MCP job은 [fixture README](../../../tests/fixtures/issue6935/README.md)에 기록했다.

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

- [native_collision_compare_001.png](../assets/pr7274_review/native_collision_compare_001.png)
- [native_collision_compare_002.png](../assets/pr7274_review/native_collision_compare_002.png)
- [native_collision_compare_003.png](../assets/pr7274_review/native_collision_compare_003.png)
- [native_collision_overlay_001.png](../assets/pr7274_review/native_collision_overlay_001.png)
- [native_collision_overlay_002.png](../assets/pr7274_review/native_collision_overlay_002.png)
- [native_collision_overlay_003.png](../assets/pr7274_review/native_collision_overlay_003.png)
- [native_collision_review_001.png](../assets/pr7274_review/native_collision_review_001.png)
- [native_collision_review_002.png](../assets/pr7274_review/native_collision_review_002.png)
- [native_collision_review_003.png](../assets/pr7274_review/native_collision_review_003.png)
- [native_cyber_compare_028.png](../assets/pr7274_review/native_cyber_compare_028.png)
- [native_cyber_compare_029.png](../assets/pr7274_review/native_cyber_compare_029.png)
- [native_cyber_compare_030.png](../assets/pr7274_review/native_cyber_compare_030.png)
- [native_cyber_compare_031.png](../assets/pr7274_review/native_cyber_compare_031.png)
- [native_cyber_overlay_028.png](../assets/pr7274_review/native_cyber_overlay_028.png)
- [native_cyber_overlay_029.png](../assets/pr7274_review/native_cyber_overlay_029.png)
- [native_cyber_overlay_030.png](../assets/pr7274_review/native_cyber_overlay_030.png)
- [native_cyber_overlay_031.png](../assets/pr7274_review/native_cyber_overlay_031.png)
- [native_cyber_review_028.png](../assets/pr7274_review/native_cyber_review_028.png)
- [native_cyber_review_029.png](../assets/pr7274_review/native_cyber_review_029.png)
- [native_cyber_review_030.png](../assets/pr7274_review/native_cyber_review_030.png)
- [native_cyber_review_031.png](../assets/pr7274_review/native_cyber_review_031.png)
- [wasm_collision_compare_001.png](../assets/pr7274_review/wasm_collision_compare_001.png)
- [wasm_collision_compare_002.png](../assets/pr7274_review/wasm_collision_compare_002.png)
- [wasm_collision_compare_003.png](../assets/pr7274_review/wasm_collision_compare_003.png)
- [wasm_collision_overlay_001.png](../assets/pr7274_review/wasm_collision_overlay_001.png)
- [wasm_collision_overlay_002.png](../assets/pr7274_review/wasm_collision_overlay_002.png)
- [wasm_collision_overlay_003.png](../assets/pr7274_review/wasm_collision_overlay_003.png)
- [wasm_collision_review_001.png](../assets/pr7274_review/wasm_collision_review_001.png)
- [wasm_collision_review_002.png](../assets/pr7274_review/wasm_collision_review_002.png)
- [wasm_collision_review_003.png](../assets/pr7274_review/wasm_collision_review_003.png)
- [wasm_cyber_compare_028.png](../assets/pr7274_review/wasm_cyber_compare_028.png)
- [wasm_cyber_compare_029.png](../assets/pr7274_review/wasm_cyber_compare_029.png)
- [wasm_cyber_compare_030.png](../assets/pr7274_review/wasm_cyber_compare_030.png)
- [wasm_cyber_compare_031.png](../assets/pr7274_review/wasm_cyber_compare_031.png)
- [wasm_cyber_overlay_028.png](../assets/pr7274_review/wasm_cyber_overlay_028.png)
- [wasm_cyber_overlay_029.png](../assets/pr7274_review/wasm_cyber_overlay_029.png)
- [wasm_cyber_overlay_030.png](../assets/pr7274_review/wasm_cyber_overlay_030.png)
- [wasm_cyber_overlay_031.png](../assets/pr7274_review/wasm_cyber_overlay_031.png)
- [wasm_cyber_review_028.png](../assets/pr7274_review/wasm_cyber_review_028.png)
- [wasm_cyber_review_029.png](../assets/pr7274_review/wasm_cyber_review_029.png)
- [wasm_cyber_review_030.png](../assets/pr7274_review/wasm_cyber_review_030.png)
- [wasm_cyber_review_031.png](../assets/pr7274_review/wasm_cyber_review_031.png)
