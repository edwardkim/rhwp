# font-analyzer — rhwp `info --json` 계약 기반 HWP/HWPX 글꼴 분석기

HWP(CFB/OLE)·HWPX(OWPML ZIP) 컨테이너를 **직접 파싱하지 않는다**. 글꼴 목록의
유일한 신뢰 소스는 rhwp CLI의 `info --json` 출력(`fonts[]`, `schemaVersion: "1.0"`)이며,
이 도구는 그 계약 위에서 다음만 담당한다.

- 단일 파일 글꼴 목록 조회
- 디렉터리 일괄 분석: 글꼴별 사용 파일 수 집계, 실패 파일 목록
- text / JSON / Markdown 출력

포맷 해석을 rhwp 한 곳에 위임하므로 파서가 개선되면 이 도구의 결과도 함께
정확해지고, 별도의 포맷 가정(예: HWPX를 OOXML로 취급)이 끼어들 여지가 없다.

## 요구사항

- Python 3.8+ (표준 라이브러리만 사용, 추가 설치 없음)
- rhwp 실행 파일: `cargo build --bin rhwp` (또는 release/release-test 빌드)

## rhwp 실행 파일 탐색 순서

1. `--rhwp-bin` 인자
2. `RHWP_BIN` 환경변수
3. `PATH`의 `rhwp`
4. 저장소 `target/release-test` → `target/release` → `target/debug`의 `rhwp[.exe]`

`--rhwp-bin`/`RHWP_BIN`으로 명시한 경로가 잘못됐으면 다음 후보로 조용히
넘어가지 않고 즉시 오류로 종료한다.

## 사용법

```bash
# 단일 파일 (기본: text 출력)
python tools/font-analyzer/font_analyzer.py samples/field-01.hwp

# JSON / Markdown
python tools/font-analyzer/font_analyzer.py samples/field-01.hwp --format json
python tools/font-analyzer/font_analyzer.py samples/field-01.hwp --format md

# 디렉터리 일괄 집계 (하위 디렉터리까지는 --recursive)
python tools/font-analyzer/font_analyzer.py samples --format md --output out/fonts.md

# 실패 파일이 하나라도 있으면 종료 코드 1로 처리하고 싶을 때
python tools/font-analyzer/font_analyzer.py samples --strict

# 바이너리 경로를 직접 지정
RHWP_BIN=target/debug/rhwp python tools/font-analyzer/font_analyzer.py samples/field-01.hwp
python tools/font-analyzer/font_analyzer.py samples/field-01.hwp --rhwp-bin target/debug/rhwp
```

## 출력 형태

단일 파일 JSON:

```json
{
  "source": "samples/field-01.hwp",
  "format": "hwp5",
  "fonts": ["함초롬돋움", "함초롬바탕"],
  "fontCount": 2
}
```

디렉터리 JSON은 `fileCount`/`okCount`/`errorCount`, 글꼴별 집계
`fonts[] = {name, fileCount, files[]}`(사용 파일 수 내림차순), 파일별 결과
`files[]`, 실패 목록 `errors[] = {source, error}`를 담는다.

종료 코드: 성공 0, 오류(없는 입력·rhwp 실패·빈 디렉터리 등) 1.
디렉터리 모드에서 일부 파일만 실패하면 기본적으로 결과를 출력하고 0으로
끝나며(깨진 fixture가 섞인 대량 코퍼스 대응), `--strict`일 때만 1을 반환한다.

## 테스트

실제 저장소 fixture로 성공·실패 경로를 검증하는 회귀 테스트가 있다.

```bash
RHWP_BIN=target/debug/rhwp python tools/font-analyzer/tests/test_font_analyzer.py
```

- `samples/field-01.hwp` → 글꼴에 `함초롬돋움`·`함초롬바탕` 포함 (hwp5)
- `samples/hwp3-sample5-hwpx.hwpx` → 글꼴 1종 이상 (hwpx)
- 없는 파일 / 잘못된 `RHWP_BIN` / 문서 없는 디렉터리 → 종료 코드 비 0
- 임시 디렉터리에 fixture 2개를 복사한 일괄 집계 검증

`RHWP_BIN`을 생략하면 위의 탐색 순서(3→4)로 바이너리를 찾고, 어디에도 없으면
빌드 방법을 안내하는 메시지와 함께 실패한다.
