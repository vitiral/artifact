# Implementation
Now that we have a basic design and test strategy, let's start writing some
code.

First make our python module:

```bash
mkdir flash/
touch flash/__init__.py
touch flash/load.py
```

This creates our python module and get's us started with some empty files.
Let's first implement `SPC-format.question` as an object in `flash/load.py`:

```python
#!/usr/bin/python2
'''
csv loading module
'''
import csv

class Question(object):
    ''' represents a question and can be asked

    partof: #SPC-format.question
    '''
    def __init__(self, question, answer):
        self.question = question.strip()
        self.answer = answer.strip().lower()

    def __eq__(self, other):
        if not isinstance(other, Question):
            return False
        return self.question == other.question and self.answer == other.answer

    def __neq__(self, other):
        return not self == other
```

Note that we do not have the `ask()` method yet though (we will do that later).

Now let's implement `SPC-format.validate`:

```python
def validate_questions(questions):
    ''' Given a list of questions, validate them according to spec
    partof: #SPC-format.validate
    '''
    # check for duplicates
    all_qs = [q.question for q in questions]
    seen = set()
    duplicates = []
    for q in all_qs:
        if q in seen:
            duplicates.append(q)
        seen.add(q)
    if duplicates:
        raise ValueError("duplicate questions found: {}".format(duplicates))
```

Finally, let's implement `SPC-format` itself -- the loading of the file.
```python
def load_io(f):
    ''' load questions from a file '''
    reader = csv.reader(f)
    questions = []
    for row in reader:
        if len(row) == 0 or (len(row) == 1 and not row[0].strip()):
            # skip if the row contains nothing but whitespace
            continue
        if len(row) != 2:
            raise ValueError("row is invalid length of {}: {}".format(
                len(row), row))
        questions.append(Question(*row))
    return questions


def load_path(path):
    ''' given a path, load a list of validated questions
    partof: #SPC-format
    '''
    with open(path, 'rb') as f:
        return load_io(f)
```

When we've finished with all of that, type `art ls`... and nothing is
implemented. This is because we still need to tell Artifact where we have
implemented stuff. Edit `.art/settings.toml` and add `"flash/"` to
the `code_paths` list.

```toml
code_paths = ["flash/"]
```

Now try:
```
$ art ls                                                                                                                                                                 ~/tmp/learn-art
spc% tst%  | name         | parts
20.0 0.0   | REQ-purpose  | SPC-cli, SPC-format, SPC-report
0.0  0.0   | SPC-cli      |
60.0 0.0   | SPC-format   |
0.0  0.0   | SPC-report   |
```

And notice that `SPC-format` is partially impelmented!


## Implementing Unit Tests
In order for SPC-format to be completely implemented and tested we need to impelemnt
it's `tst-` subarts

> Note: `tst-` subarts contribute to _both_ tst% and spc%, the idea being
> that implementing your unit tests are necessary to being actually done.

Let's implement some tests. First create our test files:

```bash
mkdir flash/tests
touch flash/tests/__init__.py
touch flash/tests/test_load.py
```

Then write our tests in `flash/tests/test_load.py`:
```python
import os
import unittest
from StringIO import StringIO

from .. import load

script_dir = os.path.split(__file__)[0]


class TestLoadIo(unittest.TestCase):
    def test_basic(self):
        """#SPC-format.tst-basic"""
        text = '''\
        one,1
        two,2
        three,3'''
        result = load.load_io(StringIO(text))
        expected = [
            load.Question("one", "1"),
            load.Question("two", "2"),
            load.Question("three", "3"),
        ]
        self.assertEqual(result, expected)

    def test_csv(self):
        """#SPC-format.tst-load"""
        path = os.path.join(script_dir, 'example.csv')
        result = load.load_path(path)
        expected = [
            load.Question("foo", "bar"),
            load.Question("forest", "ham"),
            load.Question("I", "love"),
            load.Question("you", "too"),
        ]
        self.assertEqual(result, expected)

    def test_invalid_columns(self):
        """#SPC-format.tst-invalid_cols"""
        # extra ',' after 1
        text = '''\
        one,1,
        two,2
        three,3'''
        with self.assertRaises(ValueError):
            load.load_io(StringIO(text))

    def test_duplicate(self):
        """#SPC-format.tst-duplicates"""
        # note: extra ',' after 1
        text = '''\
        one,1,
        two,2
        three,3
        two,2'''
        with self.assertRaises(ValueError):
            load.load_io(StringIO(text))

    def test_valid_line_ending(self):
        """The last line should be able to end with '\n'."""
        text = '''\
        one,1
        two,2
        three,3
        '''
        result = load.load_io(StringIO(text))
        expected = [
            load.Question("one", "1"),
            load.Question("two", "2"),
            load.Question("three", "3"),
        ]
        self.assertEqual(result, expected)
```

Notice that we also have an extra unit test. That's okay, not every test needs
a coresponding spec in the real world either!

# Summary
We have successfully implemented and tested one artifact (`SPC-format`), along
with all of its subarts. We did this by implementing it in source code.

View the current status using `art serve`.
[You can also see the example here](examples/part2/index.html).

Notice that `SPC-format` is considered both `spc%` and `tst%` complete and so
is green. Furthermore, when you look at it you can see the lines where it is
implemented.

