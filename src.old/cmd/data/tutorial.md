# Interactive artifact tutorial
Welcome to the in depth artifact tutorial! This tutorial is designed to be run
interactively, guiding you through the requirements gathering, detailed design
phase, and implementation of a project -- as well as fixing issues that might
come up.

In order to follow along, you must have artifact installed somewhere on your
`PATH`. Check out the [Installation Guide][1] for instructions.

Feel free to keep notes in this file as you progress through the tutorial
(it will not be deleted).

Note: every time `art tutorial ...` gets called it will delete the files it
created (except this one and `tutorial.toml`). This is so that it can update the
files to be interactive. If you are taking notes or creating other design docs,
you should do so in separate files than the ones created, or use revision
control like git as you progress (recommended).

[1]: https://github.com/vitiral/artifact/blob/master/docs/Installation.md

--------------------------------------------------
## Tutorial Stage 1: follow along document
Start out by running `art tutorial` and read the created
[`tutorial.toml`](tutorial.toml) file that is created. This will give you an
overview of artifact syntax and how to write design docs

--------------------------------------------------
## Tutorial Stage 2: purpose documents
> **Run `art tutorial 2` to set the local directory to this stage**

A few changes have been made to your local directory:
 - the `flash_card_challenge.htm` file has been added
 - the `design/` folder has been added with [`purpose.toml`](purpose.toml)
 - [`.art/settings.toml`](settings-1.toml) has been updated with a new
   `artifact_paths`

> ### Exercise 1:
> Before we go any further, lets first explore the web-ui that you can use
> along side the command line tools. Type `art serve` and go to:
> http://127.0.0.1:5373/#artifacts/req-purpose
>
> I recommend doing most work in text files for this short interactive
> tutorial, but its good to know that the web-ui exists in case you find
> it helpful.
>
> Note that if you prefer to read this tutorial on the web, you can go to:
> https://github.com/vitiral/artifact/blob/master/src/cmd/data/tutorial.md

Open `flash_card_challenge.htm` in a browser (or go [here][2] and skim through
the project that we will be executing. Don't worry! You don't need to know
python to follow along with this tutorial.

Now open [`design/purpose.toml`](purpose.toml). This is a rough attempt to
translate the ideas in `flash_card_challenge.htm` into purpose statements.

Purpose statements are important because they document why your project even
exists -- something that is important to know as you develop it! Without
them, it can be easy to loose sight of what your project is trying to
accomplish and can be difficult to keep track of which features are
useful and which are not.

In addition, purpose statements allow you to specify what your project will
accomplish, but then complete it in pieces. **artifact** will help you track
which part is complete!

`purpose.toml` also contains the high level specifications of the program.
High-level specifications allows you to lay out your ideas for how a project
should be approached before you actually write any code. It also allows you to
write out "TODOs" that you think **should** be done, but you maybe won't get
done in your minimum viable product.

> ### Exercise 2:
> Review [`design/purpose.toml`](purpose.toml) and make sure it makes sense.
> Does this accurately summarize the application we are trying to build? Are
> there any purpose requirements missing?

Now run:
```
    art ls
```

This displays all the artifacts you just looked at, but colorizes them according
to whether they are complete or not. Right now, nothing is done so all you
see is red.

Now run:
```
    art ls SPC-cmd -l
```

This calls to list only one artifact (SPC-cmd), and displays it in the "long"
format (`-l`)

Try `art ls purpose -p` to search for all items with "purpose" in the
name, you can see that three purpose requirements appear.

> ### Exercise 3:
> Play around with the `art ls` command a little more to get used to it,
> we will be using it a lot. Get help with `art ls -h`

Once you are done, continue onto stage 3.


--------------------------------------------------
## Tutorial Stage 3: detailed design and test design of the loading function
> **Run `art tutorial 3` to reset the local directory to this stage**

A few changes have been made to your local directory:
 - [`design/load.toml`](load-1.toml) has been created

> ### Exercise 1:
> Read through [`design/load.toml`](load-1.toml) and see if the general plan
> makes sense to you. What would you change? Feel free to make any edits you
> think should be made. You can always return it to it's original state with
> `art tutorial 3`

The first task we are going to address is how we load the questions into
the program. This is all defined under SPC-load. Run:
```
    art ls SPC-cmd -l
```

From there you can see the parts that have to be implemented for SPC-load
to be considered done. Note that SPC-LOAD was auto-created because it is a
parent of other artifacts.

> ### Exercise 2:
> Explore each part of SPC-load using the `art ls` and/or `art serve` cmd

`load.toml` details quite a bit of the design specifications, risks and tests
in order to implement this project. Let's actually get to work and start
coding.


--------------------------------------------------
## Tutorial Stage 4: writing and linking code
> **Run `art tutorial 4` to start this stage of the tutorial**

A few changes have been made to your local directory:
 - The `flash/` directory has been created with:
    - two files, `__init__.py` and [`load.py`](load-1.py)
    - The `tests/` directory, containing `__init__.py`,
      [`test_load.py`](test_load.py) and [`example.csv`](test_data.csv)
 - [`.art/settings.toml`](settings-2.toml) was updated to include the
   `code_paths` variable

> Note: for python, a directory with an `__init__.py` file is called a "module"
> and is python's packaging mechanism.

Take a look at [`flash/load.py`](load-1.py), which contains the machinery for
loading the flash-cards file. Notice the various `#SPC-...` tags located in the
documentation strings. These tags are how artifact knows which artifacts are
implemented and where and can mark implemented artifacts as done.

Additionally, an artifact is only considered "tested" when it's TST parts are
considered done.

Run the command

    art ls SPC-load -l

Notice that it is now "defined-at" [`flash/load.py`](load-1.py). Go to
where it says it is implemented and confirm that the information is correct.

Head to [`flash/tests/test_load.py`](test_load.py) and notice that similar tags
can be found there for TST artifacts.

### Exercises
1. run `art ls ARTIFACT` on an artifact that is tagged in source. Now
   change the tag (i.e. `SPC-load` -> `SPC-format`) and run it again. Did the
   completeness change?
2. Do the same thing for an arifact in the `partof` field for a file in
  `design/`. Notice that invalid names blink red on your terminal and you get
   WARN messages. You can use this feature to help you ensure your artifact
   links are correct.
3. We will be learning about `art check` in the next step. Try it now with
   the changes you've made

--------------------------------------------------
## Tutorial Stage 5: handling errors
> **Run `art tutorial 5` to start this stage of the tutorial**

A few changes have been made to your local directory:
 - [`design/load.toml`](load-2.toml) has been changed to have a bunch of errors
 - [`src/load.py`](load-2.py) has been changed to include a few errors as well.

So far in the tutorial things have been done correctly -- but what if you
are new, or what if you have to refactor?

Here we are in the middle of refactoring our code and requirements a bit... but
we've messed some things up. It's your job to fix them. How to begin?

First of all, we can use what we already know. `art ls` can help a lot for
refactors. It can answer the question "why is that SPC at 0%? It is implemented
somewhere!"

Well, let's try it for this project:

```
    # note: -OD displayes "partof | defined-at" instead of "parts | defined-at"
    art ls -OD
```

Things don't look quite as done as they used to. In particular notice:
- `SPC-validate` is 100% tested but 0% done (that's not right!)
- `REQ-learning` is also 100% tested and 0% done
- `REQ-purpose` has droped from 75% done to only 25% done

`art ls` can help you do this kind of investigation, but if you are refacting
then tracing errors this way is tedious. Those artifacts used to be
implemented... isn't there some way to find where they used to be tagged?

There is, run `art check` is the commnad you want. It analyzes your project for
errors and displays them in a way that makes them easier to fix. Some of the
errors it finds are:
 - invalid `partof` fields: if you've renamed (or misspelled) an artifact but
    forgot to update artifacts that were parts of it, this will help you.
 - dangling locations in code: you might THINK writing `#SPC-awesome-func`
    in your code links to something, but unless that spec actually exists
    it isn't doing anything. `art check` has your back.
 - recursive links: artifact's completeness algorithm doesn't work if there are
    recursive partof links (i.e. A is partof B which is partof A)
    `art check` will help you narrow down where these are comming from.
 - hanging artifacts: if you've written a SPC but haven't linked
    it to a REQ, then you probably want to (otherwise what exactly are you
    specifying?). The same goes for tests that are not testing any specs or
    risks.

> ### Exercise:
> use `art check` to find errors and fix them. Keep running
> `art check` and fixing errors until there are no errors, then run
> `art ls` to see if the current status makes sense.

--------------------------------------------------
## Documenting and Hosting your own project
To start documenting your own project, run `art init` in your project and
edit `.art/settings.toml` with the paths on where to find your
design docs and code.

Have your build system export your design documents as html for easy viewing.
See: https://github.com/vitiral/artifact/blob/master/docs/ExportingHtml.md

--------------------------------------------------
## Additional Resources

This tutorial gave you a good feature overview of artifact but you are probably
hungry to know quality best practices (you are, aren't you?). No worries!
The author of this tool has written an EXTREMELY SHORT ebook for just that, in
which artifact plays a prominent role. Check it out here:
    https://vitiral.gitbooks.io/simple-quality/content/

Seriously, its completely free and like 9 pages. You owe it to yourself to at
least skim through it -- even if you are an experienced developer and already
know this stuff.

--------------------------------------------------
## Summary and Final Words

Here are a few parting words of advice:

1. You should always write a good README and other documentation for your users
   -- design docs SHOULD be used for bringing developers of your project up
   to speed but they aren't the best format for general users.
2. Keep your design docs fairly high level -- don't try to design every detail
   using artifact. Using artifact does not mean that you shouldn't use code
   comments!
3. Use `art ls` and `art check` often, and fix those error messages!
4. follow the [artifact best practices][3]
5. Don't be afraid to refactor your design docs. It is actually easier than it
   might sound, as the tool will help you find broken links and incomplete
   items in real time. Not to mention that if you use revision control
   (you should), your artifacts can be tracked with your project -- no more
   having your documentation and your code be wildly out of sync!

This tutorial took you part of the way through developing a simple project
using artifact. Try using artifact for one of your smaller personal projects and
see the benefits that design documents can give. Have some fun with the tool,
try to break it. If you find bugs or have any suggestions, please open a ticket
at: https://github.com/vitiral/artifact/issues

Good luck!

[2]: http://wiki.openhatch.org/Flash_card_challenge
[3]: https://github.com/vitiral/artifact/blob/master/docs/BestPractices.md
