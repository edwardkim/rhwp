# PR #3472 검토 — 자동번호가 뒤 컨트롤의 갭을 훔치던 어순 파괴 정정

Issue: #3466 / author: planet6897 / reviewer: edwardkim / milestone: v1.0.0
연작: planet6897 7건 누적 검토(2026-07-28), 체리픽 1순위

## metadata (작성 시점 참고값)

| 항목 | 값 |
|---|---|
| head | `5889572775` (기능 커밋 `009f26bef` → 누적 `91fd2e8be`) |
| 규모 | +126 -0 (`src/model/paragraph.rs` +25, 테스트 +101) |
| CI | 원 PR head SUCCESS, mergeable CLEAN |

## 변경

자동번호(제어문자 0x12)는 8 코드 유닛을 점유하며 가시 placeholder 한 글자를 남기는데,
`control_text_positions()` 가 갭을 컨트롤에 순서대로 배분해 자동번호가 다음 컨트롤(수식)의
갭을 가져갔다. 결과는 표시 흔들림이 아니라 **읽는 뜻이 달라지는 어순 파괴**다.

수정은 자동번호 placeholder 를 갭이 아니라 자기 글자 위치로 잡는다. 판별자 셋을 모두
요구한다 — stride==8, placeholder 공백, 대기 컨트롤이 AutoNumber/NewNumber. **탭이 같은
stride signature 라 stride 만으로 판정하면 반대 방향으로 깨진다**는 함정을 컨트리뷰터가
수정 중 잡아 가드 테스트로 고정했다. HWPX `newNum`(순수 8 갭)은 조건 3 으로 종전 경로 유지.

## 검증

- focused 4건: 제보 최소 재현 `[5,9]→[0,5]`, 수식 2개 누적 무밀림, 무자동번호 무회귀, 탭 가드
- **red-check 실증**: `src/model/paragraph.rs` 만 devel 로 되돌리면 수정 대상 2건만 FAILED,
  가드 2건은 양쪽 통과 — 테스트가 결함을 실제로 검출한다
- 누적 branch 전체 게이트: release-test **4253 passed / 0 failed**, svg_snapshot golden
  무변화, fmt·clippy 클린

## 시각 판정

`control_text_positions()` 는 렌더가 쓰는 함수지만 이 수정은 잘못 배정된 컨트롤 위치를
문서 어순대로 되돌리는 것이고, golden 무변화 + 무자동번호 문단 무회귀 테스트로 파급이
차단됐다. PR 본문의 red→green 실측을 근거 기록으로 삼고 신규 sweep 은 수행하지 않는다.
제보 #3466 의 별개 증상(수식 미렌더 58건 등)은 이 PR 범위 밖이며 이슈에 남는다.

## 권고

**merge (통합 PR 경유).** 진단 정확, 함정 자체 발견·가드화, 실증 완료.
