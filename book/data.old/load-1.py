#!/usr/bin/python2
'''
csv loading module
'''
import csv


class Question(object):
    ''' represents a question and can be asked

    partof: #SPC-question
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



def validate_questions(questions):
    ''' given a list of questions, validate them according to spec
    partof: #SPC-validate
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
    partof: #SPC-load
    '''
    with open(path, 'rb') as f:
        return load_io(f)
