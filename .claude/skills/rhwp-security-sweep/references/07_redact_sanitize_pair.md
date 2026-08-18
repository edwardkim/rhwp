# redact + sanitize 짝

권위는 `mydocs/manual/cli_commands.md` 의 해당 절과 레시피 3·4·10 이다.
이 장은 기존 CLI 를 설명하고 소비 규칙을 고정한다. 새 명령·새 탐지 규칙을 발명하지 않는다.
## 명제

본문만 지우면 미리보기와 작성자가 남는다.

속성만 지우면 본문의 전화번호가 남는다.

배포 전 정리는 이 둘이 짝이다.

## 순서

```bash

rhwp edit redact   초안.hwp     -o 마스킹본.hwp --no-raw --verify --json

rhwp edit sanitize 마스킹본.hwp -o 배포본.hwp --json

```

레시피 10 실측: redactedCount 3, verify.identical true, removedCount 10.

## redact 적용 계약

- `-o` 또는 `--in-place` 필수. 없으면 exit 2.

- 기본 산출 이름 없음. 원본 경로 `-o` 거부.

- `--mask` 는 비영숫자 한 글자. 두 글자면 exit 2.

- `--verify` 차이 시 exit 3. `verify.identical` 로도 읽는다.

- 탐지 0건이면 출력 파일을 만들지 않는다. `output` 부재.

- `redactedCount` 는 실제 치환 횟수. findingCount 와 다르면 같은 값이 여러 곳.

탐지는 `pii_scan`, 변경은 `replace_all_native`. 새 편집 로직이 없다.

## sanitize 계약

본문은 건드리지 않는다. `export-text` 전후가 같다.

지우는 대상 셋:

1. OLE 요약 정보(title/subject/author/keywords/comments/lastSavedBy/revisionNumber/dateString/createdAt/lastSavedAt/lastPrintedAt). 바이트 길이를 바꾸지 않고 비운다.

2. HWPX `<opf:metadata>` 중립 블록.

3. 미리보기 PrvText·PrvImage. `--keep-preview` 는 이미지만 남긴다. 미리보기 텍스트는 언제나 대상.

`removed[]` 는 거짓 보고를 하지 않는다. HWP5 직렬화기가 빈 PrvText 를 본문 앞부분으로 다시 채우므로, 미리보기 텍스트는 지금 본문과 다를 때만 지우고 보고한다.

두 번째 실행은 `removedCount: 0` — 첫 실행이 실제로 지웠다는 증거.

## 미리보기 누수

한컴에서 저장한 실무 문서는 마스킹 전 본문이 미리보기에 남아 있을 수 있다.

redact 만 하고 sanitize 를 건너뛰면 지운 값이 미리보기로 샌다.

레시피 3 의 `preview.text` 실측이 그 경로다.

## 중간 산출물

초안·마스킹본은 공유 경로에 두지 않는다. 내보내는 파일은 최종본 하나.
