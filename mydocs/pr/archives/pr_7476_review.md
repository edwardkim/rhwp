---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-04
---

# PR #7476 검토 — 수정: #7418 재조판 줄 채움(condense·목록 마커·구두점 폭)과 host 글·칸 조각 기하를 한/글에 맞춤

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 초기 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f`에서 체리픽을 시작했고, 현재는 `upstream/devel 6b3faf77d8085441f9f26d88d65a49791e910352`를 포함한 후보를 검토합니다.

## 접수·기여자·출처

- 원 PR: https://github.com/edwardkim/rhwp/pull/7476, 작성자 planet6897, base `devel`, 정확한 head `d0c18feef2e9a13403482d7b8663a66a422bfaa4`.
- reviewer jangster77을 지정했습니다. 원 contributor 변경과 메인터너 충돌 보정을 구분해 기록합니다.
- 사전 선택: collaborator_external_pr 9.1.1 체리픽 통합 경로; intake/local_validation/visual_fixture_evidence/multi_pr_update_branch/post_merge 적용.
- source CI는 통합 head 검증을 대체하지 않습니다. 모든 renderer·페이지 변경은 직접 Native/fresh WASM 전쪽 TSV와 영향 경계 review/overlay를 검토합니다.

## 체리픽 계획

| source commit | 판정 | 변경 |
| --- | --- | --- |
| `23da3567ecb49bce69d56f43cd041b8cb6eff9cc` | stacked/rebased duplicate | 수정(조판): 칸 문단의 줄 상자도 문단 좌우 여백만큼 들여 잡는다 (#7407) |
| `b07b421a9d2398eac87e1e5155584e6f4cbddce5` | stacked/rebased duplicate | 수정(조판): 한 줄에 안 들어가는 토큰을 잰 폭으로 자른다 (#7407) |
| `7a81c50b701bbd590a47ca351a0a0aca9f5c6907` | stacked/rebased duplicate | 문서(증적): #7407 A2b 시각 증적과 코퍼스 변화 3건 판독 |
| `6a9a03bc235a5c274bef91dfa103337527e98582` | stacked/rebased duplicate | 수정(조판): 칸 문단을 다시 조합하는 경로도 문단 좌우 여백을 뺀다 (#7422) |
| `d155e91375e8a616384c5c07067a076a1d706bd4` | stacked/rebased duplicate | 문서(증적): #7422 재조합 경로 칸 상자 수정의 시각 증적 |
| `93c73a032efd9c8d72405622cd3d24553bc93f1b` | candidate | 수정(조판): 칸 안 여백 축소를 줄바꿈으로 안 되는 칸에만 건다 (#7413) |
| `1752f6fc69bee0dc3d003b202aa84bc3d73ec6e5` | candidate | 수정(조판): condense 로 새 낱말을 시작하려면 줄이 자연폭 안이어야 한다 (#7418) |
| `c96cc071e93a28fba67daffcc0c786966df0db5b` | candidate | 문서(조사): #7418 condense 규칙 재현 프로브 — 한/글 저장 줄 208/208 |
| `0928fba0918818bdb0d7a7e7668bac3b093929e2` | candidate | 수정(조판): 줄 맨 앞 공백은 condense 로 줄이지 않는다 (#7418) |
| `37d2e41891b27559f8b99b03cbbbc1a3a8342400` | candidate | 수정(조판): 공백을 줄여 넣는 판정에는 줄바꿈 여유를 얹지 않는다 (#7418) |
| `178b658b5e724de7a0ca3631217fac76eaa9986e` | candidate | 수정(조판): 글머리표 폭을 줄 나눔 상자에서 빼고 condense 판정을 한/글에 맞춘다 (#7418) |
| `fc34d8c73d019901854cbb5c8c655635ae000385` | candidate | 수정(조판): 줄 끝에 건 공백이 문단 끝이면 빈 행을 만들지 않고, 칸·끝 공백 검사를 더한다 (#7418) |
| `055a054ea081b4f6474d64803a8cc49dd35fee13` | candidate | 수정(조판): 글머리표 자리의 soft hyphen 은 반각 하이픈으로 그리고 잰다 (#7418) |
| `c6dd5efcc22c92affd2a88ce2e9b760026f65b87` | candidate | 수정(조판): 번호·개요 폭도 줄 나눔 상자에서 뺀다 — 번호를 문서 순서로 미리 계산한다 (#7436) |
| `8d7f57abcce796660614f2ee3602491ba54e69cc` | candidate | 검사(#7418): #2214 fifth line 경계를 한/글 환경에서 다시 정하고, 번호 파생값을 Debug 동일성에서 뺀다 |
| `bd8596aacc562fe98e6199a38d241c1b287f771d` | candidate | 수정(조판): 공백 최소값은 자간 적용 전 공백 폭을 기준으로 줄인다 (#7418) |
| `66b058c5f4dffa470d5d78b1c70f99ce726b2957` | candidate | 수정(조판): 목록 마커 영역을 머리 모양 속성으로 정한다 (#7418) |
| `f18e4e42458296db504043fb8c9802ebf5ee8512` | candidate | 검증: #7418 머리 모양 검사에 수정 전 일치 수를 적는다 |
| `47ccfb7375a479f5461259386d1eca6de3009d09` | candidate | 수정(조판): 목록 마커 글자 폭·글자 모양·빈 영역을 한/글에 맞춘다 (#7418) |
| `8d98868683ddf7cfaf0d0a999636fa34ac2f8a34` | candidate | 수정(조판): 저장 줄 없는 글자처럼 취급 표 host 줄의 줄간격을 계상한다 (#7418, #7344) |
| `5472db690924061b5f75fa4afc731967287b1164` | candidate | 수정(조판): 쪽 나누기 선언만으로 빈 문단 넘침을 흡수하지 않는다 (#7429) |
| `5c262e4aca18959268536eb4dab8ce374c4041f6` | candidate | 수정(조판): 칸 선언이 모든 행을 담으면 낡은 표 선언 높이로 행을 늘리지 않는다 (#7418) |
| `38dafafd4af7c9f3e605a94fb58ea6f463720e53` | candidate | 수정(조판): 중첩 표 행 안의 중첩 표를 행 경계에서 나누고, 일반 격자 묶음의 빈 꼬리도 쪽 바닥에서 자른다 (#7418) |
| `719339ab2db1824eb3c759d9cf5a561577fbedff` | candidate | 수정(조판): 쪽 경계에 걸친 칸은 안 여백을 깎지 않는다 (#7080) |
| `7baef3c9dc5ed7696170f2f6ed47354aba8d5f93` | candidate | 수정(조판): 빈 꼬리 문단 흡수를 문서 근거가 있을 때만 하고, 없으면 줄이 온전히 들어가야 한다 (#7429) |
| `22378bb40280ad325e3502b9d8d6eda557fdc201` | candidate | 검사: 78494 의 쪽 핀을 한 쪽 밀린 현재 흐름으로 옮긴다 (#7422, #6776) |
| `4bf04154e52f53c10664f76ea9ffd93113118d6e` | candidate | 빌드: WMF/EMF 골든도 LF 로 체크아웃한다 |
| `74205c26fb751800a0672c6649fc7eaaa24b8a26` | candidate | 수정(조판): 선언 빈 꼬리 압축은 행으로 나뉘는 열이 하나인 묶음에만 (#7418) |
| `af3a05b926a3ed3bd349f6cd8fdd7a0b24a8ce83` | candidate | 검사: hy_ladder3 칸 넘침 래칫을 7 → 11 로 조인다 (#7418) |
| `c75bbee725d44e22ef6593c18dcf57e2569cfee8` | candidate | 수정(조판): ASCII 구두점은 영문 슬롯 글꼴로 재고 그린다 (#7418) |
| `8041f80f113aec2a2717fd75ffc774315c24b11a` | candidate | 검사: 78494 의 쪽 핀을 정본 쪽으로 되돌린다 (#7422, #6776) |
| `ea41e252b047547407908594236cb33b004b32f0` | candidate | 검사: 구두점 영문 슬롯 단위 시험을 통합 시험으로 옮긴다 (#7418) |
| `2f0f6ca3c0a23f54392190815bafad7a9ecb5013` | candidate | 수정(조판): 구두점 영문 슬롯은 영문 슬롯이 옛 한컴 영문 글꼴일 때만 건다 (#7418) |
| `2d213d5d3e68035f6ce052ebc6328a738d0261d1` | candidate | 수정(조판): 구두점 영문 슬롯은 옛 영문 글꼴이 라틴 전용 글꼴로 치환될 때만 (#7418) |
| `397a5357c51a5ef40a77834061e81674ca25664c` | candidate | 수정(조판): 영문 슬롯 구두점은 run 을 쪼개지 않고 글자 폭만 영문 슬롯으로 잰다 (#7418) |
| `6d72723b601ed5a8ef0143be4951a3677bab18e2` | candidate | 검사: #4962 공개 coverage 골든의 판정 행을 영문 슬롯 구두점 측정에 맞춘다 (#7418) |
| `853cd9c497fd19274395d6c6d38a2ff2196d0d1e` | candidate | 수정(조판): 글자 단위 한글 문단은 한글 글자 앞도 줄 끝 후보다 (#7418) |
| `de25d9512557586d5ac51b8cbd1c9fc202b2ce16` | candidate | 수정(조판): 칸 안 여백 축소 판정은 칸 마지막 줄의 줄간격을 세지 않는다 (#7418, #7413) |
| `7065555f66ccf0a7a0616cab3ab38c2c9780bf74` | candidate | 수정(조판): 쪽을 넘어 이어지는 목록 문단 조각도 마커 영역만큼 들어간다 (#7418) |
| `6a9ca07c848abc8aa3203131364e275cb55fc623` | candidate | 수정(조판): 마지막 행 안에서 이어지는 2열 표 조각도 바깥 위 여백을 연다 (#7418) |
| `219687eec48ee77ae6fa41404f6fa8d433cb5c3d` | candidate | 수정(조판): 표 host 글 뒤 자리차지 표와 칸 중첩 표 행의 기하를 한/글에 맞춘다 (#7418, #6854) |
| `fe76d00f2682d6e59b5fa4a934aac65913e19664` | candidate | 검사: 표 host 글·칸 조각 기하를 한/글 정본 좌표로 잠근다 (#7418, #6854) |
| `2576efc2d1372889378d36da56fb397bddcd0b80` | candidate | 수정(조판): host 글 선방출이 다른 표의 쪽 나눔을 바꾸지 않게 좁힌다 (#7418) |
| `16a78a62f65009a884d46d5422e03b8fcca918c2` | candidate | 수정(조판): host 글 위치는 기존 host 배치 모델로 정하고 임시 규칙 둘을 걷는다 (#7418, #6854) |
| `ef1921b2700ac602bba98e81726bd4d123457bee` | candidate | 수정(조판): host 글 기록의 쪽 간 누수·저장 줄 근거 여백 축소·글 앞 앵커 분리 (#7418, #7413, #6950) |
| `d18b1c1e84881023edc85edb70484cb5396d0093` | candidate | 검사: 정본 근거로 기준값 셋을 옮긴다 (#7418) |
| `375d701b32fd69fa24c6749e6e9450834649e9ff` | candidate | 문서(증적): #7418 대표 쪽 Native·fresh WASM Visual Sweep 증적 |
| `e5637516ac3bc42888ec505f87ca139751c656e5` | candidate | 수정(조판): 합성 host 줄이 품은 줄간격을 TAC host 규칙이 다시 더하지 않는다 (#7418) |
| `d3d829abc86350e1862f8c6a805793366817dbd4` | candidate | 검사: #7413 칸 여백 시험 파일을 rustfmt 로 정리한다 (#7418) |
| `d0c18feef2e9a13403482d7b8663a66a422bfaa4` | candidate | 검사: #7413 시험의 쓰지 않는 재귀 인자를 뺀다 (#7413) |

## 변경 범위와 검토 계획

- 원 PR 변경 4982줄 추가·486줄 삭제·137파일입니다.
- 생산 경로: `src/diagnostics/ir_field_sweep.rs`, `src/document_core/commands/document.rs`, `src/document_core/mod.rs`, `src/document_core/queries/cursor_rect.rs`, `src/document_core/queries/rendering.rs`, `src/model/paragraph.rs`, `src/parser/hwpx/header.rs`, `src/renderer/composer.rs`, `src/renderer/composer/line_breaking.rs`, `src/renderer/float_placement.rs`, `src/renderer/height_measurer.rs`, `src/renderer/layout.rs`, `src/renderer/layout/paragraph_layout.rs`, `src/renderer/layout/table_cell_content.rs`, `src/renderer/layout/table_layout.rs`, `src/renderer/layout/table_partial.rs`, `src/renderer/layout/text_measurement.rs`, `src/renderer/layout_frame.rs`, `src/renderer/mod.rs`, `src/renderer/pagination.rs`, `src/renderer/pagination/engine.rs`, `src/renderer/style_resolver.rs`, `src/renderer/typeset.rs`, `src/renderer/typeset/paragraph.rs`, `src/renderer/typeset/paragraph/format.rs`, `src/renderer/typeset/paragraph/overflow.rs`, `src/renderer/typeset/section/tail.rs`, `src/renderer/typeset/state/commands.rs`, `src/renderer/typeset/state/data.rs`, `src/renderer/typeset/state/transition.rs`, `src/renderer/typeset/table/block/entry.rs`, `src/renderer/typeset/table/block/prepare.rs`, `src/renderer/typeset/table/continuation/fragment/budget.rs`, `src/renderer/typeset/table/continuation/fragment/emit.rs`, `src/renderer/typeset/table/scan/block_fit.rs`, `src/serializer/hwpx/header.rs`, `src/wasm_api.rs`, `src/wasm_api/tests.rs`.
- 실제 호출 경로의 측정→예약/컷→paint 소비를 추적하고 정상 대조군·원본 저장 정보의 유효성을 확인합니다. 문단·행·개체·각주 소유와 누락/중복을 기존 검사 의미로 판단하며 픽셀 핀을 승인 근거로 사용하지 않습니다.
- 원본·기준 PDF와 검토 commit의 실제 파일/해시 일치를 확인합니다. 통합 출력과 PDF의 전체 쪽수가 다르거나 미달쪽이 있으면 재검토합니다. 원 PR의 부분 개선 주장을 전체 피델리티 완료로 확대하지 않습니다.
- 합성/실물 경계 회귀·전체 nextest threads8·Native Skia3·필수 lint/정책·fresh WASM은 누적 후보에서 순차 수행합니다. 로그는 ignored output에만 저장합니다.
- 1,000줄 초과 규모이므로 대형 PR 예외 검토를 추가합니다. 포함된 선행/후속 변경과 기준·baseline 교정의 독립 근거를 따로 검토하며 CI green만으로 수용하지 않습니다.
- #7435 자체는 CI 실패로 이번 선택에서 제외했습니다. 이 green head에는 동일 선행 칸 여백 수정과 후속 rustfmt/unused 인자 제거가 포함됩니다. green #7476의 전체 누적 diff에 속하는 의존 변경으로 출처를 남기고 검토합니다.

## 실행 결과

고유 source commit 63개 체리픽을 완료했습니다. 출처와 보정은 [적용 원장](../assets/planet6897_green_20261002/applied_commits.json)에 기록했습니다. 현재 후보 `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`의 Native Clippy는 exit0이며 전체 nextest는 실행 중입니다. 최종 회귀·시각 검증은 미완료입니다.

### 체리픽 보정 1: 글머리표 폭과 이어지는 문단

- 원인: #7418의 첫 구간 글머리표 폭 예약과 현재 devel의 부분 재조판·이어지는 문단 내어쓰기 보정이 같은 경로를 수정했습니다.
- 보정: 첫 가용 구간에서 마커 폭을 한 번 예약하며, 배치는 원 PR의 동일 폭 함수를 사용합니다. 마커를 다시 그리지 않는 이어지는 문단에서도 본문 내어쓰기를 유지합니다.
- 현재 결과: 충돌을 해소했습니다. 원 PR의 신명조 공백 자연 폭 변경과 기존 12pt 이상 공백 보정의 차이는 통합 후 독립 기준 PDF 및 기존 회귀로 재검증합니다. 이 단계는 테스트 통과 또는 승인 판정이 아닙니다.

### 체리픽 보정 2: 번호·개요 마커 폭

- 문서 순서로 선계산하는 번호·개요 마커 폭을 원 PR대로 적용하면서, 배치에서 같은 `list_marker_hang_px` 결과를 사용합니다. 앞 보정의 이어지는 문단 내어쓰기를 유지합니다. 충돌 해소 단계이며 검증은 통합 후보에서 수행합니다.

### 체리픽 보정 3: 머리 모양 기하

- 원 PR 후속 commit은 단순 문자열 폭을 머리 모양 속성의 공통 기하로 바꾸며, 이어지는 조각도 같은 내어쓰기를 유지합니다. 앞 보정과 의미가 같으므로 새 공통 기하 경로로 합쳤습니다. 검증 대기 상태입니다.

### 체리픽 보정 4: 기존 보류 자료와 합성 마커 겹침

- #7445로 분리한 #4961의 렌더 count/hash 핀은 복구하지 않고 현재 보류 기록을 보존합니다. 새 합성 문서의 의도된 마커 겹침 3건은 원 PR이 제공한 한컴 PDF 근거와 함께 적용하되 통합 출력과 독립 비교 후 유효성을 판단합니다. 기존 실물 문서 허용값을 완화하지 않았습니다.

### 체리픽 보정 5: 합성 사다리 넘침 허용값

- `af3a05b926`은 hy_ladder3 허용값을 7→11로 늘립니다. 원 기여자는 한컴 저장 줄 일치가 19/29→24/29로 개선되어 추가 줄 높이가 사다리 바닥을 넘는다고 설명합니다. 근거 없는 완화와 구분하려면 현재 후보에서 29개 칸의 저장 줄·실제 줄 및 넘침 소유를 직접 확인해야 합니다.
- 지금은 기존 7을 유지하며 검사 변경 수용을 보류합니다. #7445로 분리된 task2097 자료도 복구하지 않았습니다. 코드 변경 없는 원 commit의 출처는 이 검토 기록에 보존합니다.

### 체리픽 보정 6: 언어별 글꼴 크기와 영문 슬롯

- 현재 devel의 언어별 글꼴 크기 벡터와 원 PR의 옛 영문 슬롯 치환 판정을 모두 유지했습니다. 두 정보의 역할이 달라 한쪽 선언을 삭제하면 현재 글꼴 보정 또는 새 구두점 분기가 깨집니다. 통합 검증 대기 상태입니다.

### 체리픽 보정 7: run 보존과 렌더 해시 보류

- 구두점 폭 측정 변경은 run을 쪼개지 않는 원 PR 후속 구조로 적용했습니다. 이미 #7445로 분리한 missing-face 사례의 레이아웃 해시 핀은 복구하지 않았습니다. SVG 골든 변경은 독립 시각 검증 대상이며 통과로 간주하지 않습니다.

### 체리픽 보정 8: 표 호스트 내용 높이와 저장 프레임

- 원 PR의 호스트 내용 끝(마지막 줄간격 제외)을 예산과 배치에서 같이 소비하도록 적용했습니다. 자동 병합이 같은 오프셋 분기를 중복 삽입한 충돌은 중복 없이 하나로 합쳤습니다.
- 현재 devel의 단 소유가 같은 저장 프레임 재사용, 첫 조각 바깥 위 여백의 단일 예약, 캡션 호스트 줄간격 공유 함수를 보존했습니다. 원 PR의 배치 모델 없음 판정도 유지합니다. 구문 확인 후 통합 회귀·시각 검증으로 판단합니다.

### 체리픽 보정 9: 임시 호스트 규칙 제거

- 원 PR의 후속 수정대로 배치 모델 없음에 따른 임시 분기를 제거했습니다. 현재 devel의 저장 프레임 소유와 바깥 위 여백 예약은 유지하며 기존 배치 모델을 통해 호스트 글 위치를 계산합니다. 충돌 해소 후 통합 검증 대기입니다.

### 통합 후 보정: 미사용 글머리표 중복 함수 정리

- 최종 원 PR의 머리 모양 기하가 줄 나눔·배치의 공통 경로가 되었습니다. 이전 devel의 단순 글머리표 문자열/폭 함수는 참조가 없어 두 중복 함수를 제거했습니다. 새 원 PR의 머리 모양 속성 계산은 유지합니다. 통합 lint와 회귀 검증 대기입니다.

### 누적 후보 검증 시작

- 검증 코드 head: `ec5ca7c3057a89c9a82bb59a78956a4d5eee567d`. Native Clippy exit0(34.38초). 전체 nextest release-test/threads8/no-fail-fast 실행 중이며 통합 시각 검증은 아직 미완료입니다. 원 PR의 green CI와 구분합니다.

### 메인터너 보정: #7445 80250 baseline 재등록 제거

- 분석: 원 PR의 2열 마지막 행 여백 변경에서 자동 병합된 `80250_regulatory_analysis.hwp` baseline 행 때문에 body_overflow 16개 분할 모두 동일한 입력 누락으로 실패했습니다. 이 문서는 사용자 지시와 독립 검토에 따라 이미 #7445에 보존·분리했습니다. 16개의 독립 제품 결함이 아닙니다.
- 수정: 재등록 행과 그 설명만 제거해 최신 devel의 제외 결정을 복구했습니다. 제품의 2열 바깥 위 여백 변경과 다른 문서 baseline, 정상 회귀를 유지했습니다. 원본 HWP/PDF 증적은 삭제하지 않았습니다.
- 결과: diff에서 해당 4줄 외 baseline 변경이 없음을 확인했습니다. 최초 전체 실행의 실패 기록을 보존하고 후속 후보에서 16개 분할을 전부 재검증합니다. 정상 검사 완화·새 skip 분기·일괄 #7445 이관은 없습니다.

### #7445 baseline 복구 재검증 결과

- body_overflow 기존 16개 분할을 nextest release-test/threads8/no-fail-fast로 모두 실행해 16PASS/0FAIL, exit0(62.314초)입니다. 80250 재등록으로 발생했던 최초16FAIL은 해결했습니다. 다른 정상 baseline·공차는 변경하지 않았습니다.
- 최초 전체 검증의 다른 실패와 통합 시각 게이트는 아직 완료되지 않았습니다.

### 메인터너 보정 사전 분석: 자모 목록 마커의 공백

- 정상 한컴 PDF4쪽의 16/18번 보기는 자모 ㄱ~ㄹ을 각각 두 번 표시합니다. 현재 render tree도 각 자모를 두 번 표시하지만 새 마커 기하 경로는 뒤 간격을 글자 공백 대신 마커 영역으로 예약하여 문자열이 `ㄱ.`입니다. 기존 검사는 `ㄱ. `와 완전 일치해 실패했습니다.
- 동일 원본과 독립 PDF의 전체 Native TSV는 4/4쪽 90% 이상(95.13~98.65%)이고 쪽수4/4 일치입니다. 번호 글자를 바꾸거나 누락한 것이 아닙니다. 모델 간격 표현에 종속된 끝 공백 비교만 제거하고 실제 표시 자모 종류·각2회·문항별 쪽 소유 계약을 유지하겠습니다.

### 자모 목록 공백 교정 결과

- 끝 공백 비교만 제거해 표시 자모와 각2회 출현·문항별 쪽 소유를 유지했습니다. 파생 manifest를 다시 준비한 뒤 기존 시험지 검사 3개를 nextest release-test/threads8/no-fail-fast로 실행했습니다. 실제 결과는 focused 로그의 최종 summary로 확인합니다. 0개 실행했던 준비 전 호출은 성공 증거에서 제외했습니다.

- 최종 focused 결과는 3PASS/0FAIL, exit0(0.051초)입니다. 신규 검사·현재 좌표 기대값·허용치 변경은 없습니다. [동일 원본 전체4쪽 TSV](../assets/planet6897_green_20261002/exam_social_native.tsv)를 보존했습니다.

### 메인터너 보정 사전 분석: 축소 본문에서 낡은 저장 프레임 허용

- #6950 기존 검사는 본문 아래 여백을 18000HU 늘린 상태에서 표가 본문 하단을 약36px 넘는 실제 결함을 검출했습니다. 검사 허용값을 늘리지 않습니다.
- 동일 영역을 XML에서 바꾼 별도 진단 입력에서도 `page_avail=138.8`, 실제 행합178.6, `fits=false`인데 저장 프레임 허용89.9가 행 전체 수용을 허락했습니다. 저장 앵커425.1 기준 끝603.7만 확인하고, 선방출된 호스트 뒤 실제 표 원점에서 같은 높이를 수용할 수 있는지는 확인하지 않았습니다. 진단 사본은 output에만 두며 한컴 정답지로 취급하지 않습니다.
- 첫 조각의 선언 프레임은 저장 앵커와 현재 공유 배치 원점에서 모두 본문 경계에 들어갈 때만 행 경계 증거로 사용하겠습니다. 실제 배치 원점과 다른 낡은 저장 끝으로 본문 초과를 허용하지 않습니다. 기존 #6950 경계·이월·형제 표 검사로 검증하고 원 PR 정상 호스트 사례도 이어 확인합니다.

### 축소 본문 저장 프레임 보정 결과

- 저장 앵커의 끝뿐 아니라 예산·배치 공유 원점에서 선언 프레임을 수용할 수 있는지 검사했습니다. 잘못된 저장 허용으로 온전한 행을 본문 아래까지 수용하지 않습니다.
- 기존 #6950 검사 28개를 release-test/threads8/no-fail-fast로 실행하여 28PASS/0FAIL, exit0입니다. 본문 축소 3단계와 이월·형제 표·호스트 소유 대조군을 기대값 변경 없이 통과했습니다. 로그: `output/pr-review/planet6897-green-20261002/nextest-6950-corrected.log`.
- 최초 실패 33개 중 focused에서 28개를 처리했고 5개가 남아 있습니다. 원 PR 정상 호스트 문서의 시각 검증과 최종 전체 테스트는 별도 필수이며 이 결과로 대체하지 않습니다.

### 한국어 주석 보완

- 새로 들어온 문단 여백 상자·확정 원점·두 열 종료 조각 설명의 영문 주석을 한국어로 바꿨습니다. 식별자·수식·관측값은 보존하고 실행 코드는 바꾸지 않았습니다.
- `cargo fmt --all -- --check`와 `git diff --check`로 형식을 확인합니다. 코드 변경이 없는 설명 보완이며 최종 코드 검증을 대신하지 않습니다.

### 누적 후보 검증 상태 갱신 (2026-10-02)

- GitHub를 다시 조회한 결과 선택 당시의 원 PR head와 CI green 상태가 유지됩니다. #7435는 현재도 non-green이며 선택에 포함하지 않았습니다. 누적 후보는 `review/planet6897-green-20261002`, source 고유 커밋63개입니다.
- 최초 전체 nextest는 10,283개 중10,250PASS/33FAIL/50SKIP, exit100으로 완료했습니다. 그 뒤 PR별 보정과 focused 재검증으로 최초 실패28개를 처리했으며 5개가 남았습니다. 전체 재실행 통과로 바꾸어 보고하지 않습니다.
- 각주 빈 번호·합성 사다리·다열 표 대조군·중첩 표 후속 원점의 잔존 5개와 whole fixture 시각 보류를 [현재 검증 기록](../assets/planet6897_green_20261002/review_progress.json)에 기록했습니다. 원 PR별 기존 분석·커밋 출처는 위 내용을 유지합니다.
- **현재 통합 승인/머지 보류**입니다. 원 PR의 green CI는 누적 후보의 실패 또는 미완료 Native/fresh WASM 시각 검증을 대체하지 않습니다. 새 통합 PR 생성·push·머지는 하지 않았습니다.

### 호스트 배치 정상 대조군 후속 결과

- `issue_7418_host_text_and_split_row_geometry` 기존7개를 현재 코드 보정 후 실행해 7PASS/0FAIL입니다. 두 열 종료 조각의 바깥 여백과 원 PR 호스트 배치 개선을 유지합니다.
- #7243은 한컴2020 HWPX·2024 HWPX·2024 HWP PDF의26쪽 괘선을 직접 추출했습니다. 모두77.515/145.760/193.868px로 같습니다. 현재 상대 원점 차이를 DPI 오차로 정당화하지 않습니다.

### 메인터너 보정 사전 분석: 조각의 예약 여백과 후속 흐름 끝

- 두 열 종료 조각 여백 확대만 임시 제외한 진단에서 #7243 두 검사가 모두 PASS였습니다. 진단 변경은 복원했습니다. 정상 두 열 표의 위 여백을 없애는 해결은 채택하지 않습니다.
- 표 조각 예산은 바깥 아래 여백을 실제로 예약한 `fragment_outer_bottom_overhead`를 알고 있지만, 확정한 호스트 배치의 `occupied_bottom`은 비캡션에서 선언 `outer_margin_bottom`을 무조건 더합니다. #7243 26쪽은 아래 여백 예약0인데141HU를 더한 후속 원점을 저장합니다. 배치는 이 저장 원점을 재사용해 후속 표가 약1.8px 밀립니다.
- 조각 예산의 예약값을 확정 배치의 점유 끝에도 그대로 사용하고, 원점 기반 가용 높이 계산도 같은 값을 빼도록 보정합니다. 위 여백과 표/중첩 Enter 높이는 유지합니다. #7243와 #6950 호스트 경계 및 #7418 정상 두 열 대조군을 함께 재검증합니다.

### 예약 여백 공통 소비 보정 결과

- 비캡션 조각의 원점 기반 예산과 확정 `occupied_bottom`이 실제 예약된 바깥 아래 여백을 소비하도록 맞췄습니다. 위 여백 재개만으로 아래 여백도 다시 더하는 처리를 제거했습니다.
- #7243 두 검사, #6950 기존28개, #7418 원 PR 정상 대조7개를 현재 보정에서 함께 실행해37PASS/0FAIL입니다. release-test/threads8/no-fail-fast, exit0, `output/pr-review/planet6897-green-20261002/nextest-fragment-reserved-margin.log`. #7243 픽셀 기대값은 변경하지 않았으며 두 열 표의 정상 위 여백도 유지했습니다.
- 최초 전체 실행의 잔존 실패는5개에서3개로 줄었습니다. 합성 사다리·빈 각주 검사·다열 표 대조군 및 독립 전체 시각/최종 전체 검증은 아직 남아 있습니다.

### 메인터너 보정 9 사전 분석: 분기 없는 HWPX 문단 여백의 단위

- 원본 `samples/task2070/hy_ladder3.hwpx`는 수동 생성본입니다. 독립 한컴 2020 PDF는 2쪽, 현재 Native는 1쪽이며, 1쪽 진단 일치율은 62.41613%입니다. 기존 넘침 허용값 7→11 변경을 승인 근거로 채택하지 않습니다.
- 한컴 2020에서 같은 원본을 HWP로 저장한 뒤 HWP 파서로 읽은 문단 IR을 대조했습니다. 원본의 분기 없는 `HWPUNIT` 들여쓰기 -2440/-4880은 한컴 HWP에서 -4880/-9760 IR 값입니다. 입력이나 저장 줄을 수정하지 않은 독립 단위 근거입니다.
- `parse_para_shape_margin_value_child`는 분기 없는 값을 그대로 IR에 넣고, `HwpUnitChar` case는 2배로 변환합니다. 이후 문단 스타일 해석은 공통 IR 값을 2로 나누므로 분기 없는 음수 들여쓰기만 실제의 절반으로 배치됩니다. 파서 → 공통 IR → 스타일 들여쓰기 → 줄 가용 폭/배치 경로의 단위 불일치입니다.
- 명시적 `unit="HWPUNIT"` 자식만 공통 IR 단위로 정규화하고, 단위가 없는 이전 속성 형식과 switch/default의 왕복 계약은 유지하는 방향을 검증합니다. 기존 파서 검사에 숫자 계약을 보완하고, 한컴 PDF와 원본 사다리 전체 쪽을 다시 비교합니다. 별도 회귀 fixture나 허용값 증가는 추가하지 않습니다.

- 기존 #4898 기록도 대조했습니다. 당시 여백과 고정 줄간격을 함께 2배 변환한 안은 HWP 저장 축에서 58건을 깨뜨렸습니다. 따라서 이번에는 한컴 저장본으로 입증한 여백 자식만 변환하고 고정 줄간격은 변경하지 않습니다. 평문 HWPX 직렬화도 여백을 역변환해 원본 숫자를 보존합니다. 기존 #4898 검사에 IR→물리 단위 및 재파싱 보존을 함께 검증합니다. 속성 형식·switch/default·줄간격은 변경하지 않습니다.
- 수정 전 기존 파서 검사에 독립 단위 기대값을 보완한 실행은 실패했습니다(-2260, 기대 -4520). 이는 전체 회귀의 새 실패 수가 아니라 단위 결함 재현입니다.

#### 보정 9 결과

- 파서 기존 검사 54개, 평문/switch 왕복 2개, CHAR 홀수 단위 왕복 2개가 통과했습니다. unit-tier 분류 검사도 통과했습니다. 원본 HWPX·넘침 baseline은 변경하지 않았습니다.
- 1쪽 Native 진단은 62.41613%→90.86083%입니다. 직접 비교 PNG에서 음수 들여쓰기와 첫 여섯 셀 줄 구성이 기준에 맞게 회복됐습니다. 아래 증적은 부분 개선의 증거이며 전체 fixture 승인 근거가 아닙니다.
- 한컴은 2쪽이고 Native는 여전히 1쪽입니다. 다음 문단이 쪽 밖으로 밀리는 측정/배치 불일치는 별도 보정 대상입니다. fresh WASM 및 전체 쪽 비교는 아직 통과하지 않았으며 #7476 머지 보류를 유지합니다.
- [독립 한컴 PDF](../../../pdf/task2070/hy-ladder3-2020.pdf), [수정 전 PNG](../assets/planet6897_green_20261002/hy_ladder3_p1_before.png), [단위 보정 PNG](../assets/planet6897_green_20261002/hy_ladder3_p1_margin_fixed.png), [해시·검증 기록](../assets/planet6897_green_20261002/hy_ladder3_margin_validation.json).
- 앞선 #7243 여백 예약 보정의 새 Native 26쪽은 93.84389%이며, [직접 비교 PNG](../assets/planet6897_green_20261002/regulatory86712_p26_reserved_margin.png)를 확인했습니다. 전체 64쪽의 다른 시각 보류는 별도로 유지합니다.

### 메인터너 보정 10 사전 분석: 미저장 셀의 TAC 흐름 높이

- 단위 보정 뒤에도 사다리 전체 쪽은 1/2로 다릅니다. `RHWP_DIAG_TACCAP`에서 실측 표+후행 간격은 1330.4px인데 host 저장 줄 기반 상한은 55.5px이고 실제 조판 사용 높이는 76.8px입니다. paint는 커진 셀을 그려 뒤 문단까지 쪽 밖으로 밀지만, fit·상한은 작은 host 줄만 예약합니다.
- 원본은 모든 셀 글줄의 저장 LineSeg가 없는 수동 생성본입니다. 셀은 실제 텍스트로 다시 조판하면서 host의 옛 짧은 줄을 현재 개체 높이의 근거로 쓰는 모순입니다. 독립 한컴 PDF는 표 뒤 `NEXT PARAGRAPH`를 2쪽에 둡니다.
- 현재 `tac_fit`의 편집 성장 하한 → `typeset_tac_table`의 실제 전진 → `tac_reconcile`의 사후 상한을 추적했습니다. 미저장 텍스트 셀의 실제 높이도 같은 하한으로 연결하고 후속 저장 사다리의 되감기를 막는 방향으로 보정합니다. 저장 셀 줄이 있는 정상 문서는 기존 계약을 유지합니다. 쪽/문단 소속과 전체 2쪽 비교로 결과를 판단하며 baseline은 변경하지 않습니다.

#### 보정 10 결과

- 최종 변경에서 #7418 7개·#5699 2개·#6950 28개·#7243 2개, 기존 관련 검사 총 39개가 통과했습니다. 생성 suite가 재배정된 #7243은 wrapper로 실제 2개 실행을 다시 확인했습니다.
- 원본 사다리의 Native 전체 2쪽은 기준 PDF와 쪽수가 일치합니다. 1쪽 90.86083%, 2쪽 100.00000%이며, `NEXT PARAGRAPH`는 1쪽에서 사라지고 2쪽에 한 번 나타납니다. 실측 셀 높이를 작은 host 줄로 되돌리는 사후 상한도 제거했습니다.
- [1쪽 비교](../assets/planet6897_green_20261002/hy_ladder3_p1_current_band.png), [2쪽 비교](../assets/planet6897_green_20261002/hy_ladder3_p2_current_band.png), [현재 코드 해시·검증 기록](../assets/planet6897_green_20261002/hy_ladder3_current_band_validation.json).
- 넘침 원장 partition14는 원본의 잘못된 들여쓰기 복원 뒤 12줄을 관측해 기존 7줄 기준에서 실패했습니다(그 외 78개 입력은 증가 없음). 7→12 변경은 아직 하지 않았습니다. fresh WASM 전체 2쪽 및 기준 PDF의 잘림 경계를 확인한 뒤 의도된 변화 여부를 판단합니다. 이 원본 표 자체는 한컴도 1쪽에 통배치하고 아래쪽 내용을 자릅니다. 쪽수 복원을 원본 모든 셀 내용의 가시성 확보로 확대하지 않습니다.

### 메인터너 보정 11 사전 판정: 사다리 원장의 의도된 변화

- `bf5eced2a` fresh WASM 전체 2쪽도 Native와 동일하게 90.86083%/100.00000%로 통과했습니다. 기준 PDF와 2/2쪽이며, 실제 WASM SVG·WASM render tree 출처를 확인했습니다.
- 기존 7줄은 들여쓰기를 절반으로 읽고 뒤 문단까지 같은 쪽 밖으로 보내던 Native 출력의 관측값입니다. 올바른 내어쓰기와 한컴처럼 2쪽으로 이월되는 뒤 문단을 복구한 현재 출력에서 12줄을 관측합니다. 한컴 원본 PDF도 TAC 표를 1쪽에 통배치하고 아래 셀을 자르므로, 표를 임의로 나눠 원장 수만 낮추지 않습니다. 이 판단은 source가 제안한 11줄을 그대로 채택한 것이 아닙니다.
- 전체 Native/fresh WASM ≥90% 및 쪽·뒤 문단 소속을 확인한 뒤 사다리의 기존 원장 한 행만 7→12로 갱신합니다. 다른 입력의 허용값이나 전체 threshold는 변경하지 않습니다. 기존 partition14와 인접 원장 검사를 실행한 뒤 결과를 기록합니다. 신규 회귀 검사나 입력은 추가하지 않습니다.

#### 보정 11 결과

- 기존 overflow-cell partition14가 통과했습니다(79개 입력, 다른 입력의 증가 없음). 최초 전체 실패 중 사다리 1건을 추가로 처리해 집중 검증으로 처리한 항목은 31/33개입니다. 최종 전체 재실행은 아직 아닙니다.
- [Native 2쪽 TSV](../assets/planet6897_green_20261002/hy_ladder3_native.tsv), [fresh WASM 2쪽 TSV](../assets/planet6897_green_20261002/hy_ladder3_wasm.tsv)는 `bf5eced2a`에서 생성한 전체 PNG를 재사용해 같은 실루엣 계산식으로 산출했습니다. 추가 원문 재출력으로 기록하지 않습니다.
- [fresh WASM 1쪽 비교](../assets/planet6897_green_20261002/hy_ladder3_p1_fresh_wasm.png), [fresh WASM 2쪽 비교](../assets/planet6897_green_20261002/hy_ladder3_p2_fresh_wasm.png). Mac `--no-opt` 대체 빌드 통과이며 Docker 최적화 빌드 통과는 아닙니다.
- 다른 실물 fixture의 잔존 시각 차이와 최종 전체 게이트 때문에 원 PR #7476의 최종 판정은 계속 보류입니다. 사다리 보류 사유만 이번 근거로 해소했습니다.

### 보정 10의 기존 검사 보완

- 전체 Native/fresh WASM 선행 기준을 통과했으므로, 기존 host 줄간격 검사 한 개에 이미 저장소에 있던 사다리의 쪽·뒤 문단 소속 계약을 보완합니다. 신규 test 함수나 fixture는 추가하지 않습니다. 이전 출력의 1쪽/기준 2쪽 차이를 의미로 검사하고, 절대 픽셀 위치를 기대값으로 추가하지 않습니다.

- 보완한 기존 #7418 검사 7개가 모두 통과했습니다. 사다리 계약은 정확한 2쪽, 1쪽 표 유지, 뒤 문단의 2쪽 단일 출현, 표의 임의 분할 금지를 확인합니다. test 함수 수는 7개 그대로이며 새 fixture를 추가하지 않았습니다. 생성 suite는 003으로 재배정되어 wrapper의 실제 7개 실행을 확인했습니다.

### 메인터너 보정 준비: 평문 여백의 패키지 버전 계약

- 전체 실패 `task903_hwpx_h_01_para_shape_margin_children_are_parsed`는 유효합니다. 원본 패키지 xmlVersion1.2의 문단10 들여쓰기 -2800은 이번 한컴2020 재저장에서도 HWP IR -2800으로 유지됩니다. 이전 5b8be2ac8은 이 값을 -5600으로 잘못 확대했습니다.
- 원본 내용을 그대로 둔 통제 실험: header.xml의 head version만 1.4로 바꾸면 HWP IR은 -2800 그대로입니다. version.xml의 xmlVersion을1.4로 함께 바꾸면 -5600이 됩니다. 반대 방향으로 hy_ladder3 패키지 xmlVersion을1.2로 낮추면 원본 -2440/-4880이 그대로 저장됩니다. h01의 xmlVersion1.3도 -2800입니다. 처리 기준은 파일명·한컴 제품명이 아닌 실제 패키지 XML 버전입니다.
- 수정 계획: version.xml의 xmlVersion이1.4 이상일 때만 평문 HWPUNIT 여백을 공통 IR 2배로 읽고, 기존1.2/1.3은 원값을 유지합니다. 평문 저장 시에도 같은 읽기 출처를 보존하여 역변환을 적용합니다. switch/default·고정 줄간격은 변경하지 않습니다. 기존 #903 정답 기대값을 유지하고 #4898·#6875·기존 파서 단위 검사 및 정상 hy_ladder3 전2쪽을 검증합니다. 새 회귀 함수·문서는 추가하지 않습니다.

### 패키지 단위 보정 결과와 별도 시각 보류

- `version.xml`의 xmlVersion을 읽어 1.4 이상에서만 평문 물리 여백을 2배 공통 IR로 변환합니다. 읽기 출처를 문단 모양에 보존하여 평문 저장 때 동일한 역변환을 적용합니다. 구버전·판본 미상은 기존 원값, switch/default·고정 줄간격은 기존 계약입니다. 기존 HWP 파서는 이 출처를 사용하지 않습니다.
- #903 기존 독립 HWP 대조 기대값은 유지했습니다. #903/#4898/#6875 기존5개 PASS, 파서 기존54개 PASS, Clippy --lib release-test exit0. #4898 기존 함수에서1.2/1.3/1.4의 평문 왕복을 검증했으며 새 함수·fixture는 없습니다. 이 보정의 목적은 기존 파싱·저장 계약이며 새 렌더링 golden을 등록하지 않았습니다. [한컴 통제 실험](../assets/planet6897_green_20261002/plain_margin_version_oracle.json), [검증 증적](../assets/planet6897_green_20261002/plain_margin_version_validation.json).
- 정상 hy_ladder3는 원본/기준/Native/fresh WASM 모두2쪽, 전쪽 최저90.86083%를 유지했습니다. Mac fresh WASM --no-opt exit0이며 Docker 최적화 검증이 아닙니다.
- h01은 원본/기준/Native/fresh WASM 모두9쪽입니다. 2~9쪽은98% 이상이나1쪽은양쪽53.03132%입니다. 비교 PNG에서 제목 프레임과 이후 본문/표의 위쪽 이동을 직접 확인했습니다. 단위 실패 해결과 전체 문서 승인 판단을 분리합니다. [1쪽 review](../assets/planet6897_green_20261002/plain_margin_version_h01_p1_review.png). 9쪽 문서는 이 브랜치에서 추가 보정할 보류이며 #7445로 이관하거나 renderer 기대값을 느슨하게 하지 않습니다. 전체 회귀 재실행 전이므로 PR 최종 판정은 보류입니다.

### 메인터너 검토 준비: 이미 이관한 Q29 물리쪽의 남은 반대 단정

- 현재17개 재실행은15PASS/2FAIL입니다. #3930 혼합 검사에 `p296은Q29를가지면안된다`는 반대 단정이 남아 있습니다. 독립 한컴2024 PDF의 물리296쪽을 pdftotext로 다시 읽으면 Q27/Q28/Q29가 있고 Q29 표제와 응답이 모두 있습니다. 과거 #7382 보정146도 같은 PDF/원본 SHA로296쪽의Q27/Q29를 확인하고 실패 물리쪽 전제를 #7445 issuecomment-5883936903에 이관했습니다. 현재 단정은 독립 기준과 반대로 기대하므로 renderer를 바꿔 통과시키면 기준에서 멀어집니다.
- 이 단계는 이미이관한Q29의 잘못된 부정 단정 하나와 전용 상수만 제거합니다. 원본384쪽과 PDF, 나머지 같은 쪽 배치·셀넘침·저장전후13쪽 동일성·바탕쪽/IR 계약을 유지합니다. 현재296쪽을 새로운golden으로 등록하지 않으며, 전체90% 미달 문서의 쪽수나 새로운 렌더링 검사는 추가하지 않습니다. 기존 #7445 기록을 연결하고 새 공개 댓글은 게시하지 않습니다.

- 실행 결과: 잘못된Q29 부재 단정 한곳과 미사용상수만 제거했습니다. 나머지 혼합 계약과 같은파일 기존3함수 모두PASS(nextest release-test threads8, exit0). 저장전후13쪽 결과동일성·기존바탕쪽/IR/표/그림조건을 유지했습니다. 독립 PDF SHA는 기존 #7445 증적과동일합니다. [검증 증적](../assets/planet6897_green_20261002/handbook_q29_inverse_assertion_validation.json). 큰문서의 전체피델리티 이슈나 원본/PDF는 제거하지 않았고 새로운현재쪽 golden도 등록하지 않았습니다.
- 직전17개 집중재실행은15PASS/2FAIL이며 이중#3930은본단계기존3검사로해결했습니다. 현재남은재현FAIL은 `hwpx_sample2.hwpx`의 off_canvas partition1입니다. 전체 nextest/최종visual은아직미완료입니다.

### 이번 단계 종료 재검증 요약

- head `8927c6ccb`에서 이전 전체 실패17개 이름을 모두 확인했습니다. generated suite 재배정으로16개 실행15PASS/1FAIL과 별도 oracle partition7의1PASS를 합쳐16PASS/1FAIL입니다. 전체10,281개 재실행 결과가 아닙니다. [최종 집중 증적](../assets/planet6897_green_20261002/prior17_final_8927c6ccb.json).
- 잔존FAIL은29쪽 hwpx_sample2의8쪽표 용지밖1건입니다. h01의1쪽53.03132%와 큰문서 미달쪽, 최신215쪽전쪽Native/freshWASM·전체회귀 검증은 추가보류입니다. 이들을 완료로 간주하거나 blanket baseline 변경·새골든등록·원본삭제를 하지 않았습니다. PR 최종승인/머지준비는미완료입니다.

### h01 1쪽 저장 TAC 앞 빈 줄 보정 사전 분석

- 현재 Native 전9쪽을 재실행해1쪽53.03132%, 나머지98% 이상을 확인했습니다. 첫 로고 표의 괘선은 Native98.2px/PDF100.051px입니다. 호스트의 첫 저장 줄은 빈 줄(text_height100HU+gap44HU), 표 소유 줄은 vpos144HU이며 line_height4091HU가 표3525HU+바깥여백566HU와 정확히 같습니다. 빈 줄의 line_height에는 문단 최대 표 높이가 반복되므로 text_height와 다음 원점이 실제 빈 줄 점유 근거입니다.
- composer `stored_tac_lines`는 공백 텍스트 캐리어의 앞 빈 줄은 수용하지만 빈 컨트롤 캐리어의 단일 표와 PageNumberPos를 거절합니다. typeset `stored_tac::prepare`의 공통 pen/end가 생성되지 않아 일반 TAC paint/flow가 앞144HU와 뒤 간격을 잃습니다. 쪽번호 위치 지정은 본문 줄을 그리는 개체가 아니므로 저장 빈 줄 연속성·표 소유 높이·단일 마지막 소유 줄을 같은 계약으로 수용합니다. 가시 객체·편집/합성/비연속 줄은 제외합니다.
- 기존 공통 측정/paint 계획을 사용하고 문서별 수치 보정은 추가하지 않습니다. 첫 표 보정 후 나머지 제목/본문 차이를 별도로 확인하며 기존 관련 검사와 정상 사다리/양돈 자료, Native/fresh WASM 전9쪽을 검증합니다. 신규 검사·fixture·잠정 golden은 추가하지 않습니다.

- 첫 후보는 Native53.03132%로 무변화였습니다. 구역 첫 문단의 `empty_control_stream_position`이 축 보정량이 있다는 이유로 먼저 거절합니다. 초기 진단은 보정량을 잘못 추정했으며 실제 보정량8을 더한ts32는 cc33의 마지막 문단부호입니다. 기존 `stored_text_starts_on_hwp5_axis`는 문단 끝을 넘는 경우만 확인하므로 이 개체 줄을 구별하지 못했습니다. 축 증거가 있는 완전8유닛 스트림만 재사용하고 미확정 HWPX/합성은 계속 거절합니다. 컨트롤 소속 생산 → composer 줄 계획 → typeset pen/end → paint 공통 배치의 연결을 재검증합니다.

- 진단 재확인: cc33/4컨트롤·axis8·offset없음·분할dirty없음·실측47px입니다. 24는 마지막 표의8유닛 슬롯이고32는 문단부호입니다. 완전 제어 스트림의 실제 인라인 개체 시작을 문단부호로 옮기는 보정만 거절하며, 텍스트·불완전 스트림·중간 슬롯의 기존 판정은 유지합니다. 진단용 출력은 최종 코드에서 제거합니다.

- 축 판정 후 첫 표는98.2→100.2px로 PDF100.051px에 맞습니다. 다음 제목 프레임은186.6px/PDF190.192px이며 그 후 본문 차이가 남습니다. p3은 인라인 날짜 표의 소유 줄2131HU가 float offset2346HU 앞에 들어가고, 다음 저장 원점과 차이9067HU가 offset2346+float높이6155+바깥여백566과 정확히 같습니다. 기존 공통 상자 helper가 offset0·표1개만 수용하여 혼합 줄에서 위여백과 전체 점유를 버립니다. 같은 줄의 TAC 소유 높이가 offset 앞 공간에 들어가며 다음 줄이 전체 상자를 닫는 경우만 공통 offset+바깥상자 점유를 생산하여 예약/paint에 전달합니다. 그 밖의 양수 offset, 가시 글자·그림·재조판은 유지합니다.

### h01 시각 보정 중간 결과

- Native/fresh WASM 전9쪽 같은 일치율, 최저1쪽94.28413%, 나머지98% 이상·미달0입니다. 제목/본문/표 위치를 review PNG로 직접 확인했고 마지막 표의 약3px 차이는 잔존합니다. Native/fresh WASM 정상#6797 11쪽·사다리2쪽 렌더 트리는 이전 검증 출력과 전쪽 바이트 동일합니다. 기존 관련62검사62PASS입니다.
- 새 단일 빈 줄 캐리어에 각주가 있는 경우 공통 단축 대신 기존 일반 예약을 유지하도록 마지막 guard를 보완했습니다. 최종 빌드·lint·출력 동일성·집중 회귀를 재검증합니다. 검증된 기존 h01 검사1개에 저장 빈 줄 점유와 제목 상자의 관계를 보완하며 새 함수/fixture나 절대px 핀을 추가하지 않습니다. 통합PR 준비·최종 전체 검증은 미완료입니다.

### h01 보정 최종 단계 결과

- 마지막 각주 예약 guard·기존 h01 관계 검사 보완 후 기존 관련62건62PASS입니다. Native/fresh WASM 빌드·root/WASM/workspace Clippy·workspace 빌드·fmt·최신base suite 정책·변경 문서 링크 exit0입니다. Mac fresh WASM no-opt 대체 빌드이며 Docker 최적화 검증은 아닙니다.
- 마지막 생산 코드에서 Native full-font print SVG/렌더 트리와 fresh WASM raw SVG/렌더 트리가 실제 시각 비교 출력과 전9쪽 바이트 동일함을 확인했습니다. 앞선 직접 raster 비교의 Native/WASM 전9쪽 최저94.28413%·미달0 근거를 연결하며 추가 재래스터화로 보고하지 않습니다. 빈 줄 점유·양수offset 바깥상자 관계는 이전 실제 render tree에서2조건 FAIL, 현재2조건 PASS를 확인했고 보완한 Rust 함수도 PASS입니다. 새 test 함수/fixture·baseline·기준 PDF 변경0건입니다.
- [최종 원장](../assets/planet6897_green_20261002/h01_correction_validation.json), [Native 전9쪽 TSV](../assets/planet6897_green_20261002/h01_corrected_native.tsv), [WASM 전9쪽 TSV](../assets/planet6897_green_20261002/h01_corrected_wasm.tsv), [1쪽 review](../assets/planet6897_green_20261002/h01_corrected_native_p1_review.png), [1쪽 overlay](../assets/planet6897_green_20261002/h01_corrected_native_p1_overlay.png). 첫 페이지에서 마지막 표 약3px 및 제목 글자 미세 차이는 잔존합니다.
- h01의90% 미달 보류 사유를 해소했습니다. 다른 시각 보류와 최신215쪽 전체 비교·최종 전체 회귀는 별도 미완료이며 #7476/통합 PR 전체를 승인 완료로 표현하지 않습니다. 이 단계 커밋 후 전체 nextest를 실행합니다.

### 전체 검사에서 확인한 메인터너 보정의 반례(#7103)

- h01 메인터너 보정에서 추가한 빈 줄 검증이 음수 줄간격을 무조건 거절해 기존 복수 TAC 표의 원문 계획을 무효화했습니다. 기여자 원 PR의 결함으로 분류하지 않습니다. 원문 빈 줄 높이300HU·간격-92HU·다음 원점208HU는 정상이며, 실제 전진량과 저장 원점 연결을 검사하도록 수정했습니다.
- 기존4건4FAIL→4PASS, 기존2건의 절대 PDF 좌표를 원문 표 높이·저장 줄 간격·본문 포함·내용 순서로 변경한 뒤에도4PASS입니다. Native/fresh WASM 전1쪽95.54807%, h01 전9쪽 SVG/렌더 트리는 양 backend에서 기존 시각 증적과 바이트 동일합니다. 새 test 함수/fixture/golden은 추가하지 않았습니다. [검증 원장](../assets/planet6897_green_20261002/tac7103_correction_validation.json), [review](../assets/planet6897_green_20261002/tac7103_corrected_native_p1_review.png). 다른 전체 실패와 시각 보류는 미완료입니다.

### 2024 호환 경로 반례 복원

- 메인터너 h01 보정의 단일 TAC 선행 줄 계획이 기존 2024 회수량 적립과 후속 쪽 경계 재적합을 건너뛰어 세대 검사2건이 실패했습니다. 2024의 해당 입력은 기존 일반 경로가 담당하도록 복원했고 기존4검사4PASS입니다. 기본2022·해제 후 pi13=2쪽,2024 pi13=1쪽이며 실제 줄이 단 내부에 포함됩니다. h01 Native/fresh WASM 전9쪽 출력은 이전 시각 증적과 동일합니다. [검증](../assets/planet6897_green_20261002/compat5524_correction_validation.json). Native2024 독립 PDF의 전체 시각 일치율은 미검증이므로 그 범위의 완료나 통합 승인으로 표현하지 않습니다.

### 각주 대조군의 유지보수 보정과 기존 검사 보완(#598)

- 확정 TAC 줄 pen에 저장 원점을 다시 더하던 paint 경로, 문단 앞 간격을 뺀 reset owner 비교, HWP5 쪽 첫 간격을 원점으로 빼던 두 소비자, 감추기 메타데이터 때문에 표 앞 빈 줄을 잃던 경로를 단계별로 보정했습니다. 기여자 원 PR의 잘못으로 일괄 분류하지 않고 통합 브랜치와 메인터너 보정의 공통 경로 반례로 기록합니다.
- 새 한컴2020 PDF로 Native/fresh WASM 전6쪽 TSV가 동일하며 최저1쪽98.10308%·90미달0입니다. 원문2010 저장본의 engine2020 재출력 PDF를 커밋했고 쪽수6/6·쪽별 텍스트 동일을 확인했습니다.5쪽 원문 나눔고딕과 한컴 바탕 대체 글꼴 차이는 남습니다. 색 있는 표 배경의 실루엣100%를 글꼴 일치로 해석하지 않습니다.
- 기존 각주 검사5함수는 유지하고, 낡은 절대 클릭 좌표를 실제 페이지0 마커의 구역·문단·제어·번호 소속으로 대체했습니다. 커서 offset과 삭제 후 마커 부재를 유지하며,3쪽 첫 줄의 문단 소속·2쪽 중복 부재를 기존 첫 함수에 보완했습니다. 최초 실패2건을 포함해5건5PASS입니다. 신규 `#[test]` 함수/fixture/golden 추가0·검사 삭제0입니다.
- [검증 원장](../assets/planet6897_green_20261002/footnote598_final_validation.json), [Native 전6쪽](../assets/planet6897_green_20261002/footnote598_final_native_all6.tsv), [WASM 전6쪽](../assets/planet6897_green_20261002/footnote598_final_wasm_all6.tsv), [3쪽 review](../assets/planet6897_green_20261002/footnote598_final_native_p3_review.png), [5쪽 review](../assets/planet6897_green_20261002/footnote598_final_native_p5_review.png). 다른 전체 회귀 실패와12/215쪽 시각 보류는 남아 있어 통합 PR 승인/준비 완료로 표현하지 않습니다.

### 어울림 그림 뒤 문단의 단 하단 넘침 보정(#6812)

- 통합 후보의 기존 회귀20건 중1건은 그림을 추가했을 때 본문 둘째 줄이 단 하단3.64px 밖에 그려졌습니다. 세 줄 소속과 하단 포함을 확인하는 검사 관계는 정상이며, 기대값을 완화하지 않았습니다.
- 메인터너 분석에서 첫 TAC 표 소유 줄을 Square 그림의 빈 안내 줄로 제외해 측정은 후행 간격0, paint는12px를 소비하는 불일치를 확인했습니다. 사진·도형 소유 helper에는 표가 포함되지 않으므로 실제 표 제어의 소속 줄과 원본 줄 높이 증거를 보완했습니다. 문서 번호·고정 위치에 따른 예외를 추가하지 않았습니다.
- 기존20건20PASS와 관련12건12PASS(exit0), 새 검사·픽스쳐·golden 갱신0입니다. [검증 원장](../assets/planet6897_green_20261002/6812_table_owned_line_validation.json), [Native 전11쪽](../assets/planet6897_green_20261002/6812_social6797_current_native_all11.tsv), [WASM 전11쪽](../assets/planet6897_green_20261002/6812_social6797_current_wasm_all11.tsv), [7쪽 review](../assets/planet6897_green_20261002/6812_social6797_current_p7_review.png).
- #6797은11/11쪽·최저92.15032%·미달0, #598 각주6쪽은 이전 출력과 동일합니다. fresh WASM 전17쪽 SVG는 Native와 동일합니다. 7쪽 첫 표의6.667px 상향은 이번 보정 이전 실행 파일에서도 확인되었으며 이전 원점 보정의 잔차로 재검토합니다. 전체 회귀 및 다른 문서 시각 보류가 남아 있어 승인/PR 준비 완료가 아닙니다.

### 생성 SVG의 두부문자 재확인과 macOS 표준 폰트 경로 보완

- `form-002/page-0.actual.svg`를 재생성하니 API의 기본 Full 임베딩에서 휴먼명조 굵은 face가 local()로 남아 파란 핵심 목표 줄이 두부문자로 표시됐습니다. 명시 경로를 쓴 기존 확인본만으로 기본 API 경로를 확인했다고 보지 않았습니다. 공통 기본 탐색에 macOS 표준 `~/Library/Fonts`를 포함하여 설치된 윤곽선 글꼴을 공급했습니다.
- 기존 폰트 경로5건5PASS와 unit 정책 검사exit0, 새 test 함수0입니다. 현재 파일은 이전 정상 확인 SVG와 해시 동일이며 새 Chrome 캡처도 픽셀 동일합니다. [현재 PNG](../assets/planet6897_green_20261002/form002_font_after_chrome.png), [검증](../assets/planet6897_green_20261002/form002_font_preview_validation.json). CSS를 제외한 비교 원문과 golden 기대값은 변경하지 않았고, 표 기하에 따른 스냅샷 실패는 계속 보류입니다.175MB 진단 SVG는 요청 경로에 남기고 `.gitignore`로 커밋에서 제외합니다.

## 기존 table-text 검사 보정 — 2026-10-04

- 원 기여자의 재조판 간격 변경 뒤 기존 golden은59글자의 x 문자열 차이로 실패했으며, 내용140글자·세로 위치·전체223요소는 유지됐습니다. 해당1쪽 문서는 독립 한컴2020 PDF와 현재 release-test Native·fresh WASM 전쪽 비교를 마쳤습니다. 두 실행 실루엣100%이며 review에서 실제 표와 숫자를 확인했습니다. 엄격 픽셀70.58156%는 글꼴 획 차이도 포함하므로 완전 픽셀 일치로 보고하지 않습니다.
- 메인터너 보정은 기존 `svg_snapshot::table_text_page_0` 한 개를 표18칸의 내용·행열 소유·칸 내부 표시·수치와 증감 제목 가운데 정렬 검사로 전환합니다. 새 함수/fixture/production 변경과 golden 갱신은 없습니다. 셀 상대 허용량은 렌더 트리의 소수점 반올림만 처리하며 절대 위치를 고정하지 않습니다.
- 집중 해당1건1PASS(exit0). 기존 snapshot 묶음도6PASS·form002 배치1FAIL(exit100)로 재확인했습니다. form002 실패나 최종 전체 검증을 이 결과로 승인하지 않습니다. 필수 gate 결과는 [검증 기록](../assets/planet6897_green_20261002/tabletext_semantic_validation.json)에 기록합니다. [Native TSV](../assets/planet6897_green_20261002/tabletext_current_native_all1.tsv), [WASM TSV](../assets/planet6897_green_20261002/tabletext_current_wasm_all1.tsv), [검토 PNG](../assets/planet6897_green_20261002/tabletext_current_p1_review.png).

## #6797 7쪽의 저장 원점 재보정 — 2026-10-04

- 독립 재확인에서 #598 보정 후7쪽 첫 표가6.667px 위로 이동하여 이전98.05666%에서92.15032%로 낮아졌습니다. 원 기여자의 표 내용 변경과 별개로 메인터너가 HWP5에도 확대 적용한 쪽 원점 규칙의 영향입니다. 원본 첫문단69의 글자처럼 취급되는 묶음 도형과 제목이500HU 저장 원점을 함께 소유하는데 일반 텍스트 문단 앞 여백으로 판정했습니다.
- 공통 `stored_first_margin_is_page_relative`에서 첫문단 TAC 도형을 제외해 페이지네이터·렌더러의 저장 원점 판정을 함께 바로잡았습니다. 해당 표만 기존 정상 위치로 돌아오고 다른10쪽 렌더 트리는 수정 전과 동일합니다. 새 검사·fixture·기대값 변경은 없습니다.
- 현재 Native 전체11쪽90% 미달0건, 최저95.80181%,7쪽98.05666%입니다. 관련 기존 #6797·#598·#6972·#6812 총32검사32PASS(exit0). #598 전6쪽 현재 Full SVG/렌더 트리는 기존 검증본과 바이트 동일하며 새 raster 실행으로 주장하지 않습니다. 실루엣 점수를 글꼴 획의 완전 일치로 보고하지 않습니다. fresh WASM·필수 gate 결과는 [검증 기록](../assets/planet6897_green_20261002/social6797_origin_validation.json)에 남깁니다. [Native 전쪽 TSV](../assets/planet6897_green_20261002/social6797_origin_native_all11.tsv), [7쪽 review PNG](../assets/planet6897_green_20261002/social6797_origin_p7_review.png). 통합 전체 검증·PR 판정은 보류 상태입니다.

- #6797 최종 단계 결과: 새 WASM 실제 전11쪽 raster·TSV도 Native와 동일하며 미달0건/최저95.80181%,7쪽98.05666%입니다. Native/WASM Clippy·workspace build/all-target Clippy·fmt·고정 base manifest·문서 링크/metadata 검사 모두 exit0. [WASM TSV](../assets/planet6897_green_20261002/social6797_origin_wasm_all11.tsv). 기존 전체 실패 중 남은4함수의 현재 집중 재실행은4FAIL(exit100)로 확인하여 전체 PR 승인으로 보고하지 않습니다.


### 메인터너 후속 보정: #5731의 셀 그림 프레임과 빈 자르기 선택

이 단계는 기여자의 해결 범위를 다시 주장하는 것이 아니라 통합 브랜치의 기존 실패를 독립 원문과 한컴 출력으로 재검증한 메인터너 보정입니다. 이전 TAC 선언 높이 재확장 제거 뒤 작은 초기 셀 높이에 둘째 그림이 제한되어 캡션과 겹쳤습니다. 마지막 저장 앵커+그림 높이+유효 안 여백이 개체 프레임을 정확히 닫는 경우에만 그 프레임을 조판과 실제 배치가 함께 소비합니다. 일반 TAC 표의 낡은 선언을 다시 최소 높이로 적용하지 않습니다.

기존35KB 픽스처는 그림을1×1로 치환한 자료였습니다. 본문·서식·메타데이터를 바꾸지 않고 같은 fixture 경로에 원본 BinData6개를 복원했습니다. 동일 원문으로 생성한 MCP2020 직접 PDF와 한컴 HWP 재저장 뒤 PDF는7쪽 모두 동일한96dpi 픽셀 해시입니다. 정본에는 총6그림이 나옵니다. 기존 회귀 주석의 “한컴도7개”와 3쪽의 그림2개 표시 전제는 잘못됐습니다.

첫 그림의 자르기 선택은 폭이 있지만 높이가 역전되어 비어 있습니다. rhwp가 이 선택을 None으로 바꾸면서 원본 전체를 표시한 것이 남은3쪽80.43%의 원인이었습니다. 빈 선택을 보존하고 그 선택 안의 픽셀만 그리는 계약을 SVG/Native Skia/Canvas/HTML/Studio DOM/CanvasKit에 적용했습니다. 원본 그림 개체·저장 글줄 공간·셀 소유와 캡션은 유지합니다. 측정 결과는 `fit_measured_for_host`와 `resolve_row_heights_with_common_fit`에서 같은 그림 프레임을 소비하며, 실제3쪽에서 최종 셀 경계와 두 캡션·그림 자리·뒤 본문까지 확인했습니다.

Native 전7쪽 최저95.59735%, 90% 미달0쪽입니다. 최종 Native CLI에서 재산출한 SVG7개·렌더 트리7개가 비교 산출물과 동일하여 래스터/TSV를 재사용합니다. 기존 테스트1개는 절대 픽셀 고정 대신 쪽수·원문 그림 보존·캡션 내용/순서·셀 내부 포함·빈 선택의 실제 출력으로 변경했습니다. 관련 기존21검사 모두 통과했으며 새 테스트 함수는 추가하지 않았습니다. Studio1,814 PASS/실패0/skip2와 기존 CanvasKit 실제 자르기4모드도 통과했습니다. fresh WASM 전쪽·Native Skia 공식 회귀·최종 전체 검증은 아직 완료하지 않았으므로 이 중간 결과만으로 통합 PR 준비 완료나 승인 완료를 선언하지 않습니다.

- #5731 추가 완료: 새 Mac 로컬 no-opt WASM 전7쪽 TSV는 Native와 동일합니다(최저95.59735%, 미달0쪽). Native Skia 전체 단위4,109 PASS/실패0/skip13. [보정 검증 JSON](../assets/planet6897_green_20261002/cell5731_frame_crop_validation.json)에 정본 PDF 두 경로 비교·그림 복원·전체 TSV·진행 중 검사 상태를 보존합니다. 공식 Skia 개별2종과 보정 전 새 관계 검사 재현은 후속 검증으로 남깁니다.

- #5731 보정 커밋 `14d205d83`: 공식 Native Skia 단위4,109 PASS에 이어 missing-picture2/2, direct-PDF4/4 PASS 및 실제3쪽 PNG 출력0. 직접 Skia3쪽의 한컴 정본 비교도98.07425%입니다. 동일한 기존 관계 검사를 보정 전 Rust 소스에서 실제 실행하자 둘째 그림이 앞 캡션을 덮는 관계 위반으로1 FAIL(exit100)이 재현됐습니다. 보정 소스는 파일별 SHA를 대조해 원상 복원했으며 같은 검사의 수정 후 실행을 진행 중입니다. 새 테스트 함수·기준선 허용치 변경은 없습니다.

- #5731 수정 후 동일 검사1 PASS(exit0)까지 확인했습니다. 보정 전은 컴파일 오류가 아닌 실제 캡션 겹침1 FAIL(exit100)입니다. 13개 보정 Rust 파일의 SHA는 앞서 lint·전쪽 Native/WASM·Skia 검증에 사용한 소스와 동일하게 복원됐습니다. 이 단계 검증은 완료이며, 통합 브랜치의 나머지 차단과 최종 전체 검증을 이어갑니다.

### 메인터너 후속 보정: #1749 재조판 표의 조각 마지막 간격

- 독립 한컴2020 출력에서도4쪽 표의 별표 줄은50/67/10,5쪽은67/22/67/16/67/4입니다. 기존 회귀의 의미 기대값은 유지합니다. 원본 HWPX 셀의 첫5문단에는 저장 LineSeg가 없으므로 저장 프레임 수용 조건을 완화하지 않았습니다.
- 재조판 마지막 줄 뒤 간격을 현재 조각 끝의 점유로 잘못 더하여10개 별표 줄이5쪽으로 밀렸습니다. 같은 trailing trim을 컷 선택·조각 예약·table_partial 행 배치가 소비하도록 보정했습니다. 저장 줄·개체·중첩 표는 기존 계약을 유지합니다.
- 기존3검사3 PASS, 관련26검사26 PASS(exit0), Native 전5쪽 최저91.10644%/미달0쪽입니다.5쪽은83.28153→92.61078%. 표 내부 앞 간격과 괘선 표현 차이는 남아 있으며 fresh WASM 전5쪽도 Native와 동일하며 필수 Rust lint/빌드/정책 gate가 모두 통과했습니다. 새 회귀 함수·허용치 변경은 없습니다.
- [정본 PDF](../../../pdf/task1749/saved_bounds_cumulative_page_break-hwpx-2020.pdf), [검증 JSON](../assets/planet6897_green_20261002/savedbounds1749_reflow_validation.json), [Native 전쪽 TSV](../assets/planet6897_green_20261002/savedbounds1749_reflow_native_all5.tsv), [4쪽 review](../assets/planet6897_green_20261002/savedbounds1749_reflow_p4_review.png), [5쪽 review](../assets/planet6897_green_20261002/savedbounds1749_reflow_p5_review.png). 다른 보류·최종 전체 검증이 남아 통합 승인 보류를 유지합니다.

### form002 바깥 위여백 단계 — 전체 승인 보류

- 원본 `form-002.hwpx`의 선언 첫 조각 높이69446HU와 바깥 위·아래283HU가 본문70012HU를 닫습니다. A4 정규화 본문70014HU와의2HU 차이는 원본 정수 단위로 비교합니다. 폭0 저장 앵커와 구역·단·쪽번호 설정은 별도 가시 글줄을 만들지 않습니다.
- 첫 조각 배치 원점, 이어받기 예산, 표 paint가 동일한 위여백 소유를 소비하도록 보정했습니다. Native 전10쪽 상단94.48→98.3px, 독립 한컴 PDF98.292px와 대응합니다.
- 기존 관련 회귀42건42PASS·실패0. Native 전10쪽은9쪽 미달→4쪽 미달(1쪽89.18899%,4쪽83.46681%,8쪽66.25781%,10쪽84.21871%). 첫 여백 보정만으로 전체 승인을 선언하지 않으며 golden과 기존 테스트는 유지합니다.
- [Native 전쪽 TSV](../assets/planet6897_green_20261002/form002_outer_margin_native_all10.tsv), [1쪽 review](../assets/planet6897_green_20261002/form002_outer_margin_native_p1_review.png), [8쪽 review](../assets/planet6897_green_20261002/form002_outer_margin_native_p8_review.png), [단계 검증](../assets/planet6897_green_20261002/form002_outer_margin_validation.json). fresh WASM 전10쪽 TSV는 Native와 바이트 동일합니다. Mac no-opt WASM·Studio SHA `b47ceadea2e369126397a5abf512eedf257d566a8a81d46f31fb67da71463e2c`, Docker 최적화 대체가 아닙니다. fmt·Native/WASM Clippy·workspace build/all-target Clippy·manifest·문서 링크 검사 모두exit0입니다. 다음은 안내 표 앞뒤 간격·남은 분할 높이를 보정합니다.

### form002 원본 물리 프레임 단계 — 시각 검증 충족, 기존 golden 교체 대기

- 첫 폭0 앵커의 구역 설정이 object-only 판정에서 빠져 첫 표 조각의 빈 하단 밴드가 소실되었습니다. 본문을 닫는 저장 프레임을 기존 첫 조각 예약·paint 계약에 연결했습니다. 내용과 선언 높이 차이가0.5px 미만인 경우도 같은 원본 프레임 소유를 유지하여 말미 접기의 재차감이 발생하지 않게 했습니다.
- noAdjust 원본의 문단 간 저장 쪽 경계가 단일 남은 프레임임을 확인한 경우, 첫 조각이 소비한 공간을 선언 행 높이에서 뺀 잔여를 다음 조각에 전달합니다. 원본8쪽 해당 셀 잔여는20801HU입니다. 일반 내용 컷·편집본·다중 남은 프레임의 기존 처리 범위는 넓히지 않습니다.
- Native와 fresh WASM 각각 전10쪽 모두90% 이상, TSV 바이트 동일. 최저8쪽92.96827%,1쪽97.53277%,4쪽98.57131%,10쪽98.34023%. 기존 관련42건42PASS·실패0, 필수 lint·build·manifest 모두exit0. fresh Mac no-opt WASM SHA `4792539457a5da53ff2d1dd8578b5feca7e8f55045d00529b7ee06385bb3d76c`·Studio 동일이며 Docker 최적화 통과로 보고하지 않습니다.
- 요청된 최신 `.actual.svg`를 다시 생성하고 실제 Chrome 확인: 파란 한글 두부 없음, 실제 체크박스 보존. 8쪽 안내 표의 작은 위치 차이는 남으며90% 충족과 구분합니다. 기존 고정 SVG 기대값1건은 여전히FAIL(exit100)입니다. 다음 단계에서 기존 검사 하나를 원본 표·쪽·분할 문단 소유 의미 검사로 바꿉니다. 신규 회귀 함수·golden 갱신0건, 최종 전체 회귀·남은 다른 문서 보류는 미완료입니다.
- [정본 PDF](../../../pdf/hwpx/form-002-hwpx-2020.pdf), [검증](../assets/planet6897_green_20261002/form002_stored_frame_validation.json), [Native 전쪽 TSV](../assets/planet6897_green_20261002/form002_stored_frame_native_all10.tsv), [WASM 전쪽 TSV](../assets/planet6897_green_20261002/form002_stored_frame_wasm_all10.tsv), [1쪽 review](../assets/planet6897_green_20261002/form002_stored_frame_native_p1_review.png), [8쪽 review](../assets/planet6897_green_20261002/form002_stored_frame_native_p8_review.png), [최신 Chrome SVG 확인 PNG](../assets/planet6897_green_20261002/form002_stored_frame_current_chrome.png).

### form002 기존 회귀의 의미 검사 전환

- 전10쪽 Native/fresh WASM 최저92.96827% 검증 후, 기존 `svg_snapshot::form_002_page_0` 하나를 수정했습니다. 고정 SVG 바이트 대신 독립 정본의10쪽·원본5개 표의 쪽별 소유/행·열 구조와1→2쪽의13/15번 문단 소유를 검사합니다. PDF에서 확인한 주사제형화 개발 문구와PFC/GMP 재개 문구도 보존하는지 확인합니다. 절대px·새 test 함수·새 fixture·golden 자동갱신은 없습니다.
- 실제 라우트 `regression_suite_007`, 기존 SVG 묶음7건7PASS·실패0(exit0). 이전 원문/진단 SVG는 요청 경로에 보존하고 커밋에서 제외합니다. [검증](../assets/planet6897_green_20261002/form002_semantic_validation.json). Rust test 필수 fmt·Native/WASM Clippy·workspace build/all-target Clippy·manifest·문서 링크 모두exit0입니다.76076 기존 겹침 차단과 최종 전체 검증은 계속 대기합니다.

### 76076 남은 겹침 차단 — 정확한 입력 기준 재검증 시작

- 현재 head `16b047b10`에서 기존 partition13 재실행1FAIL(exit100),76076 겹침 증가1건을 확인했습니다.진단의0-based `page=5`, 즉 물리6쪽 본문 표의 글줄과 footer 쪽번호가 겹칩니다. 아직 회귀 제외·baseline 기대값 변경은 하지 않았습니다.
- `samples/issue1891/76076_regulatory_analysis.hwpx`는 실제 OLE HWP5·2018 저장본·82쪽이며 SHA `49bbcc49…`입니다. 기존2020 PDF 대응 기록(#4764)의 원문 `samples/76076_regulatory_analysis.hwp` SHA `3308ba85…`와 다르므로 정확한 입력의 정본을 새로 산출했습니다. 변환용 `.hwp` 사본은 원문과 바이트 동일하며 output 안에서만 사용했습니다.
- 실제 한컴2020 MCP job `fc7c5c7b-5997-43d1-8759-f0f8e642a29b`,38초 성공,82쪽638,575B,Creator Hwp2020·Producer Hancom PDF1.3.0.550. [정본 PDF](../../../pdf/issue1891/76076-exact-input-hwp2020-20261004.pdf), [입력·실패 분석](../assets/planet6897_green_20261002/76076_exact_reference_analysis.json). Native 전82쪽 TSV를 전체 페이지 범위로 산출 중입니다. 시각 결과 확인 후 이 문서에 한정해 보정 또는#7445 이관 여부를 판단합니다.
- `upstream/devel` 재확인 `6b3faf77d8085441f9f26d88d65a49791e910352`, 현재 분기에서 누락된 devel 커밋0입니다. PR 생성·전체 최종 검증은 계속 보류합니다.

### 원 PR 최신 head·CI 재확인 — 통합 검증과 구분

- 2026-10-04 API 재조회: 원 PR은 OPEN, head `d0c18feef2e9a13403482d7b8663a66a422bfaa4`로 기존 접수 기록과 같습니다. 원본 저장소의 해당 SHA check 35건은 skipped 4건, success 31건이며 실패·진행 중인 check는 없습니다.
- 현재 통합 후보 `a73100f16`에서 form002는 Native/fresh WASM 전10쪽 최저92.96827%와 기존 관련42건·SVG 묶음7건의 통과를 확인했습니다. 원 PR CI 통과를 통합 후보 전체 통과로 대체하지 않습니다. 76076 실제 물리6쪽의 본문/쪽번호 겹침, 다른 시각 보류 및 최종 전체 회귀·Skia 검증이 남아 있어 최종 승인·PR 제출은 계속 보류합니다.
- [정확한 source SHA별 check 증적](../assets/planet6897_green_20261002/source_ci_refresh_after_form002.json). 원 PR mergeability와 통합 분기 충돌 여부는 별개이며, 원 PR의 직접 병합은 수행하지 않았습니다.

### 76076 대용량 피델리티의 제한적 이관 — 결함 해결과 구분

- 정확한 원본/한컴2020 정본82쪽으로 Native 전쪽 TSV를 완료했습니다(exit1은 시각 미달).43쪽이90%미만, 최저7쪽14.15054%, 누락0쪽입니다.5쪽72.51548%,6쪽89.26879%,7쪽14.15054% review를 직접 확인했습니다.6쪽 표 마지막 행이 쪽번호와 겹치고 정본은7쪽으로 나눕니다.7쪽 표 이어받기와 이후 본문 원점도 다릅니다.
- 대용량 실제 PR 차단 입력에 한정해 [#7445 이관 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5974222442)을 게시했습니다. API로 UTF-8 본문 일치·BOM 없음을 확인했습니다. `text_overlap_baseline` 자동 수집에서 `issue1891/76076_regulatory_analysis.hwpx` 한 입력만 보류하며 정상 검사·다른 원장·baseline 수치는 변경하지 않습니다. 원본과 새 정본 PDF는 그대로 보존합니다.
- [전82쪽 TSV](../assets/planet6897_green_20261002/76076_exact_native_all82.tsv), [분석·원본/PDF/범위 증적](../assets/planet6897_green_20261002/76076_exact_reference_analysis.json), [6쪽 review](../assets/planet6897_green_20261002/76076_exact_native_p6_review.png), [7쪽 review](../assets/planet6897_green_20261002/76076_exact_native_p7_review.png). fresh WASM 전82쪽은 미실행이므로 검증 완료로 보고하지 않습니다. 전체 Native/fresh WASM90% 이상·표 행/본문 소유·쪽번호 비겹침 확인 뒤 해당 입력을 복원합니다.
- 기존 text-overlap16개 분할을 모두 재실행하여16PASS·실패0·exit0을 확인했습니다(47.634초). 최신 라우트는 `regression_suite_025`입니다. 최종 전체 회귀·Skia·다른 시각 보류는 계속 남아 있습니다.

- 이 단계의 fmt·Native/WASM Clippy·workspace build/all-target Clippy·base 대비manifest·문서링크 검사 모두exit0입니다. production source 변경0·신규test 함수0·다른 원장 변경0이며 `.log`는output 안에만 남깁니다. 다음 단계에서 전체nextest를8threads로 실행합니다.

### 전체 회귀의 새 차단4건와 #6764 제한적 분리

- 현재 후보 `670e90c0e`의 전체 nextest는10268건실행·10264PASS·4FAIL·50skip·exit100으로 완료됐습니다(실행446.251초, 컴파일 별도). 실패는 #6764 public-table presence, #7359 page-top spacing, #6855 rewind next-page, #5701 rewound-host follower입니다. 기존 집중 검사 통과가 최종전체 통과를 대체하지 않는다는 반례가 확인됐습니다.
- #6764는Native202쪽/한컴204쪽이고 표 자체는184→185쪽에 있습니다. 같은 표가 정본186쪽에서 시작하므로 고정183쪽 전제는 현재 전체 피델리티와 맞지 않습니다. 관련183~185쪽 Native/fresh WASM TSV는 바이트 동일하고16.46720%/13.14854%/30.24961%입니다. 내용으로 대응시킨Native184/정본186도26.55007%이며 이전 표 이어받기와 새 표의 시작이 다릅니다. 이는 전체202쪽을 다시 시각 승인한 결과가 아닙니다.
- [기존 #7445 이관 댓글](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5869952956)에 두 함수가 명시돼 있음을 확인하고 [현재 실패 추가 기록](https://github.com/edwardkim/rhwp/issues/7445#issuecomment-5974346179)을 게시했습니다. 현재 실제 차단인 `repaired_public_table_keeps_its_leading_rows_inside_the_paper`와 `issue_6855_rewound_line_starts_the_next_page`만 제거합니다. 원문·PDF·다른baseline·production source는 유지합니다. 통과한 같은파일 `issue_6855_band_page_paints_nothing_below_the_paper`는 그대로 재실행1PASS·실패0입니다.
- [검증·제외 범위](../assets/planet6897_green_20261002/cbta6764_blocker_analysis.json), [Native TSV](../assets/planet6897_green_20261002/cbta6764_blocker_native.tsv), [WASM TSV](../assets/planet6897_green_20261002/cbta6764_blocker_wasm.tsv), [내용 대응 review PNG](../assets/planet6897_green_20261002/cbta6764_native184_pdf186_review.png). 원본은samples와기존#7445자산에동일SHA로보존하며 새fixture/test함수0건입니다.
- fmt·Native/WASM Clippy·workspace build/all-target Clippy 통과. 최초manifest는fmt 이후source길이에따른파생suite drift로실패했고, 다시prepare→fmtcheck→all-target Clippy→base대비manifest를수행하여모두exit0을확인했습니다. 공개댓글본문도API로일치·BOM없음을확인했습니다. 이문서의피델리티해결이나최종전체통과로세지않으며 다음은#7359와#5701을개별처리합니다.

### #5701 정확한 슬라이스 정본 및 현재 Skia 완료

- 후보 `6521fe466` Skia 필수검사: lib4109건·missing-picture2건·direct-PDF4건 모두PASS·exit0입니다. 기본 전체 회귀의 알려진 #7359/#5701 차단과 시각 보류는 별도이며 PR 준비 완료로 보고하지 않습니다.
- #5701의 기존26KB IR 슬라이스를 바이트 동일하게 한컴2020에 전달해 정본을 생성했습니다. job `a49327ba-9db6-49d4-bcb6-0d0ff3a6a7f3`,3쪽74,414B입니다. Native/fresh WASM도3쪽이므로 기존2쪽 전제는 독립 출력과 다릅니다. 그러나 전3쪽 일치율74.52520%·16.00393%·1.68954%이며 양 backend 동일합니다.
- 첫쪽 review에서 정본이 이월한 표 마지막 두 행을 Native가 한쪽에 남기고, 표 호스트의 앞/뒤 글줄과 후속 문단도 잘못 소유하는 것을 직접 확인했습니다. `dump-pages`는 Table pi7 뒤 PartialParagraph pi7 전체0..4와 FullParagraph pi8을 같은쪽에 둡니다. paint 종료의 하단 보정만으로 쪽 소유를 복구할 수 없습니다. 작은3쪽 입력은 현 브랜치 보정 대상이며 테스트 숫자만 바꾸거나 삭제하지 않습니다.
- [새 정본 PDF](../../../pdf/issue5701/1270000-202200012-slice-p76-rewound-host-2020.pdf), [분석·정확한 입력/출력 SHA](../assets/planet6897_green_20261002/slice5701_exact_reference_analysis.json), [첫쪽 review](../assets/planet6897_green_20261002/slice5701_before_native_p1_review.png). 원본 문서를 그대로 보존했으며 production/test 변경0건입니다.
- #7359 전체103쪽 Native TSV 완료:90%미만24쪽·최저79쪽17.93029%, 누락0. [전103쪽 TSV](../assets/planet6897_green_20261002/chemical7359_current_native_all103.tsv). fresh WASM 전103쪽은 미실행이며 해당 회귀의 보정/이관은 아직 결정하지 않았습니다.

### #5701 저장 글 앞 앵커의 공통 계획 — 부분 보정

- 기존 IR 슬라이스의 같은 입력을 한컴2020에서 다시 저장한 진단 사본을 만들었습니다. host pi7의 원본 되감김63298→21050은 정상 재저장에서는35440→37600의 단조 줄로 바뀌고, 표 선언 높이도 전체16행20480HU에서 첫14행17920HU로 바뀝니다. 원본 픽스처는 변경하지 않았으며 진단 사본은 회귀 fixture로 추가하지 않습니다. 재저장 진단 사본의 정확한 PDF도 새로 생성했고, 이전 정확한 입력 PDF와 전3쪽 PNG가 바이트 동일합니다.
- `ParagraphFloatPlacement::from_stored_head_host`가 실제 글 앞 제어문자와 유효 저장 줄을 검증하고 기존 공통 앵커 계산을 소비하도록 보완했습니다. entry의 통째 fit → prepare의 첫 조각 → 행 스캐너 → 확정 배치가 같은 점유 계획을 소비합니다. 진단 사본의 표 소유가16행 통째 배치에서 독립 정본과 같은14행/2행으로 바뀌었습니다.
- 최초 적용에서 정상 대조군#6797의7쪽이98.05666→65.34522%로 악화됐습니다. 글 앞 앵커에 문단 앞 간격과 글 끝 남은 폭을 적용한 오류였습니다. 저장 앞 앵커의 원점은 앞 간격 전 문단 시작으로 잡고, 폭에 따른 후행 흐름 전환은 실제 글 끝 제어문자만 소비하도록 수정했습니다. 재검증7쪽98.05666%, Native 전11쪽 렌더 트리가 기존 검증 출력과 바이트 동일하며 관련 기존39검사39PASS입니다. 이 음성 결과와 보정을 숨기지 않고 원장에 남깁니다.
- 진단 사본 전3쪽 Native 일치율76.98390%·27.45380%·100%입니다. 표 앞에 있어야 할 후속 pi8 첫 줄 소유와 첫 조각 위 바깥여백이 남아 있습니다. 원본 슬라이스의 렌더 트리도 이전 출력과 동일하므로 이번 결과를 원본의 해결로 보고하지 않습니다. 기존2쪽 assertion·픽셀 허용치·test함수/fixture 추가·회귀 삭제0건입니다. 이3쪽 문서는 현 브랜치 보정 대상으로 유지합니다.
- [부분 보정 원장](../assets/planet6897_green_20261002/slice5701_head_plan_validation.json), [Native 전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_head_plan_native_all3.tsv), [정확한 재저장 진단 사본](../assets/planet6897_green_20261002/slice5701_hancom_resaved_diagnostic.hwp), [그 입력의 정본 PDF](../../../pdf/issue5701/1270000-202200012-slice-p76-hancom-resaved-2020.pdf), [현재 첫쪽 review](../assets/planet6897_green_20261002/slice5701_head_plan_native_p1_review.png). 최종 전체 nextest·Skia와 다른 문서의 시각 보류는 별도 미완료입니다.

- 현재 보정 v2의 기존39검사·fmt·Native/WASM Clippy·workspace build·all-target Clippy·고정base manifest·fresh WASM은 모두exit0입니다. Mac 로컬 no-opt 빌드이며 Docker 최적화 검증이 아닙니다. pkg/Studio WASM SHA는 `e3958626e7b8b87a66de70557ac03bf763455091ea200f752924fa4c84d425f8`로 같습니다. [fresh WASM 전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_head_plan_wasm_all3.tsv)도76.98390%·27.45380%·100%로 Native와 같고 전3쪽 PNG도 바이트 동일합니다. 정상 대조군 fresh WASM7쪽98.05666%·exit0을 확인했으며 현재 review PNG를 직접 판독했습니다. 잔존 차이를 해결한 뒤 최종 전체 검증을 진행합니다.

### #5701 여러 저장 호스트 줄의 바깥 프레임 여백 — 부분 보정

- 사양 표69의 바깥4방향 여백은 개체의 속성입니다. 기존 `column_rowbreak_fragment_opens_outer_top`의 한 저장 줄 전제 때문에 같은 유효 글 앞 앵커의 여러 줄에서는 위 바깥여백을 누락했습니다. 앵커 생성의 저장 줄 판정을 `stored_host_lines_are_valid`로 공유하고, 첫 조각 prepare·이어받기 행 예산·실제 table_partial 출력이 같은 여백 개방 결과를 소비하도록 수정했습니다. 원본 되감김·편집 줄과 표 위에서 끝나지 않는 호스트는 이 새 경로가 아닙니다.
- Native 진단 사본 전3쪽은96.60598%·18.71808%·100%입니다.1쪽 표 괘선·내용 및2쪽 이어받기 표 위치는 정본과 겹치지만, 후속 pi8 첫 줄을 여전히2쪽에 가져와 이후 본문이 밀립니다.2쪽 수치 악화도 기록하며 1쪽90% 통과만으로 내용 소속까지 해결했다고 판정하지 않습니다. 기존39검사는39PASS·exit0이고, 정상#6797 전11쪽·사용자 요청 form002 전10쪽·원본#5701 전3쪽 Native 렌더 트리는 직전 검증 출력과 모두 바이트 동일합니다. 기존 회귀를 삭제하거나 기대값을 변경하지 않았습니다.
- [전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_outer_top_native_all3.tsv), [1쪽 review](../assets/planet6897_green_20261002/slice5701_outer_top_native_p1_review.png), [2쪽 review](../assets/planet6897_green_20261002/slice5701_outer_top_native_p2_review.png), [여백 계약·검증 원장](../assets/planet6897_green_20261002/slice5701_outer_top_validation.json). 새 fixture/test함수0건입니다. fresh WASM과 필수 lint 완료 뒤 이 단계를 확정하며, 후속 첫 줄 소유 보정은 다음 단계입니다.

- 이 여백 보정의 fmt·Native/WASM Clippy·workspace build·all-target Clippy·고정base manifest·fresh WASM은 모두exit0입니다. Mac 로컬 no-opt이며 pkg/Studio SHA `a434b2fe690ccf0eb9c58f27daef83e1f7f8d96eb159ca6483e98ec1167911b8`가 같습니다. [fresh WASM 전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_outer_top_wasm_all3.tsv)는96.60598%·18.71808%·100%로 Native와 같고 전3쪽 PNG도 바이트 동일합니다. 정상 #6797 전11쪽과 form002 전10쪽 fresh WASM 렌더 트리·raw SVG도 각각 직전 검증 출력과 바이트 동일합니다. 원본/기존회귀는 유지하며 첫 줄 소유 문제와 최종 전체검증은 다음 단계에서 계속합니다.

### #5701 표 위 후속 첫 줄과 이어받기 아래여백 보정

- 후속 문단 전체만 선행 배치하던 상태에는 부분 소비한 줄 컷이 없었습니다. 저장 되감김 전 줄이 현재 실제 단 너비의 줄 구성과 같은 원점이고 표 상단 전에서 끝나는 경우에만 `ParagraphFragment`와 끝 컷을 함께 확정합니다. state가 같은 조각을 적용하고 다음 쪽은 그 컷 뒤부터 시작합니다. whole-fit은 이미 소비한 줄을 다시 그리지 않으며 첫 이어받기 예산은 현재 표가 차지한 공간을 뺍니다.
- 최초 줄 소속 보정은98.97935%·87.24739%·100%였습니다.2쪽의 남은3.8px 차이는 문단 첫 줄이 아니라 실제 이어받을 줄이 표의 아래 바깥여백을 닫는 데서 발생했습니다. 같은 시작 컷·현재 행 높이로 마지막 행 수용 예산과 재스캔을 결정하고, emit의 `occupied_bottom`·`NextLine`을 실제 terminal 흐름이 소비하도록 수정했습니다.
- 최신 source의 Native/fresh WASM 전3쪽은98.97935%·97.81243%·100%, 양쪽exit0·전3쪽 PNG 바이트 동일입니다.1쪽 host4줄·후속 첫1줄·표14행,2쪽 표2행·후속 나머지4줄,3쪽 마지막 `하였음`을 직접 판독했습니다. 유효 한컴 재저장 입력과 정확한 독립 PDF의 결과이며, 원본 수동IR 슬라이스의 개선으로 바꾸어 보고하지 않습니다.
- 기존 관련40검사40PASS·exit0. fmt·Native/WASM Clippy·workspace build·all-target Clippy·고정base manifest·fresh WASM 모두exit0입니다. 실제 단 너비를 쓰는 최종 source에서 다시 확인했으며 pkg/Studio SHA `d23cc9bee193ac0113e8c698513485e5c9ec39594e1f5b77027ab5475ddbd14a`가 같습니다. Mac 로컬 no-opt 검증입니다. 정상#6797 전11쪽 렌더 트리는 그대로이며 form002 전9쪽은 같고10쪽의 빈 문단2개의 줄/빈run4노드만 이동했습니다. form00210쪽은 새 Native/fresh WASM PNG가 이전 검증 PNG와 바이트 동일하고98.34023%입니다. 빈 문단의 물리 점유를0으로 취급하지 않으며 최종 전체 흐름 검증은 별도로 수행합니다.
- [Native 전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_prefix_native_all3.tsv), [fresh WASM 전3쪽 TSV](../assets/planet6897_green_20261002/slice5701_prefix_wasm_all3.tsv), [1쪽 review](../assets/planet6897_green_20261002/slice5701_prefix_native_p1_review.png), [2쪽 review](../assets/planet6897_green_20261002/slice5701_prefix_native_p2_review.png), [3쪽 review](../assets/planet6897_green_20261002/slice5701_prefix_native_p3_review.png), [소비 경로·정확한SHA·게이트 원장](../assets/planet6897_green_20261002/slice5701_prefix_validation.json). 신규/변경/삭제 회귀 함수0건입니다. 정상화된 기존 픽스처와 독립3쪽/내용 소속에 맞춘 기존 검사의 교정은 다음 단계이며, 최종 전체 검증과 PR 준비는 아직 완료하지 않았습니다.

### #5701 기존 픽스처 정상화와 의미 회귀 교정

- 기존 수동 IR 추출본은 원문 쪽의 저장 좌표를 남겼고, 독립 한컴 PDF도 기존 검사 전제인 2쪽이 아닌 3쪽입니다. 같은 내용을 한컴2020에서 정상 재저장한 입력으로 기존 픽스처를 교체했습니다. [이전 IR 원본](../assets/planet6897_green_20261002/slice5701_original_ir_before_hancom_resave.hwp)을 바이트 동일하게 보존하고, 두 입력의 독립 PDF도 유지합니다. 원본 IR 입력의 피델리티 해결로 보고하지 않습니다.
- 교체 파일의 SHA `86dcf229…`는 전3쪽 Native/fresh WASM 검증에 사용한 재저장 진단 사본과 같습니다. 각 쪽 98.97935%·97.81243%·100%와 직접 판독 근거를 재사용합니다. 새 회귀 함수·새 회귀 픽스처는 없으며 기존 검사 한 개를 수정했습니다.
- 고정 2쪽·후속 문단 전체가 표 아래라는 전제를 독립 정본의 3쪽, 표 14행/2행, 후속 첫 줄 1쪽/나머지4줄 2쪽의 소속 검사로 교정했습니다. 문단·모든 표 칸의 원문 순서/누락/중복, 앞뒤 상자 비겹침과 본문 안쪽 포함도 검사합니다. 자동 불릿은 원본 스타일의 문자로 함께 검증하며 절대 픽셀 핀은 없습니다.
- 최종 같은 검사 소스와 픽스처로 보정 전 `519127d09`는 후속 첫 줄 0/1 때문에 1FAIL·exit100, 현재 `518d99009`는 관련 41건 41PASS·exit0입니다. fmt·Native/WASM Clippy·workspace build·all-target Clippy·고정 base manifest 모두 exit0입니다. 최초 파생 suite drift와 공유 target 재사용 오류는 실행 근거에서 제외하고, 재준비/명시적 재컴파일 뒤 결과만 수용했습니다.
- [정확한 입력·검사 해시와 전후 검증](../assets/planet6897_green_20261002/slice5701_regression_correction_validation.json). 최종 전체 회귀·Skia 및 다른 문서의 시각 보류는 미완료이며 통합 PR 준비 완료로 판정하지 않습니다.

### #5701 교정 후 전체 회귀 — 13건 차단 확인

- 검증 head `e580fa109a800ce3d5ba84b0d668981037cb0f8e`, base `6b3faf77d8085441f9f26d88d65a49791e910352`. 전체 nextest를 locked/release-test/shared target/8threads/no-fail-fast로 완료했습니다. 10,266건 실행·10,253PASS·13FAIL·50skip·exit100, 실행405.402초·컴파일2분54초입니다. 실행 중 소스 변경은 없습니다.
- 교정한 #5701 내용/쪽 소속 검사는 PASS입니다. 기존 #7359 캡션 간격 실패는 남고, #6761 표 뒤 간격·synam001 호스트 제목 간격·#6267 호스트 겹침·oracle partition6의 작은 exclusion probe 2→3쪽 및 text-overlap 8개 분할의 신규 겹침이 검출됐습니다. 집중40/41PASS나 정상 대조군의 시각 통과를 전체 무회귀로 일반화할 수 없습니다.
- [최종 summary·13개 함수·신규 겹침 입력](../assets/planet6897_green_20261002/whole_after_slice5701_validation.json). 새 실패는 정상 원본과 독립 PDF로 재검토하고, 작은 문서는 현 브랜치에서 보정합니다. 임계값 완화·일괄 회귀 삭제·새 회귀 함수 추가는 하지 않았습니다. 최종 승인/PR 생성은 계속 보류하며 다음은 작은 exclusion probe 쪽수 실패를 분석합니다.

### #1789 표 위 잉크와 표 뒤 빈 줄 점유 보정

- 정상 HWPX `samples/task1789/exclusion_probe_line_spacing.hwpx`와 독립 한컴2020 정본은 2쪽입니다. 전체 회귀의 3쪽 실패는 첫 글줄 뒤 간격까지 배제 프로브에 포함해 위원구성 줄을 표 아래로 밀었기 때문입니다. 프로브는 잉크 높이를 사용하고 뒤 간격은 기존 순차 흐름에서 소비하도록 보정했습니다.
- 첫 Table의 호스트 LINE_SEG를 단 원점으로 삼으면 측정과 출력의 기준이 갈라져 표 뒤 빈 줄의 16px 공간을 표 안에서 소비했습니다. 실제 본문 흐름으로 원점을 역산하고, 재조판 글줄도 표 예약에 사용한 공통 원점을 전달합니다. 문서 ID/수치 예외, 좌표 clamp, 빈 줄 삭제는 없습니다.
- 기존 #1789 검사 한 건의 절대 픽셀 핀을 독립 정본의 2쪽, 위원구성은 표 위, 빈 줄·회의내용·다섯 항목은 표 뒤, 행정사항·첨부·결재는 둘째쪽이라는 관계 검사로 교정했습니다. 본문 누락·중복도 원문과 대조합니다. 새 회귀 함수·픽스처·회귀 삭제·기준 PDF 변경0건입니다.
- 같은 교정 검사로 수정 전 `9f9845afe`는 3/2쪽 1FAIL·exit100, 수정 후 관련65건65PASS·exit0입니다. 첫 보정의 #6950 재조판 대조군 실패는 공통 호스트 원점 전달로 해결했습니다. fmt·Native/WASM Clippy·workspace build·all-target Clippy·고정 base manifest 모두 exit0입니다.
- 최종 Native/fresh WASM 전2쪽은 96.54137%·94.44785%, 미달0이며 실제 PNG/트리도 전쪽 바이트 동일합니다. Mac no-opt 로컬 빌드이고 Docker 최적화 검증은 아닙니다. 정상 form002 전10쪽·#6797 전11쪽·#5701 전3쪽의 Native/fresh WASM 트리는 이전 독립 PDF 검증 출력과 모두 같습니다.
- [검증·정확한 SHA·소비 경로](../assets/planet6897_green_20261002/probe1789_origin_validation.json), [Native 전2쪽 TSV](../assets/planet6897_green_20261002/probe1789_origin_native_all2.tsv), [fresh WASM 전2쪽 TSV](../assets/planet6897_green_20261002/probe1789_origin_wasm_all2.tsv), [1쪽 review](../assets/planet6897_green_20261002/probe1789_origin_native_p1_review.png), [2쪽 review](../assets/planet6897_green_20261002/probe1789_origin_native_p2_review.png). 이 보정 완료 후 전체 nextest를 재실행합니다. 다른 시각 보류·최종 Skia·PR/CI/후속처리는 아직 미완료이며 통합 승인 보류입니다.
