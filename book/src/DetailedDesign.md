# Detailed Design

Now that we have our high level design, let's start desiging how we are
actually going to _build_ our flash card application. The first thing
we might want to design is: how does the user specify questions?

We already answered this to some degree in `SPC-format`. Let's expand it a bit.

```markdown
# SPC-format
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

Let's also define a couple of unit tests.

> Personally I like to create a single `TST-unit` which I link to all
> lower-level items and create subarts for each of them. This is one area where
> there could definitely be improvement in artifact (maybe .TST-subarts, i.e.
> `SPC-name.tst-invalid` which complete testing?)

```markdown
# TST-format

Unit tests for testing [[SPC-format]]:
- [[.invalid]]: At least the following unit tests **will** be implemented:
  - Test invalid number of columns (0, 1, and 3).
  - Test duplicate names.
- [[.load]]: The unit tests for load shall include:
  - Loading a raw string and validating it.
  - Loading a valid csv file path and validating it.
  - Try to load a csv file with 3 columns, expect error.
```

> Note that because `TST-format` has the same postfix as `SPC-format` they are
> automatically linked (no need to specify `partof`).
