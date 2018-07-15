The `art export html $DEST` command can be used to create a static site which
is included on github. For an example, see
[artifact's own design docs](http://vitiral.github.io/artifact/index.html)

In order to make a github page for your site that hosts your design documents:
- [Activate github pages](https://pages.github.com/) (we will be using the
    `index.html` option)
- Run `art export html`, which will generate an `index.html` file among other
  necessary files and folders.
- Run `git add index.html css` to add the generated files.
- Push to master or to `gh-pages` branch.

That's it! You should be able to navigate to
`http://<username>.github.io/<repo-name>/` to view your page!
