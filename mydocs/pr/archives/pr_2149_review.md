# PR #2149 검토 - Task #2146 NO_LS 라벨 셀 행높이 선언 신뢰

- PR: https://github.com/edwardkim/rhwp/pull/2149
- 작성자: `planet6897`
- base: `devel`
- 원 head: `89bfe8a9871e745ef721b01a6f70c3cabc48afb0`
- 체리픽 커밋: `f91234b62`
- 포함 README: `samples/task2146/README.md`

## 결론

핵심 개선은 확인했다. r0 사선/Fixed-모순 NO_LS 라벨 셀의 렌더 높이를 선언
52.4px로 유지하는 테스트가 통과한다.

중요한 범위 제한도 README와 보고서에 명확하다. 이 PR만으로
`21761835_jeonjik_exemption_table.hwp` 전체 페이지 수가 한글 6쪽과 일치하지는 않는다.
rhwp는 여전히 7쪽이고, 잔여 팽창 +473px은 후속 백로그다.

## README 반영

README는 다음을 명시한다.

- 한글 2022: 6쪽
- 수정 전 rhwp: 7쪽
- 이번 수정 대상: r0 헤더 행 +26.9px 팽창 제거
- 잔여: r0 수정 후에도 rhwp 7쪽 유지, 원인은 24개 행의 이질 팽창

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `target/release-test/rhwp dump-pages samples/task2146/21761835_jeonjik_exemption_table.hwp --page 0`
  - rhwp: 7쪽
- MCP 기준 PDF 생성:
  - `pdf/task2146/21761835_jeonjik_exemption_table-2020.pdf`
  - sha256 `565b72bcf3b9149b73ea6b85df8b480280c90a794e7cad5697cc00734aed7498`
  - `pdfinfo`: 6쪽
- focused test: `issue_2146_no_ls_label_cell_declared_height` pass
- 관련 회귀:
  - `issue_1891`, `issue_1842` 포함 관련 suite pass
- 전체 검증:
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## maintainer 보정

누적 체리픽 상태에서 clippy가 `tests/issue_2146_no_ls_label_cell_declared_height.rs`의
doc list indentation(`clippy::doc_lazy_continuation`)으로 실패했다. 기능 코드는 변경하지 않고
문서 주석 continuation 들여쓰기만 보정했다.

## 리스크

- 전체 문서 페이지 수 불일치는 그대로 남는다.
- `no_ls_short_label_cell` 판정은 사선 셀 또는 Fixed 줄간격 모순 셀로 좁혀져 있다. README가
  설명하는 #1891 회귀 방지 범위와 일치한다.

## 권고

누적 체리픽 PR에 포함 가능하다. 단 이 PR을 “21761835 전체 페이지 수 해결”로 설명하면 안 되며,
r0 행높이 축 해결로만 기록해야 한다.
