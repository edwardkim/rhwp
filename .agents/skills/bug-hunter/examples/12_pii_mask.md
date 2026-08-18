# 예제 — 개인정보 탐지 → 마스킹

이슈 #5324. playbook 카탈로그. gym 아님. F13.

## 정답지

문서 안에서 탐지된 PII 목록과 마스킹 후 재스윕 0건.
외부 시스템 업로드·실명 조회는 범위 밖.

## 명령

```bash
rhwp inspect hidden-text 파일.hwp --json
rhwp inspect unicode 파일.hwp --json
rhwp edit redact --dry-run 파일.hwp --json
# 적용은 요청 있을 때만. 원본은 -o 분리
```

보안 스윕 본문은 `rhwp-security-sweep`. 여기서는 제출 직전 산출이
요건(마스킹)을 만족하는지 재독한다.

## 읽는 법

dry-run 없이 원본을 덮지 않는다. 탐지 목록을 UTF-8 파일로 남기고
콘솔에 주민번호를 찍지 않는다.
