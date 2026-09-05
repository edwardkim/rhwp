# 06 — sanitize 제출 정리

`edit sanitize <파일> [--keep-preview] [-o <출력>] [--json]` (#3719 §6-11).

작성자·제목·주제·최종수정자·작성/수정 일시·미리보기를 지운다.
**본문 내용은 건드리지 않는다.** `export-text` 전후가 같아야 한다.

이 단계는 "값이 들어갔다" 다음의 "제출할 수 있다" 다. 채움 없이
sanitize 만 하는 요청도 이 장을 연다. 새 메타 편집 로직을 만들지 않는다.

```bash
rhwp edit sanitize output/날인본.hwp -o output/제출본.hwp --json | jq '.removedCount'
rhwp edit sanitize output/제출본.hwp -o /tmp/재확인.hwp --json | jq .removedCount
```

두 번째가 `0` 이면 첫 실행이 실제로 지웠다는 증거다 (멱등).

## 봉투

```
{"schemaVersion":"1.0","source","keepPreview","removedCount",
 "removed":[{"field","before"}],"output","outputFormat"}
```

`removed[]` 는 거짓 보고를 하지 않는다. 지운 것만 실린다.

## 지우는 대상

1. **OLE 요약 정보** (`\x05HwpSummaryInformation`)
   `title` · `subject` · `author` · `keywords` · `comments` ·
   `lastSavedBy` · `revisionNumber` · `dateString` 과
   `createdAt` · `lastSavedAt` · `lastPrintedAt` (FILETIME → ISO 8601).
   속성 오프셋 표가 절대 위치를 담고 있어 **바이트 길이를 바꾸지 않고**
   비운다.
2. **HWPX 저작자 메타** (`Contents/content.hpf` 의 `<opf:metadata>`).
   직렬화기가 원본에서 그대로 splice 하는 유일한 저작자 경로. 중립
   블록으로 교체.
3. **미리보기** (PrvText · PrvImage). ZIP 엔트리와 HWP5 계약 스트림.

`--keep-preview` 는 미리보기 **이미지**만 남긴다. 미리보기 텍스트는
언제나 대상.

HWP5 직렬화기는 PrvText 가 비면 본문 앞부분으로 다시 채우므로,
미리보기 텍스트는 **지금 본문과 다를 때만**(예전 판 잔재) 지우고
보고한다.

HWPX 원본의 `/HwpSummaryInformation` 은 파일에 없던 계약 fallback
상수라 HWPX 로 저장할 때는 손대지 않고, HWP5 로 변환할 때만 처리한다.

그래서 두 번째 실행은 `removedCount: 0` 이다.

## 산출

기본 이름: `<입력명>_sanitized.<입력과 같은 확장자>`.
형식 보존은 다른 edit 와 같다 (#3383).

제출 파이프라인에서의 위치:

```
fill-fields [--verify]
  → [insert-image]
  → sanitize
```

sanitize 를 채움보다 먼저 하면 채움 저장이 작성자·일시를 다시 남길 수
있다. 제출 직전이 맞다.

## 본문 불변 확인

```bash
rhwp export-text 날인본.hwp --json > /tmp/before.json
rhwp edit sanitize 날인본.hwp -o 제출본.hwp --json
rhwp export-text 제출본.hwp --json > /tmp/after.json
jq -n --slurpfile a /tmp/before.json --slurpfile b /tmp/after.json \
  '$a[0].text == $b[0].text'
```

본문이 달라지면 이 스킬이 sanitize 를 재구현하지 않는다. 산출을 버리고
보고한다.

## 하지 말 것

- 본문 PII 를 sanitize 로 지우기 → 그건 `edit redact` (security-sweep)
- 누름틀 값을 비우기
- 미리보기 텍스트를 남기려고 새 플래그를 만들기 (`--keep-preview` 는
  이미지만)
- removedCount 0 을 실패로 처리 (멱등)
