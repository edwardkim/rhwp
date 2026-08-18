# 예제 12_auto_number_false — auto_number false

이 워크스루는 gym 과제가 아니다. 기존 helper 와 `rhwp build-from-ingest` 만 쓴다.
교차: `AN-false`. 동작: 원본 번호 유지.

## 입력

사용자가 시험지 원본(또는 그 경로)을 준다. 원본은 읽기만 한다.

## 명령

```bash
bash .claude/skills/rhwp-exam-ingest/helpers/check_deps.sh --json
rhwp build-from-ingest "$TMP/ingest.json" --media-dir "$MEDIA" -o "$OUT"
rhwp export-text "$OUT" -o "$TMP/txt"
rhwp dump "$OUT" > "$TMP/dump.txt"
```

## ingest 요지

- 스키마 `version: "1"`. 미지 필드 없음.
- 교차 픽스처/정지를 `AN-false` 로 확인.
- `auto_number` 정책을 10장에 맞게 고정.

## 정지

실패 봉투는 `references/14_failure_envelopes.md`. 성공이 아니면 `-o` 산출을 사용자에게 주지 않는다.

## 한계

Picture #182, 수식 이미지, 표 Picture. 새 CLI / exam_paper 수정 없음.
