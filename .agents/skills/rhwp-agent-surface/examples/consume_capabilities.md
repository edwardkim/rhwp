# 레시피 — 처음 오는 에이전트가 표면을 읽는 법

목표는 명령 이름을 외우지 않고 `rhwp capabilities` 로 층을 파악하는 것이다.

## 1. CLI 자기서술 1회

```bash
rhwp capabilities > /tmp/caps.json
```

`--json` 을 붙이지 않는다. 이미 JSON 이다. 붙이면 exit 2.

읽을 것:

```bash
python -c "import json;d=json.load(open('/tmp/caps.json',encoding='utf-8'));
print(d['version'], d['schemaVersion']);
print('json 명령', sum(1 for c in d['commands'] if c.get('json')));
print(d['exitCodes']);
print(d['jsonContract']['stdout'])"
```

`available:false` 가 있으면 그 명령은 지금 바이너리에 없다.

## 2. 이름이 생각나지 않으면 검색

```bash
rhwp capabilities --search redact
rhwp capabilities --search "표 병합" --json
```

AND. 하위명령 요약도 대상이다. 0건은 exit 0 + 안내 한 줄.

`--search` 와 `--mcp` 를 같이 주지 않는다.

## 3. 무상태 선언

```bash
rhwp capabilities --mcp | python -c "import sys,json;d=json.load(sys.stdin);
print(d['protocol'], len(d['tools']));
print(sorted(t['name'] for t in d['tools'])[:8])"
```

개수는 계약이 아니다. `hwp_open` 이 이 목록에 없어도 정상.

## 4. 세션 목록은 서버에 묻는다

`tools/list` (호스트가 이미 붙어 있으면 그 결과).
호스트 부착 자체는 `rhwp-mcp-session`.

## 5. 판정 필드를 열어 본다

```bash
rhwp ir-diff a.hwp b.hwpx --json; echo exit:$?
# identical:false + exit 3 → 오류가 아니라 데이터
```

`isError` 나 exit 만 보고 끝내지 않는다.
