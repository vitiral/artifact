pub static DATA: &'static str = r##"
[REQ-cmd]
partof = "REQ-purpose"
text = '''
The interface **will** be a simple command-line one that can be called
with a path to the questions to ask. The program will go through all the
questions, prompting the user with a question and return whether the response
was valid or not.
'''

[SPC-response]
partof = "REQ-purpose-learning"
text = '''
when an answer is correct, a happy message **shall** be displayed. Otherwise,
an error message with the correct answer **shall** be displayed
'''

[SPC-random]
partof = "REQ-purpose-learning"
text = '''
The questions **shall** be presented randomly
'''

[SPC-weighted]
partof = "REQ-purpose-learning"
text = '''
The questions **shall** be given a higher weight when they are missed
'''
"##;
