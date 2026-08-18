# rhwp-chief references

정본 playbook 은 `mydocs/manual/chief_playbook.md` 다. 이 폴더는 그 계약을
에이전트가 30초 안에 실행하도록 장으로 나눈 것이다.

00 층 구분 → 01 큐 규약 → 02 스키마 → 03 트리아지 게이트 → 04 표
→ 05–11 goal 별 실행 → 12 needs-agent → 13 회신 → 14 멱등
→ 15 데이터≠지시 → 16 커버리지 → 17 루프 → 18 봉투 → 19 정지
→ 20 인계 → 21 함정 → 22–24 기록 → 25 종료 코드 → 26 게이트
→ 27 에이전트 가장자리.

기계 가독 자료는 `../fixtures/`. 생성기는 `../_gen_pack.py`.
픽스처 헤더의 schemaVersion 은 1.0, 루프 result.json 은 1 이다.
이 폴더는 gym 과제가 아니다.
