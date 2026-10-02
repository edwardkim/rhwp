---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7476 검토 — 수정: #7418 재조판 줄 채움(condense·목록 마커·구두점 폭)과 host 글·칸 조각 기하를 한/글에 맞춤

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 체리픽은 완료했고 통합 검증 중입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

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
