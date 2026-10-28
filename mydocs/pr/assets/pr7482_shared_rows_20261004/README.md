# PR #7482 공통 TAC 줄 배치 시각 증적

코드 source: `0373fb43c4189a138482f72ebbb5ecb0a081b4a5`. Native와 root wrapper로 생성한 fresh WASM을 동일 입력/PDF·96dpi·검증된 글꼴 공급으로 출력했다. `provenance.json`은 각 PNG와 실행 산출물 해시를 고정한다. 이 폴더는 공개 fixture의 PNG만 포함한다.

- `tac-mixed-one-row`: `samples/issue7482/tac-mixed-one-row.hwpx`, `pdf/issue7482/tac-mixed-one-row-hwpx-2020.pdf`; B/첫 표/A/둘째 표/C가 같은 줄에 순서대로 놓인다.
- `tac-mixed-explicit-break`: 같은 이름의 `samples/issue7482/` 입력과 `pdf/issue7482/`의 `-hwpx-2020.pdf`; 명시적 개행 뒤 둘째 표/C가 다음 줄에 놓인다.
- `full-width`: `samples/issue7481/synth_square_host_full_width_table_no_ls.hwp`, `pdf/synth_square_host_full_width_table_no_ls-2020.pdf`; 공개 NO_LS 합성 입력과 독립 한컴 PDF다. 빈 host의 줄 점유와 뒤 표 위치를 확인하며 Paper 장식의 잔여 차이가 있다.
- `square-follower`: `samples/issue7482/tac-after-plain-paragraphs.hwpx`, `pdf/issue7482/tac-after-plain-paragraphs-hwpx-2020.pdf`; 첫 쪽의 22개 문단이 Square 표 옆에서 이어지고 큰 TAC는 둘째 쪽에 통째로 배치된다. 수정 전 첫 줄 124.0px → 현재 91.4px, 독립 PDF ink 91.03px. 두 쪽의 새 review/overlay를 포함한다. 50% 정책 검증으로 확대하지 않는다.
- `square-columns`: `tests/fixtures/issue_6970/synth_no_ls_square_wrap.hwp` p3 / `pdf/synth_no_ls_square_wrap-2020.pdf`; 변경 없는 공개 입력과 독립 PDF의 12pt=16px 후속 줄 간격을 검사한다. 이전 c8에서 확인한 11.47px 겹침을 제거했고, 뒤 빈 문단도 16px 간격을 유지한다. raw 74.11148%와 왼쪽 단의 기존 줄바꿈 차이는 남는다.
- `distribution`: `samples/task1768/distribution_doc.hwpx` p3; 상단 목록·라벨/표/뒤 문단 순서를 확인했다. 실루엣 85.83661%와 표 왼쪽 약 7.8px 등 잔여 차이는 #7482 전용 사용자 예외로 평가하며 완전 일치를 주장하지 않는다.

입력 생성과 독립 PDF job/hash는 `samples/issue7482/{README.md,provenance.json}`, 검증 집계와 남은 범위는 `validation-summary.json`, 정식 검토는 [PR review](../../pr_7482_review.md), 상세 기록은 별도 로컬 review branch의 `mydocs/working/davindev_7482_7518_requested_verification.md` 마지막 절을 따른다. ignored 로컬 증적은 `output/pr-review/davindev-20261003/candidate-final11-{native,wasm}-visual/<key>/{run_manifest.json,pages,review,overlay}`다.

PR 본문에는 이 asset을 포함한 정확한 head SHA의 raw URL로 대표 review/overlay PNG를 표시한다. 이 폴더 준비만으로 원격 push·PR 생성·review·merge가 승인되지는 않는다.

최종 Native/fresh WASM 각각 22문서 27쪽을 직접 판독했다. 공개 대표 PNG는 모두 새 source에서 다시 생성한 28개다. 집중 24/24, 전체 10,265/10,265(50 skipped), 필수 lint·Skia 및 Studio 18문서/45검사 PASS. 50% 정책은 미검증이며 이 증적을 그 정책의 완료 근거로 쓰지 않는다.
