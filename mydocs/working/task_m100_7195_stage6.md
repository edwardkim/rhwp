---
kind: investigation
status: active
---

# #7195 6단계 — 통합 후 조기 분할 원인과 실패 증감 검증

## 승인과 고정 비교 대상

2026-09-17 작업지시자가 통합 후 큰 여백의 원인 수정과 실패 증감 검증을 승인했다.
원격 변경과 기준값·ignore 변경은 범위에 포함하지 않는다.

- 최신 devel: `fcbd00e0fabc4b309a887357033f92e2d511cd75`, giant HWPX 48쪽.
- 수정 전 통합 제품: `cb151503146af73df18e10017f0038ca66ecd530`, 같은 입력 49쪽.
- 시작 HEAD: 기록 커밋 `f73aecb14` (제품은 cb151과 동일).
- 전체 검증 기준: stage5의 9931 PASS / 30 FAIL / 47 skipped.
  `output/7195/stage5/regression-comparison.json`의 실패 이름을 고정하고
  해결·잔존·신규를 개별 비교한다. 총 실패 수가 같아도 신규 실패가 있으면 별도로 보고한다.

## 조사·검증 순서

1. 40쪽 s0 pi0 ci3의 시작 컷1182에서 유닛 높이, 중첩 예약, 재시도 예산과
   최종 배치 원점을 대조한다. 계측 코드는 진단 후 제거한다.
2. 원인 규칙의 반례를 정식 회귀 검사에 추가하여 수정 전 실패를 확인한다.
3. 해당 규칙만 수정하고 같은 컷의 점유/paint, 이어받기 소유를 검사한다.
4. 영향 페이지와 기존 승인 86712의 4–8·11·12쪽을 직접 비교한다.
5. 선행 검증 후 전체 회귀를 실행하고 stage5의 실패 집합과 대조한다.
   페이지 수 감소만으로 성공을 판정하거나 새 실패를 기존 이슈로 묶어 숨기지 않는다.

기준 PDF `pdf/table_giant_cell_overfill-hwpx-2024.pdf`의 부속서 I는 물리41쪽이다.
수정 전 rhwp의 같은 내용은 물리40쪽이므로 같은 숫자의 페이지를 동일 내용으로
간주하지 않는다. 이 앞선 쪽 차이와 이번 조기 분할은 별도다.

## 원인 계층과 수정 범위

계측 결과 최초 가설인 혼합 중첩 예약의 이중 차감은 이 조각의 원인이 아니었다.
`cut-trace.log`에서 시작 유닛1182부터 13개의 빈 문단(원본 pi639–651)이 각각
38.9333px, 합계506.1333px를 소비한다. 원본 XML의 줄 높이1200HU·줄간격720HU와
빈 문단 자체를 확인했다. 이 조각의 `mixed_nested_extra`와 `trailing_trim`은 0이다.

- 측정: `row_cut_content_height`가 실제 선택 유닛의 높이와 패딩을 합산한다.
  통합본의 컷1182→1210, 점유1002.44px는 본문1009.1467px 안에 든다.
- 배치: `table_partial.rs`는 빈 문단을 실제 배치해 `para_y`를 전진시키지만,
  중첩 표 원점을 `has_preceding_text == false`일 때 `inner_area.y`로 되돌린다.
  이 플래그는 가시 글자 존재 여부이지 앞 줄의 공간 소비 여부가 아니다.
- 음성 대조: 새 `giant_continuation_nested_origin_preserves_owned_empty_lines`는
  수정 전 실제 제품에서 `nested top 79.34`가 빈 줄 끝567.34보다 위로 돌아가 실패했다.
  로그: `output/7195/stage6/origin-contract-before.log`.
- 수정: 기존 비분할 경로와 같이 `para_y_before_lines`를 사용한다.
  저장 square 앵커 경로와 앞에 가시 텍스트가 있는 경로는 그대로 둔다.
  유닛 생산·컷 선택·예약 높이·원본 높이·패딩·페이지수 계약은 변경하지 않는다.

고정 devel fcbd의 실제 CLI로 같은40쪽을 다시 내보내 확인했다.
`devel-p40/render_tree_040.json`에서도 pi639–651 빈 줄 y84.1–551.3을 배치한 뒤
pi652 표를 y79.3에 놓는다(높이985.6px). 따라서 원점 역행은 devel에도 있는 기존 결함이다.
48쪽이라는 페이지수 통과는 이 배치 오류가 없다는 증거가 아니다.

초기 진단에서 시도한 “부속서 전체가 같은 쪽에 있어야 한다” 검사는 앞의 실제 빈 문단과
기존 PDF 쪽 소속 차이를 고려하지 않은 가정이어서 정식 계약으로 채택하지 않았다.
그 실행 로그 `contract-before.log`는 새 계약의 음성 대조 증거가 아니다.
이번 기대값은 페이지수/부속서 전체 fit이 아니라, 선택된 빈 줄의 공간을 뒤 표가 역행하지
않고 본문 경계를 지키는 배치 불변식이다. PDF와의 완전한 일치를 의미하지 않는다.

검증은 cb151 제품에 현재 소스·테스트 diff를 적용한 격리 review worktree에서 수행한다.
`output/7195/stage6/verify.mjs`는 각 명령·시각·HEAD와 정확한 diff 및 SHA256을 보존한다.

## 작은 경계 검사와 직접 출력 비교

- 새 원점 계약: 수정 전 FAIL → 수정 후 PASS (`origin-contract-after.log`).
- Native binary SHA256: `8a1d5db6d67ce4380d29edf772bb9cd0e26daf18c0523c0ee93968583257ca2b`.
- giant 전체49쪽의 렌더 트리를 비교한 결과 **40쪽만 변경**되었다.
  같은 77개 노드와 TextRun 순서를 유지하고 pi652 표의 y만79.3→585.5로 이동했다.
  표 높이는492.5px, 하단1078.0px로 본문 하단1084.7px 이내다.
  `dump-pages` 전체 결과는 해시까지 동일하다. 컷·쪽 소속·49쪽 총수는 변경하지 않았다.
  증적: `output/7195/stage6/giant-comparison.json`.
- 기존 승인 86712의 4–8·11·12쪽은 stage5와 Native 렌더 트리/SVG가 모두 동일하다.
  증적: `output/7195/stage6/approved-pages/comparison.json`.
- Native Visual Sweep 39–42쪽을 실행했다. 기본 Chrome 검색 실패 후 설치된 Puppeteer
  Chrome152 경로를 `VISUAL_SWEEP_CHROME`으로 지정해 재실행했다.
  `native-sweep/giant/{compare,overlay,review}`를 보존했다. rsvg 보조 실행과 구분한다.
- 39·40·41·42쪽 PNG를 직접 확인했다. 40쪽의 큰 공간은 위의 실제 빈 문단 소유와
  일치하도록 이동했지만, PDF41쪽의 부속서 전체 한쪽 배치와는 여전히 다르다.
  42쪽에 보이는 기존 표/텍스트 경계 문제도 그대로다. 이 차이는 통과 판정하지 않는다.
  앞선 쪽 소속과 기존 overflow 전체의 해결은 이번 원점 수정의 입증 범위가 아니다.

이번 전체 실행은 실패 증감 조사이며, 남은 PDF 차이를 승인하거나 제출 게이트를 대신하지 않는다.

## 집중 검사·fresh WASM

- 집중 검사: **38 실행 /37 PASS /1 FAIL**. 실패는 기존
  `issue_7140_nested_row_unit_paint_height::hwpx_page_count_matches_the_oracle`
  (49/48) 그대로다. 새 원점 계약과 #2279·#2308·#3798·#7195·#7150·
  stored_nested_content_flow 대조군은 통과했다. `focused.{json,log,patch}` 참조.
- `cargo fmt --all -- --check` 통과. 테스트 변경 후 파생 harness drift를 검출하여
  격리 review worktree에서 `--prepare` 재실행 후 suite policy 통과했다.
  파생 파일은 source 변경에 포함하지 않는다.
- Docker fresh WASM: `docker compose --env-file .env.docker run --rm
  -e CARGO_BUILD_JOBS=4 wasm`, 성공(7m54s).
  SHA256 `72bc8f1cb03f99d4d259150a0967b42ab0e22cefd1c959dfd1c2eb66b814c6e6`,
  11,221,576bytes. `wasm-verification.json`에 dirty source diff hash를 명시했다.
- 실제 WASM API에서 86712 승인7쪽의 트리/SVG가 모두 이전과 동일하다(64쪽).
  giant는49쪽 유지. Studio 7700이 제공하는 WASM 해시도 새 산출물과 일치했다.
- Chrome152 fresh WASM Visual Sweep 39–42쪽 완료. 직접 본 40쪽은 Native와 같은
  빈 문단 후 표 배치이고, 네 쪽 PNG 모두 Native와 해시까지 동일하다.
  증적: `output/7195/stage6/wasm-sweep/giant/{compare,overlay,review}`.
- 같은 PDF에 대한 전후 전수 text/layout ledger는 `fidelity-before`, `fidelity-after`다.
  최초 끝 index48 요청은 PDF48쪽 범위를 초과해 실패했으며 0–47로 정정했다.
  text·owner·page-boundary·overlap·clip 후보 결과는 동일하다.
  table-fragment 후보64개 중40→41쪽 pi652의 원점/본문하단거리만 바뀌었다.
  후보 수를 실제 결함 수로 해석하지 않는다.

## 전체 실패 집합 비교

전체 회귀 완료: **9962 실행 /9932 PASS /30 FAIL /47 skipped**, exit100.
실제 테스트415.221초, 컴파일 포함 약12분19초.
정확한 검증 대상은 cb151 + `regression.patch`이며 diff SHA256은
`7c6f7e8a1fce4223f68669e1eed06c25320b60ddfa14f10cd36c628ea2883987`이다.

| 비교 시점 | PASS | FAIL | skipped |
| --- | ---: | ---: | ---: |
| stage4 통합 전 | 9903 | 28 | 47 |
| stage5 devel 통합 후 | 9931 | 30 | 47 |
| stage6 원점 수정 후 | 9932 | 30 | 47 |

- stage5 대비 **기존 실패 해결0 /신규 실패0 /동일 실패 잔존30**이다.
- 실패30건 모두 panic 원문을 추출했으며 메시지와 집계 검사의 현재값 TSV 블록도 동일하다.
  테스트 묶음 재배치에 영향받지 않도록 개별 테스트 이름으로 비교했다.
- PASS +1은 새 원점 계약 추가 때문이며, 기존 실패가 해결된 것으로 세지 않는다.
- 통합 전후 증가한2건은 동일 giant HWPX의49/48쪽 차이를 검출하는
  `issue_7140_nested_row_unit_paint_height::hwpx_page_count_matches_the_oracle`과
  `oracle_page_count_baseline::page_counts_do_not_drift_from_hancom_oracle_partition_5`다.
  이번 수정에서도 남아 있다.
- 기계 판독 증적: `output/7195/stage6/failure-comparison.json`.
  전체 명령·로그·소스 diff: `regression.{json,log,patch}`.

판정: 이번 좁은 수정으로 추가 실패나 기존 실패 수치 악화는 검출되지 않았다.
그러나 기존 실패는 줄지 않았으므로 전체 회귀 통과 또는 통합 조판 문제 해결로 보고하지 않는다.
기존 실패의 후속 이슈 분리 방침은 유지하며, 기준값·ignore를 조정해 성공으로 바꾸지 않았다.

Clippy3종·Native Skia 등 제출용 통합 게이트는 이번 진단 결과와 별개이며 아직 완료하지 않았다.
커밋·원격 push·PR·기준값·ignore 변경은 하지 않았다.
