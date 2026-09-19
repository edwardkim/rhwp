---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-19
---

# PR #7275 검토

## 판정

**메인터너 보정 후 수용 가능 — 원 contributor head 직접 승인은 아님.** 필수 로컬 검증과 통합 code candidate CI를 확인했다. 전체 PDF 일치를 의미하지 않는다.

## Metadata와 체리픽

- 원 PR [#7275](https://github.com/edwardkim/rhwp/pull/7275): 수정(layout): 자리차지 표의 왼끝이 저장 horzOffset 이 아니라 그 안쪽 바깥여백 상자다 (#7063)
- 작성자 planet6897, nondraft, 검토 전 reviewer jangster77 지정.
- 원 head `108009618bdf6785e7dc02de7df2f67fab0cd8af` → cherry-pick `e9a5158cfa98fb9f47c5a08c18f3f46b03328d0c`.
- base `5b481c8b2c5a0aba46eaaa55fe506178caf1ab4c`, branch `codex/planet-7272-7275-20260919`.
- #7272 → #7273 → #7274 → #7275 순서로 충돌 없이 적용. 원 commit rewrite 없음.
- 메인터너 보정 #7274 `7f0a71b1923c1c1342008393cee40b0ca460b71d`, #7275 `e93d7f23684396163876c2a10733f912472fe932`. 최종 컴파일 source `e93d7f23684396163876c2a10733f912472fe932`.
- integration code candidate: `93ae8441134c094d9de0db70a33c29affebf274a`. 원 PR CI를 보정 통합 head CI로 세지 않는다.

## 분석·검증 결과와 남은 범위

자리차지(TopAndBottom), 비글줄참여, Para/Column 기준 Left/Inside 표의 horzOffset은 바깥 여백 상자의 원점이다. 표 테두리는 outMargin.left 안쪽에 놓인다. Paper/Page 기준·다른 정렬·Square·글줄참여는 이 helper의 적용 대상이 아니다. 저장 오프셋이0이 아닌14쪽(Column158HU)도 양성 사례다.

원 PR의 `float_placement.rs:1425` → `table_layout.rs:5029` 공통 x계산 → 일반/partial 배치까지 추적했다. 기존 native-empty-host와 stored-reset의 가로 보정은 제거해 한 번만 적용하고 세로 보정은 유지한다. 다만 wrapper unwrap(`table_layout.rs:2839`)은 부모와 같은 depth로 자식을 놓으며 기준 영역에 자식 여백을 이미 포함하는 **세 번째 소비 지점**이었다. 원 PR은 이 경로를 놓쳐 외곽 테두리와 자식 원점에 여백을 재가산했다.

정상 대조군 `samples/80168_regulatory_analysis.hwp` 6쪽 세로 괘선 좌단은 원 PR 적용 전79.3467px, 원 PR 적용 후81.2267px였다. 독립 한컴 PDF의 x는79.317px, y=882.715..988.679px(PyMuPDF get_drawings,96/72환산)이다. 원 PR의55focused와 원격 CI가 모두 통과해도 이 회귀는 남았다.

메인터너 보정 `e93d7f236`은 이미 적용된 여백의 소유를 공통 x계산에 전달하고, wrapper 테두리의 추가 inset을 제거한다. wrapper 안쪽 기준 영역도 같은 일반 helper를 사용한다. 특정 문서 ID·임의 clamp·출력 숨김은 없다. partial은 자체 기준 영역이므로 false를 전달한다. 저장본 확인을 편집 후 전체 재조판 입증으로 확대하지 않는다.

추가한 실제 fixture 검사 `block_wrapper_keeps_its_pdf_left_edge_when_margin_is_already_owned`는 원 PR source에서2PASS/1FAIL(exit100,좌단81.227)이고 보정 후3PASS다. 원 PR의 기존 두 검사는 #7275 적용 전 source에서 모두 의도한 x결함으로 실패(exit100)했다. 최종 focused56PASS, wrapper 좌단79.3467px로 복구했다. 전체 source14문서711쪽 전후 비교는 x변화만 있고 구조·텍스트·y·w·h·페이지 수 차이는0이다.

`hwpx_sample2` 14쪽39.90→43.68px(PDF43.66),19쪽37.79→39.67px(PDF39.66)로 의도한 원점을 맞춘다. 기준값을 옮긴 issue2214/2215·2230·3738·6655/4771·5877은 각각 별도 문서의 PDF/그림/괘선과 대조했다. form002·issue157 golden의 x평행이동도 실제 변경과 일치한다. 구현 출력을 그대로 새로운 기대값으로 삼아 회귀를 감추지 않았다.

남은 차이: hwpx_sample2의 세로 배치(특히14쪽)와 Center 경로는 비범위이며 #7063을 닫지 않는다. ghost는 기존부터 rhwp32쪽/PDF31쪽으로 같은 내용이 rhwp15쪽↔PDF14쪽에 있다. 동일 물리 페이지 비교와 별도 내용 대응 비교를 모두 보존하며 행 높이·열 폭·테두리·페이지 내용 차이가 남는다. 로고 문서의 placeholder, 여러 문서의 글꼴·세로 간격도 해결했다고 쓰지 않는다.

작성자의1,139문서 전수A/B와 우측overflow+206에 대한 설명은 작성자 증거다. 이 검토에서 전수 재실행하지 않았으므로 전체206건을 독립 승인했다고 주장하지 않는다. 직접 검증한14문서의 x규칙과 대조군, 필수 전체 회귀 결과로 이번 부분 개선 범위를 판정한다.

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
| wrapbox / 6,23 | `samples/80168_regulatory_analysis.hwp` | `pdf/80168_regulatory_analysis-2022.pdf` | 6: 22.05591%, 23: 13.59764% |
| margin / 14,15,19,20 | `samples/hwpx_sample2.hwpx` | `pdf/hwpx_sample2-hwpx-2020.pdf` | 14: 6.04629%, 15: 41.32429%, 19: 20.2467%, 20: 32.42545% |
| physical / 1 | `samples/tac-img-02.hwp` | `pdf/tac-img-02-hwp-2020.pdf` | 1: 48.27579% |
| selection / 1 | `samples/issue1949_giant_cell_nested_tables_perf.hwp` | `pdf/issue1949_giant_cell_nested_tables_perf-hwp-2024.pdf` | 1: 13.59921% |
| logo / 1 | `samples/hwpx/opengov/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177).hwpx` | `pdf/36389312_결재문서본문_특정소방대상물 화재발생 알림(화재번호 2026-177)-2024.pdf` | 1: 24.57559% |
| policy / 23,167,168 | `samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp` | `pdf/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구)-hwp-2024.pdf` | 23: 18.71489%, 167: 20.5149%, 168: 33.68731% |
| stack / 4-8 | `samples/issue2004_cell_image_stack.hwp` | `pdf/issue2004_cell_image_stack-2022.pdf` | 4: 87.18035%, 5: 82.6125%, 6: 45.26229%, 7: 71.08918%, 8: 72.12596% |
| ghost / 14-15 | `samples/issue5877/fragment_ghost_vrules.hwp` | `pdf/fragment_ghost_vrules-2020.pdf` | 14: 6.32984%, 15: 7.25392% |
| kwater / 17 | `samples/k-water-rfp.hwp` | `pdf/k-water-rfp-2022.pdf` | 17: 59.81834% |
| social / 1 | `samples/exam_social.hwp` | `pdf/exam_social-2022.pdf` | 1: 12.83001% |

기존 Git 추적 입력/PDF는 이름을 바꾸거나 중복 추가하지 않았다. 신규 입력은 기존 tracked 파일 SHA 대조로 중복이 없음을 확인했다. PDF Creator 연도/빌드 또는 형식1.4만으로 기준을 배제하지 않았다.

새 기준 `pdf/fragment_ghost_vrules-2020.pdf`: 기존 tracked HWP(저장 Hancom2020 11.0.0.4278)로 MCP job `3ca7750f-3791-409d-90ea-29fda830adcb`를 실행했다. engine2020 / 실행 Hancom11.0.0.9136 / 전처리 없음 / status SUCCEEDED 확인 후 다운로드. 31쪽·565927bytes·PDF1.4·Creator Hwp2020 0.0.0.0·Producer Hancom PDF1.3.0.550. SHA256 `71dd147ab8d9be9f0d79b6e8d42c62d293cbb88e9a7cb645715aa6896343e22f`로 client/server 일치. 원문 HWP는 중복 추가하지 않았다.

원 PR wrapper 실패본은 `before_wrapbox_*_006.png`, 최종본은 `native_wrapbox_*_006.png`로 구분한다. ghost의 `*_ghost_mapped_*_015.png`는 PDF14쪽에 대응한다. 보조값은6.938%이며 기존 페이지 배분·세로 차이가 남는다.

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

- [before_wrapbox_compare_006.png](../assets/pr7275_review/before_wrapbox_compare_006.png)
- [before_wrapbox_overlay_006.png](../assets/pr7275_review/before_wrapbox_overlay_006.png)
- [before_wrapbox_review_006.png](../assets/pr7275_review/before_wrapbox_review_006.png)
- [native_ghost_compare_014.png](../assets/pr7275_review/native_ghost_compare_014.png)
- [native_ghost_compare_015.png](../assets/pr7275_review/native_ghost_compare_015.png)
- [native_ghost_mapped_compare_015.png](../assets/pr7275_review/native_ghost_mapped_compare_015.png)
- [native_ghost_mapped_overlay_015.png](../assets/pr7275_review/native_ghost_mapped_overlay_015.png)
- [native_ghost_mapped_review_015.png](../assets/pr7275_review/native_ghost_mapped_review_015.png)
- [native_ghost_overlay_014.png](../assets/pr7275_review/native_ghost_overlay_014.png)
- [native_ghost_overlay_015.png](../assets/pr7275_review/native_ghost_overlay_015.png)
- [native_ghost_review_014.png](../assets/pr7275_review/native_ghost_review_014.png)
- [native_ghost_review_015.png](../assets/pr7275_review/native_ghost_review_015.png)
- [native_kwater_compare_017.png](../assets/pr7275_review/native_kwater_compare_017.png)
- [native_kwater_overlay_017.png](../assets/pr7275_review/native_kwater_overlay_017.png)
- [native_kwater_review_017.png](../assets/pr7275_review/native_kwater_review_017.png)
- [native_logo_compare_001.png](../assets/pr7275_review/native_logo_compare_001.png)
- [native_logo_overlay_001.png](../assets/pr7275_review/native_logo_overlay_001.png)
- [native_logo_review_001.png](../assets/pr7275_review/native_logo_review_001.png)
- [native_margin_compare_014.png](../assets/pr7275_review/native_margin_compare_014.png)
- [native_margin_compare_015.png](../assets/pr7275_review/native_margin_compare_015.png)
- [native_margin_compare_019.png](../assets/pr7275_review/native_margin_compare_019.png)
- [native_margin_compare_020.png](../assets/pr7275_review/native_margin_compare_020.png)
- [native_margin_overlay_014.png](../assets/pr7275_review/native_margin_overlay_014.png)
- [native_margin_overlay_015.png](../assets/pr7275_review/native_margin_overlay_015.png)
- [native_margin_overlay_019.png](../assets/pr7275_review/native_margin_overlay_019.png)
- [native_margin_overlay_020.png](../assets/pr7275_review/native_margin_overlay_020.png)
- [native_margin_review_014.png](../assets/pr7275_review/native_margin_review_014.png)
- [native_margin_review_015.png](../assets/pr7275_review/native_margin_review_015.png)
- [native_margin_review_019.png](../assets/pr7275_review/native_margin_review_019.png)
- [native_margin_review_020.png](../assets/pr7275_review/native_margin_review_020.png)
- [native_physical_compare_001.png](../assets/pr7275_review/native_physical_compare_001.png)
- [native_physical_overlay_001.png](../assets/pr7275_review/native_physical_overlay_001.png)
- [native_physical_review_001.png](../assets/pr7275_review/native_physical_review_001.png)
- [native_policy_compare_023.png](../assets/pr7275_review/native_policy_compare_023.png)
- [native_policy_compare_167.png](../assets/pr7275_review/native_policy_compare_167.png)
- [native_policy_compare_168.png](../assets/pr7275_review/native_policy_compare_168.png)
- [native_policy_overlay_023.png](../assets/pr7275_review/native_policy_overlay_023.png)
- [native_policy_overlay_167.png](../assets/pr7275_review/native_policy_overlay_167.png)
- [native_policy_overlay_168.png](../assets/pr7275_review/native_policy_overlay_168.png)
- [native_policy_review_023.png](../assets/pr7275_review/native_policy_review_023.png)
- [native_policy_review_167.png](../assets/pr7275_review/native_policy_review_167.png)
- [native_policy_review_168.png](../assets/pr7275_review/native_policy_review_168.png)
- [native_selection_compare_001.png](../assets/pr7275_review/native_selection_compare_001.png)
- [native_selection_overlay_001.png](../assets/pr7275_review/native_selection_overlay_001.png)
- [native_selection_review_001.png](../assets/pr7275_review/native_selection_review_001.png)
- [native_social_compare_001.png](../assets/pr7275_review/native_social_compare_001.png)
- [native_social_overlay_001.png](../assets/pr7275_review/native_social_overlay_001.png)
- [native_social_review_001.png](../assets/pr7275_review/native_social_review_001.png)
- [native_stack_compare_004.png](../assets/pr7275_review/native_stack_compare_004.png)
- [native_stack_compare_005.png](../assets/pr7275_review/native_stack_compare_005.png)
- [native_stack_compare_006.png](../assets/pr7275_review/native_stack_compare_006.png)
- [native_stack_compare_007.png](../assets/pr7275_review/native_stack_compare_007.png)
- [native_stack_compare_008.png](../assets/pr7275_review/native_stack_compare_008.png)
- [native_stack_overlay_004.png](../assets/pr7275_review/native_stack_overlay_004.png)
- [native_stack_overlay_005.png](../assets/pr7275_review/native_stack_overlay_005.png)
- [native_stack_overlay_006.png](../assets/pr7275_review/native_stack_overlay_006.png)
- [native_stack_overlay_007.png](../assets/pr7275_review/native_stack_overlay_007.png)
- [native_stack_overlay_008.png](../assets/pr7275_review/native_stack_overlay_008.png)
- [native_stack_review_004.png](../assets/pr7275_review/native_stack_review_004.png)
- [native_stack_review_005.png](../assets/pr7275_review/native_stack_review_005.png)
- [native_stack_review_006.png](../assets/pr7275_review/native_stack_review_006.png)
- [native_stack_review_007.png](../assets/pr7275_review/native_stack_review_007.png)
- [native_stack_review_008.png](../assets/pr7275_review/native_stack_review_008.png)
- [native_wrapbox_compare_006.png](../assets/pr7275_review/native_wrapbox_compare_006.png)
- [native_wrapbox_compare_023.png](../assets/pr7275_review/native_wrapbox_compare_023.png)
- [native_wrapbox_overlay_006.png](../assets/pr7275_review/native_wrapbox_overlay_006.png)
- [native_wrapbox_overlay_023.png](../assets/pr7275_review/native_wrapbox_overlay_023.png)
- [native_wrapbox_review_006.png](../assets/pr7275_review/native_wrapbox_review_006.png)
- [native_wrapbox_review_023.png](../assets/pr7275_review/native_wrapbox_review_023.png)
- [wasm_ghost_compare_014.png](../assets/pr7275_review/wasm_ghost_compare_014.png)
- [wasm_ghost_compare_015.png](../assets/pr7275_review/wasm_ghost_compare_015.png)
- [wasm_ghost_mapped_compare_015.png](../assets/pr7275_review/wasm_ghost_mapped_compare_015.png)
- [wasm_ghost_mapped_overlay_015.png](../assets/pr7275_review/wasm_ghost_mapped_overlay_015.png)
- [wasm_ghost_mapped_review_015.png](../assets/pr7275_review/wasm_ghost_mapped_review_015.png)
- [wasm_ghost_overlay_014.png](../assets/pr7275_review/wasm_ghost_overlay_014.png)
- [wasm_ghost_overlay_015.png](../assets/pr7275_review/wasm_ghost_overlay_015.png)
- [wasm_ghost_review_014.png](../assets/pr7275_review/wasm_ghost_review_014.png)
- [wasm_ghost_review_015.png](../assets/pr7275_review/wasm_ghost_review_015.png)
- [wasm_kwater_compare_017.png](../assets/pr7275_review/wasm_kwater_compare_017.png)
- [wasm_kwater_overlay_017.png](../assets/pr7275_review/wasm_kwater_overlay_017.png)
- [wasm_kwater_review_017.png](../assets/pr7275_review/wasm_kwater_review_017.png)
- [wasm_logo_compare_001.png](../assets/pr7275_review/wasm_logo_compare_001.png)
- [wasm_logo_overlay_001.png](../assets/pr7275_review/wasm_logo_overlay_001.png)
- [wasm_logo_review_001.png](../assets/pr7275_review/wasm_logo_review_001.png)
- [wasm_margin_compare_014.png](../assets/pr7275_review/wasm_margin_compare_014.png)
- [wasm_margin_compare_015.png](../assets/pr7275_review/wasm_margin_compare_015.png)
- [wasm_margin_compare_019.png](../assets/pr7275_review/wasm_margin_compare_019.png)
- [wasm_margin_compare_020.png](../assets/pr7275_review/wasm_margin_compare_020.png)
- [wasm_margin_overlay_014.png](../assets/pr7275_review/wasm_margin_overlay_014.png)
- [wasm_margin_overlay_015.png](../assets/pr7275_review/wasm_margin_overlay_015.png)
- [wasm_margin_overlay_019.png](../assets/pr7275_review/wasm_margin_overlay_019.png)
- [wasm_margin_overlay_020.png](../assets/pr7275_review/wasm_margin_overlay_020.png)
- [wasm_margin_review_014.png](../assets/pr7275_review/wasm_margin_review_014.png)
- [wasm_margin_review_015.png](../assets/pr7275_review/wasm_margin_review_015.png)
- [wasm_margin_review_019.png](../assets/pr7275_review/wasm_margin_review_019.png)
- [wasm_margin_review_020.png](../assets/pr7275_review/wasm_margin_review_020.png)
- [wasm_physical_compare_001.png](../assets/pr7275_review/wasm_physical_compare_001.png)
- [wasm_physical_overlay_001.png](../assets/pr7275_review/wasm_physical_overlay_001.png)
- [wasm_physical_review_001.png](../assets/pr7275_review/wasm_physical_review_001.png)
- [wasm_policy_compare_023.png](../assets/pr7275_review/wasm_policy_compare_023.png)
- [wasm_policy_compare_167.png](../assets/pr7275_review/wasm_policy_compare_167.png)
- [wasm_policy_compare_168.png](../assets/pr7275_review/wasm_policy_compare_168.png)
- [wasm_policy_overlay_023.png](../assets/pr7275_review/wasm_policy_overlay_023.png)
- [wasm_policy_overlay_167.png](../assets/pr7275_review/wasm_policy_overlay_167.png)
- [wasm_policy_overlay_168.png](../assets/pr7275_review/wasm_policy_overlay_168.png)
- [wasm_policy_review_023.png](../assets/pr7275_review/wasm_policy_review_023.png)
- [wasm_policy_review_167.png](../assets/pr7275_review/wasm_policy_review_167.png)
- [wasm_policy_review_168.png](../assets/pr7275_review/wasm_policy_review_168.png)
- [wasm_selection_compare_001.png](../assets/pr7275_review/wasm_selection_compare_001.png)
- [wasm_selection_overlay_001.png](../assets/pr7275_review/wasm_selection_overlay_001.png)
- [wasm_selection_review_001.png](../assets/pr7275_review/wasm_selection_review_001.png)
- [wasm_social_compare_001.png](../assets/pr7275_review/wasm_social_compare_001.png)
- [wasm_social_overlay_001.png](../assets/pr7275_review/wasm_social_overlay_001.png)
- [wasm_social_review_001.png](../assets/pr7275_review/wasm_social_review_001.png)
- [wasm_stack_compare_004.png](../assets/pr7275_review/wasm_stack_compare_004.png)
- [wasm_stack_compare_005.png](../assets/pr7275_review/wasm_stack_compare_005.png)
- [wasm_stack_compare_006.png](../assets/pr7275_review/wasm_stack_compare_006.png)
- [wasm_stack_compare_007.png](../assets/pr7275_review/wasm_stack_compare_007.png)
- [wasm_stack_compare_008.png](../assets/pr7275_review/wasm_stack_compare_008.png)
- [wasm_stack_overlay_004.png](../assets/pr7275_review/wasm_stack_overlay_004.png)
- [wasm_stack_overlay_005.png](../assets/pr7275_review/wasm_stack_overlay_005.png)
- [wasm_stack_overlay_006.png](../assets/pr7275_review/wasm_stack_overlay_006.png)
- [wasm_stack_overlay_007.png](../assets/pr7275_review/wasm_stack_overlay_007.png)
- [wasm_stack_overlay_008.png](../assets/pr7275_review/wasm_stack_overlay_008.png)
- [wasm_stack_review_004.png](../assets/pr7275_review/wasm_stack_review_004.png)
- [wasm_stack_review_005.png](../assets/pr7275_review/wasm_stack_review_005.png)
- [wasm_stack_review_006.png](../assets/pr7275_review/wasm_stack_review_006.png)
- [wasm_stack_review_007.png](../assets/pr7275_review/wasm_stack_review_007.png)
- [wasm_stack_review_008.png](../assets/pr7275_review/wasm_stack_review_008.png)
- [wasm_wrapbox_compare_006.png](../assets/pr7275_review/wasm_wrapbox_compare_006.png)
- [wasm_wrapbox_compare_023.png](../assets/pr7275_review/wasm_wrapbox_compare_023.png)
- [wasm_wrapbox_overlay_006.png](../assets/pr7275_review/wasm_wrapbox_overlay_006.png)
- [wasm_wrapbox_overlay_023.png](../assets/pr7275_review/wasm_wrapbox_overlay_023.png)
- [wasm_wrapbox_review_006.png](../assets/pr7275_review/wasm_wrapbox_review_006.png)
- [wasm_wrapbox_review_023.png](../assets/pr7275_review/wasm_wrapbox_review_023.png)
