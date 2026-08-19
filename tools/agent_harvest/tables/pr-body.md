> **PR base 브랜치가 `devel` 인지 확인해주세요** (`main` 아님).

## 변경 요약

`rhwp-agent`로 `samples/` 실제 문서를 열어 **표 격자 실측** 봉투를 남긴다.
본 CLI(`src/main.rs`)는 건드리지 않는다. 편집 로직을 만들지 않는다.

작업 문서: [tools/agent_harvest/tables/WORKING.md](tools/agent_harvest/tables/WORKING.md)

명령: tables·table-inspect·table-csv·merged-tables

값은 전부 실제 stdout 이다. 고정 표본 `form-01.hwp` · `hwp3-sample.hwp` · `hwp_table_test.hwp` 재실행은 `test_replay.py` 다.

## 테스트

- [x] **`cargo fmt --all -- --check` 통과**
- [x] `python tools/agent_harvest/tables/test_replay.py`
- [ ] `cargo clippy -- -D warnings`
- [ ] 샘플 SVG — N/A (조회 실측, 렌더 변경 없음)
