# 24 — 함정 노트 (재현 시 시간 절약)

README 함정 절을 에이전트 언어로 펼친다. 새 함정을 도구 없이
주장하지 않는다.

## 캡처·창

1. 고정 창은 A3 를 크롭해 가짜 diff 를 만든다. 하네스가 판형을 읽는다.
   `--window-size` 를 바깥에서 붙이지 말 것.
2. Chrome 실패는 한 번 재시도하고 stderr 를 표면화한다. 바깥 루프 금지.
3. 같은 out-dir 의 PNG 는 재사용된다. 글꼴 변경 후 반드시 새 디렉터리
   또는 PNG 삭제 (F14).

## 숫자

4. diff% 는 랭킹용. 자간 미세 차가 누적된다. math 6–11% 가 정상 구간에
   가깝다.
5. text-report 는 순서·좌표를 모른다. 줄바꿈 버그를 확정하지 말 것.
6. `reference_only=0` 이 시각 동일을 뜻하지 않는다. path 글리프, 숨김
   텍스트, 추출기 매핑.
7. page-count 차이는 전역 page-break 패치 근거가 아니다 (F11).
8. 절대 임계 CI 게이트를 만들지 말 것 (F05).

## 환경

9. 배경 셸 한글 argv 는 cp949 로 깨진다. 키·글롭.
10. Windows 는 `venv\Scripts\python.exe`. `venv/bin/python` 폴백 금지.
11. `--break-system-packages` 금지 (F15).
12. 시스템 `python tools/fidelity_compare/...` 는 ImportError 를
    "도구 버그"로 위장한다 (F09).
13. stale `target/release-test/rhwp` 는 방금 고친 렌더러가 아니다.
    `target/pr-review` 를 명시.
14. Edge 를 `CHROME_BIN` 에 넣지 말 것.

## 기준

15. `samples/` 동반 PDF 는 참고 등급 (F17).
16. `save_as(PDF)` 맞춰찍기는 PageCount 가드. 축소본을 오라클로 쓰지 말 것.
17. `rhwp export-pdf` 는 한컴 출력이 아니다 (F01).
18. bunjang 을 2022 기준으로 승격하지 말 것.
19. 암호화 PDF 빈 추출을 전량 소실로 읽지 말 것 (F13).

## 글꼴

20. 전면 두부는 하네스 오염 (F14).
21. HMKMM/HMKMG EBDT `.notdef`. outline 우선. HY신명조는 보존.
22. 글꼴 바이너리를 저장소에 커밋하지 말 것.
23. Linux 만 fontconfig 스텁. Windows 에 이식하지 말 것.

## 범위·이웃

24. gym/ 금지 (F06).
25. visual-regression / bug-hunter 재작성 금지 (F07, F08).
26. 새 CLI 발명 금지.
27. 원본 덮어쓰기 금지 (F18).
28. 질문이 이미 답이면 다음 단 금지 (F16).

## 에이전트 자가 점검

이 장을 연 이유가 "숫자가 이상해서" 이면 4–8 을 먼저 읽는다.
"윈도에서 안 돌아서" 이면 9–14. "한컴과 같다/다르다" 문장을 쓰기
전이면 15–19 와 12장.
