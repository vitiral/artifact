import re

NAME_VALID_CHARS = "A-Z0-9_"
NAME_VALID_STR = r"(?:REQ|SPC|TST)-(?:[{0}]+-)*(?:[{0}]+)".format(NAME_VALID_CHARS)
NAME_VALID_RE = re.compile(r"^{}$".format(NAME_VALID_STR), re.IGNORECASE)

REQ = "REQ"
SPC = "SPC"
TST = "TST"

class Name:
    def __init__(self, ty, key, raw):
        self.ty = ty
        self.key = key
        self.raw = raw

    @classmethod
    def from_str(cls, text):
        match = NAME_VALID_RE.match(text)
        if not match:
            raise ValueError("Invalid name: {}".format(text))

        return cls(
            ty=match.group(1).upper(),
            key=text.upper(),
            raw=text)

    def __hash__(self):
        return hash(self.key)

    def __cmp__(self, other):
        if not isinstance(other, self.__class__):
            raise TypeError(type(other))
        return self.key.__cmp__(other.key)

    def __repr__(self):
        return self.raw
