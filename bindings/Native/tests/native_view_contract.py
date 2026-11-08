#!/usr/bin/env python3
"""Exercise the actual host view ABI without opening a document."""

import argparse
import copy
import importlib.util
from pathlib import Path

EXAMPLE = Path(__file__).resolve().parents[1] / "examples" / "native_host.py"
SPEC = importlib.util.spec_from_file_location("rhwp_view_host", EXAMPLE)
module = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(module)


def run(library):
    host = module.NativeHost(library)
    state = {"config": {"pages": [{"width": 100, "height": 200}, {"width": 200, "height": 300}],
                        "viewport": {"width": 150, "height": 100}, "zoom": 1,
                        "deviceScale": 2, "pageGap": 20}, "scroll": {"x": 0, "y": 0}}
    def request(state, **command):
        return host.host_request(op="view", state=state, command=command)
    result = request(state, action="state")
    assert result["contentSize"] == {"width": 200, "height": 520}
    assert result["rasterScale"] == 2
    before = request(state, action="pageAt", x=100, y=50)["query"]
    result = request(state, action="zoomAt", zoom=2, x=100, y=50)
    after = request(result["state"], action="pageAt", x=100, y=50)["query"]
    assert before == after
    mapped = request(result["state"], action="pointInViewport", page=after["page"],
                     x=after["point"]["x"], y=after["point"]["y"])["query"]
    assert mapped == {"x": 100, "y": 50}
    result = request(state, action="scroll", x=1e9, y=1e9)
    assert result["state"]["scroll"] == result["scrollLimit"]
    assert result["visible"][-1]["page"] == 1
    for invalid in (dict(action="state", unexpected=True), dict(action="zoomAt", zoom=-1, x=0, y=0)):
        try:
            request(state, **invalid)
        except module.NativeCallError as error:
            assert error.code == "invalidArgument"
        else:
            raise AssertionError("invalid view request was accepted")
    broken = copy.deepcopy(state)
    broken["config"]["deviceScale"] = 0
    try:
        request(broken, action="state")
    except module.NativeCallError as error:
        assert error.code == "invalidArgument"
    else:
        raise AssertionError("invalid deviceScale was accepted")
    print('{"ok":true,"contract":"native-view","anchoredZoom":true,"coordinateRoundtrip":true}')


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("library", type=Path)
    run(parser.parse_args().library)
