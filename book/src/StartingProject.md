The primary teaching method that this book will employ is "learning by doing".
This is an approach that many developers are familiar with and is used in some
of the most effective tutorials on software development.

The project we will be implementing is the [flash card challenge][2] created
by Open Hatch. There are several reasons this project was chosen:
- It has a clear goal with targeted users.
- It is simple to define and yet can be extremely broad.
- The guide is written in python which largely reads like pseudo code.
  You should be able to follow along in any language.

One of the best tutorials on C (in my opinion), [learn C the hard way][3] has
this to say about itself:

> [This tutorial] teaches real robust C coding and defensive programming
> tactics on real hardware rather than abstract machines and pedantic theory.
> The book emphasizes breaking your code on purpose, and in the process teaches
> a plethora of important topics.

There are three important aspects to the "Learn The Hard Way" method that
this tutorial will use:
 1. It is designed for absolute beginners: you should know the basics of a
    programming language and revision control, but that's all you need.
 2. It focuses on teaching simple concepts and tools which deliver immediate
    real-world value.
 3. It uses exercises as the primary teaching method. You must
    **actually do the excersies** if you want to understand how the tools and
    processes you are reading about are useful.

## Before we start

Before we start, install the following tools:
- [git](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git)
- [artifact](https://github.com/vitiral/artifact/blob/master/docs/Installation.md)

Now run the following:
```
mkdir ~/learn-art # or whatever directory you want
cd ~/learn-art
git init
art init
```

This will set up your project as an artifact tutorial project and initialize
git.  It is your job to know how to use git as we progress through the
tutorial. I recommend committing the files you have now, including the `./art/`
directory that was created.

> ##### Exercise 1:
> Create a `README.md` file in this directory and take notes while you read the
> [flash card challenge][2] webpage. Pay close attention to:
> - what is the use case (how will this software be used)?
> - what are the inputs/outputs of the program?
>
> Then write a paragraph answering the question "how would I develop
> this application, knowing only what I know now?"

[1]: https://github.com/vitiral/artifact
[2]: http://wiki.openhatch.org/Flash_card_challenge
[3]: https://learncodethehardway.org/c/

