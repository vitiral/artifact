# High Level Design

Once you know the purpose of your project, it is important for you to write
down the approach you plan to take. This is important because:
- There may be gaps when you don't work on your project. If you go on vacation
  for a month, having a reference of your thoughts at the time you were focused
  can jumpstart your productivity.
- It is important to be able to reference a design doc for new contributors and
  newbie developers.

Your high level requirements should go in your `README.md`, just below your purpose
section:

```markdown
# Execution Method
The minimum viable product shall be a command line utility
that is given the path to one or more question files as
arguments

Additional arguments will include:
- `-t`: specify the time allowed for each question
- `-T`: specify the total time allowed for the whole quiz
- `-r NUM`: repeat questions only a certain number of times.
    By default there is no limit

The program will ask one question at a time, recording how
many answers the user got correct/incorrect and weighting
future questions accordingly.

When the program is complete it will report:
- time taken, broken up by whole quiz and each question
- the user's score


# Final Results
When the program is complete a report shall be printed with:
- time taken, broken up by whole quiz and each question
- the user's total score
- questions ranked by ones the user had the most difficulty


# Question File Format
The user shall be able to easily configure the quiz
questions through a simple csv format consisting of two
columns: the question and the answer.
```

Again, just like the purpose documentation, this documentation aims to be
brief and help you during your design process.

> ### Exercise 1:
> What are some other items that we can detail at a high level?
> Try writing them out yourself in this section.
