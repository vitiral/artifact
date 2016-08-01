pub static data: &'static str = r##"#TUTORIAL=true
import os
import unittest
from StringIO import StringIO

from .. import load

script_dir = os.path.split(__file__)[0]


class TestLoadIo(unittest.TestCase):
    def test_basic(self):
        ''' partof: #TST-load-csv '''
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
        ''' see: TST-load-csv '''
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
        ''' partof: #TST-load-unit-colums '''
        # extra ',' after 1
        text = '''\
        one,1,
        two,2
        three,3'''
        with self.assertRaises(ValueError):
            load.load_io(StringIO(text))

    def test_duplicate(self):
        ''' partof: #TST-load-unit-duplicate '''
        # extra ',' after 1
        text = '''\
        one,1,
        two,2
        three,3
        two,2'''
        with self.assertRaises(ValueError):
            load.load_io(StringIO(text))

    def test_valid_line_ending(self):
        # the last line should be able to end with \n
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
"##;
