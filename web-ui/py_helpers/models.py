"""Representations of data."""

import aenum

strty = basestring


class Model(object):
    """Represents data that can be de/serialized."""
    # a list of (type, required, cls-attr, dict-key)
    # the type can be a tuple of (deserializer, serializer)
    ATTR_MAP = ()

    @classmethod
    def deserialize(cls, dct):
        """Deserialize a dict into the Model."""
        def get_attr(m):
            """given a map instance, return the attr."""
            ty, required, m = m[0], m[1], m[2:]

            def getter(k):
                """maybe get from dict."""
                if required:
                    v = dct[k]
                else:
                    v = dct.get(k)
                if v is None:
                    assert not required, "value is None for required type"
                    return v
                return v
            if len(m) == 1:
                a = m[0]
                v = getter(a)
            elif len(m) == 2:
                a, k = m
                v = getter(k)
            else:
                assert False, "invalid map item"
            if v is not None:
                if isinstance(ty, tuple):
                    v = ty[0](v)
                elif issubclass(ty, Model):
                    v = ty.deserialize(v)
                else:
                    assert isinstance(
                        v, ty), "invalid type: {} | {}".format(v, ty)
            return (a, v)

        attrs = [get_attr(m) for m in cls.ATTR_MAP]
        return cls(**dict(attrs))

    def serialize(self):
        """Serialize the Model into a dict."""
        dct = {}

        def set_attr(m):
            """given a map instance, return the attr."""
            ty, m = m[0], m[2:]
            if len(m) == 1:
                k = m[0]
                v = getattr(self, k)
            elif len(m) == 2:
                a, k = m
                v = getattr(self, a)
            else:
                assert False, "invalid map item"
            if v is not None:
                if isinstance(ty, tuple):
                    v = ty[1](v)
                elif issubclass(ty, Model):
                    v = v.serialize()
            dct[k] = v
        for m in self.ATTR_MAP:
            set_attr(m)
        return dct


class ArtifactToml(Model):
    """Artifact as represented in .toml files."""
    ATTR_MAP = [
        (strty, False, 'partof'),
        (strty, False, 'done'),
        (strty, False, 'text'),
    ]

    def __init__(self, partof, done, text):
        self.partof = partof
        self.done = done
        self.text = text


class Type(aenum.Enum):
    """Artifact type."""
    req = "REQ"
    spc = "SPC"
    tst = "TST"


class Name(Model):
    """An artifact name object."""

    def __init__(self, raw, value, ty):
        self.raw = raw
        self.value = value
        self.ty = ty

    @classmethod
    def deserialize(cls, dct):
        raw = dct
        value = raw.upper()
        return cls(raw, value, Type(value[:3]))

    def serialize(self):
        return self.raw

    def __repr__(self):
        return self.serialize()

    def __eq__(self, other):
        return self.value == other.value

    def __ne__(self, other):
        return not self == other


def deserialize_names(names):
    """Deserialize a list of names."""
    return [Name.deserialize(n) for n in names]


def serialize_list(ls):
    """serialize a list of models."""
    return [i.serialize() for i in ls]


NameList = (deserialize_names, serialize_list)


class Loc(Model):
    """Code implementation location."""

    ATTR_MAP = [
        (strty, True, 'path'),
        (int, True, 'line',),
    ]

    def __init__(self, path, line):
        self.path = path
        self.line = line

    def __eq__(self, other):
        return self.path == other.path and self.line == other.line

    def __ne__(self, other):
        return not self == other


class Artifact(Model):
    """Python representation of an artifact."""

    ATTR_MAP = [
        (int, True, 'id'),
        (int, True, 'revision',),
        (Name, True, 'name', 'name'),
        (strty, True, 'definition', 'def'),
        (strty, True, 'text', 'text'),
        (NameList, True, 'partof', 'partof'),
        (NameList, True, 'parts', 'parts'),
        (Loc, False, 'code', 'code'),
        (strty, False, 'done', 'done'),
        (int, True, 'completed', 'completed'),
        (int, True, 'tested', 'tested'),
    ]

    def __init__(
            self,
            id,
            revision,
            name,
            definition,
            text,
            partof,
            parts,
            code,
            done,
            completed,
            tested):
        self.id = id
        self.revision = revision
        self.name = name
        self.definition = definition
        self.text = text
        self.partof = partof
        self.parts = parts
        self.code = code
        self.done = done
        self.completed = completed
        self.tested = tested
