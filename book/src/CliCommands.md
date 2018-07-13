
The `ls` command is one of the most important command to learn in artifact,
as it helps you manage the artifacts in your project, see how they are linked,
and view how completed/tested they are.

Type:
    art ls SPC-learn-ls -l

This will show you this artifact, pretty printed on your terminal.

Try:
    art ls learn -p

This searches in the "name" field for all artifacts that have "learn"
in their name.

Let's say you wanted to find an artifact, and all you knew was that it mentioned
files in it's text field. You could run:

    art ls file -p text --text
    # OR
    art ls file -p T -T

This will search for the pattern "file" in the text field
(specified with `-p T`). It will also display a short piece of the text field
(specified with `-T`).

Now let's say that you see that SPC-learn-valid is what you were looking for,
but you want an expanded view:

   art ls SPC-learn-valid -l

Now you see that SPC-learn-valid has not been tested or implemented and that it
is partof SPC-LEARN. From there you could decide what to do.

See [[SPC-cmd-serve]] next.
'''

[SPC-cmd-serve]
text = '''
type `art serve` and goto the following link in your browser:

http://127.0.0.1:5373/#artifacts/req-toml

As you can see, this tutorial has been rendered via the web. You can explore,
click the links and even edit artifacts in place (edits will change this file).
