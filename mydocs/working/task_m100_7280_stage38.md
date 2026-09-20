# Task #7280 Stage 38 — R2ai 어울림 꼬리 조회·종료/배치 명령 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2ah](task_m100_7280_stage37.md), 시작 head `62648b2c2`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: 구현 완료, 고정 제품 SHA 집중 검증 예정. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 실제 호출 경로

`typeset_wrap_around_paragraph`의 전폭 꼬리 및 매칭 실패 처리를 다음 경계로 나눈다.

- `controls/wrap_tail::classify_prefix`: 저장 줄 순서의 cs/sw 접두 길이와 전폭 꼬리 판정.
  `WrapPrefix`에 len/판정만 보존한다. `PageLayoutInfo`는 읽기 전용 차용이며 너비 조회는
  기존 `.all`의 단락 평가 위치에 둔다. 모든 줄의 너비를 미리 계산하지 않는다.
- `WrapPrefix::suffix_height`: **기존 문단 포맷 이후** 빈 문단/줄 개수/1:1 대응 조건을
  같은 순서로 판단하고, 수용할 경우 같은 줄 범위 전진량과 spacing_after를 반환한다.
- `wrap_tail::mismatch_starts_new_page`: 밴드 종료 뒤 저장 vpos/단 수/고정 하단 앵커 조건을
  판정한다. 페이지 전환은 하지 않는다. 원래 인덱스 guard·비교·상수·반환 의미를 보존한다.
- `state::end_following_wrap`: 네 matching 필드 초기화 → 기존 `close_square_band` 호출.
- `state::commit_wrap_tail`: PartialParagraph 추가 → 높이 전진 → vpos dirty 설정.
  페이지 전환과 fit 재판정은 하지 않는다.

실제 조정 순서는 저장 접두부 조회 → 현재 단 너비로 포맷 → 안정 형상/꼬리 높이 →
현재 가용 높이 확인 → 접두부 흡수 기록 → 밴드 종료 → 새 높이/예산 재조회 → 필요 시
쪽 전환 → 꼬리 기록이다. 가용 높이 조회에는 진단 출력이 있으므로 기존 호출 횟수와
`!current_items.is_empty()` 단락 조건을 그대로 남긴다. 밴드 종료 전후 높이를 한 snapshot으로
묶지 않는다. 전체 흡수 true/호출자 continue와 일반 문단 fallback false도 보존한다.

매칭 실패 Query의 `has_items`와 단 수는 종료 뒤 scalar로 전달한다. 원래 인덱스 guard보다
scalar 읽기가 앞서지만 부수효과·상태 변경은 없으며 이전 문단 인덱싱은 여전히 guard 뒤에 있다.
`close_square_band` 본체와 다른 호출부는 변경하지 않는다. 본문/미주 인덱스를 합치지 않는다.

기존 조건·상수·조판 정책·IR/API·Rust 테스트/assertion·baseline/golden/ignore·CI 변경은 없다.
이번 절편은 Query/Command 경계 분리이며 후속 어울림 전체 조정자의 이동/상태 읽기 캡슐화는 남는다.

## 2. 검증 계획과 한계

`output/7280/stage38/verify-wrap-tail.mjs`는 조회식·명령·인자를 원본과 대조하고,
새 호출을 원래 블록으로 펼쳐 전체 조정 함수 및 나머지 parent/state/controls 불변을 확인한다.
정적 대조는 실행 분기 커버리지 또는 직접 시각 판정이 아니다.

고정 review worktree에서 suite 준비 → baseline 대비 manifest/unit-tier → fmt → native Clippy →
집중 nextest를 순차 실행한다. 기존 324건에 `issue_2098_page_bottom_fixed_anchor_stays_on_page_1`
1건을 더해 325건을 선택한다. baseline 전수 로그의 같은 PASS 이름들과 대조한다.

기존 #4090 검사는 좌측 prefix·전폭 tail 렌더 위치/존재를, #6128은 접힌 줄과 뒤 문단의 경계를
검사한다. #2098은 합성 하단 앵커의 페이지 소유를 검사하는 대조군이며 이 입력이 반드시
매칭 실패 helper의 내부 분기를 실행한다고 주장하지 않는다. 새 분기 계측을 하지 않았으므로
모든 단 수/저장 vpos/빈 문단 조합의 실행은 미검증이다.

주 checkout의 파생 suite는 건드리지 않고 변경 Rust 파일만 rustfmt로 정리한다.
전체 fmt는 review worktree에서 suite 준비 후 검사한다. 생성 suite/manifest는 stage하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM·직접 시각 대조는
구현계획 §7의 책임 묶음/제출 전 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 3. 후속

후속 어울림 전체 조정자를 새 Query/Command 경계에 연결하고 남은 상태 읽기 경계를 정리한다.
이후 R2 책임 묶음 통합 검증 범위를 확정한다. R3 표 분할, R4 각주/미주, R5 구역·페이지 수명과
최종 상태 캡슐화는 별도 책임으로 유지한다.
