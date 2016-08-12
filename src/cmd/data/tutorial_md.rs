pub static DATA: &'static str = r##"
This is 2nd and 3rd part of the tutorial. If you would like to reset the
directory at the intro run `rst tutorial`

Note: every time `rst tutorial ...` gets called it will delete the files it
created. This is so that it can update the files to be interactive. If you are
taking notes or creating other artifacts, you should do so in separate files
than the ones created.


##################################################
# Tutorial Stage 2: high-level requirements and design specifications
> Run `rst tutorial 2` to reset the local directory to this stage

A few changes have been made to your local directory:
 - `tutorial.toml` has been removed
 - the `flash_card_challenge.htm` file has been added
 - the `reqs/` folder has been added with `purpose.toml` and `high_level.toml`
     in it
 - `.rst/settings.toml` has been updated with a new `artifact_paths`

Open `flash_card_challenge.htm` in a browser and skim through the project
that we will be executing. Don't worry! You don't need to know python to
follow along with this tutorial.

Now open `reqs/purpose.toml`. This is a rough attempt to translate the ideas
in `flash_card_challenge.htm` into purpose statements.

Purpose statements are important because they document why your project even
exists -- something that is important to know as you develop it! Without
high-level requirements, it is easy to loose sight of what your project is
trying to accomplish and can be difficult to keep track of which features are
useful and which are not.

In addition, purpose statements allow you to specify what your project will
accomplish, but then complete in pieces. **rst** will help you track which part
is complete!

> ## Exercise 1:
> Review `reqs/purpose.toml` and make sure it makes sense. Think about things
you > think should be added to the purpose documentation and make notes or add
> artifacts in a separate file. You can always return it to it's original state
> with `rst tutorial 2`

Now open `high_level.toml` in the same directory. This is mostly the high-level
specifications and requirements of the command/program itself.

High-level specifications allows you to lay out your ideas for how a project
should be structued before you actually write any code. It also allows you to
write out "TODOs" that you think **should** be done, but you maybe won't get
done in your first iteration.

> ## Exercise 2:
> Review the `reqs/high_level.toml` document. Which items do you think should be
> done immediately, and which will have to wait?

Now run:
```
    rst ls
```

This displays all the artifacts you just looked at, but colorizes them according
to whether they are complete or not. Right now, nothing is done so all you
see is red.

Now run:
```
    rst ls REQ-cmd -l
```

This calls to list only one artifact (REQ-cmd), and displays it in the "long"
format (`-l`)

Try `rst ls cmd -p` to search for all items with "cmd" in the name. In this case
there is only one.

> ## Exercise 3:
> Play around with the `rst ls` command a little more to get used to it, we will
> be using it a lot. Get help with:
>     `rst ls -h`

Once you are done, continue onto stage 3.


##################################################
# Tutorial Stage 3: detailed design and test design of the loading function
> Run `rst tutorial 3` to reset the local directory to this stage

A few changes have been made to your local directory:
 - `reqs/load.toml` has been created

> ## Exercise 1:
> Read through `reqs/load.toml` and see if the general plan makes sense to you.
> What would you change? Feel free to make any edits you think should be
> made. You can always return it to it's original state with `rst init -t 3`

The first task we are going to address is how we load the questions into
the program. This is all defined under `SPC-cmd-load`. Run:
```
    rst ls SPC-load -l
```

From this you can see the parts that have to be implemented for `SPC-load`
to be considered done. Note that SPC-load was auto-created because it is a
parent of other artifacts.

> ## Exercise 2:
> Explore each part of SPC-load using the `rst ls` cmd.

This document details quite a bit of the design specifications, risks and tests
in order to create this function. Let's actually get to work and start coding.
That is the focus of the next part


##################################################
# Tutorial Stage 4: writing and linking code
> Run `rst tutorial 4` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `flash/` has been created with two files, `__init__.py`
     and `load.py`
 - `.rst/settings.toml` was updated to include the `code_paths` variable

> Note: for python, a directory with an `__init__.py` file is called a "module"
> and is python's packaging mechanism.

Take a look at `flash/load.py`. This contains the machinery for loading our
file. Notice the various `#SPC` tags located in the documentation strings. This
is how rst knows which artifacts are implemented and where. If an artifact is
implemented in code in this way it is marked as 100% "completed".

> Note: only `SPC` and `TST` artifacts can be completed in this way.

There are two ways that an artifact is considered "done":
 - if it is a SPC or TST and has a #ART tag somewhere in source code
 - if it's `parts` are done

Additionally, an artifact is only considered "tested" when it's TST parts are
considered done.

Run the command

    rst ls SPC-load-format

Notice that it is now "implemented-at" `flash/load.py`. Go to where it says it
is implemented and confirm that the information is correct.

Head to `flash/tests/test_load.py` and notice that similar tags can be found
there for TST artifacts.

## Exercises
 1. run `rst ls ARTIFACT` on an artifact that is tagged in source. Now change
    the tag so that it is mispelled and run it again. Did the completeness
    change?
 2. do the same thing for an arifact in the `partof` field for a file in
   `reqs/`. Notice that invalid names blink red on your terminal and you get
    WARN messages. You can use this feature to help you ensure your artifact
    links are correct.


##################################################
# Tutorial Stage 5: handling errors
> Run `rst tutorial 5` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `reqs/load.toml` has been changed to have a bunch of errors
 - `src/load.py` has been changed to include a few errors as well.

So far in the tutorial things have been done correctly -- but what if you
are new, or what if you have to refactor?

Here we are in the middle of refactoring our code and requirements a bit... but
we've messed some things up. It's your job to fix them. How to begin?

First of all, we can use what we already know. `rst ls` can help a lot for
refactors. It can answer the question "why is that REQ at 0%? It is implemented
somewhere!"

There is a cmd specifically for finding errors in a project,
and that is `rst check`

`rst check` analyzes your project for errors. Some errors it finds are:
 - invalid `partof` fields: if you've renamed (or misspelled) an artifact but
    forgotten to update artifacts that were parts of it, this will help you.
 - dangling locations in code: you might THINK writing #SPC-awesome-func
    in your code links to something, but unless that spec actually exists
    it isn't doing anything. `rst check` has your back.
 - recursive links: rsk's completeness algorithm doesn't work if there are
    recursive partof links (i.e. A is partof B which is partof A)
    `rst check` will help you narrow down where these are comming from
 - hanging artifacts: if you've written a SPC but haven't linked
    it to a REQ, then you probably want to (otherwise what exactly are you
    specifying?). The same goes for tests that are not testing any specs.

> ## Exercise:
> use `rst check` to find errors and fix them. Keep running `rst check` until
> there are no errors, then run `rst ls` to see if the current status makes
> sense.

Feel free to explore at this point -- the interactive tutorial is over so this
is your project now. Enjoy!


##################################################
# Documenting your own project
To start documenting your own project, run `rst init` in your project and edit
`.rst/settings.toml` with the paths to find your code-implementations and
documents.


##################################################
# Additional Resources
It is the plan of this author to write a "hacker's intro to requirements" book
in the github wiki of this project. Until then, here are some useful references

## Wikipedia page:
> https://en.m.wikipedia.org/wiki/Software_requirements_specification

This is an excellent introduction to the purpose of writing requirements
as well as an initial template for how you might structure your requirements.
Well worth the read for any developer.


##################################################
# Summary and Final Words

Here are a few parting words of advice:

 1. Still write a good README and other documentation for your users --
      requirements SHOULD be used for bringing developers of your project up
      to speed but they aren't the best format for general users.
 2. Keep your artifacts fairly high level -- don't try to design every detail
      using rst. Using rst does not mean that you shouldn't use code comments!
 3. Use `rst ls` and `rst check` liberally, and fix those error messages!
 4. Keep names short and simple. Avoid unnecessary nesting. If you have web and
      cmdline ui elements, consider naming them just `REQ-web` and `REQ-cmd`
      instead of `REQ-ui-web` and `REQ-ui-cmd`. Trying to nest too deep can
      quickly get confusing.
 5. Don't be afraid to refactor your artifacts. It is actually easier than it
      might sound, as rst will help you find broken links and incomplete
      items in real time. Not to mention that if you use revision control
      (you should), your artifacts can be tracked with your project -- no more
      having your documents and your code be wildly out of sync!

This tutorial took you part of the way through developing a simple project using
**rst**. I leave it as an exercise to finish the project in whichever language
you are most comfortable. Have some fun with the rst tool, try to break it. If
you find bugs or have any suggestions, please open a ticket at:
https://github.com/vitiral/rst/issues

Good luck!
"##;
