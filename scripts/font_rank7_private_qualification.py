#!/usr/bin/env python3
"""Run the bounded, local-only rank-7 Q3 qualification projection."""

from __future__ import annotations

import font_rank8_private_qualification as shared


shared.TARGET_FACE = "KoPubWorld돋움체 Light"
shared.EXACT_TTF_SHA256 = (
    "069494cce21a4222c88e537f256b6f46fee209375aba769f82431b2d382bc84f"
)
shared.Q0_SHA256 = "a15c7c4c51c08cd5c1251bda6551bc8e031a2360b3e1adf0246bb0712faf93bb"
shared.Q2_SHA256 = "159d712f648062a229df376e9fceaeef2b8d984364aa378f59c44d064cf7abfe"
shared.Q0_CANONICAL = (
    "a3be826dda103dd7ec69b3db9cb3b1529e82d55520bcdbeadd1bc7ddfaae1c42"
)
shared.Q2_CANONICAL = (
    "a84562e5a5a606cdc62fa8da3917df88ae85d40bc885d9b112f9a0d0a327a153"
)
shared.PUBLIC_KIND = "font-rank7-private-qualification-projection"
shared.PRIVATE_KIND = "font-rank7-private-qualification-detail"
shared.STAGE = "W8-R7-Q3"
shared.COHORT_KIND = "font-rank7-private-cohort"
shared.COHORT_DOCUMENTS = 5


if __name__ == "__main__":
    try:
        raise SystemExit(shared.main())
    except shared.OracleStage2Error as error:
        raise SystemExit(str(error)) from error
