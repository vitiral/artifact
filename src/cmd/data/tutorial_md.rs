pub static data: &'static str = r##"
##################################################
# Tutorial Stage 1: introduction to rsk
> Run `rsk tutorial` to start this stage of the tutorial
Congradulations! You have installed rsk!

A tutorial file has been created for you in this directory, open the file
`tutorial.rsk` in your text editor of choice and read through it,
following all the directions.

When you are finished, go to the next section.

##################################################
# Tutorial Stage 2: high-level requirements and design specifications
> Run `rsk tutorial 2` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `tutorial.rsk` has been removed
 - the `flash_card_challenge.htm` file has been added
 - the `docs/` folder has been added with `purpose.rsk` and `high_level.rsk` in it
 - `.rsk/settings.rsk` has been updated with a new `artifact_paths`
 
open `flash_card_challenge.htm` in a browser and skim through the project 
that we will be executing. Don't worry! You don't need to know python to 
follow along with this tutorial.

Now open `docs/purpose.rsk`. This is a rough attempt to translate the ideas
in `flash_card_challenge.htm` into what are known as "high-level requirements"

High-level requirements are important because they document why your
project even exists -- something that is important to know as you develop it!
Without high-level requirements, it is easy to loose sight of what your project
is trying to accomplish and can be difficult to keep track of which features
are useful and which are not.

> ## Exercise 1:
> Review `docs/purpose.rsk` and make sure it makes sense. Think about things you think should
> be added to the purpose documentation, and feel free to make changes.
> You can always return it to it's original state with `rsk tutorial 2`

Now open `high_level.rsk` in the same directory. This is mostly the high-level 
specification of the command/program itself.

High-level specifications allows you to lay out your ideas for how a project
should be structued before you actually write any code. It also allows you to write out
"TODOs" that you think **should** be done, but you maybe won't get done in your
first iteration.

> ## Exercise 2:
> Revew the `docs/high_level.rsk` document. Which items do you think should be done 
> immediately, and which will have to wait? Feel free to make any edits you think 
> should be made. 
> 
> ### Some thoughts in answer:
> Probably `SPC-cmd-load` and `SPC-cmd-response` has to be done immediately, so that 
> an initial implementation can be shown to work, but both `SPC-cmd-random`
> and `SPC-cmd-weighted` can wait for at least a little bit.

Now run:
```
    rsk ls
```
    
This displays all the artifacts you just looked at, but colorizes them according
to whether they are complete or not. Right now, nothing is done so all you
see is red.

Now run:
```
    rsk ls REQ-cmd -l
```
    
This calls to list only one artifact, and displays it in the "long" format

> Note: try `rsk ls cmd -p` to search for all items with "cmd" in the name

> ## Exercise 3:
> Play around with the `rsk ls` command a little more to get used to it, we will
> be using it alot. Get help with:
>     `rsk ls -h`
    
Once you are done, continue onto stage 3.

##################################################
# Tutorial Stage 3: detailed design and test design of the loading function
> Run `rsk tutorial 3` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `docs/load.rsk` has been created

> ## Exercise 1:
> Read through `docs/load.rsk` and see if the general plan makes sense to you.
> What would you change? Feel free to make any edits you think should be
> made. You can always return it to it's original state with `rsk init -t 3`

The first task we are going to address is how we load the questions into
the program. This is all defined under `SPC-cmd-load`. Run:
```
    rsk ls SPC-load -l
```

From this you can see the parts that have to be implemented for `SPC-load`
to be considered done.

> ## Exercise 2:
> Explore each part of SPC-load using the `rsk ls` cmd.

This document details quite a bit of the design specifications, risks and tests
in order to create this function. Let's actually get to work and start coding.


##################################################
# Tutorial Stage 4: writing and linking code
> Run `rsk tutorial 4` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `flash/` has been created with two files, `__init__.py`
     and `load.py`
 - `.rsk/settings.rsk` was updated to include the `code_paths` variable
 
> Note: for python, a directory with an `__init__.py` file is called a "module"
> and is python's packaging mechanism.

Take a look at `flash/load.py`. This contains the machinery for loading our file.
Notice the various `#ART-*` tags located in the documentation. This is how we mark
that an artifact is "completed". Note that only `SPC` and `TST` artifacts can be
completed in this way.

There are two ways that an artifact is considered "done":
 - if it is a SPC or TST and has a #ART tag in source
 - if it's parts are done
 
Additionally, an artifact is only considered "tested" when it's TST parts are 
considered done.

Run the command

    rsk ls SPC-load-format

Notice that it is now "implemented-at" `flash/load.py`.

Head to `flash/tests/test_load.py` and notice that similar tags an be found there
for TST artifacts.

## Exercises
 1. run `rsk ls ARTIFACT` on an artifact that is tagged in source. Now change the tag
      so that it is mispelled and run it again. Did the completeness change?
 2. do the same thing for an arifact in the `partof` field. Notice that invalid names
      blink red on your terminal.


##################################################
# Summary and Final Words

This tutorial took you part of the way through developing a simple product using
requirements. I leave it as a exercise for the reader to finish the project in
whichever language you are most comftorable. Have some fun with the rsk tool,
trying to break it or find bugs. If you find any, please post them at: 
https://github.com/vitiral/rsk/issues
"##;
