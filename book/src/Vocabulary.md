# Vocabulary

It is time we briefly discuss design vocabulary.

## Specifying meaning
It is very important that your design documents have clear meaning. Without
clear meaning, it is difficult to know your [definition of done][1].

- **Shall – Requirement:**  Shall is used to indicate something that is
  contractually binding, meaning it must be implemented and its
  implementation verified. Don’t think of “shall” as a word, but
  rather as an icon that SCREAMS “This is a requirement.”  If a statement
  does not contain the word “shall” it is not a requirement.
- **Will - Facts or Declaration of Purpose:** Will is used to indicate a
  statement of fact. Will statements are not subject to verification.
  "The application **will** be written in python" is an example.
  **Will** statements are meant to be notes to inform design
  or specify implementation details.
- **Should – Goals, non-mandatory provisions:** Should is used to indicate a
  goal which must be addressed by the design team but is not formally
  verified. Why include **should** statements? Because you may have a very
  important issue that you want to communicate to the developers, but can’t
  think of a way to do so in the form of a verifiable requirement. We have
  already seen an example in our purpose statement: "the flash quizzer
  **should** use effective memorization techniques". There is no way to
  validate that we are using the best methodologies, but we should aim for that
  goal.

**Reference**: these definitions are modified from an article at
[reqexperts.com][2] (October 9th, 2012. Lou Wheatcraft)

[1]: https://www.agilealliance.org/glossary/definition-of-done/
[2]: http://reqexperts.com/blog/2012/10/using-the-correct-terms-shall-will-should/

> ##### Exercise 1:
> Review the documentation we've written so far. When did we use "shall",
> "will" and "should"? Did we use them correctly?

## Testing your software
There are three main categories of testing every developer should know:
- [unit testing][10] which is testing isolated pieces of code
- [integration testing][11] which is testing modules of your code integrated
	together, but not the entire application
- [system testing][12], also known as end-to-end testing.

> ##### Exercise 2:
> Read at least the intro for all three wikipedia links above.

[10]: https://en.wikipedia.org/wiki/Unit_testing
[11]: https://en.wikipedia.org/wiki/Integration_testing
[12]: https://en.wikipedia.org/wiki/System_testing

### Testing Methodologies

The above specify which pieces of your software should be tested,
these specify how to determine what tests to write.

#### [Functional Testing][20]
What is the functionality we are programming for? It is important that
we test at least the basics. This includes the:
- [boundary conditions][21]: test the extreme inputs and outputs of your
    function
- typical use cases: test how the function or application will typically be used
    (i.e. a few values in the middle of the boundary conditions)
- error cases: make sure it throws an exception when invalid
    inputs are used. This should also test that your application can recover
    in case of recoverable faults.

#### [White Box Testing][22]
Look at your code. As the programmer, what kind of inputs are YOU concerned
about? Spend some time focusing on these.

You as the developer have the most insight into the internals of your
application, so you are probably the most qualified individual for trying to
break it. Always observe Murphy's law: **what can go wrong will go wrong.**
If you can see something that might break, even if the scenario seems
impossible for a user to hit, make sure it doesn't break anyway.

#### [Risk Based Testing][23]
What is the worst thing that your function/application could do. Can it
segfault? Can it recurse infinitely? Can it delete user data? Could it crash
the whole operating system? Can it introduce security vulnerabilities?

It's important to ask what the worst case scenarios are and test for them.
This is especially true if your program overwrites files or exposes a port
to the internet -- data loss and security vulnerabilities are serious
problems that can be introduced in the most simple software.


[20]: https://en.wikipedia.org/wiki/Functional_testing
[21]: https://en.wikipedia.org/wiki/Boundary_testing
[22]: https://en.wikipedia.org/wiki/White-box_testing
[23]: https://en.wikipedia.org/wiki/Risk-based_testing
