# 확장한 인라인 호스트 프레임과 뒤 본문의 공간 검사

## 원 해결과 독립 근거

전체 Rust `3991`에서 기존 `issue_6448_tac_cell_uses_measured_band_and_moves_tail_to_page3`가
3쪽 대신 2쪽으로 실패했다. 원 이슈 #6448, PR #6472와 통합 PR #6480을 다시 대조했다.
실제 통합 `19b89d967b1505cd4bdcdbba7d1f1413f32a1505`는 넓은 declared-fit 특례를
복원하지 않고, 독립 Print의 HEAD / 표 / TAIL 3쪽 흐름을 보호한다. 이후 #8191의
`6097253468`은 셀 글자 높이 캐시의 정상/불일치 구분을 추가했다. 기대값을 완화하지 않는다.

같은 `samples/issue6448/tac_cell_leftover_fits.hwpx`와
`pdf/tac_cell_leftover_fits-2020.pdf`를 고정 base
`238de6de41a208648f4ade6b494fe433a81c1fce`와 후보
`20e49891adf822c320ee2ddb3a3f40ff3bdf6e0f`에서 대조했다. base Native247은
전 3쪽 최저 raw 99.04290%, fresh WASM070은 99.30014%다. 후보 Native248과
fresh WASM071은 모두 2쪽이어서 전쪽 TSV를 산출할 수 없다. 두 후보 제품은 삭제한
실행 불가능 분기 이전 runtime이므로 진단 제품으로 구분하며 최종 head 제품으로 표기하지 않는다.
입력/PDF SHA, 실제 명령과 로그는 ignored `output/pr-review/semanticist21-20261019/`의
`6448-base-candidate-3992-result.json`에 있다.

## 생산 결과와 실제 소비

`inline_box_rows::normalized_host → rendering의 inline_frames 투영 →
RenderNormalizationOverlay::register_rendered_text_partitions → TAC 표 배치 →
뒤 일반 문단 fit`을 추적했다. 원본 host 높이 1,000HU가 실제 표 높이 68,864HU로 바뀌고
뒤 문단의 원점도 같은 증가분을 소비한다. 이 자체는 실제 표 공간을 보존하는 보정이다.
그러나 기존의 짧은 host 판정은 투영된 큰 줄을 정상 저장 프레임으로 오인하여 거짓이 된다.
결과적으로 표 뒤의 strict fit이 사라지고, 높이 933.5733px인 본문에
947.52px를 수용한다. 원본의 높이 누락이라는 출처를 투영된 숫자만으로 다시 추정한 것이 원인이다.

원본 저장 줄이 단일 TAC 표의 선언 상자를 담지 못했고 실제 투영 줄이 커진 경우를
overlay에 원본·투영 포인터 모두로 보존한다. 문자 재줄바꿈 여부와 구분하며, 표의
셀 내용·물리 컷을 무조건 dirty로 만들지 않는다. 실제 TAC 배치가 이 출처를 소비해
바로 뒤 본문의 전체 줄 전진을 검사한다. 문서 번호·행 수·목표 화면 좌표의 분기는 추가하지 않는다.
정상 저장 줄과 확장이 없는 표는 기존 경로를 유지한다.

## 검증 순서

현재 실행 중인 `3991`의 검증 worktree와 source는 고정한 채 끝까지 실행한다.
수정 후 기존 #6448의 수정 전 FAIL / 수정 후 PASS, 정상 인라인·저장 줄·공동 앵커
대조군과 같은 입력의 전쪽 Native/fresh WASM·직접 판독을 확인한다. 새 입력이나 골든을
추가하지 않으며, 그 뒤 변경 범위의 필수 lint·전체 회귀를 완료한다. sample16·편람의
삭제된 원본·검사·파생 참조는 재생성하지 않는다. 아직 보정 성공이나 PR 준비 완료를 선언하지 않는다.

## 공유 target의 차트 의존 산출물 불일치

같은 전체 실행에서 OOXML 차트 4검사가 추가로 실패했다. 소스의 제목은
`dominant-baseline="text-before-edge"`이고 기본 frame은 RGB134·1px인데, 실제 실행은
이 속성이 없고 RGB140·0.75px를 출력했다. 현재 worktree와 제출 checkout의
`crates/rhwp-ooxml-chart/src/renderer.rs` SHA-256은 모두
`757959ae78ca2a9d7dd8b7ebae6e2c2dd9f6bf40be845f887adf8d86574f8fb6`이다.

실제 공유 target의 `librhwp_ooxml_chart-8de1c9d4ed8a0220.rlib`에는
`text-before-edge`가 없고 옛 테두리 문자열이 있으며 dep-info에도 현재의
`cartesian_layout.rs`가 없다. 원본 파일 mtime은 10월 25일 00:55, 해당 artifact는
16:53이다. 다른 base worktree에서 만든 의존 산출물이 과거 mtime을 보존한 후보
source보다 새로워 재사용된 정황과 실제 binary 불일치를 확인했다. 네 실패를
현재 차트 기대값의 오류로 단정하거나 assertion을 완화하지 않는다.

실행 중인 전체 검증이 끝난 뒤 실제 후보 workspace의 source anchor mtime을 갱신해
같은 `target/pr-review`에서 해당 로컬 crate와 소비자를 다시 빌드한다. 파일 내용과
Git diff는 바꾸지 않고 target·deps를 삭제하지 않는다. 재빌드 로그의 실제 crate와
새 artifact의 문자열·dep-info, 같은 4검사의 실행 결과를 확인한다. Native/fresh WASM
최종 제품도 같은 source 출처를 확인하며 이전 제품을 새 head 제품으로 바꾸어 쓰지 않는다.
`ooxml-stale-dependency-3997.json`에 source/artifact SHA와 관측을 보존했다.

#6448의 같은 문서는 `body_overflow_baseline` partition 9에서도 신규 1건을 만들었다.
이것은 별도 원본 실패가 아니라 위 2쪽 합침의 뒤 본문 넘침이다. 원장에 1을 더하지 않고
구현 보정 뒤 이 입력의 overflow 0과 전쪽 Print 대조를 함께 확인한다.

## 첫 재빌드 결과와 빠른 소비 경로 보정

`7bb8b13271`의 실제 workspace source anchor mtime만 갱신한 뒤 Native249를
같은 공용 target·잡 8개로 빌드했다. 276.21초·exit 0이며 네 로컬 crate의 실제 재컴파일을
확인했다. 차트 artifact에는 현재 `text-before-edge`가 있고 dep-info에도 현재
`cartesian_layout.rs`가 있다. 같은 focused harness에서 차트의 기존 10실패를 포함해
69 PASS·1 FAIL이다. 기대값은 변경하지 않았다.

남은 #6448은 여전히 2쪽이었다. 추가한 overlay 출처는 일반 `place_table_with_text`의
strict fit에 전달했지만 그 앞의 `physical_tac::prepare → commit_stored_tac_control → return`
단축을 놓쳤다. 새로 커진 줄이 이 단축에서 원본 저장 프레임으로 승인되어 strict fit을
우회했다. 이 소비 누락을 보정하여 같은 출처를 physical TAC의 승인 조건에도 전달한다.
원본 저장 줄과 투영 줄의 소유를 구분하며 정상 저장 프레임의 단축은 유지한다.

전체 `3991` 결과는 11,690 PASS·17 FAIL·45 SKIP, exit 100이다. 첫 재빌드의 focused
성공을 전체 회귀 성공으로 확대하지 않는다. 로그·source/artifact SHA와 실패 가족별 분류는
`expanded-inline-host-4004-result.json`, `full-rust-failure-families-4003.json`에 있다.
새 단축 보정의 검증은 아직 남아 있다.

Native250도 같은 70검사에서 69 PASS·#6448 1 FAIL이었다. raw parser와 Core의
편집 IR은 모두 host 1,000HU·표 68,000HU를 보존하므로, IR을 이미 덮어썼다는 가설은
실제 읽기 전용 probe로 기각했다. `commit_stored_tac_control`의 전체 호출을 다시 추적하면
TAC 함수 내부 이전에 `try_place_stored_tac_paragraph`의 computed/stored 두 계획과
`flow_table::prepare_coanchored_first_line`도 조기 반환한다. 이 세 수용도 같은 확장 출처를
제외해야 실제 공통 배치에 도달한다. `physical_tac`와 일반 배치의 출처 소비를 포함해
전체 저장 TAC commit 지점을 대조했으며, 이번 두 입구에도 동일 overlay 조건을 전달한다.
실제 재검증 전 성공으로 보고하지 않는다. probe와 실행은
`6448-source-frame-4009-result.json`, `expanded-inline-host-focused-4006-result.json`에 있다.

## 모든 저장 우회 경로 보정 후 Native251 확인

`bdea9b6e9d`의 추가 가드와 `addcac4560`의 공통 접두 프레임 보정을 함께 빌드했다.
4011의 기존 70건은 70 PASS / 0 FAIL이며 #6448의 기존 3쪽 검사가 통과한다.
4012의 동일 원본·독립 Print는 3/3쪽, 전쪽 Native 최저 raw 99.05213%다.
기대값·쪽수·기준값을 바꾸지 않았다. fresh WASM과 최종 필수 검증은 계속 진행한다.
