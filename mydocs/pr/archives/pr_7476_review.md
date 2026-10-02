---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-10-02
---

# PR #7476 검토 — 수정: #7418 재조판 줄 채움(condense·목록 마커·구두점 폭)과 host 글·칸 조각 기하를 한/글에 맞춤

## 최종 판정

**머지 보류.** 원 PR 최신 head의 CI는 green이지만 통합 코드·직접 시각 검증은 아직 실행 전입니다. `review/planet6897-green-20261002`에서 최신 devel `e5098bc91be44a49367a7f2895a14fcd4f4c2c7f` 위에 순차 체리픽해 검토합니다.

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

체리픽·충돌 보정·최종 검증은 아직 미실행입니다. 분석·코드·결과 보고·commit 순서로 단계별 갱신합니다.
