from __future__ import annotations

COMMAND = 'edit-insert-shape'
FAMILY = 'bbox'
CLAIM_ID = 'V-w2-edit-insert-shape'
SCHEMA_VERSION = "1.0"

def decide_row(f0: str, f1: str, f2: str, f3: str) -> str:
    family = FAMILY

    right = int(f0); page_w = int(f1); bottom = int(f2); page_h = int(f3)
    if min(right, page_w, bottom, page_h) <= 0:
        return "USAGE"
    if right > page_w or bottom > page_h:
        return "BBOX_OOB"
    return "BBOX_OK"

    raise ValueError(family)
