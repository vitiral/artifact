## Table Of Contents
- <a style="font-weight: bold; color: #0074D9" title="REQ-BAZ" href="#REQ-BAZ">REQ-baz</a>
- <a style="font-weight: bold; color: #0074D9" title="REQ-FOO" href="#REQ-FOO">REQ-foo</a>
- <a style="font-weight: bold; color: #FF4136" title="REQ-LIB" href="#REQ-LIB">REQ-lib</a>
- <a style="font-weight: bold; color: #FF851B" title="REQ-PURPOSE" href="#REQ-PURPOSE">REQ-purpose</a>
- <a style="font-weight: bold; color: #0074D9" title="SPC-BUILD" href="#SPC-BUILD">SPC-build</a>
- <a style="font-weight: bold; color: #0074D9" title="SPC-FOO" href="#SPC-FOO">SPC-foo</a>
- <a style="font-weight: bold; color: #3DA03D" title="SPC-FOO_DONE" href="#SPC-FOO_DONE">SPC-foo_done</a>
- <a style="font-weight: bold; color: #FF851B" title="TST-BUILD" href="#TST-BUILD">TST-build</a>
- <a style="font-weight: bold; color: #FF851B" title="TST-FOO" href="#TST-FOO">TST-foo</a>


## REQ-baz
<details>
<summary><b>metadata</b></summary>
<b>partof:</b> <i>none</i></a><br>
<b>parts:</b> <i>none</i></a><br>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> src/baz.rs[0]<br>
<b>spc:</b>100.00&nbsp;&nbsp;<b>tst:</b>0.00<br>
<hr>
</details>

implemented directly in source!

Not a partof anything...


## REQ-foo
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #FF851B" title="REQ-PURPOSE" href="#REQ-PURPOSE">REQ-purpose</a></li>
<b>parts:</b><br>
<li><a style="font-weight: bold; color: #0074D9" title="SPC-FOO" href="#SPC-FOO">SPC-foo</a></li>
<li><a style="font-weight: bold; color: #3DA03D" title="SPC-FOO_DONE" href="#SPC-FOO_DONE">SPC-foo_done</a></li>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> <i>not implemented</i><br>
<b>spc:</b>87.50&nbsp;&nbsp;<b>tst:</b>89.30<br>
<hr>
</details>

foo needs to do the foo thing

## REQ-lib
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #FF851B" title="REQ-PURPOSE" href="#REQ-PURPOSE">REQ-purpose</a></li>
<b>parts:</b> <i>none</i></a><br>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> <i>not implemented</i><br>
<b>spc:</b>0.00&nbsp;&nbsp;<b>tst:</b>0.00<br>
<hr>
</details>

Lib is definitely a library

## REQ-purpose
<details>
<summary><b>metadata</b></summary>
<b>partof:</b> <i>none</i></a><br>
<b>parts:</b><br>
<li><a style="font-weight: bold; color: #0074D9" title="REQ-FOO" href="#REQ-FOO">REQ-foo</a></li>
<li><a style="font-weight: bold; color: #FF4136" title="REQ-LIB" href="#REQ-LIB">REQ-lib</a></li>
<li><a style="font-weight: bold; color: #0074D9" title="SPC-BUILD" href="#SPC-BUILD">SPC-build</a></li>
<li><a style="font-weight: bold; color: #FF851B" title="TST-BUILD" href="#TST-BUILD">TST-build</a></li>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> <i>not implemented</i><br>
<b>spc:</b>62.50&nbsp;&nbsp;<b>tst:</b>53.60<br>
<hr>
</details>

The purpose of this project is is to test a basic
project... that's it!


## SPC-build
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #FF851B" title="REQ-PURPOSE" href="#REQ-PURPOSE">REQ-purpose</a></li>
<b>parts:</b><br>
<li><a style="font-weight: bold; color: #FF851B" title="TST-BUILD" href="#TST-BUILD">TST-build</a></li>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> build.rs[0]<br>
<b>spc:</b>100.00&nbsp;&nbsp;<b>tst:</b>75.00<br>
<hr>
</details>

This has a build file.

Unit tests:
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/build.rs[5]" style="color: #0074D9"><b><i>.tst-unit</i></b></span>


## SPC-foo
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #0074D9" title="REQ-FOO" href="#REQ-FOO">REQ-foo</a></li>
<b>parts:</b><br>
<li><a style="font-weight: bold; color: #3DA03D" title="SPC-FOO_DONE" href="#SPC-FOO_DONE">SPC-foo_done</a></li>
<li><a style="font-weight: bold; color: #FF851B" title="TST-FOO" href="#TST-FOO">TST-foo</a></li>
<b>file:</b> design/foo.md<br>
<b>impl:</b> src/foo/mod.rs[0]<br>
<b>spc:</b>75.00&nbsp;&nbsp;<b>tst:</b>78.60<br>
<hr>
</details>

This is the spec for foo, it does lots of foo.

It is some foo subparts:
- <span title="Not Implemented" style="color: #FF4136"><b><i>.no</i></b></span>: not done
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/src/foo/fab.rs[3]" style="color: #0074D9"><b><i>.yes</i></b></span>: done


## SPC-foo_done
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #0074D9" title="REQ-FOO" href="#REQ-FOO">REQ-foo</a></li>
<li><a style="font-weight: bold; color: #0074D9" title="SPC-FOO" href="#SPC-FOO">SPC-foo</a></li>
<b>parts:</b> <i>none</i></a><br>
<b>file:</b> design/foo.md<br>
<b>impl:</b> this is done<br>
<b>spc:</b>100.00&nbsp;&nbsp;<b>tst:</b>100.00<br>
<hr>
</details>

This is done and is weird?

## TST-build
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #FF851B" title="REQ-PURPOSE" href="#REQ-PURPOSE">REQ-purpose</a></li>
<li><a style="font-weight: bold; color: #0074D9" title="SPC-BUILD" href="#SPC-BUILD">SPC-build</a></li>
<b>parts:</b> <i>none</i></a><br>
<b>file:</b> design/purpose.md<br>
<b>impl:</b> build.rs[4]<br>
<b>spc:</b>50.00&nbsp;&nbsp;<b>tst:</b>50.00<br>
<hr>
</details>

direct link to REQ-purpose

- <span title="Not Implemented" style="color: #FF4136"><b><i>.no</i></b></span>


## TST-foo
<details>
<summary><b>metadata</b></summary>
<b>partof:</b><br>
<li><a style="font-weight: bold; color: #0074D9" title="SPC-FOO" href="#SPC-FOO">SPC-foo</a></li>
<b>parts:</b> <i>none</i></a><br>
<b>file:</b> design/foo.md<br>
<b>impl:</b> <i>not implemented</i><br>
<b>spc:</b>57.10&nbsp;&nbsp;<b>tst:</b>57.10<br>
<hr>
</details>

Partially done foo test with some subparts

- <span title="Not Implemented" style="color: #FF4136"><b><i>.no1</i></b></span>
- <span title="Not Implemented" style="color: #FF4136"><b><i>.no2</i></b></span>
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/src/foo/test.rs[4]" style="color: #0074D9"><b><i>.yes1</i></b></span>
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/src/foo/test.rs[6]" style="color: #0074D9"><b><i>.yes2</i></b></span>
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/src/foo/test.rs[7]" style="color: #0074D9"><b><i>.yes3</i></b></span>
- <span title="/home/rett/open/artifact/artifact-test/interop_tests/settings/src/foo/fab.rs[9]" style="color: #0074D9"><b><i>.yes4</i></b></span>


