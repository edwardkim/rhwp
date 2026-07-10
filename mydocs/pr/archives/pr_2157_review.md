# PR #2157 검토 - Issue #2151 HWP3 pgy=0 그림 유령 페이지

- PR: https://github.com/edwardkim/rhwp/pull/2157
- 작성자: `planet6897`
- base: `devel`
- 원 head: `a36dd82cb1858872dc2bdac43e3c96799d0743f8`
- 체리픽 커밋: `86a7ffaa5`
- 포함 README: 없음

## 결론

blocking finding 없음. HWP3 그림 호스트 문단이 `pgy=0`으로 저장된 뒤 다음 문단에서
거짓 쪽 경계가 생기는 문제를 `prev_last_pgy` 리셋 조건 보강으로 막는 변경이며,
샘플 2종의 페이지 수가 HWP 2020 PDF 기준과 일치한다.

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `target/release-test/rhwp dump-pages samples/hwp3-sample14.hwp --page 0`
  - rhwp: 11쪽
- `target/release-test/rhwp dump-pages samples/hwp3-sample11.hwp --page 0`
  - rhwp: 151쪽
- MCP 기준 PDF 생성:
  - `pdf/hwp3/hwp3-sample14-2020.pdf`
    - sha256 `306bdbf044eee6352bb2f430650697609ad09e8a8263076588fb34444a47d377`
    - `pdfinfo`: 11쪽
  - `pdf/hwp3/hwp3-sample11-2020.pdf`
    - sha256 `9706818d6e12d7c6190633bf66881ec337897f374e5fbfd79492083489082a5b`
    - `pdfinfo`: 151쪽
- focused test: `issue_2151_hwp3_ghost_page` pass
- 기존 HWP3 회귀:
  - `issue_554` HWP3/HWP5/HWPX samples pass
  - `issue_929` HWP3 samples pass
- 전체 검증:
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 변경은 HWP3 parser의 페이지 경계 추론에 영향을 준다. 이번 검증에서는 기존 HWP3 regression suite를
  함께 돌려 무회귀를 확인했다.

## 권고

누적 체리픽 PR에 포함하고 admin merge 가능하다.
