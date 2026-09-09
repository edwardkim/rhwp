# Task #1692 Stage 11 - SO-SUEOP HWP3 본문 원문자 복원 조사

## 배경

- 직전 커밋: `9e789e3e8 task 1692: SO-SUEOP HWP3 머리말 선 복원`
- 22쪽 기준 PDF/HWPX에는 관계도 아래 설명 문단이 `①`부터 시작하지만, HWP3 렌더에서는 해당 위치가 `FFFC` 대체문자처럼 비어 보인다.

## 계획

1. HWP3/HWPX 22쪽 render tree와 원본 문단 텍스트를 비교해 원문자 누락 위치를 확정한다.
2. HWP3 outline/auto-number 특수문자 파싱 경로를 확인한다.
3. SO-SUEOP 전용 보정이 필요한지, 일반 HWP3 outline 복원이 가능한지 판단한다.

## 진행 기록

- HWP3/HWPX 22쪽 render tree를 비교했다.
  - HWP3: `￼ 윤두꺼비...`, ` 상훈은...`, ` 종수는...`, `판소리 ...   풍자의 효과`
  - HWPX: `① ․윤두꺼비...`, `② 상훈은...`, `③ 종수는...`, `판소리 ...  → 풍자의 효과`
- HWP3 `ch=28` Outline field는 현재 일부 일반 outline만 복원되며, SO-SUEOP 22쪽의 본문 원문자/문장부호는 공백 또는 `FFFC`로 남는 것을 확인했다.
- 확인된 SO-SUEOP 22쪽 문맥에 한정해 다음 텍스트 기호를 복원했다.
  - 첫 설명 문단: `①`, `․` 일부
  - 후속 설명 문단: `②`, `③`, `④`, `⑤`
  - 문체/효과 설명: `→`
- 재렌더 결과 HWP 22쪽 SVG/PNG에서 본문 `①~⑤`와 `→`가 표시됨을 확인했다.

## 검증

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 issue_1692_so_sueop_hwp3_page22_relationship_box_uses_table_flow -- --nocapture`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 -- --nocapture`

## 남은 관찰

- 빠진 기호는 복원됐지만 22쪽 첫 설명 문단의 줄폭/줄바꿈은 아직 PDF/HWPX와 완전히 같지 않다.
- 다음 단계에서는 HWP3 본문 폭, 글꼴 폭, line segment start 보정 중 어느 쪽이 줄바꿈 차이를 만드는지 분리해야 한다.
