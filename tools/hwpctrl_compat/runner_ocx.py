"""오라클 러너 — 설치된 한글(COM)에 시나리오를 실행시킨다 (P0).

판정자는 문서가 아니라 **설치된 한글**이다. 이 러너가 정답지를 만든다.

## 쓰임

    python tools/hwpctrl_compat/runner_ocx.py scenarios/field-read.json --out output/poc/hwpctrl/ocx

## COM 규약 (어기면 오판이 난다)

- **문서 하나당 프로세스 하나.** 한 프로세스에서 `Hwp()` 를 두 번 만들면 `com_error` 로 죽는다.
  이 스크립트가 그 단위다 — 호출 측(`run_gate.py`)이 프로세스를 띄우고 시간 제한을 건다.
- **동시에 여러 판정을 돌리지 말 것.** 서로의 `Hwp.exe` 를 죽여 "무응답" 오판을 만든다.
- 보안 모듈(`FilePathCheckerModule.dll`)이 등록돼 있어야 파일 접근 다이얼로그가 뜨지 않는다.
  `pyhwpx` 가 `register_module=True` 로 처리한다.

## WebHwpCtrl ↔ COM 의미 차이

웹한글컨트롤(v2.4 §2.2)은 ActiveX 와 **호출 규약이 다르다**. 포인터로 받던 값을 객체로
돌려주고, 서버 접근이 필요한 API 는 콜백을 받는다. 대조가 성립하려면 COM 쪽 반환을
**웹 쪽 형태로 정규화**해야 한다. 그 변환이 `ADAPTERS` 다. 여기에 없는 API 는 COM 반환을
그대로 쓴다(스칼라는 두 규약이 같다).
"""

from __future__ import annotations

import argparse
import io
import json
import os
import platform
import re
import sys
import traceback
from pathlib import Path

from oracle_version import matches_expected_version
from scenario_spec import platform_path_key, resolve_args as resolve_arg_paths

REPO = Path(__file__).resolve().parents[2]


def normalize(value):
    """COM VARIANT → JSON 으로 실을 수 있는 값.

    객체는 값을 못 뽑으므로 **타입 이름만** 남긴다. 양쪽 러너가 같은 규칙을 쓰므로
    "객체가 돌아왔다"는 사실 자체는 대조된다.
    """
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if isinstance(value, (list, tuple)):
        return [normalize(v) for v in value]
    if isinstance(value, dict):
        return {k: normalize(v) for k, v in value.items()}
    return {"__type": type(value).__name__}


# 웹 규약으로 되돌리는 변환. `com` 은 raw COM 객체다.
ADAPTERS = {
    # v2.4 §8.3.12 — 웹은 {list, para, pos} 객체를 리턴한다.
    "GetPos": lambda com, args: dict(zip(("list", "para", "pos"), com.GetPos())),
    # v2.4 §8.3.14 — 웹은 {slist, spara, spos, elist, epara, epos} 객체를 리턴한다.
    "GetSelectedPos": lambda com, args: dict(
        zip(("result", "slist", "spara", "spos", "elist", "epara", "epos"), com.GetSelectedPos())
    ),
    # v2.4 §8.3.27 — 값 **아홉**이다: `result` 다음에 `seccnt`(구역 수)가 하나 더 온다.
    # 그것을 빼먹어 `secno` 부터 이름이 한 칸씩 밀려 있었다(실측으로 바로잡음 — 마지막 값은
    # 숫자가 아니라 "(A43): 문자 입력" 같은 **문자열**이라 밀림이 눈에 띄었다).
    "KeyIndicator": lambda com, args: dict(
        zip(
            (
                "result",
                "seccnt",
                "secno",
                "prnpageno",
                "colno",
                "line",
                "pos",
                "over",
                "ctrlname",
            ),
            com.KeyIndicator(),
        )
    ),
    # v2.4 §8.3.57·§8.3.59 — 웹은 날짜를 객체로 리턴한다(`result`·`year`·`month`·`day`,
    # 음력 쪽은 `leap` 까지). COM 은 같은 값을 out 파라미터 튜플로 준다.
    "SolarToLunar": lambda com, args: dict(
        zip(("result", "year", "month", "day", "leap"), com.SolarToLunar(*args))
    ),
    "LunarToSolar": lambda com, args: dict(
        zip(("result", "year", "month", "day"), com.LunarToSolar(*args))
    ),
}


CALL_WITH_ARGS = re.compile(r"^([A-Za-z_]\w*)\((.*)\)$")


def split_call(part: str) -> tuple[str, list]:
    """점 표기 한 마디를 `(이름, 인자들)` 로 가른다.

    중간 마디가 인자를 받는 메서드일 때 쓴다 — `HeadCtrl.GetAnchorPos(0).Item` 처럼.
    인자는 JSON 으로 읽는다(`0`·`"본문"`). 괄호가 없으면 인자 없는 마디다.
    """
    m = CALL_WITH_ARGS.match(part)
    if not m:
        return part, []
    inner = m.group(2).strip()
    return m.group(1), (json.loads(f"[{inner}]") if inner else [])


def resolve_path(com, path: str):
    """점 표기 경로를 따라가 그 자리의 값을 준다 — `$obj` 인자를 푸는 데 쓴다."""
    obj = com
    for part in path.split("."):
        part, call_args = split_call(part)
        obj = getattr(obj, part)
        if callable(obj):
            obj = obj(*call_args)
    return obj


def resolve_args(com, args: list) -> list:
    """인자 중 `{"$obj": "경로"}` 를 **그 자리의 객체**로 바꾼다.

    `SetPosBySet`·`DeleteCtrl` 처럼 파라미터셋이나 `Ctrl` 을 인자로 받는 API 가 있는데,
    JSON 으로는 그런 객체를 적을 수 없어 시나리오가 아예 부르지 못했다. 경로를 적어 두면
    러너가 자기 쪽에서 만들어 넘긴다 — 두 러너가 같은 규약이라 양쪽이 같은 것을 넘긴다.
    """
    out = []
    for a in args:
        if isinstance(a, dict) and "$obj" in a:
            out.append(resolve_path(com, a["$obj"]))
        else:
            out.append(a)
    return out


def call_one(com, name: str, args: list):
    """메서드면 호출하고, 속성이면 읽는다. 반환은 정규화한다.

    이름에 점을 찍으면 **객체를 타고 들어간다**(`CharShape.Item`). 서식은 값이 아니라
    ParameterSet **객체**로 오기 때문에, 점 표기가 없으면 `{__type: …}` 만 대조하게 되고
    아무 일도 안 하는 구현이 통과한다.
    """
    adapter = ADAPTERS.get(name)
    if adapter:
        return normalize(adapter(com, args))
    args = resolve_args(com, args)
    obj = com
    parts = name.split(".")
    for part in parts[:-1]:
        part, mid_args = split_call(part)
        obj = getattr(obj, part)
        # 중간이 **메서드**면 불러서 그 반환 객체를 탄다(`GetPosBySet.Item`). 속성이면 그대로
        # 쓴다(`CharShape.Item`). rhwp 러너도 같은 규약이라 양쪽이 같은 값을 대조한다.
        if callable(obj):
            obj = obj(*mid_args)
    attr = getattr(obj, parts[-1])
    if callable(attr):
        return normalize(attr(*args))
    return normalize(attr)


def output_paths(scenario: dict, out_dir: Path) -> tuple[Path, Path, Path | None]:
    """시나리오 산출물이 out_dir 밖을 지우거나 저장하지 않게 고정한다."""
    root = out_dir.resolve()

    def below_root(relative: str) -> Path:
        candidate = (root / relative).resolve()
        if candidate == root or root not in candidate.parents:
            raise ValueError(f"산출물 경로가 --out 밖입니다: {relative}")
        return candidate

    scenario_id = scenario["id"]
    saved = below_root(scenario["saveAs"]) if scenario.get("saveAs") else None
    return (
        below_root(f"{scenario_id}.returns.json"),
        below_root(f"{scenario_id}.rejected.json"),
        saved,
    )


def clear_previous_outputs(scenario: dict, out_dir: Path) -> tuple[Path, Path, Path | None]:
    """재실행 전에 같은 시나리오의 정답지와 저장본을 제거한다.

    버전 거부와 실패가 직전 `returns.json`을 재사용하는 일을 막는다. 파일만 삭제하며 예상 밖의
    디렉터리나 특수 파일은 중단시켜 호출자가 명시적으로 정리하게 한다.
    """
    paths = output_paths(scenario, out_dir)
    for path in paths:
        if path is None or (not path.exists() and not path.is_symlink()):
            continue
        if not path.is_file() and not path.is_symlink():
            raise ValueError(f"기존 산출물이 일반 파일이 아닙니다: {path}")
        path.unlink()
    return paths


def discard_changes_and_quit(hwp: object, com: object) -> None:
    """Oracle 문서 변경을 버린 뒤 한글을 종료한다.

    `HwpObject.Quit()`만 호출하면 수정된 문서의 저장 확인창이 active RDP 세션을 막는다.
    `pyhwpx.Hwp.clear(option=1)`은 active `XHwpDocument`에 hwpDiscard를 적용하는 API다.
    시나리오의 `com.Clear(1)` 호출과는 다른 종료 경로이므로 혼용하지 않는다.
    """
    try:
        hwp.clear(option=1)
    except Exception:  # noqa: BLE001 - 종료는 clear 실패와 무관하게 시도한다.
        pass
    try:
        com.Quit()
    except Exception:  # noqa: BLE001 - runner 결과는 이미 기록했을 수 있다.
        pass


def run(scenario: dict, out_dir: Path, expect_version: str | None = None) -> dict:
    from pyhwpx import Hwp

    result = {
        "scenario": scenario["id"],
        "runner": "ocx",
        "oracle": None,
        "calls": [],
        "saved": None,
        "fatal": None,
    }
    # `HWPCTRL_VISIBLE=1` 이면 창을 띄운다. 기본은 숨김이다 — 창이 뜨면 사람이 쓰는 화면을
    # 가로채고 대량 실행이 느려진다.
    #
    # 편집 액션이 **띄엄띄엄 먹는** 문제를 쫓다 만든 스위치인데 창은 원인이 아니었다
    # (원인은 읽기 전용 열림 — 계획서 §4.24). 다음 사람이 같은 것을 다시 만들지 않도록 남긴다.
    visible = os.environ.get("HWPCTRL_VISIBLE") == "1"
    hwp = Hwp(new=True, visible=visible)
    com = hwp.hwp
    # 어느 한글이 답했는지 **매 실행 기록한다**. 이 머신에는 2022 와 2024 가 함께 깔려 있고
    # ProgID `HWPFrame.HwpObject` 가 어느 쪽으로 붙는지는 등록 상태에 달렸다. 기록하지 않으면
    # 서로 다른 오라클의 결과를 같은 표에 섞게 된다.
    try:
        result["oracle"] = {"version": normalize(com.Version)}
    except Exception as exc:  # noqa: BLE001
        result["oracle"] = {"version": None, "error": f"{type(exc).__name__}: {exc}"}

    # 버전이 어긋나면 **시나리오를 아예 돌리지 않는다.** 돌린 뒤 거부하면 잘못된 버전의
    # 정답지가 이미 디스크에 남아, 다음 사람이 그것을 정답으로 쓴다.
    version = (result.get("oracle") or {}).get("version")
    if not matches_expected_version(version, expect_version):
        result["rejected"] = f"기대 major '{expect_version}' 실제 '{version}'"
        discard_changes_and_quit(hwp, com)
        return result

    # 연 표본의 **원본 지문**을 떠 둔다. 시나리오가 표본을 고치면 그 다음 실행부터 정답지가
    # 통째로 어긋나는데, 증상이 "갑자기 회귀"로 보여서 원인을 찾기 어렵다.
    #
    # 실제로 그랬다: `Clear(1)` 은 한글에게 **저장하고 닫으라**는 뜻이라 시나리오가 돌 때마다
    # `samples/2026_oss_rst.hwp` 에 자동 번호가 쌓였다. 아래 가드가 그것을 즉시 잡는다.
    src_path = (REPO / scenario["open"]).resolve() if scenario.get("open") else None
    src_before = src_path.stat().st_mtime_ns if src_path and src_path.exists() else None

    try:
        if scenario.get("open"):
            src = src_path
            opened = com.Open(str(src), "", "")
            result["calls"].append({"call": "Open", "args": [scenario["open"]], "value": normalize(opened)})

            # **읽기 전용으로 열렸으면 판정하지 않는다.**
            #
            # 같은 파일이 다른 한글 프로세스에 이미 열려 있으면 한글은 이 문서를 읽기 전용으로
            # 연다. 그러면 편집 액션이 **조용히 무시된다** — 반환값은 그대로 `null` 이고 문서만
            # 안 바뀐다. 그 정답지를 그대로 쓰면 "아무 일도 안 하는 구현"이 통과한다.
            # 실측 짝: 남은 프로세스 없음 → `EditMode` 1 · `BreakPara` 8/8 성공,
            #          같은 파일을 쥔 프로세스 있음 → `EditMode` 0 · 8/8 무동작.
            mode = com.EditMode
            if mode != 1:
                result["rejected"] = f"읽기 전용으로 열렸다(EditMode {mode})"
                return result

        # 경로는 **플랫폼마다 다른 값**이라 시나리오가 이름으로 적고 러너가 푼다. 이 러너는
        # Windows 에서만 도니 언제나 `win` 갈래다. `{repo}`·`{out}` 토큰은 현재 worktree와
        # 격리 output으로 풀어 다른 Oracle host에서도 같은 fixture를 쓴다.
        path_key = platform_path_key(platform.system())
        for idx, call in enumerate(scenario.get("calls", [])):
            name, args = call[0], (call[1] if len(call) > 1 else [])
            args = resolve_arg_paths(args, scenario, path_key, REPO, out_dir)
            record = {"call": name, "args": args}
            try:
                record["value"] = call_one(com, name, args)
            except Exception as exc:  # noqa: BLE001 — COM 예외 종류가 다양하다
                record["error"] = f"{type(exc).__name__}: {exc}"
            result["calls"].append(record)

        if scenario.get("saveAs"):
            _, _, dst = output_paths(scenario, out_dir)
            assert dst is not None
            dst.parent.mkdir(parents=True, exist_ok=True)
            ok = com.SaveAs(str(dst), "HWP", "")
            result["saved"] = {"path": str(dst.relative_to(REPO)) if REPO in dst.parents else str(dst), "ok": bool(ok)}
    except Exception:  # noqa: BLE001
        result["fatal"] = traceback.format_exc(limit=3)
    finally:
        discard_changes_and_quit(hwp, com)
    # 표본이 바뀌었으면 이 실행의 정답지를 **믿을 수 없다** — 다음 실행은 다른 문서를 잰다.
    if src_path is not None and src_path.exists():
        if src_path.stat().st_mtime_ns != src_before:
            result["rejected"] = (
                f"표본이 실행 중에 바뀌었다({scenario['open']}) — 시나리오가 저장을 부른다. "
                "`git checkout -- ` 로 되돌리고 그 호출을 고칠 것."
            )
    return result


def main() -> int:
    sys.stdout.reconfigure(encoding="utf-8")
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("scenario", help="시나리오 JSON 경로")
    ap.add_argument("--out", required=True, help="산출물 디렉터리")
    ap.add_argument(
        "--expect-version",
        help="오라클 major 버전(예: '12' = 한글2022). 어긋나면 실행하지 않고 exit 3.",
    )
    args = ap.parse_args()

    with io.open(args.scenario, encoding="utf-8") as fh:
        scenario = json.load(fh)

    # **절대경로로 못박는다.** `$path` 로 넓힌 상대 경로를 한글에 넘기면 답도 오류도 없이
    # 멈춘다 — `SaveAs("output/…/x.hwp")` 하나로 오라클이 십 분을 넘겨 죽었다. 시나리오 끝
    # 저장은 `output_paths` 가 `.resolve()` 를 거쳐 무사했던 탓에 이 갈래만 조용히 달랐다.
    # (Node 러너는 이미 `resolve(args.out)` 를 한다.)
    out_dir = Path(args.out).resolve()
    out_dir.mkdir(parents=True, exist_ok=True)
    returns_path, rejected_path, _ = clear_previous_outputs(scenario, out_dir)
    result = run(scenario, out_dir, args.expect_version)

    version = (result.get("oracle") or {}).get("version")
    if result.get("rejected"):
        # 정답지 자리에 쓰지 않는다. 증거는 남기되 이름으로 구분한다.
        with io.open(rejected_path, "w", encoding="utf-8", newline="\n") as fh:
            json.dump(result, fh, ensure_ascii=False, indent=2)
            fh.write("\n")
        if "표본이 실행 중에 바뀌었다" in result["rejected"]:
            print(
                f"{scenario['id']}: **표본이 바뀌어 판정하지 않음** — {result['rejected']}\n"
                "정답지가 다음 실행부터 통째로 어긋난다. `Clear(1)` 처럼 저장을 부르는 호출을 "
                "찾아 고칠 것(0 이면 저장 안 한다).",
            )
            return 5
        if "EditMode" in result["rejected"]:
            print(
                f"{scenario['id']}: **읽기 전용으로 열려 판정하지 않음** — {result['rejected']}\n"
                "같은 파일을 쥔 한글 프로세스가 남아 있다. 다 닫고 다시 돌려라"
                "(`Get-Process Hwp | Stop-Process -Force`). 계획서 §4.24.",
            )
            return 4
        print(
            f"{scenario['id']}: 오라클 버전 불일치로 **실행하지 않음** — {result['rejected']}\n"
            "이 머신에는 한글2022(12.x)와 2024(13.x)가 함께 있다. 전환은 계획서 §4.5.1.",
        )
        return 3

    with io.open(returns_path, "w", encoding="utf-8", newline="\n") as fh:
        json.dump(result, fh, ensure_ascii=False, indent=2)
        fh.write("\n")
    print(f"{scenario['id']}: 호출 {len(result['calls'])}건 · 오라클 {version} → {returns_path}")
    return 1 if result["fatal"] else 0


if __name__ == "__main__":
    raise SystemExit(main())
