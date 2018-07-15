# Detailed Design

Now that we have our high level design, let's start desiging how we are
actually going to _build_ our flash card application. The first thing
we might want to design is: how does the user specify questions?

We already answered this to some degree in `SPC-format`. Let's expand it a bit.

```markdown
# SPC-format
partof:
- REQ-purpose
###
The user shall be able to easily configure the quiz
questions through a simple csv format consisting of two
columns: the question and the answer.

The format of the csv file **shall** be a csv file of the form:

    City, Capitol

> Note: whitespace will be ignored

## [[.question]]
The `Question` class shall be the primary datatype used for questions in the
application. Quetions shall:
- Store the question and answer.
- Provide a method `ask` to ask the user the question
  and validate the answer.

## [[.validate]]
Input questions **shall** be validated to:
- Guarantee against duplicates.
- Guarantee that the data format is correct.
```

There are a few things here, so let's take a look:
- We expanded _how_ the user can configure the questions (the CSV format itself)
- We created two subartifacts, `.question` and `.validate`.
  - Having the `[[.subart]]` anywhere within the text is enough to create
    these.
  - We can now link these subarts in code and they are necessary for our
    artifact to be considered "complete".

# Define Tests

Let's also define a couple of unit tests. You could do this using a new
`TST-format` (or any other name) artifact.

However, artifact has what are called tst-subarts specifically for the purpose
of defining unit test coverage that you want. Simply add the following section
to `SPC-format`:

```markdown
## Unit Tests:
- Invalid: make sure invalid inputs don't work
  - [[.tst-invalid_cols]]: Test invalid number of columns (0, 1, and 3).
  - [[.tst-duplicates]]: Test duplicate names.
- Make sure loading works.
  - [[.tst-basic]]: Loading a raw string and validating it.
  - [[.tst-load]]: Loading a valid csv file path and validating it.
```

These will allow us to implement testing for SPC-format without having to
create new artifacts.
