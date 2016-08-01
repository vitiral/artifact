pub static data: &'static str = r##"
[REQ-purpose]
text = '''
Write a flash card quizzer from scratch and learn about rsk
while doing so.

The example tutorial can be found here: http://wiki.openhatch.org/Flash_card_challenge
or at: {repo}/flash_card_challenge.htm

The program should be easy to understand and follow along so that it can
reach maximum audience for instructing in rsk
'''

[REQ-purpose-config]
text = '''
The command **shall** get the questions from a simple comma-separated
text file, allowing for any thing to be quizzed on easily
'''

[REQ-purpose-learning]
text = '''
The flash program should do things in a way that is most condusive to learning, such
as:
 - doing items in a random order
 - doing missed items more often
 - telling the answer after a guess is missed
'''
"##;
