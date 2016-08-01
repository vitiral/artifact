
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
 - the `docs/` folder has been added
 
open `flash_card_challenge.htm` in a browser and skim through the project 
that we will be executing. Don't worry! You don't need to know python to 
follow along with this tutorial.

Now open `docs/purpose.rsk`. This is a rough attempt to translate the ideas
in `flash_card_challenge.htm` into what are known as "high-level requirements"

High-level requirements are important because they document why your
project even exists -- someting that is important to know as you develop it!
Without high-level requirements, it is easy to loose sight of what your project
is trying to accomplish and can be difficult to keep track of which features
are useful and which are not.

> ## Exercise 1:
> Review `docs/purpose.rsk` and make sure it makes sense. Think about things you think should
> be added to the high-level, and feel free to make changes.
> You can always return it to it's original state with `rsk init -t 2`

Now open `high_level.rsk` in the same directory. This is mostly the high-level 
specification of the command/program itself.

High-level specifications allow you to lay out your ideas for how a project
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
    rsk ls SPC-cmd-load -l
```
    
This calls to list only one artifact, and dispaly it in the "long" format

> Note: try `rsk ls load -p` to search for all items with "load" in the name

> ## Exercise 3:
> Play around with the `rsk ls` command a little more to get used to it, we will
> be using it alot. Get help with:
>     `rsk ls -h`
    
Once you are done, continue onto stage 2.

##################################################
# Tutorial Stage 3: detailed design and test design of the loading function
> Run `rsk init -t 3` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `docs/load.rsk` has been created

> ## Exercise 1:
> Read through `docs/load.rsk` and see if the general plan makes sense to you.
> What would you change? Feel free to make any edits you think should be
> made. You can always return it to it's original state with `rsk init -t 3`

The first task we are going to address is how we load the questions into
the program. This is all defined under `SPC-cmd-load`. Run:
```
    rsk ls SPC-cmd-load -l
```

From this you can see the parts that have to be implemented for `SPC-cmd-load`
to be considered implemented.

> ## Exercise 2:
> Explore each part of SPC-cmd-load using the `rsk ls` cmd.

This document details quite a bit of the design specifications, risks and tests
in order to create this function. Let's actually get to work and start coding.


##################################################
# Tutorial Stage 4: writing and linking code
> Run `rsk init -t 4` to start this stage of the tutorial

A few changes have been made to your local directory:
 - `flash/` has been created with two files, `__init__.py`
     and `load.py`
 - `docs/settings.rsk` has been created with a few variables defined
 
> Note: in python a directory with an `__init__.py` file is called a "module"
> and is python's packaging mechanism.

Take a look at `flash/load.py`. This contains our loading function
