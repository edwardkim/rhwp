# #6963 단계 13 — 사용자 확인에 따른 링크 경계 정책 정정

- 기준: 로컬 `01417901fcb886e5c373d622aa29fe9d210c4647`.
- 사용자 확인: 삭제 취소 보정(2번)은 정상. 시작 경계에서 링크 서식을 상속시키는 보정은 잘못된 방향.
- 확정 계약: 링크 앞·뒤 경계에 새로 입력한 글자는 일반 텍스트이며 링크 밖이다.
  시작에서는 링크 start/end를 함께 뒤로 옮기고, 끝에서는 기존 링크 범위를 유지한다.
  엄격한 내부(start < offset < end)에서만 링크가 확장된다.
- 시작 char-shape 경계는 기존처럼 뒤로 이동해 일반 서식을 사용한다. 인접 링크 사이도
  어느 링크에도 새 글자를 포함하지 않는다. 굵기·글꼴 등 기존 글자 서식을 무조건 초기화하지 않는다.
- 단계 12의 '시작 글자 링크 서식 상속' 판정과 해당 UI 기대값은 철회한다.
  삭제 조각에 의한 undo 보정은 유지한다.
- 검증: native 시작·끝·내부·인접 링크, HWP/HWPX 왕복, 실제 WASM/Studio history,
  fresh WASM과 브라우저. 원격 반영·코멘트는 계속 승인 대기.
- 별도로 확인된 빈 링크 필드의 재삽입 거부는 이번 경계 정책 정정과 구분하며 아직 미해결이다.

## 원인 계층과 보정

시작 필드 범위만 이동하는 첫 시도는 HWPX 저장 재열기에서 실패했다. HWPX는 raw UTF-16
갭의 FIELD_BEGIN 슬롯도 소비하므로 새 글자의 raw 위치를 BEGIN 앞에 두어 범위와 슬롯을
함께 이동한다. 8 code unit은 HWP 필드 마커의 사양상 폭이며 화면 좌표 보정값이 아니다.

인접 링크의 서식 적용 범위는 다음 표시 글자 시작까지의 제어 슬롯 갭을 포함하고 있었다.
링크 끝에서 서식 범위를 마지막 표시 글자 직후에 끝내어 원래 일반 서식 run을 보존한다.
같은 글자 범위 계산은 서식 ID 수집과 적용 양쪽에 쓰인다. 색을 강제로 검정으로 바꾸거나
굵기·글꼴을 초기화하지 않는다.

## 검증 기록

- native hyperlink 편집 16/16 통과: 문단 시작·일반 글자 뒤·인접 링크 사이 입력,
  끝 이어 쓰기·내부 확장·삭제 undo 조각·HWP/HWPX 왕복.
- `issue_6788_mixed_char_format`: 17/17 통과.
- `apply_char_format_contract`: 4/4 통과.
- 로그: `/private/tmp/pr6984-boundary-native.log`, `pr6984-boundary-mixed-format.log`,
  `pr6984-boundary-char-format.log`. 초기 실패 로그는 수정 원인 확인 후 최종 실행으로 교체했다.

수동 확인 기준: 새 링크 `링크`의 맨 앞에서 `X`를 입력하면 문서 텍스트는 `X링크`이고,
`X`는 일반 서식, 링크의 표시 문자열은 `링크`여야 한다. Home 위치는 이제 링크 밖이므로
기존 링크 고치기를 열려면 `링`과 `크` 사이로 커서를 옮긴다.

- fresh WASM 빌드 성공 (`scripts/wasm-pack-locked.sh --target web --out-dir pkg --no-opt`).
  SHA-256: `fe9590d4b8b2ff2120a739155bb5e68fbec0467356de05212798f46aca00255c`.
- 실제 WASM/Studio command/history 15개 검증 그룹 통과. 삭제 undo 및 경계 입력의
  일반/링크 서식, HWP/HWPX 왕복을 포함한다. 로그: `/private/tmp/pr6984-boundary-wasm.log`.
- `cargo fmt --all -- --check`, suite manifest `--check`, `git diff --check`: 통과.
- 브라우저 fresh 문서에서 링크 삽입 → Home → X: 검정/밑줄 없는 X를 확인했다.
  링크 고치기의 표시 문자열은 `링크`이며 X가 포함되지 않는다.
  화면: `/private/tmp/pr6984-boundary-outside.jpg`, `pr6984-boundary-link-range.jpg`.
- 7794 서버는 새 WASM으로 갱신했다. 사용자 기존 탭의 문서는 수정하지 않았다.
- 새 보정 head의 전체 Rust lint/release-test와 base 통합·CI는 이후 승인 단계에 남아 있다.
  GitHub push·comment는 실행하지 않았다.
