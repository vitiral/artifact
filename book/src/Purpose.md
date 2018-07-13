One of the most critical pieces of documentation is your purpose documentation.
Without purpose documentation, it is easy to get lost and forget what your
project was trying to accomplish.

We are going to start by writing our requirements in our `README.md`.  Later we
are going to use artifact to track them.

Open up your `README.md` in your favorite plain-text editor and write out
something like the following:

```
## Purpose:
Write a flash card quizzer from scratch and learn about
quality best practices while doing so.

The example tutorial can be found here:
    http://wiki.openhatch.org/Flash_card_challenge

It should be easy for users to input questions to the
quizzer in a simple and open format. Additionally, the
quizzer should use effective memorization techniques.
Some possible ideas include:
- asking items in a random order
- telling the correct answer after the user answers incorrectly
- asking items more often if they were answered incorrectly
- allowing users to configure time limits, so they can
  compare results between quizzes.
```

Notice that we try to keep our purpose documentation as brief and simple as
possible. This is important for all documents, but is especially important for
high level docs. There is a basic rule: documentation that is not brief and
clear will not be read. You want to keep your docs readable, otherwise they
will only weigh down your project.

> ##### Exercise 1:
> In your `README.md`, break down the purpose documentation above into some high
> level requirements. Then give a high level specification for how you would
> approach those requirements. What programming language would you use? What
> libraries would you use? What would be your overall approach to each problem?

> ##### Exercise 2:
> Assume your program user interface would be over the command line. What kind
> of arguments and user configuration would you accept? Would you let the user
> use only one quiz file at a time, or use multiple of them? Write down your
> answers.

> ##### Exercise 3:
> Skim through the [markdown format][1] specification. Markdown is a great
> format for keeping docs, since it allows you to write docs in plain text
> (so they can be revision controlled) but the simple formatting rules
> render beautifully on sites like github.
>
> Markdown is easy to learn, easy to read and has become the defacto standard
> for writing docs for open source projects. It is worth learning!

[1]: https://gitbookio.gitbooks.io/markdown/content/
