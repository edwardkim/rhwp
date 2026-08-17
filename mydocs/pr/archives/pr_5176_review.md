# PR #5176 검토 - HWP5 캡션 공통속성 bit29 보정

- PR: https://github.com/edwardkim/rhwp/pull/5176
- 작성자: `planet6897`
- base: `devel`
- 원 head: `9f98942a0f724628e649725e11e6ddf88bbea21b`
- 누적 검토 브랜치: `review/planet6897-hwp-contracts-20260818`
- 체리픽 커밋: `e8c9fb1cb`, `cdbdf039b`, `681704f3d`
- 제외 커밋: `eb17b9c1e329` (동일 overflow-cell baseline이 `upstream/devel`에 이미 존재)

## 결론

blocking finding 없음. 표, 그림, GSO 도형의 캡션 유무와 공통속성 `attr bit29`를
직렬화 직전에 일치시켜 CTRL_HEADER 뒤 캡션 레코드 해석이 어긋나지 않도록 한다.
테스트를 `tests/cases` 입력으로 이동해 파생 suite 정책도 따른다.

## 검증

- 체리픽 충돌 없음
- focused: `issue_5136_caption_attr_bit29` 4 passed
  - 캡션 있는/없는 표와 GSO 각각의 bit29 상태를 확인
- 누적 전체 Rust 회귀: 6,735 passed, 38 skipped, 3 slow
- 구조 확인: `git diff --check upstream/devel...HEAD` pass

## Fixture와 시각 증적

- 관련 fixture: `samples/hwp3-table-caption.hwp`
- 변경은 HWP5 레코드 구조·개방 안전성 보정이며 renderer 외관 변경은 아니다.
  이 검토에서는 PDF를 새로 만들지 않았고, 실제 한글 앱의 개방성은 별도 Windows/MCP
  증적이 필요할 때 보강한다.

## 리스크와 권고

실제 한글 2020/2022 개방 실행은 이 macOS 회귀 범위에 포함되지 않는다. 다만 bit29
양방향 계약과 전체 회귀가 통과했으므로 누적 통합 PR 후보에 포함할 수 있다.
