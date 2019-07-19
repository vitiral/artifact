check: fix lint test
	# SHIP IT!

ship: check
	rm -rf pycheck/
	virtualenv --python=python3 pycheck
	pycheck/bin/pip install .
	py3/bin/python setup.py sdist
	# run: py3/bin/twine upload dist/*


init:
	# python2
	virtualenv --python=python2 py2
	py2/bin/pip install -r requirements.txt
	py2/bin/pip install pytest
	# python3
	virtualenv --python=python3 py3
	py3/bin/pip install -r requirements.txt
	py3/bin/pip install pytest yapf pylint twine

fix:
	py3/bin/yapf --in-place -r anchor_txt tests

lint:
	py3/bin/pylint anchor_txt

test2:
	# Testing python2
	py2/bin/py.test -vvv

test3:
	# Testing python3
	py3/bin/py.test -vvv

test: test2 test3

clean:
	rm -rf py2 py3 dist anchor_txt.egg-info
