# Artifact Intro

This is an introduction to how we start integrating the Artifact tool into our
design and development workflow. It is intended to be interactive, so please
follow along with everything installed!

> **Exercise 1: ensuring your environment works:**
>
> You should have at least done the [Starting Project](./StartingProject.html)
> chapter before attempting this one.
>
> Run `art ls`, you should see something like:
>
> ```
> spc% tst%  | name         | parts
> 0.0  0.0   | REQ-purpose  |
> ```

## Converting our README.md into an artifact.

The first thing we want to do is use the `README.md` file we have already been writing
as our artifact file.

To do this, let's make a couple of changes:
- Move our `README.md` into `design/purpose.md`
- Clean up the headers so they are artifacts.

To move your README.md, simply type:
```
mv README.md purpose.md
```

> **Check In:** run `art ls`. It shows nothing because we have not specified
> any artifacts.

We now need to convert our headers into artifacts. Let's start with our purpose.
Change the `# Purpose` line to `# REQ-purpose`. Your file should now look something like:

```markdown
# REQ-purpose
Write a flash card quizzer ...

```

Do the same thing to your specifications:
- `# Execution Method` -> `# SPC-cli`
- `# Final Results` -> `# SPC-report`
- `# Question File Format` -> `# SPC-format`

Now `art ls` should show:
```bash
$ art ls
spc% tst%  | name         | parts
0.0  0.0   | REQ-purpose  |
0.0  0.0   | SPC-cli      |
0.0  0.0   | SPC-format   |
0.0  0.0   | SPC-report   |
```

This is closer, but notice that none of them are linked. Let's fix that.

For `SPC-cli` make it look like this:

```markdown
# SPC-cli
partof:
- REQ-purpose
###
The minimum viable product ...
```

Do the same for `SPC-format` and `SPC-report`, also making them partof
`REQ-purpose`. You should now have:
```
$ art ls
spc% tst%  | name         | parts
0.0  0.0   | REQ-purpose  | SPC-cli, SPC-format, SPC-report
0.0  0.0   | SPC-cli      |
0.0  0.0   | SPC-format   |
0.0  0.0   | SPC-report   |

```

Now is also a good time to run `art serve`. This will serve your project
locally so that you can view and edit it through the Web UI.

[Here is an example of the project in its current state](examples/part1/index.html)


