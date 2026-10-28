# 한컴 회전·뒤집기 저장 관례 오라클

`ShapeComponentAttr.flip` 저장 워드의 비트가 무엇을 뜻하는지 **한글이 실제로 쓰는 값**으로
정한다. HWP 는 폐쇄 포맷이라 이 질문에 스펙 답이 없다 — 편집기를 정답지로 쓰는 수밖에 없다.

## 언제 쓰나

rhwp 편집 명령이 `flip` 비트나 `rotate_image` 를 세우거나 지우려 할 때, **그 근거가
한컴 관례인지** 확인할 때 쓴다. 근거 없이 비트를 세우면 저장본이 한컴과 갈라지고,
되돌릴 경로가 없으면 undo 도 성립하지 않는다.

## 무엇을 재나

세 가지를 다르게 잰다. 앞의 두 개는 한글 없이도 돈다.

| 모드 | 한글 필요 | 재는 것 |
|---|---|---|
| `--survey` | 아니오 | 한컴 저장본 전수에서 각 비트와 "회전됨" 의 상관 |
| `--resave` | 예 | 한글이 **그대로 저장**할 때 비트를 보존하는지 정규화하는지 |
| `--set-rotation DEG` | 예 | 한글이 회전을 DEG 로 **바꿔 저장**할 때 비트를 어떻게 두는지 |

`--survey` 는 특정 비트를 가정하지 않는다. 32비트 전부에 대해 회전 개체·비회전 개체에서의
출현을 세서 **어느 비트가 회전을 따라 움직이는지 데이터가 말하게** 한다. 처음에 bit19 를
회전 표식으로 짚었다가 실측에서 정반대로 뒤집힌 전례가 있어 그렇게 만들었다.

## 사용

```
cargo build --release --bin rhwp

# 표본이 많으면 커맨드라인 길이 한계에 걸린다 — 목록 파일을 쓴다.
python tools/hangul_rotation_oracle/oracle.py --survey --list samples.txt
python tools/hangul_rotation_oracle/oracle.py --survey --list samples.txt --detail

python tools/hangul_rotation_oracle/oracle.py --resave samples/ta-pic-001-r.hwp
python tools/hangul_rotation_oracle/oracle.py --set-rotation 0 samples/ta-pic-001-r.hwp
```

`--exe` 기본값은 `target/release/rhwp.exe`. **출처가 분명한 빌드를 쓸 것** — 오래된 exe 는
유령 회귀를 만든다(`tools/loadsave_sweep/README.md` 의 같은 경고).

오라클 자체의 파싱·판정 계약은 한글도 rhwp 도 없이 검증한다:

```
python tools/hangul_rotation_oracle/test_oracle.py
```

## 전제

- Windows + 한글 설치(COM 모드만). `--survey` 는 불필요.
- Python 3.11+, `pywin32`. `pyhwpx` 는 쓰지 않는다.
- `rhwp dump` 의 변환 줄. 이 도구는 그 줄을 판정 근거로 파싱하므로
  `mydocs/manual/dump_command.md` 의 형식과 함께 움직인다. 형식이 바뀌면
  `test_oracle.py` 가 먼저 깨진다.

  `dump` 에는 그림 출력 경로가 셋(본문 `shape.rs`, 표 셀 `table.rs`, story `story.rs`)
  있고 변환 줄은 앞의 둘에서만 나온다. story 경로(한 줄 이름 형식)는 변환을 내지 않으므로
  그 경로로만 나오는 개체는 표본에서 빠진다 — 표본 수가 충분한지는 `=== 표본:` 줄로 본다.

## 함정 (`tools/hwp_oracle_pdf.ps1` 에서 확인된 것과 같다)

- `SetMessageBoxMode(0x00020000)` 없이는 대화상자가 사람을 기다리며 멈춘다.
- 시작 전 잔여 `Hwp.exe` 를 정리한다. 떠 있는 인스턴스에 붙으면 그 창의 상태를
  물려받아 저장 확인 대화상자가 뜬다.
- `Open(path, "", "")` — 형식을 `"HWP"` 로 못박으면 `.hwpx` 가 빈 문서로 조용히 열린다.
- `FilePathCheckerModule` 미등록이면 파일 접근 확인 대화상자가 뜬다. 등록은 DLL 설치 +
  `regsvr32` + 레지스트리가 필요해 이 도구는 하지 않고 경고만 낸다.
- 한글 COM 은 실패 뒤 인스턴스가 오염된다 — 문서 1건당 자식 프로세스로 격리한다.
- 이 장비에 한글이 여러 버전 깔려 있으면 ProgID 가 어디에 붙는지는 등록 상태에 달렸다.
  **매 실행 버전을 기록**하므로 표에 섞이지 않는다(`# 한글 버전:` 줄).
- 여러 판정을 동시에 돌리지 말 것. 서로의 `Hwp.exe` 를 죽여 무응답 오판을 만든다.

## 알려진 한계

`--set-rotation` 은 한글 2024(13.0.0.645) 에서 회전 액션 이름을 찾지 못했다. 후보
(`ShapeObjDialog`·`ShapeObjectDialog`·`ShapeObjPropertyDialog` × `RotateAngle`·`RotAngle`·
`Rotation`)를 모두 시도하고 `ROTATE_FAIL` 로 보고한다 — **조용히 성공한 척하지 않는다.**
이름을 알아내면 `ROTATE_ACTIONS`·`ROTATE_FIELDS` 에 추가하면 된다. `--survey` 와
`--resave` 는 이 모드 없이도 판정을 낸다.

## 측정 결과

[`EVIDENCE.md`](EVIDENCE.md) 에 장비·한글 버전과 함께 기록한다. 요약: `flip` bit19 와
`rotate_image` 는 **회전 상태의 함수가 아니다** — 한컴 저장본 5660건과 한글 2024 재저장이
같은 방향을 가리킨다.
