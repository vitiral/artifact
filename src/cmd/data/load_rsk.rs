pub static data: &'static str = r##"
[REQ-question-duplicate]
partof = "REQ-purpose-learning"
text = '''
duplicates **shall not** be allowed, as they would be extremely confusing
'''

[REQ-load]
text = '''
loading **shall** be from a csv file in a simple format. This is where
the questions will be gotten from
'''

[SPC-question]
text = '''
The `Question` class **shall** be the primary datatype used for questions
in the application. Quetions shall:
 - store the question and answer
 - provide a method `ask` to ask the user the question
     and validate the answer
'''

[SPC-load-format]
text = '''
The format of the csv file **shall** be a csv file of the form:
```
City, Capitol
```

Where whitespace is ignored
'''

[SPC-load-validate]
partof = "REQ-question-duplicate, SPC-load-format"
text = '''
input questions **shall** be validated to meet the
linked requirements, printing and returning an error
if they are not met.
'''

[RSK-load]
partof = "REQ-load"  # RSK links must be explicit
text = '''
A user could give invalid data. Input data must be checked
for validity and an error must be raised if invalid.
'''
[TST-load]
partof = "RSK-load"
text = '''
Plaintext tests related to loading the questions.
> Note: this is automatically linked to SPC-load,
> but RSK links must be explicit
'''
[TST-load-unit-colums]
partof = "SPC-load-validate"
text = "test invalid number of colums"
[TST-load-unit-duplicate]
partof = "SPC-load-validate"
text = "test duplicate names"

[TST-load-csv]
partof = "SPC-load-format"
text = '''
lesser tests that validate that a full csv file load works
(other funcationality validated by TST-load-unit)
'''
"##;
