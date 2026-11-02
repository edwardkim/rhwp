# 예제 24_verify_export_text — export-text 게이트

이 워크스루는 gym 과제가 아니다. 기존 helper 와 `rhwp build-from-ingest` 만 쓴다.
교차: `T09`. 동작: 중복 번호 검사. 그림이 필수이면 F18로 먼저 중단한다. 아래 명령은 텍스트 입력 또는 제한 재현용이다.

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
- 교차 픽스처/정지를 `T09` 로 확인.
- `auto_number` 정책을 10장에 맞게 고정.

## 정지

실패 봉투는 `references/14_failure_envelopes.md`. 성공이 아니면 `-o` 산출을 사용자에게 주지 않는다.

## 한계

현재 build-from-ingest는 이미지 파일을 읽지 않고 placeholder만 생성한다. 그림·수식·표 이미지가 필수이면 F18로 중단하며 아래 빌드 명령을 실행해도 완성 문서로 제공하지 않는다. 경로·배치 재시도로 해결되지 않는다. 빌더 구현은 변경하지 않는다.
