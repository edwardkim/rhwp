# task_m100_4384 Stage 1 — 접수증 날인선 판정을 본문 문구에서 PUA 마커로

- **이슈**: [#4384](https://github.com/edwardkim/rhwp/issues/4384)
- **브랜치**: `fix/issue-4384-receipt-signature-marker`
- **분기 기준**: `upstream/devel` `e48fe8694`
- **상태**: 전체 게이트 통과, PR 게시
- **기록일**: 2026-08-10 KST

## 1. 오탐 우려는 실측으로 반증됐다

이슈는 `"접 수 증"` 이 한국 공문서에 흔해 오탐이 난다고 우려했다. `~/hwpdocs_10k/` 10,000건
실측 결과:

- 정확한 리터럴 `"접 수 증"`(반각 스페이스 포함) 매치 **0건**
- `"Filing Receipt"` 매치 **0건**
- 오탐 **0건** (9,948/10,000 파싱 성공)

공백 없는 `"접수증"` 은 33건 있었고 다수가 실제 접수증 서식(TAC 표 안)이었지만, **어느 문서도
`U+F081C` 채움 기법을 쓰지 않아** 구조 조건 자체가 맞지 않았다(10k 전체에서 F081C 사용 문서는
3건뿐, 전부 무관한 용도).

저장소 fixture 678개에서도 `복학원서.hwp`/`.hwpx` 외에는 0건이다.

## 2. 미탐은 확인됐다

`samples/복학원서.hwp` 스크래치 복사본에서 `edit replace-text` 로 "접 수 증"→"접 수 표",
"Filing Receipt"→"Filing Notice" 로 바꾸자 render-tree 에서 **`(인)` 날인선이 사라졌다.**
이슈의 미탐 주장은 정확했다.

## 3. layout.rs — 일반화 성공

`samples/복학원서.hwp` 문단 16 을 직접 덤프한 결과, 표 앞 F081C 채움줄 **안에 한컴 서명/날인 PUA
`U+F012B` 가 실제로 섞여 저장**되어 있음을 발견했다.

`tac_receipt_filler_prefix` 의 기존 주석이 *"한컴 전용 날인 기호가 같이 저장된 변형도 같은 선으로
취급한다"* 라고 **이미 이 사실을 알고 있었지만 게이트로 쓰지 않았다.**

`U+F012B` 는 `issue_937` 이 검증하는 범용 한컴 서명/날인 기호다. 제목 문구 대신 이 문자의 존재로
판정하도록 `table_contains_receipt_title` 을 `tac_filler_line_has_signature_marker` 로 교체했다.

10k 재검증: F081C 보유 3건 전부 F012B 미보유 → **새 판정도 오탐 0건, 기존보다 좁거나 같다.**
제목 편집 후에도 날인선이 유지됨을 확인해 미탐도 해소했다.

## 4. composer.rs — 일반화 불가, 근거를 코드에 남겼다

`hwp3-sample16-hwp5-2022.hwp` p83 의 LINE_SEG tag bit 를 직접 덤프해 조사했다.

두 LINE_SEG 모두 `TAG_SINGLE_SEGMENT_LINE`(bit17+18)이라 **세그먼트 비트로는 이 문서조차 구분
불가**다. 유일한 차이인 bit20(indentation)은 내어쓰기 문단의 정상적인 이어지는 줄에도 흔히 켜져,
신호로 쓰면 정상적인 2줄 문단 전반이 잘못 접힌다. "마지막 줄이 짧다"는 기하 조건 단독도 흔한 정상
조판 결과라 광범위 회귀 위험이 있다.

**일반화하지 않고 리터럴을 유지했으며, 위 조사와 기각 근거를 코드 주석으로 남겼다** — 이슈의
"일반화 실패 근거를 코드에 남긴다" 요구를 충족한다.

## 5. 사고 — 공유 stash

검증 중 `git stash` 가 worktree 간 공유 ref 임을 뒤늦게 인지하고(`stash list` 에 다른 worktree 의
"shared-stash race" 복구 기록 존재) 즉시 사용을 중단, `git show HEAD:path` 로 파일을 백업·치환하는
방식으로 전환했다. 최종 커밋 diff 가 예상한 2파일/44+9줄과 정확히 일치함을 확인했다.

## 6. 검증 (완료)

- `cargo test --profile release-test --tests` **전체 통과**(8,227줄 로그, FAILED 0건).
  `issue_2020_bokhak_receipt_seal_line_and_stamp_align`, `issue_937_*`, `issue_1116`(13개) 모두 ok.
- `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D warnings` 통과.
- 중간에 `issue_2007_intra_paragraph_saved_frame_break_is_preserved` 가 한 번 실패했으나
  baseline/fixed 양쪽 격리 재현에서 둘 다 통과해 시스템 과부하(load 48~52) 기인 flaky 로 확인했다.

남은 미래 조건은 GitHub Actions 와 작업지시자 승인, merge 다.
