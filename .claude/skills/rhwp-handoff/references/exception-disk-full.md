# 예외: 디스크 가득

## 신호

`result.json`·캡슐·저널·수거물을 쓰다 `ENOSPC` / "No space left on device" /
Windows `디스크 공간이 부족합니다` / 쓰기가 0바이트로 끝난다.

## 하지 않는 것

- 실패한 파일을 성공인 척 working doc 머리에 올린다
- sandbox 를 지워 공간을 만들려고 **이름 붙은 트리**의 산출을 지운다
- 부분 JSON 을 손으로 닫아 유효한 봉투처럼 만든다
- 새 위임을 바로 다시 돌린다 (같은 디스크)

## 하는 것

1. 추가 산출을 쓰지 않는다 (재시도가 더 채운다)
2. 이미 닫힌 파일만 유효하다. 마지막 `--verify-journal` 이 통과한 줄까지
3. working doc 에 `disk_full` 과 실패한 경로를 적고 `status: blocked` 로 둔다
4. 사람에게 공간을 요청한다. 시트 리필로 같은 디스크에 넘기지 않는다
5. 표본 exit 1 (IO) —
   `fixtures/exceptions/disk_full.json`,
   `fixtures/envelopes/disk_full.json`

## 워크스루

[`../examples/11_disk_full.md`](../examples/11_disk_full.md).
레이아웃: `fixtures/layouts/disk-full/`.
