"""Stop where full rendering commits its line-token policy.

The full renderer must select a frame-aware ``SpaceMetric`` before it
tokenizes a paragraph.  ``rhwp-render-flow`` stops only when production
diverges from the stored-font default and selects ``HalfCell``.  That
decision cannot pass through the test-only stored-metric wrapper.
"""

from pathlib import Path

import lldb


COMMAND = "rhwp-render-flow"
SOURCE_MARKER = (
    "let mut tokens = tokenize_paragraph_with_regenerated_space_metric("
)
OWNER = "fn layout_paragraph_in_frame_impl("
OWNER_END = "pub(crate) fn stored_row_metrics("


def _source_path():
    root = Path(__file__).resolve().parents[2]
    return root / "src/renderer/composer/line_breaking.rs"


def _decision_line(source):
    lines = source.read_text(encoding="utf-8").splitlines()
    owner_line = next(
        index for index, line in enumerate(lines) if OWNER in line
    )
    end_line = next(
        index
        for index, line in enumerate(
            lines[owner_line + 1 :], owner_line + 1
        )
        if OWNER_END in line
    )
    return next(
        index + 1
        for index, line in enumerate(
            lines[owner_line:end_line], owner_line
        )
        if SOURCE_MARKER in line
    )


def _display(value):
    if not value.IsValid():
        return "<unavailable>"
    return value.GetValue() or value.GetSummary() or str(value)


def _stop_at_flow(frame, _location, _state):
    function_name = frame.GetFunctionName() or "<unknown>"
    space_metric = frame.FindVariable("space_metric")
    if _display(space_metric) != "HalfCell":
        return False
    english_break = frame.FindVariable("english_break_unit")
    korean_break = frame.FindVariable("korean_break_unit")
    print("renderer-flow: owner=" + function_name)
    print("renderer-flow: space_metric=" + _display(space_metric))
    print("renderer-flow: english_break=" + _display(english_break))
    print("renderer-flow: korean_break=" + _display(korean_break))
    return True


def renderer_flow_command(debugger, command, result, _state):
    """Install the full-render line-flow decision breakpoint."""
    if command.strip():
        result.SetError(COMMAND + " takes no arguments")
        return
    target = debugger.GetSelectedTarget()
    if not target.IsValid():
        result.SetError("select a target before running " + COMMAND)
        return
    source = _source_path()
    try:
        line = _decision_line(source)
    except (OSError, StopIteration) as error:
        message = "cannot locate renderer decision: " + str(error)
        result.SetError(message)
        return
    breakpoint = target.BreakpointCreateByLocation(str(source), line)
    if not breakpoint.IsValid() or breakpoint.GetNumLocations() == 0:
        result.SetError("renderer decision breakpoint did not resolve")
        return
    breakpoint.SetOneShot(True)
    breakpoint.SetScriptCallbackFunction(
        __name__ + "._stop_at_flow"
    )
    result.AppendMessage(
        f"{COMMAND}: breakpoint {breakpoint.GetID()} at "
        f"{source.name}:{line}"
    )


def __lldb_init_module(debugger, _state):
    debugger.HandleCommand(
        "command script add -f "
        + __name__
        + ".renderer_flow_command "
        + COMMAND
    )
