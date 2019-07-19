# Parsing
from . import util

import re

NAME_VALID_CHARS = "A-Z0-9_"
NAME_VALID_STR = r"(?:REQ|SPC|TST)-(?:[{0}]+-)*(?:[{0}]+)".format(NAME_VALID_CHARS)

SUB_RE_KEY = "sub"
NAME_RE_KEY = "name"
NAME_SUB_RE_KEY = "name_sub"

NAME_VALID_RE = re.compile(r"^{}$".format(NAME_VALID_STR), re.IGNORECASE)

VALID_SUB_NAME_RE = re.compile(r"^\.(?:tst-)?[{}]+$".format(NAME_VALID_CHARS), re.IGNORECASE)

TEXT_SUB_NAME_STR = r"\[\[(?P<{}>\.(?:tst-)?[{}]+)\]\]".format(
        SUB_RE_KEY, NAME_VALID_CHARS)

TEXT_REF_STR = """
\[\[(?P<{1}>                # start main section
(?:REQ|SPC|TST)             # all types are supported
-(?:[{0}]+-)*               # any number of first element
(?:[{0}]+)                  # required end element
)                           # end main section
(?P<{2}>\.(?:tst-)?[{0}]+)? # (optional) sub section
\]\]                        # close text reference
""".format(NAME_VALID_CHARS, NAME_RE_KEY, NAME_SUB_RE_KEY)

TEXT_SUB_NAME_RE = re.compile(TEXT_SUB_NAME_STR, re.IGNORECASE)
TEXT_REF_RE = re.compile(TEXT_REF_STR, re.IGNORECASE | re.VERBOSE)
