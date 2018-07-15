# Simple Quality
*A short guide to quality best practices for developers.*

By Garrett Berg <vitiral@gmail.com>

## Introduction
This is a short and open-source leaflet aimed at helping software developers
improve their software quality. It is also the primary user guide for the design
documentation tool [artifact][4]. Its targeted audience are those who:
- Know at least one programming language.
- Know revision control. If you don't know one, learn git.
- Want a brief guide on how to have fewer bugs, re-designs and headaches
  in all of their projects.

This book is shiped as part of the artifact Web UI and can also be read at
(**TODO: add link**).

If you have suggestions or edits, please [open a ticket](./Feedback.html).

The goal of this book is to make developing software simpler and more fun. Think
back to the time you first learned revision control. Think about how you were
backing up files before then? Maybe you were copying folders to some `backups`
folder?  Maybe you were not backing up at all?

However you did (or didn't) track your changes, think about your life before and after
revision control. Things were a lot different, and all you had to learn in order
to use revision control were:
- Some simple vocabulary.
- An easy to use tool.
- A new way of looking at things.

This is the central premise of this book: you don't need to understand technical
jargon or complex test methodologies to realize huge gains in the quality of
your software. Some simple vocabulary, new tools and new ways of looking at
things are all you need.

> All code/documentation examples in this book are public domain. The rest of
> the book is licensed under the [GNU GPLv3 License][3].

[2]: https://github.com/vitiral/simple-quality/issues
[3]: https://www.google.com/search?q=gnu+gpl+v3&ie=utf-8&oe=utf-8
[4]: https://github.com/vitiral


## Why This Book Exists
There is a lack of good documentation unifying and extending quality best
practices. While there have been several advancements made in how to develop
quality software, they have largely been piecemeal and lacked the tools to
fit them all together. Several of the recent advancements include:
- better revision control tools and best practices
- new emphasis on unit tests as part of the development process
- linters and auto-formatters to help projects have easy to read code
- more emphasis on documentation, including inline developer documentation
- the agile process and associated tools

One of the things in common about all of these: developers tend to agree
that their lives are **better** after using them. Programming is easier,
more fun and more fulfilling. However, all of these lack the processes,
tools and vocabulary necessary to bring them together into an organized whole.

This book will give you the knowledge of how to unify and track all these quality
best practices as well as give you an intro to artifact, the open-source documentation
tool artifact. The tools presented in this book will not only allow you to write
better software, but will help your day-to-day workflow in the same ways that
good unit tests do.




