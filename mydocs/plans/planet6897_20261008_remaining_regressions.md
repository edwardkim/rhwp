---
kind: plan
status: active
canonical: mydocs/plans/planet6897_20261008_remaining_regressions.md
last_verified: 2026-10-11
---

# planet6897 누적 검토 — 남은 회귀 실패 보정

검토 branch는 `review/planet6897-20261008`, 고정 upstream base는
`790a6f8f388445e7bff2bcb165aa7b384f9ece80`이다. #7813 추가 이관 commit
`b493b6887664b2aa11c6bc484802703c8f71c894` 이후 남은 실패 9건을 처리한다.
이관 4건은 다시 출력하거나 검사하지 않는다. 전체 회귀 통과·PR 승인 상태와 구별한다.

## 범위와 독립 근거

| 대상 | 근거·처리 |
| --- | --- |
| `svg_snapshot::table_text_page_0` | 원문 음수 자간과 독립 Print의 중앙 정렬. 전진폭에 포함된 말미 자간을 점유폭으로 혼동한 검사를 수정한다. 내용·소유·포함·중앙 관계와 기존 상대 오차는 유지한다. |
| 중첩 래칫 partition 6/7/8/12/15 | 분산 정렬의 마지막 전진 간격은 잉크가 아니다. 진단의 말미 점유를 보정하고 실제 충돌 대조군으로 확인한다. 원장 수치 증가로 통과시키지 않는다. 도장 표시와의 기존 Print 교차는 별도 소유 관계로 대조한다. |
| `issue_7063_midpage_fragment_box_does_not_pull_next_page_rows` | `hwpx_sample2.hwp`의 base/current/Print 대조에서 제목·문단 소유는 보존되지만 현재 8쪽 표 하단이 짧다. 사용자가 29쪽 전면 개선 비용 때문에 #7830으로 이관하도록 지시했다. 현재 새 차이를 미해결로 보존하고 기대값은 변경하지 않는다. |
| `issue_2430_cell_rewrap_threshold_no_oversplit`, `issue_7095_terminal_fragment_box_takes_the_stored_cell_height_remainder` | 독립 KoPub Print와 base는39쪽, 현재37쪽이다. 사용자 지시로 #7831에 이관한다. 현재의 새 쪽수·이어받기 회귀를 미해결로 보존하며 현재 값으로 baseline을 바꾸지 않는다. |

## 시각 선행 조건과 1단계 결과

아래 시각 증적의 renderer source는 `d0528fb55f225fba8b4b65b9ed670a8dee9c654b`이다.
`b493b6887`은 검사 제거만 변경했으며 생산 source와 입력 바이트가 같다.
Native/fresh WASM 전쪽 TSV 최저값: table-text 99.99320%, stored-vpos 98.77240%,
hwp-multi 95.64710%, 해외투자 HWPX/HWP/복제본 각각91.97621%이다.
쪽수는 각각1/4/9/9/9/9쪽이며 누락·90% 미달은 없다.
해당 원본·Print 해시와 실행 명령은 ignored
`output/pr-review/planet6897-20261008/linux-validation-20261011/visual-audit/inputs.json`
및 각 `run_manifest.json`에 보존한다.

table-text Print/Native/WASM 같은 제목 확대 판독에서 칸 소유·중앙 배치·기호와 간격을 확인했다.
원문 말미 글자 모양의 음수 자간을 제거한 점유 끝으로 검사한다. 수치의 독립
`/usr/local/share/fonts/hancom-office-2020/Install/malgun.ttf` hmtx는 숫자0/8 모두
1128/2048em이다. 글꼴 해시는 ignored `table-text-independent-metrics.json`에 고정했다.
절대 px 위치나 배치 helper의 재인용 없이 원문 크기·장평·상대 크기·자간과 이 메트릭을 쓴다.
수정 전 전체 nextest는 제목 중앙 assertion에서 FAIL. 첫 보정은 수치 행의 같은 문제를 드러냈고,
수치까지 보정한 집중 nextest는4건 실행·4건 PASS(exit0)다.

- 수정 전: `full-nextest.log`, `remaining-table-text.log`(3PASS/1FAIL).
- 수정 후: `remaining-table-text-corrected.log`, run `c097b6ed-82e9-4eba-8715-df95dd197e43`.
- Print/Native 확대: `table-text-native-review/table-text/*title-crop.png`.
- Print/WASM 확대: `table-text-wasm-review/table-text/*title-zoom.png`.

위 상대 경로는 모두 동일 ignored `linux-validation-20261011/` 아래다.
필수3종 Clippy·최종 전체 회귀는 보정 묶음의 최종 head에서 수행한다. 나머지 실패는 아직 미해결이다.

## 2단계 구현 전 소비 경로

진단은 렌더 트리를 읽는 소비자다. `TextRunNode::layout_positions`/replay 원점 →
`layout_anomaly::glyph_band_horizontal` → `glyph_band_bbox` → text-overlap 후보 교차와
코퍼스 래칫 순서다. SVG/Canvas/Skia의 실제 글자 원점은 변경하지 않는다.
마지막 분산 간격만 걷어내되 말미 공백·손상된 positions·실제 말미 글자 충돌을 구별한다.
합성 입력의 전각 글자 폭과 명시적 replay 원점으로 독립 기대를 정한다.
현재 문서의 잉크·괘선 확대와 정상 대조군은 원장 수치와 별개로 기록한다.

## 2단계 결과 — 진단 점유와 도장 소유

`src/diagnostics/layout_anomaly/text_occupancy.rs`는 마지막 유효 replay 원점과
글자의 기본 폭·장평으로 분산 간격을 점유폭에서 제외한다. production renderer 배치는
변경하지 않았다. 전각 1em·명시 원점의 독립 경계는 수정 전 의도한 교차에서 FAIL,
수정 후 실제 글자 충돌은 유지하고 빈 분산 간격 교차만 제거한다.

- 수정 전 경계: `remaining-overlap-boundary-before.log` 1FAIL(exit100).
- 수정 후 코퍼스·경계: `remaining-overlap-after.log` 27건 실행,26PASS/1FAIL(exit100).
  partition6/7/12/15는 통과했고, partition8의 도장 교차만 남았다.
- Print와 같은 두 사진칸의 원문 안내·Group·Ellipse·㊞ 소유 및 포함 관계로
  `2`/`매`와 해당 도장만 허용한다. 다른 본문 충돌은 계속 검출한다.
  baseline 수치를 올리지 않고 이 문서의 의도하지 않은 교차0건을 검사한다.
- 정식 source는 기존 `tests/cases/text_overlap_baseline.rs` 안이다. 초기 support/newcase
  배치는 제거했고 manifest48/48에 맞춰 정상 준비했다.
- `remaining-stored-vpos-corrected.log`: 도장 소유·일반 본문 충돌 대조군·partition8
  3건 실행3PASS(exit0), run `47459175-48af-4717-9587-0e5dc0fc7b0f`.

원문4쪽 Native/fresh WASM 최저98.77240%와 같은 Print 도장 교차 확대를 확인했다.
위 시각 산출은 renderer source `d0528fb55f225fba8b4b65b9ed670a8dee9c654b` 기준이며
새 진단 source head의 fresh package라고 보고하지 않는다. 다음 단계는 승인된 실패3사례
이관, 유지 검사·필수 lint·최종 전체 nextest다.

## 3단계 — 승인된 이관과 최종 검증

#7830·#7831은 #7445의 실제 하위 이슈로 등록하고 담당자jangster77·UTF-8 본문을
API 재조회로 확인했다. 제거 전 source·SHA256은 `mydocs/pr/assets/issue7830/`,
`mydocs/pr/assets/issue7831/`에 보존한다. sample2의 실제8쪽 review·표 하단 확대도 포함했다.
입력과 Print는 다른 정상 검사 참조 때문에 공유 경로에 유지한다.

- #7830: HWP8쪽 검사 함수1개 제거, HWPX19쪽 정상 함수 유지.
- #7831: 39쪽 검사 함수1개 제거, 마지막 표 검사의 가정폭력 분기만 제거.
  정상7062 분기와 가정폭력16쪽 빈 문단 검사는 유지한다.
- 실패3사례 이관이며 전체 함수 감소는2개다. 현재 짧은 표 하단·39→37쪽 손실은
  이슈의 새 미해결 회귀로 남고, 현재 수치로 기대값을 갱신하지 않았다.

유지 검사3건은 모두 PASS했다. 최종 source
`7dd9572b7fecbc602c9e5ab47c4a6c9d6422b4bd`에서 fmt·Native/WASM/all-target
Clippy·workspace build·base 고정 manifest/unit-tier 검사를 통과했다.
전체 nextest는 **10,215건 PASS·실패0·skip45**(exit0), Studio는
**1,834건 PASS·실패0·skip2**, TypeScript 검사도 exit0이다.
실행 로그는 같은 ignored 검증 디렉터리의 `final-validation-source.json`에 연결한다.

## 현재 시각 검증 상태 정정 — 2026-10-11

완료된 전쪽 Visual Sweep을 현재의 ‘시각 미검증’으로 다시 기록하지 않는다.
아래 값은 같은 원본·독립 Print에 대한 Native/fresh WASM 각각의 전쪽 최저값이다.

| 대상 | 전체 쪽수 | Native 최저 | fresh WASM 최저 | 상태 |
| --- | ---: | ---: | ---: | --- |
| table-text | 1 | 99.99320% | 99.99320% | 전쪽 TSV 완료·제목 확대 판독 완료 |
| stored-vpos | 4 | 98.77240% | 98.77240% | 전쪽 TSV 완료·도장 소유/교차 확대 판독 완료 |
| hwp-multi | 9 | 95.64710% | 95.64710% | 전쪽 TSV 완료 |
| 해외투자 HWPX | 9 | 91.97621% | 91.97621% | 전쪽 TSV 완료 |
| 해외투자 HWP | 9 | 91.97621% | 91.97621% | 전쪽 TSV 완료 |
| hwpx-02 HWP | 9 | 91.97621% | 91.97621% | 전쪽 TSV 완료 |

이 결과는 위에 고정한 renderer source·package의 실제 실행 증거다.
이후 진단 점유 수정과 검사 이관은 해당 실제 배치를 바꾸지 않는다.
#7813 giant-cell, #7830 sample2 HWP, #7831 가정폭력 조사 문서는 승인된
후속 이슈 범위이며, 현재 회귀 보정의 미검증 문서로 중복 나열하지 않는다.
이관은 해당 출력 결함의 해결을 뜻하지 않는다.

#7768은 현재 브랜치 재확인 작업이다. 같은 입력·독립 Print의 base 전1쪽
최저92.73966%를 확인했다. Native/fresh WASM 전1쪽 TSV는 각각92.73966%, 누락0이며 같은 raster bytes를
확인했다. 양 경로 실제 review/overlay의 canonical gate는 passed다. 캡션·표·
체크박스3개·후행 용지 규격을 확대 판독하고 공개 `mydocs/pr/assets/issue7768/`에 연결했다.
이 결과는 기존 #7568 보정의 무회귀이며 새 개선으로 계산하지 않는다.

최신 `upstream/devel` `f79f2e1d6831a2187657b9c60deabc5d7c8b8e0d`로 rebase한
head는 `03a987f4aeded1d2b03415e7edecac5c833ca5cd`이다.
충돌은 `mydocs/orders/20261011.md` 한 파일뿐이며 양쪽 기록을 보존했다.
사용자 지시에 따라 이미 통과한 전체 회귀·lint·Studio를 rebase 후 반복하지 않는다.
optimized fresh WASM 빌드는 source `03a987f4a`에서 exit0으로 완료됐고 root `pkg/`와
Studio `public/`의 JS/WASM SHA-256 일치를 확인했다. WASM SHA-256은
`e235c944e4921b80a30e347bf28f74503b0be02ce6398c41d280cf79bfb72cd5`다.
명령·JS 해시·package snapshot은 ignored `final-fresh-wasm-provenance.json`에 고정했다.
#7768의 해당 package TSV·review와 실제 Studio 초기화/CanvasKit Type7 대조군 확인을 완료했다.
별도 로컬 Native Skia3종은 미실행이며 사용자 추가 테스트 금지 지시에 따라 GitHub CI에서 확인한다.

## PR 생성 직전 최신 base 정렬

최신 base `920dc29284e8a6e624861bfccf68903e19ae309b`, rebase 뒤 code head `9b4817473b2a3515ea27eefef9d20731258433e4`다.
upstream #7703의 확장 프로그램·오늘할일만 추가됐으며 오늘할일 한 파일의 충돌에서
양쪽 기록을 보존했다. `493a979d2` 대비 Rust·tests·Cargo tree가 동일함을 확인했고
사용자 지시에 따라 추가 테스트하지 않는다. 중단된 Native 재빌드는 성공으로 세지 않고
완료된7dd 빌드의 해당 SVG 경로 동일성·입력/출력 해시를 근거로 재사용했다.

제출 head의 GitHub CI·Native Skia·CodeQL·mergeability는 PR 생성 뒤 확인한다.
