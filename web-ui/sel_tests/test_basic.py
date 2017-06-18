"""TODO: these tests need to be better designed."""
from __future__ import print_function, absolute_import

import os
import time
import unittest

from selenium import webdriver
from selenium.common import exceptions
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

from . import artifact
from . import webapp

LIST_VIEW_ID = "list_view"
EDIT_VIEW_ID = "edit_view"
LIST_ID = "list"

EXAMPLE_PROJ = "web-ui/sel_tests/ex_proj"


class TestStuff(unittest.TestCase):
    """TODO: this is basically a proving ground for how to write tests in
    selenium.

    It is NOT well designed (yet).

    """
    @classmethod
    def setUpClass(cls):
        cls.app = webapp.App(webdriver.Firefox())

    @classmethod
    def tearDownClass(cls):
        app = getattr(cls, 'app', None)
        if app:
            app.quit()

    def test_req(self):
        """navigate to REQ and check that it is valid."""
        expected_parts = sorted(["REQ-purpose", "REQ-layout"])
        expected_partof = sorted([])

        app = self.app
        F = webapp.Fields

        with artifact.Artifact(EXAMPLE_PROJ) as url:
            app.driver.get(url)
            name = "REQ"
            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)

            # make sure all values look good
            app.assert_edit_view(timeout=5)
            app.get_value(name, F.name, timeout=2)
            assert app.get_items(name, F.parts) == expected_parts
            assert app.get_items(name, F.partof) == expected_partof

            # go back to list and assert values
            app.goto_list()
            app.assert_list_view(timeout=2)

            # parts are open by default, partof isn't
            assert app.get_items(name, F.parts, timeout=2) == expected_parts
            with self.assertRaises(exceptions.NoSuchElementException):
                app.get_items(name, F.partof)

            app.open_column(F.partof)
            assert app.get_items(name, F.partof, timeout=2) == expected_partof

            # now close columns and assert
            app.close_column(F.parts)
            WebDriverWait(app.driver, 1).until(
                EC.invisibility_of_element_located(
                    (By.ID, webapp.field_id(name, F.parts))))
            app.close_column(F.partof)
            WebDriverWait(app.driver, 1).until(
                EC.invisibility_of_element_located(
                    (By.ID, webapp.field_id(name, F.partof))))

    def test_edit_text(self):
        """Test editing a text field."""
        app = self.app
        F = webapp.Fields

        art = artifact.Artifact(EXAMPLE_PROJ)
        with art as url:
            app.driver.get(url)
            name = "SPC-LAYOUT"

            app.assert_list_view(timeout=10)
            app.goto_artifact(name, timeout=5)

            # do editing
            def assert_initial(edit, timeout=None):
                """assert that the text field has the initial text."""
                assert (app.get_value(name, F.raw_text, edit=edit, timeout=timeout)
                        == initial_text)
            initial_text = "This is literally just a partof REQ-layout"
            expected = "this is testing that you can edit"

            app.assert_edit_view(timeout=2)
            app.select_text(F.raw_text, timeout=2)
            assert_initial(edit=False, timeout=1)
            app.start_edit(timeout=1)
            assert_initial(edit=True, timeout=1)
            app.set_value(name, F.raw_text, expected)
            assert_initial(edit=False, timeout=1) # editing does not change read

            # undo the edit
            app.cancel_edit()
            assert_initial(edit=False, timeout=1)
            assert app.get_value(name, F.raw_text, timeout=2) == initial_text

            # edit again, save this time
            app.start_edit(timeout=1)
            assert_initial(edit=True, timeout=1)
            app.set_value(name, F.raw_text, expected)
            app.save_edit()
            time.sleep(0.5)
            app.driver.refresh()
            app.select_text(F.raw_text, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=1) == expected

            # make sure the file itself has changed
            with open(os.path.join(art.tmp_proj, "design/purpose.toml")) as purpose:
                file_text = purpose.read()
            assert expected in file_text

            # navigate away and then back... make sure it's still changed
            app.goto_list()
            app.goto_artifact(name, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=2) == expected

            # restart the server and make sure it's still changed
            assert url == art.restart(), "urls not equal"
            time.sleep(0.5)
            app.driver.get(url)  # refreshes app
            app.goto_artifact(name, timeout=5)
            app.select_text(F.raw_text, timeout=2)
            assert app.get_value(name, F.raw_text, timeout=2) == expected


# if __name__ == "__main__":
#     unittest.main()
