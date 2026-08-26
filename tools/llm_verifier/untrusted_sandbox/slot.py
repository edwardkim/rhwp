"""Closed slot set for where document-derived text may be placed.

Allowed: user-facing display, or an LLM data block wrapped with a nonce
boundary. Everything else is an instruction/criteria surface and is blocked.
"""

from __future__ import annotations

from enum import Enum


class Slot(str, Enum):
    CRITERIA = "criteria"
    SYSTEM_PROMPT = "system_prompt"
    TOOL_ARG_PATH = "tool_arg_path"
    TOOL_NAME = "tool_name"
    SHELL_COMMAND = "shell_command"
    URL_BODY = "url_body"
    RUN_PLAN = "run_plan"
    AUTHORIZATION = "authorization"
    USER_DISPLAY = "user_display"
    LLM_DATA_BLOCK = "llm_data_block"

    @property
    def is_allowed(self) -> bool:
        return self in ALLOWED_SLOTS

    @property
    def is_instruction(self) -> bool:
        return self in INSTRUCTION_SLOTS

    @property
    def is_criteria(self) -> bool:
        return self is Slot.CRITERIA


SLOT_VALUES: tuple[str, ...] = tuple(s.value for s in Slot)

ALLOWED_SLOTS: frozenset[Slot] = frozenset(
    {
        Slot.USER_DISPLAY,
        Slot.LLM_DATA_BLOCK,
    }
)

INSTRUCTION_SLOTS: frozenset[Slot] = frozenset(
    {
        Slot.CRITERIA,
        Slot.SYSTEM_PROMPT,
        Slot.TOOL_ARG_PATH,
        Slot.TOOL_NAME,
        Slot.SHELL_COMMAND,
        Slot.URL_BODY,
        Slot.RUN_PLAN,
        Slot.AUTHORIZATION,
    }
)

# Why each instruction slot cannot hold document-derived text.
SLOT_REASONS: dict[Slot, str] = {
    Slot.CRITERIA: "document_text_as_verification_criteria",
    Slot.SYSTEM_PROMPT: "document_rewrites_agent_rules",
    Slot.TOOL_ARG_PATH: "document_chooses_path",
    Slot.TOOL_NAME: "document_chooses_tool",
    Slot.SHELL_COMMAND: "document_writes_shell",
    Slot.URL_BODY: "document_chooses_destination",
    Slot.RUN_PLAN: "document_writes_run_plan",
    Slot.AUTHORIZATION: "document_approves_itself",
    Slot.USER_DISPLAY: "user_facing_display",
    Slot.LLM_DATA_BLOCK: "nonce_wrapped_data_block",
}


def parse_slot(raw: str) -> Slot:
    try:
        return Slot(raw)
    except ValueError as exc:
        raise ValueError(f"unknown slot: {raw!r}") from exc
