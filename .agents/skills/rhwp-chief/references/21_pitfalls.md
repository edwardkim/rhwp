# 21. 함정

## 발화를 goal 로 읽기

"PDF 로 바꿔줘" 가 symptom 에만 있으면 diagnose 다. 상위 시스템이 goal 을
채우지 않은 채 자연어만 넣는 버릇이 있으면, 루프를 고치지 말고 상위가
goal 을 쓰게 한다.

## 별칭

`convert` ≠ `convert-hwp`. `pdf` ≠ `export-pdf`. `tables` ≠ `extract-tables`.
별칭 사전은 곧 숨은 라우팅 표다. 만들지 않는다.

## 부분 성공

표 5개 중 4개 CSV 만 생기면 failed 다. fill 이 3필드 중 2개만 채워도
`notFound` 가 있으면 산출을 지운다. "그래도 쓸 만하다"는 회신이 아니다.

## done 폴더 재실행

고객이 같은 문서를 다시 넣고 싶으면 **새 요청 id** 다. 같은 폴더의
result.json 을 지우는 것은 운영 재시도이고, 기본 동작이 아니다.

## capabilities 캐시

`Chief.available` 은 프로세스 수명 동안 한 번 읽는다. `--watch` 로 떠 있는
동안 바이너리를 교체하면 광고 집합이 낡는다. 교체 후 루프를 재시작한다.

## 새 CLI 유혹

queue/chief/serve-queue 라는 rhwp 하위명령은 없다.
python 모듈이 큐 엔진이다.

## gym 으로 도망

큐 시나리오를 gym 팩으로 옮기지 않는다. 이 스킬의 fixtures/queues 가
실사용 기록이다.

## 트리아지 재구현

사다리를 Chief 안에 다시 짜지 않는다. 게이트 호출 + route 분기면 충분하다.
