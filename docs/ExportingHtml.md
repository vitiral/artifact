The `art export html` command can be used to create a static site which
is included on github. For an example, see
[artifact's own design docs](http://vitiral.github.io/artifact/#artifacts/REQ-1)

In order to make a github page for your site that hosts your design documents:
- [activate github pages](https://pages.github.com/) (we will be using the
    `index.html` option)
- run `art export html`, which will generate an `index.html` file and a
    `css/` folder.
- run `git add index.html css` to add the generated files
- push to master or to `gh-pages` branch

That's it! You should be able to navigate to
`http://<username>.github.io/<repo-name>/` to view your page!
