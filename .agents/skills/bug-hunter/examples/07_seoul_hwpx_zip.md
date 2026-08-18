# 예제 — 인터넷 배포 실물 왕복 (서울시)

이슈 #5324. playbook 예시 7. gym 아님.

## 정답지

https://opengov.seoul.go.kr/sanction/11678326 첨부 HWPX 원본의
ZIP 엔트리 **이름 집합**·엔트리별 크기·태그 개수.
`--verify` 4/4 가 정답지가 아니다 (함정 1, F09).

## 명령

```bash
rhwp info --json 서식.hwpx
rhwp export-tables --json 서식.hwpx
rhwp fields --json 서식.hwpx
rhwp export-svg 서식.hwpx -o svg/
rhwp export-hwpx 서식.hwpx out.hwpx --verify --verify-pages
venv/bin/python - <<'PY'
import zipfile
a, b = zipfile.ZipFile('서식.hwpx'), zipfile.ZipFile('out.hwpx')
print('missing', sorted(set(a.namelist())-set(b.namelist())))
print('added', sorted(set(b.namelist())-set(a.namelist())))
PY
```

## 읽는 법

여러 문서에서 같은 바이트 수 감소면 상수 블록 소실 신호 (#3551
header.xml 6,737B). 엔트리 수 12→12 여도 이름이 빠질 수 있다
(#3557). 값 손실이 아니면 아니라고 쓴다.

관련: `fixtures/transcripts/verify_then_zip.txt`.
