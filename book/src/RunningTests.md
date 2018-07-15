# Running Tests

If you followed along with the artifact interactive tutorial, you should feel
pretty confident by now that our load component is well designed and *should*
be implemented and tested. However, you haven't actually run any code yet, so
you can't be sure! We are going to change that.

The first thing you need to do is make sure you are running python2.7. Running:
```
python --version
pip --version
```

Should return something like:
```
Python 2.7.13
pip 9.0.1 from /usr/lib/python2.7/site-packages (python 2.7)
```

As long as they are both python2.7.X (but not python3.X), you are good to go.

> If not... python can be very difficult to configure.  Search on google for
> how to have both python2 and python3 installed. You will have to do a similar
> exercise for `pip`.

> If it is too much of a pain, you can also just use python3 (or any other
> language), it shouldn't be difficult -- you will just have to fix any errors
> that come up.
>
> If you are using another language, refer to that language's unit testing
> guide.

Now install `py.test`:
```
pip install pytest
```
> Note: it is recommended you add `--user` to the end or use a virtualenv.
> Using a virtualenv with python is out of scope of this tutorial.

Now run your unit tests:
```
py.test flash/
```

Congratulations, you've designed, written and run unit tests!

