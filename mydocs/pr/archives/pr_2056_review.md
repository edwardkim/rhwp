# PR #2056 리뷰 — #2020 첨부 문서 잔여 렌더링 차이 해결

- PR: #2056 `task 2020: 첨부 문서 잔여 렌더링 차이 해결`
- URL: https://github.com/edwardkim/rhwp/pull/2056
- 기준 브랜치: `devel`
- head branch: `task/m100-2020-remaining-visual`
- 작성자: @jangster77
- PR 생성 직후 참고 head: `17884ba805abf2ef1fedc83c9ca79a89b32e7dae`
- 관련 이슈: #2020
- 검토 경로: collaborator self-merge 후보 + 옵션 1. review 문서, visual asset, 오늘할일을 PR head 에 함께 포함한다.
- 최종 merge 조건: PR head 최신 커밋 기준 GitHub Actions 통과 + 작업지시자 승인.

## 결론

merge 후보로 본다. #2020 본문과 댓글에 남은 여권신청서, FSC HWP/HWPX, 복학원서, 국립국어원 대표 발현을
하나의 범위에서 다시 확인했고, 구조/레이아웃 회귀 기준의 자동 visual sweep 플래그는 대표 페이지 모두 0으로
정리됐다.

이 PR 은 `Closes #2020` 로 이슈를 닫는 것이 맞다. 단, 한컴 전용 폰트가 없는 공개 기본 검증 환경에서는
pixel-level glyph/font fidelity 를 close 기준으로 삼지 않는다. FSC/복학원서의 낮은 pixel proxy 값은 기준 PDF
embedded font, 한컴 폰트 부재, PUA fallback 영향으로 문서화하고 별도 font fidelity 축으로 분리한다.

## 변경 요약

- CJK 낫표 `「」` 의 수평 조판 advance 를 보정하고, 여권신청서 `2.「여권법」제9조` 줄 간격 회귀 테스트를 추가했다.
- visual sweep 에서 하단 footer bleed 와 U+F081C TAC filler false-positive 를 억제해 구조 플래그가 실제 결함만
  보도록 정리했다.
- FSC HWP/HWPX page count 5쪽 유지, HWP 2쪽 하단 14x15 표 유지, HWP/HWPX 원본과 기준 PDF 보존을 회귀 테스트와
  자산으로 고정했다.
- 복학원서 접수증의 날인선, 빨간 도장 원, `㊞` 위치, 도장 우측 하단 U+F081C marker 선을 한컴 2022 PDF 기준에
  맞춰 보정했다.
- `samples/복학원서.hwpx` 를 추가해 HWPX 기준으로도 접수증 표식 구조를 분석할 수 있게 했다.
- `mydocs/pr/assets/pr_2056_issue2020_*` 파일명으로 대표 visual sweep PNG, overlay, raw summary JSON, manifest 를 보존했다.
- PR 준비 중 clippy `manual_contains` 경고 1건을 `src/document_core/commands/object_ops/picture.rs` 에서 기계적으로
  정리했다.

## 첨부 문서와 기준 PDF

| 항목 | 보존 경로 |
|------|-----------|
| 여권신청서 HWP | `samples/issue2020/passport_application_lawgo.hwp` |
| 여권신청서 기준 PDF | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf`, `pdf/issue2020/passport_application_lawgo-2022.pdf` |
| FSC HWP/HWPX | `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwp`, `samples/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향.hwpx` |
| FSC 기준 PDF | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf`, `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` |
| 복학원서 HWP/HWPX | `samples/복학원서.hwp`, `samples/복학원서.hwpx` |
| 복학원서 기준 PDF | `pdf/issue2020/복학원서-2022.pdf` |
| 국립국어원 업무계획 HWP/PDF | `samples/2022년 국립국어원 업무계획.hwp`, `pdf/2022년 국립국어원 업무계획-2022.pdf` |

## 로컬 검증

PR 생성 전 현재 코드 기준으로 다음을 순차 실행했다.

| 명령 | 결과 |
|------|------|
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과, 2149 passed / 7 ignored |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | 통과. `tests/svg_snapshot.rs` 포함 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |
| `cargo clippy --all-targets -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 1 ignored |
| `wasm-pack build --target web --out-dir pkg` | 통과 |
| `cd rhwp-studio && npx tsc --noEmit` | 통과 |
| `cd rhwp-studio && npm test` | 통과 |

코드 검증 뒤 추가된 커밋은 visual evidence asset 과 본 review/오늘할일 문서뿐이다.

## 시각 검증

대표 visual sweep 증적은 `mydocs/pr/assets/pr_2056_issue2020_visual_sweep_manifest.md` 에 보존했다.

| 항목 | 기준 PDF | 요청 페이지 | 결과 | 대표 asset |
|------|----------|-------------|------|------------|
| 여권신청서 law.go.kr PDF | `pdf/issue2020/passport_application_lawgo-lawgo-2020.pdf` | 1 | 2/2쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_passport_lawgo_p1_review.png` |
| 여권신청서 HWP 2022 PDF | `pdf/issue2020/passport_application_lawgo-2022.pdf` | 1 | 2/2쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_passport_hwp2022_p1_review.png` |
| FSC HWP | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwp-2022.pdf` | 1-2 | 5/5쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_fsc_hwp_p1_p2_review.png` |
| FSC HWPX | `pdf/issue2020/(250813) (보도자료) 2025년 7월중 가계대출 동향-hwpx-2022.pdf` | 1-2 | 5/5쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_fsc_hwpx_p1_p2_review.png` |
| 복학원서 | `pdf/issue2020/복학원서-2022.pdf` | 1 | 1/1쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_bokhak_p1_review.png` |
| 국립국어원 업무계획 | `pdf/2022년 국립국어원 업무계획-2022.pdf` | 3 | 35/35쪽, flagged 0 | `mydocs/pr/assets/pr_2056_issue2020_niklp_p3_review.png` |

## #2020 증상별 판정

| 이슈 본문 증상 | 판정 |
|----------------|------|
| 글자 위/아래 간격이 좁아짐 | 구조 플래그 해소. 여권, 복학원서, FSC, 국립국어원 대표 페이지의 line/column drift 자동 플래그 0. |
| 특정 기호 뒤 불필요 공백 | 해결. 여권신청서 낫표 advance 회귀 테스트로 고정. |
| 이미지 위 텍스트 겹침 | 해결. FSC page count 와 2쪽 하단 표 이월 회귀 테스트로 고정. |
| 윗첨자가 일반 글자로 렌더링 | 해결. SVG 첨자 크기/baseline 테스트와 전체 snapshot 통과. |
| 글꼴이 원본과 다름 | close blocker 아님. 한컴 전용 폰트 부재로 인한 pixel-level fidelity 는 별도 축. |
| 도형 또는 원형 표시 위치 차이 | 구조 플래그 해소. 복학원서 도장 원/표식 위치 보정 및 sweep flagged 0. |
| 일부 객체가 회전된 것처럼 보임 | 구조 플래그 해소. 복학원서 대표 페이지 flagged 0. |
| 같은 페이지 하단 내용이 다음 페이지로 밀림 | 해결. FSC 5쪽 유지, 국립국어원 35쪽 유지. |

## 리스크와 merge 후 처리

- PR head 최신 커밋 기준 GitHub Actions 통과를 merge 전 확인해야 한다.
- merge 후 #2020 이 `Closes #2020` 로 자동 close 되는지 확인한다. 자동 close 가 지연되면 시간을 두고 재조회하고,
  계속 open 이면 수동 close 한다.
- issue 후속 코멘트에는 PR #2056, merge commit, CI/local 검증, visual sweep 증적, font fidelity 분리 판단을 남긴다.
- merge 후 `devel` sync 와 로컬/원격 작업 브랜치 정리를 수행한다.
